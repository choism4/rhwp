// 본문 셀 run 의 char_shape font 참조 진단 (한컴 빈폰트 드리프트).
use std::fs;
fn main() {
    let path = std::env::args().nth(1).expect("usage: <hwp>");
    let data = fs::read(&path).expect("read");
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document();
    let faces = &doc.doc_info.font_faces[0];
    let name = |id: u16| faces.get(id as usize).map(|f| f.name.clone()).unwrap_or("?".into());
    // char_shapes 테이블: raw 여부 + font_ids
    println!("=== char_shapes (raw vs model, hangul font_id → name) ===");
    for (i, cs) in doc.doc_info.char_shapes.iter().enumerate() {
        let raw = if cs.raw_data.is_some() { "RAW" } else { "mdl" };
        let h = cs.font_ids.get(0).copied().unwrap_or(0);
        if (i<3 || (i>=50 && i<=54)) { println!("  cs[{i}] {raw} hangulFont={} (={}) size={}", h, name(h), cs.base_size); }
    }
    // 본문 표 찾기 + 대사 셀 run charPrId
    println!("=== sections/paragraph runs charShapeId (본문 표 셀 텍스트) ===");
    for (si, sec) in doc.sections.iter().enumerate() {
        for (pi, p) in sec.paragraphs.iter().enumerate() {
            for ctrl in &p.controls {
                if let rhwp::model::control::Control::Table(t) = ctrl {
                    for (ci, cell) in t.cells.iter().enumerate() {
                        for cp in &cell.paragraphs {
                            let txt: String = cp.text.chars().take(14).collect();
                            if txt.contains("대사") || txt.contains("순덕") || txt.contains("정우") || txt.contains("지문") {
                                let ids: Vec<String> = cp.char_shapes.iter().map(|cs| {
                                    let fid = doc.doc_info.char_shapes.get(cs.char_shape_id as usize).map(|c| c.font_ids.get(0).copied().unwrap_or(0)).unwrap_or(0);
                                    format!("csId={}→font{}({})", cs.char_shape_id, fid, name(fid))
                                }).collect();
                                println!("  s{si}p{pi} cell{ci} <{}> runs:[{}]", txt, ids.join(", "));
                            }
                        }
                    }
                }
            }
        }
    }
}
