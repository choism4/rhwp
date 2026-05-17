// 편집틀 정리: 바탕쪽 잔상 그림 무력화 + 씬구성표 page-break 고정 + 제작진 표 그림·빈 문단 제거.
// 씬구성표 표는 1페이지에 빠듯해, 앞 콘텐츠를 지우면 페이지 중간서 시작→행 넘침→분할된다.
// 그래서 씬구성표 문단에 page_break_before 를 먼저 걸어 항상 새 페이지서 시작하게 한 뒤
// 제작진 그림·빈 문단을 제거한다.
//
// 사용: clean_template <입력> <출력> <doodleBin> <sceneTablePara> <staffPara>
use std::fs;
use rhwp::model::control::Control;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() != 5 {
        eprintln!("사용: clean_template <입력> <출력> <doodleBin> <sceneTablePara> <staffPara>");
        std::process::exit(2);
    }
    let (input, output) = (&a[0], &a[1]);
    let doodle_bin: u16 = a[2].parse().expect("doodleBin 정수");
    let scene_para: usize = a[3].parse().expect("sceneTablePara 정수");
    let staff_para: usize = a[4].parse().expect("staffPara 정수");

    let data = fs::read(input).expect("입력 읽기 실패");
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("파싱 실패");

    // 1) 도들 잔상 무력화.
    println!("doodle: {}", core.blank_bin_data_image(doodle_bin).expect("blank 실패"));

    // 2) 씬구성표 문단에 page_break_before — 제작진 제거 후에도 새 페이지서 시작 보장.
    println!(
        "page_break_before(0,{scene_para}): {}",
        core.apply_para_format_native(0, scene_para, "{\"pageBreakBefore\":true}")
            .expect("apply_para_format 실패")
    );

    // 3) 제작진 표 그림 control 전부 제거 (내림차순).
    let mut pics: Vec<usize> = {
        let p = &core.document().sections[0].paragraphs[staff_para];
        p.controls.iter().enumerate()
            .filter(|(_, c)| matches!(c, Control::Picture(_)))
            .map(|(i, _)| i).collect()
    };
    pics.sort_unstable_by(|x, y| y.cmp(x));
    for idx in pics {
        println!(
            "  staff pic ctrl[{idx}] 삭제: {}",
            core.delete_picture_control_native(0, staff_para, idx).expect("그림 삭제 실패")
        );
    }

    // 4) 비게 된 제작진 문단 삭제 (빈 페이지 방지). 씬구성표는 page_break_before 로 보호됨.
    let empty = {
        let p = &core.document().sections[0].paragraphs[staff_para];
        p.controls.is_empty() && p.text.trim().is_empty()
    };
    if empty {
        println!(
            "  빈 제작진 문단 {staff_para} 삭제: {}",
            core.delete_paragraph_native(0, staff_para).expect("문단 삭제 실패")
        );
    }

    let out = core.export_hwp_with_adapter().expect("직렬화 실패");
    fs::write(output, &out).expect("출력 쓰기 실패");
    println!("출력: {output} ({} bytes)", out.len());
}
