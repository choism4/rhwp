// 본문 표 (sec, paragraph, control) 의 모든 셀 첫 paragraph char_shape 덤프.
// 사용: dump_table_cell_charshape <hwp> <sec> <para_idx> <ctrl_idx>
use std::fs;
fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("hwp");
    let sec: usize = args.next().expect("sec").parse().expect("int");
    let para_idx: usize = args.next().expect("para").parse().expect("int");
    let ctrl_idx: usize = args.next().expect("ctrl").parse().expect("int");
    let data = fs::read(&path).expect("read");
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document();
    let para = &doc.sections[sec].paragraphs[para_idx];
    let ctrl = &para.controls[ctrl_idx];
    if let rhwp::model::control::Control::Table(t) = ctrl {
        println!("table {}x{} cells={}", t.row_count, t.col_count, t.cells.len());
        for (ci, cell) in t.cells.iter().enumerate() {
            let first = cell.paragraphs.first();
            if let Some(p) = first {
                println!("cell[{ci}] row={} col={} paras={} text='{}'", cell.row, cell.col, cell.paragraphs.len(), p.text.replace('\n', "/"));
                for (i, cs) in p.char_shapes.iter().enumerate() {
                    let id = cs.char_shape_id as usize;
                    if let Some(s) = doc.doc_info.char_shapes.get(id) {
                        let font = doc.doc_info.font_faces.first()
                            .and_then(|f| f.get(s.font_ids[0] as usize))
                            .map(|f| f.name.clone()).unwrap_or_default();
                        println!("  cs[{}] start={} doc_cs[{}] font='{}' bold={} size={}",
                            i, cs.start_pos, id, font, s.bold, s.base_size);
                    }
                }
            }
        }
    } else {
        println!("control[{ctrl_idx}] not Table");
    }
}
