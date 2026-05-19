//! 페이지 매김(쪽번호) 컨트롤 조작
//!
//! - `add_new_number_native`: 본문 구역 시작 단락에 `NewNumber(Page=N)` 삽입.
//!   `PageNumberAssigner`(`renderer/page_number.rs`)가 카운터를 N으로 리셋한다.
//! - `set_master_page_page_number_format_native`: 구역의 모든 바탕쪽 안 글상자에 들어 있는
//!   `AutoNumber(Page).format` 을 일괄 변경. HWP 스펙 표 134 (0=숫자, 4=A,B,C 등).

use crate::document_core::DocumentCore;
use crate::document_core::queries::bookmark_query::{char_offset_to_raw, find_control_insert_index};
use crate::error::HwpError;
use crate::model::control::{AutoNumberType, Control, NewNumber};
use crate::model::shape::ShapeObject;

impl DocumentCore {
    /// NewNumber(Page=number) 컨트롤을 지정 단락에 삽입한다.
    pub fn add_new_number_native(
        &mut self,
        sec: usize,
        para: usize,
        char_offset: usize,
        number: u16,
    ) -> Result<String, HwpError> {
        let section = self.document.sections.get_mut(sec)
            .ok_or_else(|| HwpError::RenderError("구역 범위 초과".into()))?;
        let paragraph = section.paragraphs.get_mut(para)
            .ok_or_else(|| HwpError::RenderError("문단 범위 초과".into()))?;

        let insert_idx = find_control_insert_index(paragraph, char_offset);

        paragraph.controls.insert(insert_idx, Control::NewNumber(NewNumber {
            number_type: AutoNumberType::Page,
            number,
        }));

        // NewNumber 는 CTRL_DATA 가 없는 컨트롤. None 자리 차지.
        if paragraph.ctrl_data_records.len() >= insert_idx {
            paragraph.ctrl_data_records.insert(insert_idx, None);
        }

        if !paragraph.char_offsets.is_empty() {
            let raw_offset = char_offset_to_raw(paragraph, char_offset, insert_idx);
            paragraph.char_offsets.insert(insert_idx, raw_offset);
        }

        // raw_stream 무효화 — modify 가 직렬화에 반영되도록.
        section.raw_stream = None;

        self.recompose_section(sec);

        Ok(r#"{"ok":true}"#.to_string())
    }

    /// 바탕쪽 footer 의 쪽번호 글상자 세로 위치를 작품명 글상자에 맞춰 정렬한다.
    ///
    /// 일부 편집틀은 쪽번호 글상자와 작품명 글상자를 서로 다른 vertical_offset 으로
    /// 배치해 쪽번호("- N -")와 작품명 텍스트의 baseline 이 어긋난다. 두 글상자는
    /// 같은 높이·valign=Center 이므로, 쪽번호 글상자의 vertical_offset 을 작품명
    /// 글상자와 동일하게 맞추면 정렬된다.
    ///
    /// 쪽번호 글상자 = textbox 안에 AutoNumber(Page) 보유.
    /// 작품명 글상자 = textbox 에 본문 텍스트 보유 + AutoNumber·그림 없음.
    ///
    /// 반환: JSON `{"ok":true,"aligned":N}`
    ///
    /// 바탕쪽은 raw 레코드(extra_child_records)로 직렬화되므로 모델 변경만으론
    /// export 에 반영되지 않는다. 모델에서 (쪽번호박스 v_off → 작품명박스 v_off)
    /// 매핑을 만든 뒤, CTRL_HEADER 레코드의 vertical_offset(데이터 offset 8) 을
    /// 직접 패치한다 (replace_text_in_master_pages 와 동일한 raw-patch 방식).
    pub fn align_master_footer_voffset(&mut self, sec: usize) -> Result<String, HwpError> {
        use std::collections::HashMap;
        let section = self.document.sections.get_mut(sec)
            .ok_or_else(|| HwpError::RenderError("구역 범위 초과".into()))?;

        // 1) 모델에서 v_off 매핑 작성 (패치 전 원본값 기준).
        let mut voff_map: HashMap<u32, u32> = HashMap::new();
        for mp in &section.section_def.master_pages {
            collect_footer_voff_map(mp, &mut voff_map);
        }
        if voff_map.is_empty() {
            for para in &section.paragraphs {
                for ctrl in &para.controls {
                    if let Control::SectionDef(sd) = ctrl {
                        for mp in &sd.master_pages {
                            collect_footer_voff_map(mp, &mut voff_map);
                        }
                    }
                }
            }
        }
        if voff_map.is_empty() {
            return Ok(r#"{"ok":true,"aligned":0}"#.to_string());
        }

        // 2) raw 레코드 패치 — 직렬화 source(Control::SectionDef) + section_def 미러.
        let mut patched: u32 = 0;
        for para in section.paragraphs.iter_mut() {
            for ctrl in para.controls.iter_mut() {
                if let Control::SectionDef(sd) = ctrl {
                    patched += patch_ctrl_header_voff(&mut sd.extra_child_records, &voff_map);
                }
            }
        }
        patched += patch_ctrl_header_voff(
            &mut section.section_def.extra_child_records, &voff_map,
        );

        // 3) 모델 master_pages 도 sync (in-memory 렌더 경로용).
        for mp in section.section_def.master_pages.iter_mut() {
            align_master_footer(mp);
        }
        for para in section.paragraphs.iter_mut() {
            for ctrl in para.controls.iter_mut() {
                if let Control::SectionDef(sd) = ctrl {
                    for mp in sd.master_pages.iter_mut() {
                        align_master_footer(mp);
                    }
                }
            }
        }

        if patched > 0 {
            section.raw_stream = None;
        }
        Ok(format!(r#"{{"ok":true,"aligned":{}}}"#, patched))
    }

    /// 구역의 모든 바탕쪽 안 글상자 내 AutoNumber(Page).format 을 일괄 변경.
    /// `format` HWP 스펙 표 134 — 0=Digit, 4=LatinUpper(A,B,C), 5=LatinLower(a,b,c).
    pub fn set_master_page_page_number_format_native(
        &mut self,
        sec: usize,
        format: u8,
    ) -> Result<String, HwpError> {
        let section = self.document.sections.get_mut(sec)
            .ok_or_else(|| HwpError::RenderError("구역 범위 초과".into()))?;

        let mut updated: u32 = 0;
        for master_page in section.section_def.master_pages.iter_mut() {
            for para in master_page.paragraphs.iter_mut() {
                updated += update_paragraph(para, format);
            }
        }

        if updated > 0 {
            section.raw_stream = None;
        }

        Ok(format!(r#"{{"ok":true,"updated":{}}}"#, updated))
    }

    /// 바탕쪽 LIST_HEADER 의 ext_flags(데이터 byte 18) 에서 overlap(0x01)·
    /// is_extension(0x02) 비트를 클리어한다.
    ///
    /// 웹 한컴독스로 편집한 골격 템플릿은 정규 바탕쪽(Odd/Even)을 확장 바탕쪽으로
    /// 잘못 표시한다. `rendering.rs` 의 master 선택은 `!is_extension` 인 것만
    /// 정규 패리티 master 로 잡으므로, 확장 표시된 Odd 바탕쪽은 홀수 본문
    /// 페이지에서 선택되지 않아 footer(쪽번호·작품명 글상자)가 통째로 소실된다.
    /// 바탕쪽은 raw 레코드로 직렬화되므로 `extra_child_records` 를 직접 패치한다.
    pub fn clear_master_page_ext_flags(&mut self, sec: usize) -> Result<String, HwpError> {
        let section = self.document.sections.get_mut(sec)
            .ok_or_else(|| HwpError::RenderError("구역 범위 초과".into()))?;

        let mut patched: u32 = 0;
        for para in section.paragraphs.iter_mut() {
            for ctrl in para.controls.iter_mut() {
                if let Control::SectionDef(sd) = ctrl {
                    patched += clear_ext_flags_in_records(&mut sd.extra_child_records);
                }
            }
        }
        patched += clear_ext_flags_in_records(&mut section.section_def.extra_child_records);

        // 모델 미러 (in-memory 렌더 경로).
        for mp in section.section_def.master_pages.iter_mut() {
            mp.is_extension = false;
            mp.overlap = false;
            mp.ext_flags &= !0x03;
        }
        for para in section.paragraphs.iter_mut() {
            for ctrl in para.controls.iter_mut() {
                if let Control::SectionDef(sd) = ctrl {
                    for mp in sd.master_pages.iter_mut() {
                        mp.is_extension = false;
                        mp.overlap = false;
                        mp.ext_flags &= !0x03;
                    }
                }
            }
        }

        if patched > 0 {
            section.raw_stream = None;
        }
        Ok(format!(r#"{{"ok":true,"patched":{}}}"#, patched))
    }
}

/// 바탕쪽 최상위 LIST_HEADER 레코드의 ext_flags(byte 18) 에서 0x03 비트를 클리어.
/// top-level = LIST_HEADER 최소 level — 하위 level 은 도형 내부 텍스트박스다
/// (`parse_master_pages_from_raw` 와 동일 규칙).
fn clear_ext_flags_in_records(records: &mut [crate::model::document::RawRecord]) -> u32 {
    use crate::parser::tags;
    let top_level = records.iter()
        .filter(|r| r.tag_id == tags::HWPTAG_LIST_HEADER)
        .map(|r| r.level)
        .min();
    let Some(top_level) = top_level else { return 0; };
    let mut count = 0;
    for rec in records.iter_mut() {
        if rec.tag_id != tags::HWPTAG_LIST_HEADER || rec.level != top_level {
            continue;
        }
        if rec.data.len() < 20 {
            continue;
        }
        // text_width(8..12)·text_height(12..16) 0×0 = MEMO 텍스트박스 — skip.
        let tw = u32::from_le_bytes([rec.data[8], rec.data[9], rec.data[10], rec.data[11]]);
        let th = u32::from_le_bytes([rec.data[12], rec.data[13], rec.data[14], rec.data[15]]);
        if tw == 0 && th == 0 {
            continue;
        }
        if rec.data[18] & 0x03 != 0 {
            rec.data[18] &= !0x03;
            count += 1;
        }
    }
    count
}

fn update_paragraph(para: &mut crate::model::paragraph::Paragraph, format: u8) -> u32 {
    let mut count = 0;
    for ctrl in para.controls.iter_mut() {
        count += update_control(ctrl, format);
    }
    count
}

fn update_control(ctrl: &mut Control, format: u8) -> u32 {
    let mut count = 0;
    match ctrl {
        Control::AutoNumber(an) => {
            if an.number_type == AutoNumberType::Page {
                an.format = format;
                count += 1;
            }
        }
        Control::Shape(shape) => {
            count += update_shape(shape.as_mut(), format);
        }
        Control::Table(t) => {
            for cell in t.cells.iter_mut() {
                for p in cell.paragraphs.iter_mut() {
                    count += update_paragraph(p, format);
                }
            }
        }
        Control::Header(h) => {
            for p in h.paragraphs.iter_mut() {
                count += update_paragraph(p, format);
            }
        }
        Control::Footer(f) => {
            for p in f.paragraphs.iter_mut() {
                count += update_paragraph(p, format);
            }
        }
        _ => {}
    }
    count
}

fn update_shape(shape: &mut ShapeObject, format: u8) -> u32 {
    let mut count = 0;
    if let ShapeObject::Group(g) = shape {
        for child in g.children.iter_mut() {
            count += update_shape(child, format);
        }
        return count;
    }
    let text_box = match shape {
        ShapeObject::Rectangle(s) => s.drawing.text_box.as_mut(),
        ShapeObject::Ellipse(s) => s.drawing.text_box.as_mut(),
        ShapeObject::Polygon(s) => s.drawing.text_box.as_mut(),
        ShapeObject::Curve(s) => s.drawing.text_box.as_mut(),
        ShapeObject::Arc(s) => s.drawing.text_box.as_mut(),
        ShapeObject::Line(s) => s.drawing.text_box.as_mut(),
        ShapeObject::Chart(s) => s.drawing.text_box.as_mut(),
        ShapeObject::Ole(s) => s.drawing.text_box.as_mut(),
        _ => None,
    };
    if let Some(tb) = text_box {
        for p in tb.paragraphs.iter_mut() {
            count += update_paragraph(p, format);
        }
    }
    count
}

#[derive(PartialEq)]
enum FooterRole {
    PageNumber,
    Title,
    Other,
}

/// 바탕쪽 글상자의 footer 역할 판정.
/// 쪽번호 = textbox 에 AutoNumber(Page). 작품명 = 텍스트 있고 AutoNumber·그림 없음.
fn shape_footer_role(shape: &ShapeObject) -> FooterRole {
    let tb = match shape.drawing().and_then(|d| d.text_box.as_ref()) {
        Some(t) => t,
        None => return FooterRole::Other,
    };
    let mut has_autonumber_page = false;
    let mut has_text = false;
    let mut has_picture = false;
    for p in &tb.paragraphs {
        if !p.text.trim().is_empty() {
            has_text = true;
        }
        for c in &p.controls {
            match c {
                Control::AutoNumber(an) if an.number_type == AutoNumberType::Page => {
                    has_autonumber_page = true;
                }
                Control::Picture(_) => has_picture = true,
                Control::Shape(s) => {
                    if matches!(s.as_ref(), ShapeObject::Picture(_)) {
                        has_picture = true;
                    }
                }
                _ => {}
            }
        }
    }
    if has_autonumber_page {
        FooterRole::PageNumber
    } else if has_text && !has_picture {
        FooterRole::Title
    } else {
        FooterRole::Other
    }
}

/// 한 바탕쪽 안에서 쪽번호 글상자의 vertical_offset 을 작품명 글상자에 맞춘다.
fn align_master_footer(mp: &mut crate::model::header_footer::MasterPage) -> u32 {
    let mut title_voff: Option<u32> = None;
    for para in &mp.paragraphs {
        for ctrl in &para.controls {
            if let Control::Shape(s) = ctrl {
                if shape_footer_role(s) == FooterRole::Title {
                    title_voff = Some(s.common().vertical_offset);
                }
            }
        }
    }
    let title_voff = match title_voff {
        Some(v) => v,
        None => return 0,
    };
    let mut count = 0;
    for para in mp.paragraphs.iter_mut() {
        for ctrl in para.controls.iter_mut() {
            if let Control::Shape(s) = ctrl {
                if shape_footer_role(s) == FooterRole::PageNumber
                    && s.common().vertical_offset != title_voff
                {
                    s.common_mut().vertical_offset = title_voff;
                    count += 1;
                }
            }
        }
    }
    count
}

/// 한 바탕쪽에서 (쪽번호 글상자 v_off → 작품명 글상자 v_off) 매핑을 수집한다.
fn collect_footer_voff_map(
    mp: &crate::model::header_footer::MasterPage,
    map: &mut std::collections::HashMap<u32, u32>,
) {
    let mut pagenum: Option<u32> = None;
    let mut title: Option<u32> = None;
    for para in &mp.paragraphs {
        for ctrl in &para.controls {
            if let Control::Shape(s) = ctrl {
                match shape_footer_role(s) {
                    FooterRole::PageNumber => pagenum = Some(s.common().vertical_offset),
                    FooterRole::Title => title = Some(s.common().vertical_offset),
                    FooterRole::Other => {}
                }
            }
        }
    }
    if let (Some(p), Some(t)) = (pagenum, title) {
        if p != t {
            map.insert(p, t);
        }
    }
}

/// CTRL_HEADER 레코드의 vertical_offset(데이터 offset 8, u32 LE) 을 매핑대로 패치.
/// CTRL_HEADER 데이터 = ctrl_id(4) + attr(4) + vertical_offset(4) + ...
fn patch_ctrl_header_voff(
    records: &mut [crate::model::document::RawRecord],
    map: &std::collections::HashMap<u32, u32>,
) -> u32 {
    use crate::parser::tags;
    let mut count = 0;
    for rec in records.iter_mut() {
        if rec.tag_id != tags::HWPTAG_CTRL_HEADER || rec.data.len() < 12 {
            continue;
        }
        let voff = u32::from_le_bytes([rec.data[8], rec.data[9], rec.data[10], rec.data[11]]);
        if let Some(&new_voff) = map.get(&voff) {
            rec.data[8..12].copy_from_slice(&new_voff.to_le_bytes());
            count += 1;
        }
    }
    count
}
