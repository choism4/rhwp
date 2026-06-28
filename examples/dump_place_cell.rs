// 씬구성표 r=1 c=1 셀 (장 소) 의 char_shape ratios 확인.
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
                        if !(cell.row == 1 && (cell.col == 1 || cell.col == 4)) { continue; }
                        for p in &cell.paragraphs {
                            let text: String = p.text.chars().take(30).collect();
                            println!("cell[{idx}] r={} c={} h={} w={} pad_l={} pad_r={} text='{text}'",
                                cell.row, cell.col, cell.height, cell.width,
                                cell.padding.left, cell.padding.right);
                            print!("  ls: ");
                            for ls in &p.line_segs {
                                print!("{{vpos={},lh={},sw={}}} ", ls.vertical_pos, ls.line_height, ls.segment_width);
                            }
                            println!();
                            print!("  char_shapes:");
                            for cs in &p.char_shapes {
                                print!(" [{}→{}]", cs.start_pos, cs.char_shape_id);
                            }
                            println!();
                            // resolve unique char_shape ids → print their ratios
                            use std::collections::HashSet;
                            let mut ids = HashSet::new();
                            for cs in &p.char_shapes { ids.insert(cs.char_shape_id); }
                            for id in ids {
                                if let Some(cs) = doc.doc_info.char_shapes.get(id as usize) {
                                    println!("  cs[{id}] size={} ratios={:?} font_ids={:?}",
                                        cs.base_size, cs.ratios, cs.font_ids);
                                }
                            }
                        }
                    }
                    return;
                }
            }
        }
    }
}
