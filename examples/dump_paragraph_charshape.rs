use std::fs;
fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("hwp");
    let sec: usize = args.next().expect("sec").parse().expect("int");
    let para_idx: usize = args.next().expect("para").parse().expect("int");
    let data = fs::read(&path).expect("read");
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document();
    let section = &doc.sections[sec];
    if para_idx >= section.paragraphs.len() { println!("para 범위 초과 (len={})", section.paragraphs.len()); return; }
    let para = &section.paragraphs[para_idx];
    println!("sec{}/para{} text='{}'", sec, para_idx, para.text.replace('\n', "/"));
    for (i, cs) in para.char_shapes.iter().enumerate() {
        let id = cs.char_shape_id as usize;
        if let Some(s) = doc.doc_info.char_shapes.get(id) {
            let font = doc.doc_info.font_faces.first()
                .and_then(|f| f.get(s.font_ids[0] as usize))
                .map(|f| f.name.clone()).unwrap_or_default();
            println!("  cs[{}] start_pos={} doc_cs[{}] font_ids={:?} font_kor='{}' bold={} size={}",
                i, cs.start_pos, id, s.font_ids, font, s.bold, s.base_size);
        }
    }
}
