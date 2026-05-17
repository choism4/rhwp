// 편집 0건 round-trip: HWP 를 load → 그대로 export. 직렬화 라운드트립 충실성 진단용.
// 사용: cargo run --release --example roundtrip_hwp -- <입력.hwp> <출력.hwp>
use std::fs;

fn main() {
    let mut args = std::env::args().skip(1);
    let input = args.next().expect("사용: roundtrip_hwp <입력.hwp> <출력.hwp>");
    let output = args.next().expect("출력 경로 필요");
    let data = fs::read(&input).expect("입력 읽기 실패");
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("파싱 실패");
    let out = core.export_hwp_with_adapter().expect("직렬화 실패");
    fs::write(&output, &out).expect("출력 쓰기 실패");
    println!("입력 {} bytes → 출력 {} bytes", data.len(), out.len());
}
