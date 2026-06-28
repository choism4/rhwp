//! HWP → HWPX 변환 (네이티브). WASM `exportHwpx` 와 동일 직렬화 경로(`export_hwpx_native`).
//!
//! 용도: HWPX masterPage 파서 자가검증. script-ai/WASM 파이프라인 없이
//! 순수 cargo 로 fresh.hwpx 를 생성한다.
//!
//! 사용법: cargo run --release --example hwp_to_hwpx -- <입력.hwp> <출력.hwpx>

use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("사용법: hwp_to_hwpx <입력.hwp> <출력.hwpx>");
        std::process::exit(2);
    }
    let input = &args[1];
    let output = &args[2];

    let data = fs::read(input).unwrap_or_else(|e| {
        eprintln!("입력 읽기 실패 {}: {}", input, e);
        std::process::exit(1);
    });

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&data).unwrap_or_else(|e| {
        eprintln!("HWP 파싱 실패: {}", e);
        std::process::exit(1);
    });

    let hwpx = doc.export_hwpx_native().unwrap_or_else(|e| {
        eprintln!("HWPX 직렬화 실패: {}", e);
        std::process::exit(1);
    });

    fs::write(output, &hwpx).unwrap_or_else(|e| {
        eprintln!("출력 쓰기 실패 {}: {}", output, e);
        std::process::exit(1);
    });
    println!("저장 완료: {} ({} bytes)", output, hwpx.len());
}
