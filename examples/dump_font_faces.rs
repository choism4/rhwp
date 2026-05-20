// DocInfo.font_faces + char_shapes 덤프 — master page 폰트 추적용.
// 사용: dump_font_faces <hwp>
use std::fs;
fn main() {
    let path = std::env::args().nth(1).expect("usage: dump_font_faces <hwp>");
    let data = fs::read(&path).expect("read");
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document();
    println!("=== font_faces[0] (Korean) ===");
    for (i, f) in doc.doc_info.font_faces[0].iter().enumerate() {
        println!("  [{i}] name='{}' alt={:?} default={:?}", f.name, f.alt_name, f.default_name);
    }
    println!("=== font_faces[1] (English) ===");
    for (i, f) in doc.doc_info.font_faces[1].iter().enumerate() {
        println!("  [{i}] name='{}'", f.name);
    }
    println!("=== doc_info.char_shapes (first 30) ===");
    for (i, cs) in doc.doc_info.char_shapes.iter().take(30).enumerate() {
        let raw = if cs.raw_data.is_some() { "raw" } else { "model" };
        println!("  [{i}] {raw} font_ids={:?} base_size={} bold={} ratios={:?}", cs.font_ids, cs.base_size, cs.bold, cs.ratios);
    }
}
