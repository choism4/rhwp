// 지정 구역의 문단 [start..=end] 범위를 삭제하고 HWP 재출력.
// 편집틀 앞부분 잔여 front matter(빈 문단 등) 정리용. 내림차순 삭제로 인덱스 시프트 회피.
// 사용: delete_para_range <입력> <출력> <section> <start> <end>
use std::fs;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() != 5 {
        eprintln!("사용: delete_para_range <입력> <출력> <section> <start> <end>");
        std::process::exit(2);
    }
    let (input, output) = (&a[0], &a[1]);
    let section: usize = a[2].parse().expect("section 정수");
    let start: usize = a[3].parse().expect("start 정수");
    let end: usize = a[4].parse().expect("end 정수");

    let data = fs::read(input).expect("입력 읽기 실패");
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("파싱 실패");
    for idx in (start..=end).rev() {
        let r = core.delete_paragraph_native(section, idx).expect("문단 삭제 실패");
        println!("  문단 {section}.{idx} 삭제: {r}");
    }
    let out = core.export_hwp_with_adapter().expect("직렬화 실패");
    fs::write(output, &out).expect("출력 쓰기 실패");
    println!("출력: {output} ({} bytes)", out.len());
}
