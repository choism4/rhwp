//! PDF 렌더러 (Task #21)
//!
//! SVG 렌더러의 출력을 svg2pdf + pdf-writer로 PDF를 생성한다.
//! 단일/다중 페이지 모두 지원. 네이티브 전용 (WASM 미지원).

/// 폰트 데이터베이스를 초기화 (시스템 폰트 + 프로젝트 폰트 로드)
///
/// generic serif/sans-serif 기본 family 를 한글 글리프 보유 CJK 폰트로 지정.
/// Linux fontconfig 가 "바탕" 같은 한글 폰트 이름을 라틴 전용 "Noto Sans" 로
/// 잘못 매칭하는 사고를 막는다. 시스템에 CJK 폰트가 없는 경우 usvg 가 자체
/// 일반 fallback 으로 진행하므로 회귀 위험은 낮다.
#[cfg(not(target_arch = "wasm32"))]
fn create_fontdb() -> usvg::fontdb::Database {
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_system_fonts();
    for dir in &[
        "ttfs",
        "ttfs/windows",
        "ttfs/hwp",
        "assets/fonts",
        "dist/assets/fonts",
        "packages/api/assets/fonts",
        "packages/api/dist/assets/fonts",
    ] {
        if std::path::Path::new(dir).exists() {
            fontdb.load_fonts_dir(dir);
        }
    }
    if std::path::Path::new("/mnt/c/Windows/Fonts").exists() {
        fontdb.load_fonts_dir("/mnt/c/Windows/Fonts");
    }
    fontdb.set_serif_family("Noto Serif CJK KR");
    fontdb.set_sans_serif_family("Noto Sans CJK KR");
    fontdb.set_monospace_family("D2Coding");
    fontdb
}

/// SVG에서 없는 한글 폰트명에 fallback 추가
#[cfg(not(target_arch = "wasm32"))]
fn add_font_fallbacks(svg: &str) -> String {
    let with_fallbacks = svg.replace("font-family=\"휴먼명조\"", "font-family=\"휴먼명조, 바탕, serif\"")
       .replace("font-family=\"HCI Poppy\"", "font-family=\"HCI Poppy, 맑은 고딕, sans-serif\"")
       // KBS V2 편집본은 "-윤고딕130/140"을 사용한다. 해당 상용 폰트가 없으면
       // fontdb가 Verdana 등으로 대체하면서 영문 헤더가 뭉개지거나 겹친다.
       // PDF 변환 단계에서만 리눅스/배포 환경에 포함 가능한 Noto Sans CJK KR로 치환한다.
       // 라틴 헤더(S#, PAGE, MEMO)는 아래 merge_ascii_text_runs에서 단어 단위로 합쳐
       // PDF 텍스트 선택/복사 매핑이 한글과 같은 CJK 폰트 경로를 타게 한다.
       // HWP/HWPX 원본 폰트 정보는 그대로 보존된다.
       .replace(
           "font-family=\"-윤고딕140,",
           "font-family=\"Noto Sans CJK KR,",
       )
       .replace(
           "font-family=\"-윤고딕140\"",
           "font-family=\"Noto Sans CJK KR,sans-serif\"",
       )
       .replace(
           "font-family=\"-윤고딕130,",
           "font-family=\"Noto Sans CJK KR,",
       )
       .replace(
           "font-family=\"-윤고딕130\"",
           "font-family=\"Noto Sans CJK KR,sans-serif\"",
       );
    // 연속 ASCII 조각(S#1. / PAGE / MEMO 등)을 단어 단위 <text>로 병합하여
    // 대체 폰트의 자연 advance 를 쓰게 한다. 장평(scale)·페이지 번호 보존은
    // merge_ascii_text_runs 내부 merge key(스타일+y+scale 완전일치)가 담당한다.
    let merged = merge_ascii_text_runs(&with_fallbacks);
    // 문장부호 글리프 겹침 보정 (v5). HWP line_seg 의 baked advance 가 렌더 폰트
    // (Noto Serif/Sans CJK) 의 실제 글리프 폭보다 좁은 `…`/`—`/`.`/`(` 등에서
    // 다음 글자가 앞 글자 위에 겹쳐 그려지는 결함을 PDF 변환 단계에서만 보정한다.
    fix_glyph_overlap(&merged)
}

// ── 문장부호 글리프 겹침 보정 (v5) ────────────────────────────────────
//
// 결함: HWP line_seg 의 char 위치는 한컴 메트릭 DB(HCR Batang 등) 기반 advance 로
// 산출되는데, PDF 렌더 폰트는 Noto Serif/Sans CJK 다. `…`(1.0em) `—`(0.89em)
// `.`(0.327em) 등은 메트릭 DB advance 가 Noto 글리프 폭보다 좁아, 다음 글자가
// 앞 글자 ink 위에 겹쳐 그려진다. 한글 음절은 side bearing 이 흡수해 ink 겹침이
// 없으므로(검출 게이트의 25% 임계 미만) 영향받지 않는다.
//
// 보정 원칙(무회귀):
//   - draw 위치(SVG <text> x)만 PDF 변환 직전에 이동. compute_char_positions /
//     줄바꿈 / 편집기 커서는 불변 → 페이지수·줄수 불변.
//   - 겹침이 실제 발생하는 인접쌍에만 적용(검출 게이트와 동일한 advance-box
//     비교; pymupdf char bbox 폭 = advance). 한글→한글 정상쌍은 25% 임계 미만
//     이라 미적용.
//   - 같은 줄(y 동일) 안에서 deficit 누적 shift 를 우측으로 전파하되, 뒤따르는
//     공백/간격(슬랙) 으로 흡수해 줄 끝 글자가 셀 보더를 넘어 클립되지 않게 한다.

/// 렌더 폰트(Noto) 글리프 메트릭 1글자 — em 단위(1000 upm 정규화).
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy)]
struct GlyphMetric {
    /// 가로 advance (em). 검출 게이트(pymupdf char bbox 폭)와 동일 기준.
    advance: f32,
}

/// Noto Serif/Sans CJK 글리프 메트릭 조회기. ttf-parser 로 1회 로드 후 캐시.
#[cfg(not(target_arch = "wasm32"))]
struct NotoMetrics {
    serif: Option<Vec<u8>>,
    sans: Option<Vec<u8>>,
}

#[cfg(not(target_arch = "wasm32"))]
fn noto_metrics() -> &'static NotoMetrics {
    use std::sync::OnceLock;
    static CELL: OnceLock<NotoMetrics> = OnceLock::new();
    CELL.get_or_init(|| {
        // create_fontdb 와 동일한 탐색 경로. 렌더 시 CWD(packages/api) 기준.
        let dirs = [
            "ttfs", "ttfs/windows", "ttfs/hwp",
            "assets/fonts", "dist/assets/fonts",
            "packages/api/ttfs",
            "packages/api/assets/fonts", "packages/api/dist/assets/fonts",
        ];
        let read_first = |names: &[&str]| -> Option<Vec<u8>> {
            for d in &dirs {
                for n in names {
                    let p = std::path::Path::new(d).join(n);
                    if let Ok(data) = std::fs::read(&p) {
                        return Some(data);
                    }
                }
            }
            None
        };
        NotoMetrics {
            serif: read_first(&["NotoSerifCJK-Regular.ttc", "NotoSerifCJK-Regular.otf"]),
            sans: read_first(&["NotoSansCJK-Regular.ttc", "NotoSansCJK-Regular.otf"]),
        }
    })
}

/// font-family 문자열로 Serif/Sans 선택 후 글리프 메트릭 조회.
/// 폰트 미적재/글리프 미존재 시 None → 호출측은 겹침 보정을 건너뛴다.
#[cfg(not(target_arch = "wasm32"))]
fn lookup_glyph_metric(font_family: &str, c: char) -> Option<GlyphMetric> {
    let m = noto_metrics();
    let is_serif = font_family.contains("Serif") || font_family.contains("바탕")
        || font_family.contains("명조") || font_family.contains("Batang");
    let data = if is_serif { m.serif.as_ref() } else { m.sans.as_ref() }
        .or(m.serif.as_ref())
        .or(m.sans.as_ref())?;
    let face = ttf_parser::Face::parse(data, 0).ok()?;
    let upm = face.units_per_em() as f32;
    if upm <= 0.0 { return None; }
    let gid = face.glyph_index(c)?;
    let adv = face.glyph_hor_advance(gid)? as f32 / upm;
    // svg2pdf/usvg(rustybuzz) 가 ASCII 숫자를 cmap 기본 글리프(proportional,
    // ~0.471em) 가 아니라 더 넓은 폭으로 셰이핑한다. 관측 렌더 폭은 템플릿/
    // 폰트크기에 따라 0.55~0.59em 으로 변동하므로, 병합 마커 런(S#11./S#31.)
    // 의 렌더 폭을 과소추정해 겹침을 놓치지 않도록 0.6em 으로 보정한다.
    // 과대추정해도 shift 가 수 px 더 우측으로 갈 뿐(셀/페이지 넘침 없음) 무해.
    let adv = if c.is_ascii_digit() { adv.max(0.6) } else { adv };
    Some(GlyphMetric { advance: adv })
}

/// 파싱된 SVG <text> 요소(겹침 보정 전용 표현).
#[cfg(not(target_arch = "wasm32"))]
struct GlyphTextElem {
    /// scale(S,1) 형식이면 true, x="" 형식이면 false.
    is_transform: bool,
    /// 현재 origin x (보정으로 갱신).
    x: f64,
    /// y 좌표 문자열(줄 식별 키). transform 의 경우 translate 의 Y.
    y: String,
    ratio: f64,
    font_size: f64,
    font_family: String,
    /// 언이스케이프된 payload(병합 ASCII run 은 여러 글자).
    payload: String,
}

/// metric DB advance 가 Noto 글리프 폭보다 좁아 겹침을 유발하는 좁은 문장부호.
/// 이런 글자(또는 이 글자로 끝나는 마커 런) 뒤에서는 작은 절대 tol 로 겹침을
/// 판정한다(한글→한글 25% 보호 규칙 우회). 한글/전각은 제외.
#[cfg(not(target_arch = "wasm32"))]
fn is_narrow_overlap_punct(c: char) -> bool {
    matches!(c,
        '.' | ',' | ':' | ';' | '!' | '?' | '~' | '/' | '-'
        | '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>'
        | '\'' | '"' | '`'
        | '\u{2026}' // …
        | '\u{2014}' // —
        | '\u{2013}' // –
        | '\u{2015}' // ―
        | '\u{00B7}' // ·
    )
}

/// payload 전체의 on-page advance 폭(렌더 폰트 기준). 한 글자라도 메트릭이 없으면
/// 그 글자는 폴백 폭(0.5em)으로 근사한다.
#[cfg(not(target_arch = "wasm32"))]
fn payload_advance_px(elem: &GlyphTextElem) -> f64 {
    let mut total = 0.0_f64;
    for c in elem.payload.chars() {
        let adv_em = lookup_glyph_metric(&elem.font_family, c)
            .map(|m| m.advance as f64)
            .unwrap_or(0.5);
        total += adv_em * elem.font_size * elem.ratio;
    }
    total
}

/// 같은 줄(y 동일, 연속) 안의 인접 <text> 글리프 겹침을 우측 shift 로 제거한다.
#[cfg(not(target_arch = "wasm32"))]
fn fix_glyph_overlap(svg: &str) -> String {
    let lines: Vec<&str> = svg.lines().collect();
    let parsed: Vec<Option<GlyphTextElem>> = {
        let mut v: Vec<Option<GlyphTextElem>> = Vec::with_capacity(lines.len());
        for l in lines.iter() {
            v.push(parse_glyph_text_line(l));
        }
        v
    };

    // 같은 줄(y 동일) 연속 요소를 그룹으로 처리. 비-text 라인이 끼면 그룹 끊김.
    let mut group: Vec<usize> = Vec::new(); // parsed 인덱스
    let mut group_y: Option<String> = None;
    let mut new_x: std::collections::HashMap<usize, f64> = std::collections::HashMap::new();

    let flush = |group: &mut Vec<usize>,
                 parsed: &[Option<GlyphTextElem>],
                 new_x: &mut std::collections::HashMap<usize, f64>| {
        if group.len() < 2 {
            group.clear();
            return;
        }
        // 검출 게이트(detect-glyph-overlap.py)는 PDF 글리프의 advance-box(=origin
        // ~ origin+advance) 끼리 비교한다(pymupdf rawdict char bbox 폭 = advance).
        // 따라서 보정도 advance-box 기준으로, 게이트와 동일한 임계로 판정한다:
        //   overlap = prev.origin + prev_adv − cur.origin
        //   flag if overlap > tol && overlap > cur_adv * 0.25
        // 한글→한글은 deficit(≈0.14em) < cur_adv 25%(≈0.24em) 라 미플래그 → 불변.
        // tol 은 게이트(1.0pt)보다 작게 잡아 잔여 여유 확보.
        const TOL: f64 = 0.5;
        let mut shift = 0.0_f64;
        for k in 1..group.len() {
            let prev_i = group[k - 1];
            let cur_i = group[k];
            let prev_right = {
                let p = parsed[prev_i].as_ref().unwrap();
                (p.x + shift) + payload_advance_px(p)
            };
            let c = parsed[cur_i].as_ref().unwrap();
            let cur_x = c.x + shift;
            // cur 의 첫 글자 advance(게이트는 cur 의 좌측 글자 폭으로 임계 판정).
            let cur_adv = match c.payload.chars().next()
                .and_then(|ch| lookup_glyph_metric(&c.font_family, ch)) {
                Some(m) => (m.advance as f64) * c.font_size * c.ratio,
                None => c.font_size * c.ratio * 0.5,
            };
            let overlap = prev_right - cur_x;
            // prev 가 narrow punctuation(. … — – ~ ( ) ! ? , : ; / 등) 또는
            // 그런 글자로 끝나는 마커 런(S#31. 등) 이면 metric DB 와 Noto 의 advance
            // 차이가 누적돼 실제 겹침이 발생한다 → 작은 절대 tol 로 판정.
            // prev 가 한글로 끝나면(한글→한글) deficit 가 작아 25% 임계로 보호한다.
            let prev_ends_narrow = parsed[prev_i].as_ref()
                .and_then(|p| p.payload.chars().last())
                .map(is_narrow_overlap_punct)
                .unwrap_or(false);
            let trigger = if prev_ends_narrow {
                overlap > TOL
            } else {
                overlap > TOL && overlap > cur_adv * 0.25
            };
            if trigger {
                // cur origin 이 prev advance-box 끝에 닿도록 우측으로 민다.
                shift += overlap;
            } else if overlap < 0.0 && shift > 0.0 {
                // prev 와 cur 사이에 여유(공백/간격) 가 있으면 그 슬랙으로 누적
                // shift 를 흡수한다 → 마커가 밀어낸 줄 끝 글자(예: "/ 낮") 가
                // 셀 보더를 넘어 클립되는 회귀 방지. 슬랙만큼만 shift 감소.
                shift = (shift + overlap).max(0.0);
            }
            // 누적 shift 를 cur(및 이후)에 전파.
            if shift > 0.0 {
                new_x.insert(cur_i, c.x + shift);
            }
        }
        group.clear();
    };

    for idx in 0..parsed.len() {
        let elem_y = parsed[idx].as_ref().map(|e| e.y.clone());
        match elem_y {
            Some(y) => {
                let same_line = group_y.as_deref() == Some(y.as_str());
                if !same_line {
                    flush(&mut group, &parsed, &mut new_x);
                    group_y = Some(y);
                }
                group.push(idx);
            }
            None => {
                flush(&mut group, &parsed, &mut new_x);
                group_y = None;
            }
        }
    }
    flush(&mut group, &parsed, &mut new_x);

    if new_x.is_empty() {
        return svg.to_string();
    }

    // 변경된 요소만 x 재작성, 나머지는 원본 라인 보존.
    let mut out = String::with_capacity(svg.len() + new_x.len() * 8);
    for (i, l) in lines.iter().enumerate() {
        if let Some(&nx) = new_x.get(&i) {
            if let Some(elem) = parsed[i].as_ref() {
                out.push_str(&rewrite_glyph_text_x(l, elem.is_transform, nx));
            } else {
                out.push_str(l);
            }
        } else {
            out.push_str(l);
        }
        out.push('\n');
    }
    out
}

/// SVG <text> 한 줄을 겹침 보정용으로 파싱. 단일/장평 두 형식 지원.
/// 멀티 글자 payload(병합된 ASCII run)도 first/last 글자만 추출해 처리.
#[cfg(not(target_arch = "wasm32"))]
fn parse_glyph_text_line(line: &str) -> Option<GlyphTextElem> {
    if line.contains("<tspan") {
        return None;
    }
    let text_start = line.find("<text")?;
    let after_text = &line[text_start + 5..];
    let tag_end_rel = after_text.find('>')?;
    let open_tag = &after_text[..tag_end_rel];
    let close_start = line.rfind("</text>")?;
    let tag_end_abs = text_start + 5 + tag_end_rel;
    if close_start <= tag_end_abs + 1 {
        return None;
    }
    let payload = &line[tag_end_abs + 1..close_start];
    let decoded = decode_xml_entities(payload);
    // 공백 전용 payload 는 보정 대상 아님.
    if decoded.trim().is_empty() {
        return None;
    }

    let font_size = parse_font_size(open_tag).unwrap_or(16.0);
    let font_family = parse_font_family(open_tag).unwrap_or_default();

    if let Some(tf_pos) = open_tag.find("transform=\"translate(") {
        let inner_start = tf_pos + "transform=\"translate(".len();
        let inner_end = open_tag[inner_start..].find(')')? + inner_start;
        let coords = &open_tag[inner_start..inner_end];
        let comma = coords.find(',')?;
        let x: f64 = coords[..comma].trim().parse().ok()?;
        let y = coords[comma + 1..].trim().to_string();
        let ratio = if let Some(sp) = open_tag[inner_end..].find("scale(") {
            let s_start = inner_end + sp + "scale(".len();
            let s_end = open_tag[s_start..].find(',').map(|p| p + s_start)
                .or_else(|| open_tag[s_start..].find(')').map(|p| p + s_start))?;
            open_tag[s_start..s_end].trim().parse().unwrap_or(1.0)
        } else {
            1.0
        };
        Some(GlyphTextElem {
            is_transform: true, x, y, ratio, font_size, font_family,
            payload: decoded,
        })
    } else {
        let x_marker = " x=\"";
        let x_pos = open_tag.find(x_marker)? + x_marker.len();
        let x_end = open_tag[x_pos..].find('"')? + x_pos;
        let x: f64 = open_tag[x_pos..x_end].trim().parse().ok()?;
        let y_marker = " y=\"";
        let y_pos = open_tag[x_end..].find(y_marker)? + x_end + y_marker.len();
        let y_end = open_tag[y_pos..].find('"')? + y_pos;
        let y = open_tag[y_pos..y_end].to_string();
        Some(GlyphTextElem {
            is_transform: false, x, y, ratio: 1.0, font_size, font_family,
            payload: decoded,
        })
    }
}

/// <text> 라인의 x(또는 translate 의 X) 만 새 값으로 교체.
#[cfg(not(target_arch = "wasm32"))]
fn rewrite_glyph_text_x(line: &str, is_transform: bool, new_x: f64) -> String {
    let nx = format_number(new_x);
    if is_transform {
        if let Some(tf_pos) = line.find("transform=\"translate(") {
            let inner_start = tf_pos + "transform=\"translate(".len();
            if let Some(rel_end) = line[inner_start..].find(')') {
                let inner_end = inner_start + rel_end;
                let coords = &line[inner_start..inner_end];
                if let Some(comma) = coords.find(',') {
                    let y = &coords[comma + 1..];
                    let mut out = String::with_capacity(line.len() + 4);
                    out.push_str(&line[..inner_start]);
                    out.push_str(&nx);
                    out.push(',');
                    out.push_str(y);
                    out.push_str(&line[inner_end..]);
                    return out;
                }
            }
        }
    } else if let Some(x_pos) = line.find(" x=\"") {
        let val_start = x_pos + " x=\"".len();
        if let Some(rel_end) = line[val_start..].find('"') {
            let val_end = val_start + rel_end;
            let mut out = String::with_capacity(line.len() + 4);
            out.push_str(&line[..val_start]);
            out.push_str(&nx);
            out.push_str(&line[val_end..]);
            return out;
        }
    }
    line.to_string()
}

/// open_tag 에서 font-family 값 추출(따옴표 안 전체).
#[cfg(not(target_arch = "wasm32"))]
fn parse_font_family(s: &str) -> Option<String> {
    let marker = "font-family=\"";
    let start = s.find(marker)? + marker.len();
    let end = s[start..].find('"')? + start;
    Some(s[start..end].to_string())
}

/// payload 의 XML 엔티티(&amp; &lt; &gt; &quot; &apos;) 를 원문자로 복원.
#[cfg(not(target_arch = "wasm32"))]
fn decode_xml_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    s.replace("&apos;", "'")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

/// 한 줄에서 추출한 단일 글자 `<text>` 정보.
///
/// rhwp svg 렌더러(`svg.rs`)는 ASCII/CJK 글자를 한 글자씩 별도 `<text>`로 출력한다.
/// 두 가지 형식이 있다 — (1) 장평 없음 `<text x="X" y="Y" {style}>c</text>`,
/// (2) 장평 적용 `<text transform="translate(X,Y) scale(S,1)" {style}>c</text>`.
///
/// 병합은 **같은 스타일(`style`)·같은 y·같은 scale** 인 연속 글자에만 적용한다.
/// `lead` 는 `<g clip-path=...>` 같은 줄 앞 래퍼로, 병합 그룹 첫 글자에서만 보존한다.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
struct TextRunLine<'a> {
    x: f64,
    y: &'a str,
    /// scale(S,1) 의 S. 장평 미적용이면 None.
    scale: Option<&'a str>,
    /// `<text` 앞의 래퍼 (`<g clip-path=...>` 등). 없으면 빈 문자열.
    lead: &'a str,
    /// transform/x/y 를 제외한 스타일 속성(`font-family=...` 부터 `>` 직전까지).
    style_attrs: &'a str,
    payload: &'a str,
}

/// HWP 원본 좌표가 문자 단위 advance로 내려오는 라틴 헤더는 대체 폰트 폭과 맞지 않아
/// 글자끼리 붙어 보인다. 같은 스타일/행/장평의 연속 ASCII 조각은 단어 단위 `<text>`로
/// 합쳐서 PDF 폰트 엔진의 정상 kerning/advance를 사용한다.
///
/// 페이지 번호(예: `- 2 -`)는 대시(바탕, scale 2.0, bold)와 숫자(`-윤명조150`, scale 없음)가
/// **서로 다른 스타일/scale** 이라 merge key 불일치로 절대 합쳐지지 않는다 → 빈칸 회귀 방지.
#[cfg(not(target_arch = "wasm32"))]
fn merge_ascii_text_runs(svg: &str) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut pending: Option<AsciiRun> = None;

    for line in svg.lines() {
        let parsed = parse_ascii_text_run_line(line);
        let Some(run) = parsed else {
            flush_pending(&mut out, &mut pending);
            out.push_str(line);
            out.push('\n');
            continue;
        };

        // 렌더러(svg.rs)는 ASCII 클러스터마다 글자별 advance 에 맞춘 textLength/
        // lengthAdjust 를 붙인다(글자별 값이 달라짐). 이를 style 비교/출력에서 제거해야
        // S#11. 같은 마커 런이 단어 단위로 병합돼 자연 advance 를 쓰고 글리프 겹침이
        // 사라진다(병합 안 하면 글자별 baked 좌표가 그대로 남아 …—.~ 외 #+숫자 등에서
        // 겹침 발생). 병합 출력에도 textLength 가 없어야 자연 kerning 이 적용된다.
        let run_style = strip_text_length_attrs(run.style_attrs);

        if let Some(existing) = &mut pending {
            // 스타일·y·scale 완전일치만 병합 (cross-style 오병합 차단).
            if existing.y == run.y
                && existing.scale.as_deref() == run.scale
                && existing.style_attrs == run_style
            {
                existing.text.push_str(run.payload);
                existing.last_x = run.x;
                continue;
            }
        }

        flush_pending(&mut out, &mut pending);
        let font_size = parse_font_size(&run_style).unwrap_or(16.0);
        pending = Some(AsciiRun {
            text: run.payload.to_string(),
            y: run.y.to_string(),
            scale: run.scale.map(str::to_string),
            lead: run.lead.to_string(),
            style_attrs: run_style,
            first_x: run.x,
            last_x: run.x,
            font_size,
        });
    }

    flush_pending(&mut out, &mut pending);
    out
}

/// SVG `<text>` style 속성에서 `textLength="..."` / `lengthAdjust="..."` 를 제거한다.
/// 렌더러가 글자별 advance 로 붙인 값이라 마커 런 병합(자연 advance 사용)을 막으므로,
/// 병합 비교·출력 단계에서만 떼어낸다(원본 SVG 의 한글 본문 텍스트에는 영향 없음).
#[cfg(not(target_arch = "wasm32"))]
fn strip_text_length_attrs(style: &str) -> String {
    let mut out = String::with_capacity(style.len());
    let mut rest = style;
    loop {
        let next = ["textLength=\"", "lengthAdjust=\""]
            .iter()
            .filter_map(|m| rest.find(m).map(|i| (i, m.len())))
            .min_by_key(|&(i, _)| i);
        match next {
            Some((i, marker_len)) => {
                out.push_str(&rest[..i]);
                // 값의 닫는 따옴표까지 건너뛴다.
                let after = &rest[i + marker_len..];
                if let Some(q) = after.find('"') {
                    rest = &after[q + 1..];
                } else {
                    rest = "";
                }
            }
            None => {
                out.push_str(rest);
                break;
            }
        }
    }
    // 제거로 생긴 이중 공백 정리(렌더 무해하지만 정돈).
    out.replace("  ", " ").trim_end().to_string()
}

#[cfg(not(target_arch = "wasm32"))]
struct AsciiRun {
    text: String,
    y: String,
    /// scale(S,1) 의 S. 장평 미적용이면 None — 병합 후에도 동일하게 보존.
    scale: Option<String>,
    lead: String,
    style_attrs: String,
    first_x: f64,
    last_x: f64,
    font_size: f64,
}

#[cfg(not(target_arch = "wasm32"))]
fn flush_pending(out: &mut String, pending: &mut Option<AsciiRun>) {
    let Some(run) = pending.take() else { return; };
    let use_center_anchor = should_center_anchor(&run);
    let x = if use_center_anchor {
        run_center_x(&run)
    } else {
        run.first_x
    };
    let style_attrs = run.style_attrs.trim_end_matches('>');
    let style_attrs = if use_center_anchor && !style_attrs.contains("text-anchor=") {
        format!("{} text-anchor=\"middle\"", style_attrs)
    } else {
        style_attrs.to_string()
    };

    out.push_str(&run.lead);
    match &run.scale {
        // 장평 보존: translate + scale(S,1) 형식 재출력. 자연 advance × scale 로
        // 전체폭만 축소되고 글자 간 겹침은 사라진다.
        Some(s) => {
            out.push_str("<text transform=\"translate(");
            out.push_str(&format_number(x));
            out.push(',');
            out.push_str(&run.y);
            out.push_str(") scale(");
            out.push_str(s);
            out.push_str(",1)\" ");
        }
        None => {
            out.push_str("<text x=\"");
            out.push_str(&format_number(x));
            out.push_str("\" y=\"");
            out.push_str(&run.y);
            out.push_str("\" ");
        }
    }
    out.push_str(&style_attrs);
    out.push('>');
    out.push_str(&run.text);
    out.push_str("</text>\n");
}

#[cfg(not(target_arch = "wasm32"))]
fn should_center_anchor(run: &AsciiRun) -> bool {
    run.text == "MEMO" && run.font_size >= 20.0
}

#[cfg(not(target_arch = "wasm32"))]
fn run_center_x(run: &AsciiRun) -> f64 {
    let char_count = run.text.chars().count();
    let estimated_last_advance = if char_count > 1 {
        (run.last_x - run.first_x) / (char_count.saturating_sub(1) as f64)
    } else {
        run.font_size * 0.5
    };
    (run.first_x + run.last_x + estimated_last_advance) / 2.0
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_font_size(s: &str) -> Option<f64> {
    let marker = "font-size=\"";
    let start = s.find(marker)? + marker.len();
    let end = s[start..].find('"')? + start;
    s[start..end].parse().ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn format_number(v: f64) -> String {
    let s = format!("{v:.4}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// 단일 글자 `<text>` 라인을 파싱한다. 장평(transform) / 비장평(x,y) 두 형식 모두 지원.
///
/// 게이트: 라틴 헤더/씬마커 폰트(`-윤명조`/`-윤고딕`/`Noto Sans CJK KR`)로 시작하고,
/// payload 가 ASCII(영숫자 + `#/().-` + 공백)이며, `<tspan` 미포함인 run 만 허용한다.
/// 한국어 본문 글자는 폰트 게이트에 걸리지 않아 병합 대상에서 자연 제외된다.
#[cfg(not(target_arch = "wasm32"))]
fn parse_ascii_text_run_line(line: &str) -> Option<TextRunLine<'_>> {
    if line.contains("<tspan") {
        return None;
    }

    let text_start = line.find("<text")?;
    let lead = &line[..text_start];
    // `<g ...>` 외의 비공백 래퍼가 앞에 붙어있으면 안전하게 제외.
    if !lead.trim().is_empty() && !lead.trim_start().starts_with("<g ") {
        return None;
    }
    let after_text = &line[text_start + 5..]; // "<text" 다음
    let tag_end_rel = after_text.find('>')?;
    let open_tag = &after_text[..tag_end_rel]; // 속성부 (`>` 제외)

    // 닫는 태그와 payload 추출.
    let close_start = line.rfind("</text>")?;
    let tag_end_abs = text_start + 5 + tag_end_rel;
    if close_start <= tag_end_abs + 1 {
        return None;
    }
    let payload = &line[tag_end_abs + 1..close_start];
    if payload.is_empty()
        || !payload
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '#' | '/' | '(' | ')' | '.' | '-' | ' '))
    {
        return None;
    }

    // 위치/장평 추출: transform 형식 우선, 없으면 x/y 형식.
    let (x, y, scale, style_attrs) = if let Some(tf_pos) = open_tag.find("transform=\"translate(") {
        let inner_start = tf_pos + "transform=\"translate(".len();
        let inner_end = open_tag[inner_start..].find(')')? + inner_start;
        let coords = &open_tag[inner_start..inner_end]; // "X,Y"
        let comma = coords.find(',')?;
        let x: f64 = coords[..comma].trim().parse().ok()?;
        let y = coords[comma + 1..].trim();
        // scale(S,1)
        let scale_pos = open_tag[inner_end..].find("scale(")? + inner_end;
        let s_start = scale_pos + "scale(".len();
        let s_end = open_tag[s_start..].find(')')? + s_start;
        let scale_inner = &open_tag[s_start..s_end]; // "S,1"
        let s = &scale_inner[..scale_inner.find(',').unwrap_or(scale_inner.len())];
        // transform 속성 전체("transform=\"...\"")의 닫는 따옴표 위치 이후가 style.
        let tf_attr_end = open_tag[s_end..].find('"')? + s_end + 1;
        let style = open_tag[tf_attr_end..].trim();
        // y 가 폰트 게이트 통과해야 함 — style 로 검사.
        if !is_marker_font(style) {
            return None;
        }
        (x, y, Some(s), style)
    } else {
        // 비장평: x="..." y="..." {style}
        let x_pos = open_tag.find(" x=\"").map(|p| p + 4).or_else(|| {
            open_tag.strip_prefix("x=\"").map(|_| 3)
        })?;
        let x_end = open_tag[x_pos..].find('"')? + x_pos;
        let x: f64 = open_tag[x_pos..x_end].trim().parse().ok()?;
        let y_pos = open_tag[x_end..].find(" y=\"")? + x_end + 4;
        let y_end = open_tag[y_pos..].find('"')? + y_pos;
        let y = &open_tag[y_pos..y_end];
        let style = open_tag[y_end + 1..].trim();
        if !is_marker_font(style) {
            return None;
        }
        (x, y, None, style)
    };

    Some(TextRunLine { x, y, scale, lead, style_attrs, payload })
}

/// 라틴 헤더/씬마커/페이지번호에 쓰이는 폰트 family 로 시작하는 style 인지.
/// add_font_fallbacks 가 윤고딕 → "Noto Sans CJK KR" 로 치환하므로 그 경우도 포함.
#[cfg(not(target_arch = "wasm32"))]
fn is_marker_font(style: &str) -> bool {
    let marker = "font-family=\"";
    let Some(start) = style.find(marker) else { return false; };
    let head = &style[start + marker.len()..];
    head.starts_with("Noto Sans CJK KR,")
        || head.starts_with("-윤명조")
        || head.starts_with("-윤고딕")
}

/// 단일 SVG를 PDF로 변환
#[cfg(not(target_arch = "wasm32"))]
pub fn svg_to_pdf(svg_content: &str) -> Result<Vec<u8>, String> {
    let fontdb = create_fontdb();
    let mut options = usvg::Options::default();
    options.fontdb = std::sync::Arc::new(fontdb);
    let svg_with_fallback = add_font_fallbacks(svg_content);
    let tree = usvg::Tree::from_str(&svg_with_fallback, &options)
        .map_err(|e| format!("SVG 파싱 실패: {}", e))?;
    let pdf = svg2pdf::to_pdf(
        &tree,
        svg2pdf::ConversionOptions::default(),
        svg2pdf::PageOptions::default(),
    )
    .map_err(|e| format!("PDF 변환 실패: {:?}", e))?;
    Ok(pdf)
}

/// 여러 SVG 페이지를 단일 다중 페이지 PDF로 생성
#[cfg(not(target_arch = "wasm32"))]
pub fn svgs_to_pdf(svg_pages: &[String]) -> Result<Vec<u8>, String> {
    if svg_pages.is_empty() {
        return Err("페이지가 없습니다".to_string());
    }
    if svg_pages.len() == 1 {
        return svg_to_pdf(&svg_pages[0]);
    }

    use pdf_writer::{Finish, Pdf, Ref};
    use std::collections::HashMap;

    let fontdb = create_fontdb();
    let mut options = usvg::Options::default();
    options.fontdb = std::sync::Arc::new(fontdb);

    let mut alloc = Ref::new(1);
    let catalog_ref = alloc.bump();
    let page_tree_ref = alloc.bump();

    // 각 페이지의 SVG를 파싱하여 chunk + page 정보 수집
    struct PageData {
        chunk: pdf_writer::Chunk,
        svg_ref: Ref,
        width: f32,
        height: f32,
    }

    let mut page_datas: Vec<PageData> = Vec::new();

    for svg in svg_pages {
        let svg_with_fallback = add_font_fallbacks(svg);
        let tree = usvg::Tree::from_str(&svg_with_fallback, &options)
            .map_err(|e| format!("SVG 파싱 실패: {}", e))?;

        let (chunk, svg_ref) = svg2pdf::to_chunk(&tree, svg2pdf::ConversionOptions::default())
            .map_err(|e| format!("SVG→chunk 변환 실패: {:?}", e))?;

        let dpi_ratio = 72.0 / 96.0; // 96 DPI → 72 pt
        let w = tree.size().width() * dpi_ratio;
        let h = tree.size().height() * dpi_ratio;

        page_datas.push(PageData {
            chunk,
            svg_ref,
            width: w,
            height: h,
        });
    }

    // 각 chunk를 재번호화하고 페이지 참조 수집
    let mut page_refs: Vec<Ref> = Vec::new();
    let mut renumbered_chunks: Vec<pdf_writer::Chunk> = Vec::new();
    let mut svg_refs_remapped: Vec<Ref> = Vec::new();

    for pd in &page_datas {
        let page_ref = alloc.bump();
        let content_ref = alloc.bump();
        page_refs.push(page_ref);

        // chunk 재번호화
        let mut map = HashMap::new();
        let renumbered = pd
            .chunk
            .renumber(|old| *map.entry(old).or_insert_with(|| alloc.bump()));

        let remapped_svg_ref = map.get(&pd.svg_ref).copied().unwrap_or(pd.svg_ref);
        svg_refs_remapped.push(remapped_svg_ref);
        renumbered_chunks.push(renumbered);
    }

    // PDF 생성
    let mut pdf = Pdf::new();
    pdf.catalog(catalog_ref).pages(page_tree_ref);
    pdf.pages(page_tree_ref)
        .count(page_refs.len() as i32)
        .kids(page_refs.iter().copied());

    // 각 페이지 생성
    let svg_name = pdf_writer::Name(b"S1");

    for (i, pd) in page_datas.iter().enumerate() {
        let page_ref = page_refs[i];
        let content_ref = alloc.bump();
        let svg_ref = svg_refs_remapped[i];

        let mut page = pdf.page(page_ref);
        page.media_box(pdf_writer::Rect::new(0.0, 0.0, pd.width, pd.height));
        page.parent(page_tree_ref);
        page.contents(content_ref);

        let mut resources = page.resources();
        resources.x_objects().pair(svg_name, svg_ref);
        resources.finish();
        page.finish();

        // 컨텐츠 스트림: SVG XObject를 페이지 크기에 맞게 배치
        let mut content = pdf_writer::Content::new();
        content.transform([pd.width, 0.0, 0.0, pd.height, 0.0, 0.0]);
        content.x_object(svg_name);

        pdf.stream(content_ref, &content.finish());
    }

    // 모든 chunk를 PDF에 추가
    for chunk in &renumbered_chunks {
        pdf.extend(chunk);
    }

    // 문서 정보
    let info_ref = alloc.bump();
    pdf.document_info(info_ref)
        .producer(pdf_writer::TextStr("rhwp"));

    Ok(pdf.finish())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod merge_tests {
    use super::merge_ascii_text_runs;

    const MYUNGJO: &str = "-윤명조130,&apos;Noto Serif CJK KR&apos;,serif";
    const NOTO: &str = "Noto Sans CJK KR,&apos;Malgun Gothic&apos;,sans-serif";
    const BATANG: &str = "바탕,&apos;Noto Serif CJK KR&apos;,serif";

    fn tf_text(x: f64, y: f64, s: &str, family: &str, extra: &str, c: &str) -> String {
        format!(
            "<text transform=\"translate({x},{y}) scale({s},1)\" font-family=\"{family}\" font-size=\"20\"{extra} fill=\"#000000\">{c}</text>",
        )
    }

    fn xy_text(x: f64, y: f64, family: &str, size: &str, extra: &str, c: &str) -> String {
        format!(
            "<text x=\"{x}\" y=\"{y}\" font-family=\"{family}\" font-size=\"{size}\"{extra} fill=\"#000000\">{c}</text>",
        )
    }

    // 본문 씬마커 S#1. (장평 0.95, -윤명조) 4글자가 단일 <text>로 병합되고 scale 보존.
    #[test]
    fn merges_scene_marker_preserving_scale() {
        let svg = [
            tf_text(255.24, 190.86, "0.9500", MYUNGJO, "", "S"),
            tf_text(263.18, 190.86, "0.9500", MYUNGJO, "", "#"),
            tf_text(271.13, 190.86, "0.9500", MYUNGJO, "", "1"),
            tf_text(279.08, 190.86, "0.9500", MYUNGJO, "", "."),
        ]
        .join("\n");
        let out = merge_ascii_text_runs(&svg);
        // 단일 병합 <text> + S#1. payload + scale 보존 + first_x 유지.
        assert_eq!(out.matches("<text").count(), 1, "should merge into one text: {out}");
        assert!(out.contains(">S#1.</text>"), "payload merged: {out}");
        assert!(out.contains("scale(0.9500,1)"), "scale preserved: {out}");
        assert!(out.contains("translate(255.24,190.86)"), "first_x preserved: {out}");
    }

    // 씬구성표 PAGE 헤더 (윤고딕→Noto Sans 치환, 장평 0.95) 병합.
    #[test]
    fn merges_page_header() {
        let svg = [
            tf_text(100.0, 50.0, "0.9500", NOTO, " font-weight=\"bold\"", "P"),
            tf_text(108.0, 50.0, "0.9500", NOTO, " font-weight=\"bold\"", "A"),
            tf_text(116.0, 50.0, "0.9500", NOTO, " font-weight=\"bold\"", "G"),
            tf_text(124.0, 50.0, "0.9500", NOTO, " font-weight=\"bold\"", "E"),
        ]
        .join("\n");
        let out = merge_ascii_text_runs(&svg);
        assert_eq!(out.matches("<text").count(), 1, "{out}");
        assert!(out.contains(">PAGE</text>"), "{out}");
    }

    // 페이지 번호 `- 2 -`: 대시(바탕,scale2.0,bold) + 숫자(-윤명조,scale없음)는
    // 스타일/scale 불일치로 절대 병합 안 됨 → 숫자가 사라지지 않음(빈칸 회귀 방지).
    #[test]
    fn page_number_digit_never_merges_away() {
        let dash_style = " font-weight=\"bold\"";
        let svg = [
            tf_text(439.0, 87.44, "2.0000", BATANG, dash_style, "-"),
            xy_text(481.0, 87.44, MYUNGJO, "20", "", "2"),
            tf_text(509.6, 87.44, "2.0000", BATANG, dash_style, "-"),
        ]
        .join("\n");
        let out = merge_ascii_text_runs(&svg);
        // 세 글자 모두 개별 <text> 로 살아있어야 함.
        assert_eq!(out.matches("<text").count(), 3, "no merge across styles: {out}");
        assert!(out.contains(">2</text>"), "digit survives: {out}");
        assert_eq!(out.matches(">-</text>").count(), 2, "both dashes survive: {out}");
        // 숫자는 scale 없는 x/y 형식 그대로.
        assert!(out.contains("x=\"481\" y=\"87.44\""), "digit form preserved: {out}");
    }

    // cross-style 인접(같은 y) ASCII 두 글자가 서로 다른 폰트면 병합 금지.
    #[test]
    fn different_font_does_not_merge() {
        let svg = [
            tf_text(10.0, 20.0, "0.9500", MYUNGJO, "", "A"),
            tf_text(18.0, 20.0, "0.9500", NOTO, "", "B"),
        ]
        .join("\n");
        let out = merge_ascii_text_runs(&svg);
        assert_eq!(out.matches("<text").count(), 2, "{out}");
    }

    // <g clip-path> 래퍼 보존 + 셀 경계(</g>)에서 병합 끊김.
    #[test]
    fn preserves_g_wrapper_and_breaks_on_close() {
        let line1 = format!(
            "<g clip-path=\"url(#cell-clip-1)\">{}",
            tf_text(10.0, 20.0, "0.9500", NOTO, "", "S"),
        );
        let line2 = tf_text(18.0, 20.0, "0.9500", NOTO, "", "#");
        let svg = [line1, line2, "</g>".to_string()].join("\n");
        let out = merge_ascii_text_runs(&svg);
        assert!(out.contains("<g clip-path=\"url(#cell-clip-1)\"><text"), "g wrapper kept: {out}");
        assert!(out.contains(">S#</text>"), "merged inside cell: {out}");
        assert!(out.contains("</g>"), "close tag preserved: {out}");
    }

    // MEMO 큰 글씨는 center-anchor 적용 경로 유지.
    #[test]
    fn memo_keeps_center_anchor() {
        let svg = [
            xy_text(100.0, 50.0, NOTO, "24", "", "M"),
            xy_text(115.0, 50.0, NOTO, "24", "", "E"),
            xy_text(130.0, 50.0, NOTO, "24", "", "M"),
            xy_text(145.0, 50.0, NOTO, "24", "", "O"),
        ]
        .join("\n");
        let out = merge_ascii_text_runs(&svg);
        assert!(out.contains(">MEMO</text>"), "{out}");
        assert!(out.contains("text-anchor=\"middle\""), "center anchor applied: {out}");
    }

    // 한국어 본문 글자(폰트 게이트 밖)는 절대 병합/변형되지 않고 그대로 통과.
    #[test]
    fn korean_body_untouched() {
        let svg = [
            tf_text(10.0, 20.0, "0.9500", MYUNGJO, "", "아"),
            tf_text(28.0, 20.0, "0.9500", MYUNGJO, "", "파"),
        ]
        .join("\n");
        let out = merge_ascii_text_runs(&svg);
        // payload 가 ASCII 아니므로 게이트 탈락 → 원본 2줄 그대로.
        assert_eq!(out.matches("<text").count(), 2, "{out}");
        assert!(out.contains(">아</text>") && out.contains(">파</text>"), "{out}");
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod overlap_tests {
    use super::*;

    const SERIF: &str = "-윤명조130,&apos;Noto Serif CJK KR&apos;,serif";

    /// Noto 폰트가 ttfs/ 에 적재되어 있어야 메트릭 의존 테스트가 의미 있다.
    fn fonts_available() -> bool {
        let m = noto_metrics();
        m.serif.is_some() || m.sans.is_some()
    }

    #[test]
    fn parse_transform_form() {
        let line = format!(
            "<text transform=\"translate(255.24,279.74) scale(0.9500,1)\" font-family=\"{SERIF}\" font-size=\"20\" fill=\"#000000\">S#11.</text>",
        );
        let e = parse_glyph_text_line(&line).expect("parsed");
        assert!(e.is_transform);
        assert!((e.x - 255.24).abs() < 1e-6);
        assert_eq!(e.y, "279.74");
        assert!((e.ratio - 0.95).abs() < 1e-6);
        assert!((e.font_size - 20.0).abs() < 1e-6);
        assert_eq!(e.payload, "S#11.");
    }

    #[test]
    fn parse_xy_form_and_entities() {
        let line = "<text x=\"100.5\" y=\"50\" font-family=\"Noto Sans CJK KR,sans-serif\" font-size=\"16\">&apos;</text>";
        let e = parse_glyph_text_line(line).expect("parsed");
        assert!(!e.is_transform);
        assert!((e.x - 100.5).abs() < 1e-6);
        assert_eq!(e.y, "50");
        assert!((e.ratio - 1.0).abs() < 1e-6);
        assert_eq!(e.payload, "'");
    }

    #[test]
    fn rewrite_keeps_y_and_scale() {
        let line = "<text transform=\"translate(10,20) scale(0.9500,1)\" font-size=\"20\">.</text>";
        let out = rewrite_glyph_text_x(line, true, 33.5);
        assert!(out.contains("translate(33.5,20)"), "{out}");
        assert!(out.contains("scale(0.9500,1)"), "{out}");
        assert!(out.ends_with(">.</text>"), "{out}");

        let xy = "<text x=\"10\" y=\"20\" font-size=\"20\">.</text>";
        let out2 = rewrite_glyph_text_x(xy, false, 33.5);
        assert!(out2.contains("x=\"33.5\""), "{out2}");
        assert!(out2.contains("y=\"20\""), "{out2}");
    }

    // 한글→한글 인접쌍은 side bearing 이 흡수(검출 게이트 25% 미만)하므로
    // 겹침 보정이 위치를 건드리면 안 된다 (줄 길이 증가 회귀 방지).
    #[test]
    fn cjk_pair_not_shifted() {
        if !fonts_available() { return; }
        // 한컴 baked advance(0.872em) 간격으로 배치된 두 한글 음절.
        let gap = 0.872 * 20.0 * 0.95; // ≈ 16.57px
        let svg = format!(
            "<text transform=\"translate(100,50) scale(0.9500,1)\" font-family=\"{SERIF}\" font-size=\"20\">건</text>\n\
             <text transform=\"translate({x2},50) scale(0.9500,1)\" font-family=\"{SERIF}\" font-size=\"20\">향</text>",
            x2 = 100.0 + gap,
        );
        let out = fix_glyph_overlap(&svg);
        // 위치(translate X) 가 그대로여야 한다. 후행 개행만 다를 수 있음.
        assert_eq!(out.trim_end(), svg.trim_end(), "CJK pair must be untouched:\n{out}");
        let hyang = parse_glyph_text_line(out.lines().nth(1).unwrap()).unwrap();
        assert!((hyang.x - (100.0 + gap)).abs() < 1e-3, "향 x unchanged: {}", hyang.x);
    }

    // 한글 뒤 마침표(.) 는 baked advance 가 한글 글리프 폭보다 좁아 겹친다 →
    // 마침표(및 이후)가 우측으로 밀려야 한다.
    #[test]
    fn punct_after_cjk_is_shifted() {
        if !fonts_available() { return; }
        // 한글 baked advance(0.79em) 로 좁게 배치 → 마침표가 한글 ink 위에 겹침.
        let x_dot = 100.0 + 0.79 * 20.0 * 0.95; // ≈ 115.0
        let svg = format!(
            "<text transform=\"translate(100,50) scale(0.9500,1)\" font-family=\"{SERIF}\" font-size=\"20\">도</text>\n\
             <text transform=\"translate({x_dot},50) scale(0.9500,1)\" font-family=\"{SERIF}\" font-size=\"20\">.</text>",
        );
        let out = fix_glyph_overlap(&svg);
        // 마침표 x 가 원래(x_dot)보다 우측으로 이동했는지 확인.
        let new_dot = parse_glyph_text_line(out.lines().nth(1).unwrap()).unwrap();
        assert!(new_dot.x > x_dot + 0.5, "dot must shift right: {} <= {}", new_dot.x, x_dot);
    }
}
