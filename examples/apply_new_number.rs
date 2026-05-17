// 지정 구역·문단 첫머리에 NewNumber(쪽번호 재시작) 컨트롤 삽입 후 HWP 재출력.
// front matter(MEMO/씬구성표) 쪽수가 본문 쪽번호에 합산돼 본문이 5쪽 등으로
// 시작하는 것을 막고 본문을 1쪽부터 세게 한다.
// 사용: apply_new_number <입력> <출력> <section> <para> <number>
use std::fs;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() != 5 {
        eprintln!("사용: apply_new_number <입력> <출력> <section> <para> <number>");
        std::process::exit(2);
    }
    let (input, output) = (&a[0], &a[1]);
    let section: usize = a[2].parse().expect("section 정수");
    let para: usize = a[3].parse().expect("para 정수");
    let number: u16 = a[4].parse().expect("number 정수");

    let data = fs::read(input).expect("입력 읽기 실패");
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("파싱 실패");
    let r = core
        .add_new_number_native(section, para, 0, number)
        .expect("NewNumber 삽입 실패");
    println!("add_new_number({section},{para},0,{number}): {r}");
    let out = core.export_hwp_with_adapter().expect("직렬화 실패");
    fs::write(output, &out).expect("출력 쓰기 실패");
    println!("출력: {output} ({} bytes)", out.len());
}
