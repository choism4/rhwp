use std::fs;
fn main() {
    let path = std::env::args().nth(1).expect("hwp");
    let data = fs::read(&path).expect("read");
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document();
    for (si, sec) in doc.sections.iter().enumerate() {
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                if let rhwp::model::control::Control::Table(t) = ctrl {
                    if t.row_count != 16 || t.col_count != 6 { continue; }
                    println!("=== sec{si} para{pi} ctrl{ci} ===");
                    for (idx, cell) in t.cells.iter().enumerate() {
                        // 모든 col 1·4 (place cells) 출력 — wrap 검사
                        if !(cell.col == 1 || cell.col == 4) { continue; }
                        for p in &cell.paragraphs {
                            let text: String = p.text.chars().take(40).collect();
                            print!("r={} c={} idx={} text='{text}' lines={} ",
                                cell.row, cell.col, idx, p.line_segs.len());
                            for cs in &p.char_shapes {
                                if let Some(c) = doc.doc_info.char_shapes.get(cs.char_shape_id as usize) {
                                    print!("[{}->id{} ratio={}] ", cs.start_pos, cs.char_shape_id, c.ratios[0]);
                                }
                            }
                            println!();
                        }
                    }
                    return;
                }
            }
        }
    }
}
