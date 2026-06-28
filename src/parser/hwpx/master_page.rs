//! Contents/masterpage{N}.xml 파싱 — 바탕쪽(Master Page) → `MasterPage` IR.
//!
//! ## 배경
//!
//! HWPX 는 바탕쪽을 본문(section*.xml) 안에 두지 않고 별도
//! `Contents/masterpage{N}.xml` 파일에 저장하며, 섹션의 `<hp:secPr>` 안에서
//! `<hp:masterPage idRef="masterpage{N}"/>` 로 참조한다 (HWP5 의 inline LIST_HEADER
//! 와 다른 2-pass 구조).
//!
//! 이 파서가 없으면 `rhwp export-pdf <hwpx>` 가 바탕쪽을 무시해 꼬리말 쪽번호
//! (footer AutoNumber)가 렌더되지 않는다. 직렬화(`serializer/hwpx/master_page.rs`)는
//! 이미 구현되어 있어 round-trip 검증이 가능하다.
//!
//! ## 스키마
//!
//! ```xml
//! <masterPage id="masterpage0" type="EVEN" pageDuplicate="0" ...>
//!   <hp:subList textWidth=".." textHeight=".." hasTextRef="0" hasNumRef="0">
//!     <hp:p ...> ... (도형/글상자 안에 footer + <hp:autoNum numType="PAGE">) ... </hp:p>
//!     ...
//!   </hp:subList>
//! </masterPage>
//! ```
//!
//! 본문 문단 파싱은 `section::parse_paragraph` 를 재사용한다(표/도형/글상자/autoNum
//! 컨트롤 처리 동일). 따라서 footer 글상자 안의 AutoNumber 컨트롤이 그대로 IR 에
//! 들어가고, 렌더러(`document_core/queries/rendering.rs`)가 HWP5 경로와 동일하게
//! 쪽번호를 그린다.

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::model::header_footer::{HeaderFooterApply, MasterPage};

use super::section::parse_paragraph;
use super::utils::{attr_str, local_name, parse_u8, parse_u32};
use super::HwpxError;

/// `Contents/masterpage{N}.xml` 한 개를 `MasterPage` IR 로 파싱한다.
pub fn parse_hwpx_master_page(xml: &str) -> Result<MasterPage, HwpxError> {
    let mut mp = MasterPage::default();
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let ename = e.name();
                let local = local_name(ename.as_ref());
                match local {
                    b"masterPage" => parse_master_page_attrs(e, &mut mp),
                    b"subList" => {
                        parse_sub_list_attrs(e, &mut mp);
                        parse_sub_list_paragraphs(&mut reader, &mut mp)?;
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                // 빈 masterPage(내용 없는 바탕쪽) — 속성만 읽는다.
                if local_name(e.name().as_ref()) == b"masterPage" {
                    parse_master_page_attrs(e, &mut mp);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("masterpage: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    Ok(mp)
}

/// `<masterPage type=.. pageDuplicate=..>` 루트 속성 → apply_to / is_extension / overlap.
fn parse_master_page_attrs(e: &quick_xml::events::BytesStart, mp: &mut MasterPage) {
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"type" => {
                let ty = attr_str(&attr);
                match ty.as_str() {
                    "ODD" => mp.apply_to = HeaderFooterApply::Odd,
                    "EVEN" => mp.apply_to = HeaderFooterApply::Even,
                    // LAST_PAGE / OPTIONAL_PAGE 등 확장 바탕쪽: 마지막/임의 쪽 적용.
                    // 기본 바탕쪽(BOTH/ODD/EVEN)과 구분하기 위해 is_extension 마킹하고
                    // apply_to 는 Both(모든 해당 쪽)로 둔다 — 렌더러가 is_extension 으로 선택.
                    "BOTH" => mp.apply_to = HeaderFooterApply::Both,
                    _ => {
                        mp.apply_to = HeaderFooterApply::Both;
                        mp.is_extension = true;
                    }
                }
            }
            b"pageDuplicate" => {
                mp.overlap = parse_u8(&attr) == 1;
                if mp.overlap {
                    mp.ext_flags |= 0x01;
                }
            }
            _ => {}
        }
    }
}

/// `<hp:subList textWidth=.. textHeight=.. hasTextRef=.. hasNumRef=..>` 속성.
fn parse_sub_list_attrs(e: &quick_xml::events::BytesStart, mp: &mut MasterPage) {
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"textWidth" => mp.text_width = parse_u32(&attr),
            b"textHeight" => mp.text_height = parse_u32(&attr),
            b"hasTextRef" => mp.text_ref = parse_u8(&attr),
            b"hasNumRef" => mp.num_ref = parse_u8(&attr),
            _ => {}
        }
    }
}

/// subList 의 직속 `<hp:p>` 들을 파싱해 `mp.paragraphs` 에 채운다.
///
/// `parse_paragraph` 가 `</hp:p>` 까지 균형 있게 소비하므로(중첩 subList/도형 포함),
/// 이 루프는 최상위 `<hp:p>` 만 만난다. `</hp:subList>` 에서 종료한다.
fn parse_sub_list_paragraphs(
    reader: &mut Reader<&[u8]>,
    mp: &mut MasterPage,
) -> Result<(), HwpxError> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if local_name(e.name().as_ref()) == b"p" {
                    let (para, _sec_def) = parse_paragraph(e, reader)?;
                    mp.paragraphs.push(para);
                }
            }
            Ok(Event::End(ref e)) => {
                if local_name(e.name().as_ref()) == b"subList" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("masterpage subList: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const NS: &str = r#" xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph""#;

    #[test]
    fn parses_type_and_overlap() {
        let xml = format!(
            r#"<masterPage{ns} id="masterpage0" type="ODD" pageDuplicate="1">"#,
            ns = NS
        ) + r#"<hp:subList textWidth="66612" textHeight="90846" hasTextRef="0" hasNumRef="0">"#
            + r#"</hp:subList></masterPage>"#;
        let mp = parse_hwpx_master_page(&xml).unwrap();
        assert_eq!(mp.apply_to, HeaderFooterApply::Odd);
        assert!(mp.overlap);
        assert_eq!(mp.text_width, 66612);
        assert_eq!(mp.text_height, 90846);
    }

    #[test]
    fn last_page_is_extension() {
        let xml = format!(
            r#"<masterPage{ns} id="m" type="LAST_PAGE" pageDuplicate="0"><hp:subList></hp:subList></masterPage>"#,
            ns = NS
        );
        let mp = parse_hwpx_master_page(&xml).unwrap();
        assert!(mp.is_extension);
        assert_eq!(mp.apply_to, HeaderFooterApply::Both);
    }

    #[test]
    fn footer_autonum_reaches_ir() {
        // footer 글상자 없이 최상위 문단에 autoNum 만 둔 최소 케이스.
        let xml = format!(
            r#"<masterPage{ns} id="m" type="BOTH" pageDuplicate="0"><hp:subList textWidth="100" textHeight="100">"#,
            ns = NS
        ) + r#"<hp:p paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0">"#
            + r#"<hp:ctrl><hp:autoNum num="3" numType="PAGE"><hp:autoNumFormat type="DIGIT"/></hp:autoNum></hp:ctrl>"#
            + r#"<hp:t/></hp:run></hp:p></hp:subList></masterPage>"#;
        let mp = parse_hwpx_master_page(&xml).unwrap();
        assert_eq!(mp.paragraphs.len(), 1);
        let has_autonum = mp.paragraphs[0]
            .controls
            .iter()
            .any(|c| matches!(c, crate::model::control::Control::AutoNumber(_)));
        assert!(has_autonum, "autoNum 컨트롤이 IR 에 들어가야 함");
    }
}
