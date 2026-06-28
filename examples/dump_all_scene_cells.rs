// body table 의 모든 scene heading cell (text 시작 S#) height 덤프.
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
                    if t.row_count < 100 { continue; }
                    let mut shown = 0usize;
                    for (ci, cell) in t.cells.iter().enumerate() {
                        if let Some(p) = cell.paragraphs.first() {
                            if p.text.starts_with("S#") && shown < 5 {
                                println!("sec{} para{} cell[{}] row={} col={} w={} h={} text='{}'",
                                    si, pi, ci, cell.row, cell.col, cell.width, cell.height, p.text);
                                shown += 1;
                            }
                        }
                    }
                    return;
                }
            }
        }
    }
}
