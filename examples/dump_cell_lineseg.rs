// 씬구성표 cell paragraph LineSeg vertical_pos dump (전체 ls).
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
                    for (idx, cell) in t.cells.iter().enumerate() {
                        if cell.row > 1 { continue; }  // 헤더+1행
                        for (pidx, p) in cell.paragraphs.iter().enumerate() {
                            let text: String = p.text.chars().take(15).collect();
                            print!("sec{si} para{pi} ctrl{ci} cell[{idx}] r={} c={} valign={:?} pi={pidx} text='{text}' ls=",
                                cell.row, cell.col, cell.vertical_align);
                            for ls in &p.line_segs {
                                print!("{{vpos={},lh={}}}", ls.vertical_pos, ls.line_height);
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
