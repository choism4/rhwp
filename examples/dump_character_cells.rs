// 본문 표의 character cell (인물명) char_shape 덤프.
// 사용: dump_character_cells <hwp>
use std::fs;
fn main() {
    let path = std::env::args().nth(1).expect("hwp");
    let data = fs::read(&path).expect("read");
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document();
    for (si, sec) in doc.sections.iter().enumerate() {
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            for ctrl in &para.controls {
                if let rhwp::model::control::Control::Table(t) = ctrl {
                    if t.row_count < 100 { continue; }  // body table only
                    let mut e_cells = 0;
                    let mut non_e_cells = 0;
                    let mut e_bold = 0;
                    for (ci, cell) in t.cells.iter().enumerate() {
                        if let Some(p) = cell.paragraphs.first() {
                            let has_e = p.text.contains("(E)");
                            if has_e {
                                e_cells += 1;
                                if let Some(cs) = p.char_shapes.first() {
                                    if let Some(sh) = doc.doc_info.char_shapes.get(cs.char_shape_id as usize) {
                                        if sh.bold { e_bold += 1; }
                                    }
                                }
                                if e_cells <= 3 {
                                    println!("(E) cell[{ci}] text='{}'", p.text);
                                    for cs in p.char_shapes.iter().take(3) {
                                        if let Some(sh) = doc.doc_info.char_shapes.get(cs.char_shape_id as usize) {
                                            println!("  cs.sp={} cs_id={} bold={} size={}", cs.start_pos, cs.char_shape_id, sh.bold, sh.base_size);
                                        }
                                    }
                                }
                            } else if !p.text.is_empty() {
                                non_e_cells += 1;
                            }
                        }
                    }
                    println!("\nsec{si} para{pi}: (E) cells {} (bold {}) / non-(E) text cells {}", e_cells, e_bold, non_e_cells);
                }
            }
        }
    }
}
