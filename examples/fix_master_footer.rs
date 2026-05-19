// 정본 골격 템플릿 정규화 — 두 가지 결함을 고쳐 HWP 재출력한다.
// (1) 바탕쪽 ext_flags 의 overlap·is_extension 비트를 클리어. 웹 한컴독스가
//     정규 바탕쪽을 확장 바탕쪽으로 잘못 표시해 홀수 본문 페이지에서
//     footer(쪽번호 글상자)가 소실되는 회귀를 고친다.
// (2) BinData 이미지 전부를 흰 1×1 로 무력화. 골격 템플릿에 남은 손글씨
//     낙서("아빠 딸" 등) 잔상을 제거한다 — 골격엔 정상 이미지가 없다.
// 사용: fix_master_footer <입력.hwp> <출력.hwp>
use std::fs;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() != 2 {
        eprintln!("사용: fix_master_footer <입력.hwp> <출력.hwp>");
        std::process::exit(2);
    }
    let data = fs::read(&a[0]).expect("입력 읽기 실패");
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("파싱 실패");
    let n = core.document().sections.len();
    for sec in 0..n {
        let r = core.clear_master_page_ext_flags(sec).expect("패치 실패");
        println!("sec{sec}: {r}");
    }
    let bin_count = core.document().doc_info.bin_data_list.len();
    for id in 1..=bin_count as u16 {
        let r = core.blank_bin_data_image(id).expect("BinData 무력화 실패");
        println!("bin{id}: {r}");
    }
    let out = core.export_hwp_with_adapter().expect("직렬화 실패");
    fs::write(&a[1], &out).expect("출력 쓰기 실패");
    println!("출력: {} ({} bytes)", a[1], out.len());
}
