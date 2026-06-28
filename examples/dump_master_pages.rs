//! round-trip 검증: HWPX 를 파싱해 각 섹션의 master_pages 가 IR 에 채워졌는지 덤프.
//!
//! 사용법: cargo run --release --example dump_master_pages -- <파일.hwpx>

use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("사용법: dump_master_pages <파일.hwpx>");
        std::process::exit(2);
    }
    let data = fs::read(&args[1]).expect("read input");
    let doc = rhwp::parser::hwpx::parse_hwpx(&data).expect("parse hwpx");

    let mut total_mp = 0usize;
    for (si, sec) in doc.sections.iter().enumerate() {
        let mps = &sec.section_def.master_pages;
        println!("섹션 {}: master_pages={}", si, mps.len());
        for (mi, mp) in mps.iter().enumerate() {
            total_mp += 1;
            println!(
                "  [{}] apply_to={:?} is_ext={} overlap={} paras={} tw={} th={}",
                mi,
                mp.apply_to,
                mp.is_extension,
                mp.overlap,
                mp.paragraphs.len(),
                mp.text_width,
                mp.text_height,
            );
        }
    }
    println!("총 master_pages={}", total_mp);
}
