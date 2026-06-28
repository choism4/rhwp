// 씬구성표 헤더 row line_seg 모든 필드 dump.
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
                    println!("=== {} sec{si} para{pi} ctrl{ci} ===", path);
                    for (idx, cell) in t.cells.iter().enumerate() {
                        if cell.row > 0 { continue; }
                        for p in &cell.paragraphs {
                            let text: String = p.text.chars().take(8).collect();
                            println!("cell[{idx}] r={} c={} h={} w={} valign={:?} padT={} padB={} text='{text}'",
                                cell.row, cell.col, cell.height, cell.width, cell.vertical_align,
                                cell.padding.top, cell.padding.bottom);
                            for ls in &p.line_segs {
                                println!("  ls: vpos={} lh={} th={} bl={} ls_={} cs={} sw={} tag={:#x}",
                                    ls.vertical_pos, ls.line_height, ls.text_height,
                                    ls.baseline_distance, ls.line_spacing,
                                    ls.column_start, ls.segment_width, ls.tag);
                            }
                            // char_shapes
                            print!("  char_shapes:");
                            for cs in &p.char_shapes {
                                print!(" [{}->{}]", cs.start_pos, cs.char_shape_id);
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
