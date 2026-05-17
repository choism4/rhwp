// 지정 구역·문단의 그림(Picture) control 을 전부 삭제하고 HWP 재출력.
// 제작진 연락처 표 등 본문 그림 잔재 제거용.
// 사용: cargo run --release --example delete_para_pictures -- <입력.hwp> <출력.hwp> <section> <para>
use std::fs;
use rhwp::model::control::Control;

fn main() {
    let mut args = std::env::args().skip(1);
    let input = args.next().expect("사용: delete_para_pictures <입력.hwp> <출력.hwp> <section> <para>");
    let output = args.next().expect("출력 경로 필요");
    let section: usize = args.next().expect("section 필요").parse().expect("section 은 정수");
    let para: usize = args.next().expect("para 필요").parse().expect("para 는 정수");

    let data = fs::read(&input).expect("입력 파일 읽기 실패");
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("파싱 실패");

    // 해당 문단의 Picture control 인덱스 수집 (내림차순 — 삭제 시 인덱스 시프트 방지).
    let mut pic_indices: Vec<usize> = {
        let doc = core.document();
        let controls = &doc.sections[section].paragraphs[para].controls;
        controls
            .iter()
            .enumerate()
            .filter(|(_, c)| matches!(c, Control::Picture(_)))
            .map(|(i, _)| i)
            .collect()
    };
    pic_indices.sort_unstable_by(|a, b| b.cmp(a));
    println!("section {section} para {para}: 그림 control {}개 발견 → 삭제", pic_indices.len());

    for idx in pic_indices {
        let r = core
            .delete_picture_control_native(section, para, idx)
            .expect("그림 삭제 실패");
        println!("  ctrl[{idx}] 삭제: {r}");
    }

    // 그림 제거 후 문단이 완전히 비면(빈 페이지 잔존 방지) 문단 자체도 삭제.
    let para_empty = {
        let p = &core.document().sections[section].paragraphs[para];
        p.controls.is_empty() && p.text.trim().is_empty()
    };
    if para_empty {
        let r = core
            .delete_paragraph_native(section, para)
            .expect("빈 문단 삭제 실패");
        println!("  빈 문단 {para} 삭제: {r}");
    }

    let out_bytes = core.export_hwp_with_adapter().expect("HWP 직렬화 실패");
    fs::write(&output, &out_bytes).expect("출력 파일 쓰기 실패");
    println!("출력: {output} ({} bytes)", out_bytes.len());
}
