// CharShape 테이블 dump.
use std::fs;
fn main() {
    let path = std::env::args().nth(1).expect("hwp");
    let ids_filter: Option<Vec<u32>> = std::env::args().nth(2).map(|s|
        s.split(',').filter_map(|x| x.parse().ok()).collect()
    );
    let data = fs::read(&path).expect("read");
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document();
    println!("=== {} char_shapes (total {}) ===", path, doc.doc_info.char_shapes.len());
    for (i, cs) in doc.doc_info.char_shapes.iter().enumerate() {
        if let Some(ref ids) = ids_filter {
            if !ids.contains(&(i as u32)) { continue; }
        }
        println!("[{i}] base_size={} bold={} italic={} font_ids={:?} ratios={:?} rel_sizes={:?}",
            cs.base_size, cs.bold, cs.italic, cs.font_ids, cs.ratios, cs.relative_sizes);
    }
}
