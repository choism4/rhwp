// 씬구성표 cell vertical_align dump.
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
                    // 첫 씬구성표 표만.
                    let mut shown = 0usize;
                    for (idx, cell) in t.cells.iter().enumerate() {
                        if cell.row > 2 || shown > 18 { continue; }
                        let text = cell.paragraphs.first().map(|p| p.text.clone()).unwrap_or_default();
                        println!("sec{si} p{pi} ctrl{ci} cell[{idx}] r={} c={} valign={:?} h={} w={} text='{}'",
                            cell.row, cell.col, cell.vertical_align, cell.height, cell.width, text);
                        shown += 1;
                    }
                    return;
                }
            }
        }
    }
}
