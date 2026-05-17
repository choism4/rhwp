// 바탕쪽(master page) 잔상 그림 무력화 — 지정 BinData 그림을 1×1 흰 이미지로 교체.
// 본문 페이지마다 반복되던 손글씨 잔상 제거용. control 구조 변경 없이 BinData 내용만 교체.
// 사용: cargo run --release --example strip_master_picture -- <입력.hwp> <출력.hwp> <bin_data_id>
use std::fs;

fn main() {
    let mut args = std::env::args().skip(1);
    let input = args.next().expect("사용: strip_master_picture <입력.hwp> <출력.hwp> <bin_data_id>");
    let output = args.next().expect("출력 경로 필요");
    let bin_id: u16 = args.next().expect("bin_data_id 필요").parse().expect("bin_data_id 는 정수");

    let data = fs::read(&input).expect("입력 파일 읽기 실패");
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("파싱 실패");
    let result = core.blank_bin_data_image(bin_id).expect("BinData 무력화 실패");
    println!("blank_bin_data_image({bin_id}): {result}");

    let out_bytes = core.export_hwp_with_adapter().expect("HWP 직렬화 실패");
    fs::write(&output, &out_bytes).expect("출력 파일 쓰기 실패");
    println!("출력: {output} ({} bytes)", out_bytes.len());
}
