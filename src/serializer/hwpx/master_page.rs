//! Contents/masterpage{N}.xml — 바탕쪽(Master Page) 직렬화.
//!
//! `serialize_hwpx` 의 ⑦B 보강. HWP5 IR 의 `SectionDef.master_pages` 를 한컴 OWPML
//! `<masterPage>` 파일로 출력한다. 쪽번호(footer AutoNumber)는 바탕쪽 안 글상자에
//! 들어 있으므로, 바탕쪽을 직렬화하지 않으면 HWPX 에서 쪽번호가 소실된다.
//!
//! ## 스키마 오라클
//!
//! 구조는 한컴 reference `samples/hwpx/exam-kor-1p.hwpx` 의 `Contents/masterpage*.xml`
//! 와 동형이다 (네임스페이스 블록, `<masterPage>` 속성, `<hp:subList>` 래퍼).
//! 본문 문단/도형/글상자는 `section::render_paragraphs_as_hp_p` 로 본문과 동일 규칙 출력.
//!
//! ## 배치
//!
//! - 파일: `Contents/masterpage{globalIdx}.xml`
//! - 섹션 참조: `<hp:secPr ... masterPageCnt="N">...<hp:masterPage idRef="masterpage{K}"/>`
//! - content.hpf manifest: `<opf:item id="masterpage{K}" href="..." media-type="application/xml"/>`
//!   (spine 에는 넣지 않는다 — reference 동일.)

use crate::model::header_footer::{HeaderFooterApply, MasterPage};

use super::context::SerializeContext;
use super::section::render_paragraphs_as_hp_p;
use super::SerializeError;

/// reference masterpage*.xml 의 루트 네임스페이스 선언 (content.hpf 와 동일 14종).
const MASTER_PAGE_NS: &str = concat!(
    r#" xmlns:ha="http://www.hancom.co.kr/hwpml/2011/app""#,
    r#" xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph""#,
    r#" xmlns:hp10="http://www.hancom.co.kr/hwpml/2016/paragraph""#,
    r#" xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section""#,
    r#" xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core""#,
    r#" xmlns:hh="http://www.hancom.co.kr/hwpml/2011/head""#,
    r#" xmlns:hhs="http://www.hancom.co.kr/hwpml/2011/history""#,
    r#" xmlns:hm="http://www.hancom.co.kr/hwpml/2011/master-page""#,
    r#" xmlns:hpf="http://www.hancom.co.kr/schema/2011/hpf""#,
    r#" xmlns:dc="http://purl.org/dc/elements/1.1/""#,
    r#" xmlns:opf="http://www.idpf.org/2007/opf/""#,
    r#" xmlns:ooxmlchart="http://www.hancom.co.kr/hwpml/2016/ooxmlchart""#,
    r#" xmlns:hwpunitchar="http://www.hancom.co.kr/hwpml/2016/HwpUnitChar""#,
    r#" xmlns:epub="http://www.idpf.org/2007/ops""#,
    r#" xmlns:config="urn:oasis:names:tc:opendocument:xmlns:config:1.0""#,
);

/// `MasterPage.apply_to` → reference `type` 문자열.
fn master_page_type(mp: &MasterPage) -> &'static str {
    if mp.is_extension {
        // 확장 바탕쪽: 마지막 쪽 / 임의 쪽. is_extension 만으로는 구분 불가하므로
        // reference 와 정합하는 LAST_PAGE 로 보수적 매핑(쪽번호 표시 자체는 영향 없음).
        return "LAST_PAGE";
    }
    match mp.apply_to {
        HeaderFooterApply::Both => "BOTH",
        HeaderFooterApply::Even => "EVEN",
        HeaderFooterApply::Odd => "ODD",
    }
}

/// 바탕쪽 하나를 `Contents/masterpage{global_idx}.xml` XML 바이트로 직렬화한다.
///
/// `global_idx` 는 문서 전역 고유(모든 섹션 통합) — manifest/idRef 와 일치해야 한다.
pub fn write_master_page(
    mp: &MasterPage,
    global_idx: usize,
    ctx: &mut SerializeContext,
) -> Result<Vec<u8>, SerializeError> {
    let id = format!("masterpage{}", global_idx);
    let type_str = master_page_type(mp);
    let duplicate = if mp.overlap { "1" } else { "0" };

    let body = render_paragraphs_as_hp_p(&mp.paragraphs, ctx);

    let xml = format!(
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes" ?>"#,
            r#"<masterPage{ns} id="{id}" type="{ty}" pageNumber="0" pageDuplicate="{dup}" pageFront="0">"#,
            r#"<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP""#,
            r#" linkListIDRef="0" linkListNextIDRef="0" textWidth="{tw}" textHeight="{th}""#,
            r#" hasTextRef="{txref}" hasNumRef="{numref}">"#,
            r#"{body}"#,
            r#"</hp:subList></masterPage>"#,
        ),
        ns = MASTER_PAGE_NS,
        id = id,
        ty = type_str,
        dup = duplicate,
        tw = mp.text_width,
        th = mp.text_height,
        txref = mp.text_ref,
        numref = mp.num_ref,
        body = body,
    );

    Ok(xml.into_bytes())
}
