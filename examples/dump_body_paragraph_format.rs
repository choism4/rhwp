// 본문 표 셀 paragraph 의 ParaShape + CharShape attribute 덤프.
// 사용: dump_body_paragraph_format <hwp>
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
                    if t.row_count < 3 { continue; }
                    println!("sec{} para{} table {}x{} cells={}", si, pi, t.row_count, t.col_count, t.cells.len());
                    let mut shown = 0usize;
                    for (ci, cell) in t.cells.iter().enumerate() {
                        if cell.row < 2 || shown >= 5 { continue; }
                        if let Some(p) = cell.paragraphs.first() {
                            let ps_id = p.para_shape_id;
                            let ps = doc.doc_info.para_shapes.get(ps_id as usize);
                            let tx: String = p.text.chars().take(20).collect();
                            println!("  cell[{}] row={} col={} text='{}'", ci, cell.row, cell.col, tx);
                            if let Some(s) = ps {
                                println!("    ps[{}] align={:?} line_sp={} type={:?} m_l={} m_r={} ind={} sb={} sa={}",
                                    ps_id, s.alignment, s.line_spacing, s.line_spacing_type,
                                    s.margin_left, s.margin_right, s.indent, s.spacing_before, s.spacing_after);
                            }
                            for cs in p.char_shapes.iter().take(2) {
                                if let Some(sh) = doc.doc_info.char_shapes.get(cs.char_shape_id as usize) {
                                    let font = doc.doc_info.font_faces.first()
                                        .and_then(|f| f.get(sh.font_ids[0] as usize))
                                        .map(|f| f.name.clone()).unwrap_or_default();
                                    println!("    cs.sp={} cs_id={} font='{}' size={} bold={} ratios={:?}",
                                        cs.start_pos, cs.char_shape_id, font, sh.base_size, sh.bold, sh.ratios);
                                }
                            }
                            shown += 1;
                        }
                    }
                }
            }
        }
    }
}
