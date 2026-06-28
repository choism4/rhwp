// Body table row 0~1 char_shape dump (header rows).
use std::fs;
fn main() {
    let path = std::env::args().nth(1).expect("hwp");
    let data = fs::read(&path).expect("read");
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document();
    for (si, sec) in doc.sections.iter().enumerate() {
        if si == 0 { continue; } // body in sec1+
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                if let rhwp::model::control::Control::Table(t) = ctrl {
                    if t.col_count != 4 { continue; }
                    println!("=== sec{si} para{pi} ctrl{ci} {}x{} ===", t.row_count, t.col_count);
                    for cell in t.cells.iter().filter(|c| c.row <= 1) {
                        let row = cell.row; let col = cell.col;
                        for p in &cell.paragraphs {
                            let text: String = p.text.chars().take(20).collect();
                            print!("  r{row} c{col} text='{text}' char_shapes:");
                            for cs in &p.char_shapes {
                                if let Some(c) = doc.doc_info.char_shapes.get(cs.char_shape_id as usize) {
                                    print!(" [{}->id{} size={} bold={} italic={} face={}]",
                                        cs.start_pos, cs.char_shape_id, c.base_size, c.bold, c.italic,
                                        doc.doc_info.font_faces.first().and_then(|fs| fs.get(c.font_ids[0] as usize)).map(|f| f.name.as_str()).unwrap_or("?"));
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
