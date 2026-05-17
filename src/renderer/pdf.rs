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
    // DEBUG: merge_ascii_text_runs 임시 비활성화 — 페이지 번호 빈칸 회귀 원인 분리용.
    // merge_ascii_text_runs(&with_fallbacks)
    with_fallbacks
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
struct TextRunLine<'a> {
    x: &'a str,
    y: &'a str,
    prefix: &'a str,
    between_x_y: &'a str,
    suffix: &'a str,
    payload: &'a str,
}

/// HWP 원본 좌표가 문자 단위 advance로 내려오는 라틴 헤더는 대체 폰트 폭과 맞지 않아
/// 글자끼리 붙어 보인다. 같은 스타일/행의 연속 ASCII 조각은 단어 단위 `<text>`로
/// 합쳐서 PDF 폰트 엔진의 정상 kerning/advance를 사용한다.
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

        let key = (
            run.y.to_string(),
            run.prefix.to_string(),
            run.between_x_y.to_string(),
            run.suffix.to_string(),
        );

        if let Some(existing) = &mut pending {
            if existing.y == key.0
                && existing.prefix == key.1
                && existing.between_x_y == key.2
                && existing.suffix == key.3
            {
                existing.text.push_str(run.payload);
                if let Ok(x) = run.x.parse::<f64>() {
                    existing.last_x = x;
                }
                continue;
            }
        }

        flush_pending(&mut out, &mut pending);
        let x = run.x.parse::<f64>().unwrap_or(0.0);
        let font_size = parse_font_size(run.suffix).unwrap_or(16.0);
        pending = Some(AsciiRun {
            text: run.payload.to_string(),
            y: key.0,
            prefix: key.1,
            between_x_y: key.2,
            suffix: run.suffix.to_string(),
            first_x: x,
            last_x: x,
            font_size,
        });
    }

    flush_pending(&mut out, &mut pending);
    out
}

#[cfg(not(target_arch = "wasm32"))]
struct AsciiRun {
    text: String,
    y: String,
    prefix: String,
    between_x_y: String,
    suffix: String,
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
    let suffix = if use_center_anchor && !run.suffix.contains("text-anchor=") {
        format!("{} text-anchor=\"middle\"", run.suffix.trim_end_matches('>'))
    } else {
        run.suffix.trim_end_matches('>').to_string()
    };

    out.push_str(&run.prefix);
    out.push_str(" x=\"");
    out.push_str(&format_number(x));
    out.push('"');
    out.push_str(&run.between_x_y);
    out.push_str(" y=\"");
    out.push_str(&run.y);
    out.push('"');
    out.push_str(&suffix);
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

#[cfg(not(target_arch = "wasm32"))]
fn parse_ascii_text_run_line(line: &str) -> Option<TextRunLine<'_>> {
    if !line.contains("font-family=\"Noto Sans CJK KR,")
        || line.contains("transform=\"")
        || line.contains("<tspan")
    {
        return None;
    }

    let text_start = line.find("<text")?;
    let x_attr = line[text_start..].find(" x=\"")? + text_start;
    let x_value_start = x_attr + 4;
    let x_value_end = line[x_value_start..].find('"')? + x_value_start;
    let y_attr = line[x_value_end..].find(" y=\"")? + x_value_end;
    let y_value_start = y_attr + 4;
    let y_value_end = line[y_value_start..].find('"')? + y_value_start;
    let tag_end = line[y_value_end..].find('>')? + y_value_end;
    let close_start = line.rfind("</text>")?;
    if close_start <= tag_end + 1 {
        return None;
    }
    let payload = &line[tag_end + 1..close_start];
    if !payload
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '#' | '/' | '(' | ')' | '.' | '-' | ' '))
    {
        return None;
    }

    Some(TextRunLine {
        x: &line[x_value_start..x_value_end],
        y: &line[y_value_start..y_value_end],
        prefix: &line[..x_attr],
        between_x_y: &line[x_value_end + 1..y_attr],
        suffix: &line[y_value_end + 1..tag_end + 1],
        payload,
    })
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
    let pdf = svg2pdf::to_pdf(&tree, svg2pdf::ConversionOptions::default(), svg2pdf::PageOptions::default())
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

    use pdf_writer::{Pdf, Ref, Finish};
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

        page_datas.push(PageData { chunk, svg_ref, width: w, height: h });
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
        let renumbered = pd.chunk.renumber(|old| {
            *map.entry(old).or_insert_with(|| alloc.bump())
        });

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
    pdf.document_info(info_ref).producer(pdf_writer::TextStr("rhwp"));

    Ok(pdf.finish())
}
