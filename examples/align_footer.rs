// 바탕쪽 footer 쪽번호 글상자를 작품명 글상자에 세로 정렬 후 HWP 재출력.
// 사용: align_footer <입력> <출력> <section>
use std::fs;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() != 3 {
        eprintln!("사용: align_footer <입력> <출력> <section>");
        std::process::exit(2);
    }
    let section: usize = a[2].parse().expect("section 정수");
    let data = fs::read(&a[0]).expect("입력 읽기 실패");
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("파싱 실패");
    let r = core.align_master_footer_voffset(section).expect("정렬 실패");
    println!("align_master_footer_voffset({section}): {r}");
    let out = core.export_hwp_with_adapter().expect("직렬화 실패");
    fs::write(&a[1], &out).expect("출력 쓰기 실패");
    println!("출력: {} ({} bytes)", a[1], out.len());
}
