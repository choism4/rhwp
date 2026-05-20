// char_shape ID 사용처 — 본문/master/footer 별 카운트.
// 사용: dump_char_shape_usage <hwp> <cs_id1> [cs_id2 ...]
use std::collections::HashMap;
use std::fs;
fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("usage: dump_char_shape_usage <hwp> <cs_id> [cs_id ...]");
    let target_ids: Vec<u32> = args.map(|s| s.parse().expect("cs id int")).collect();
    let data = fs::read(&path).expect("read");
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document();
    let mut counts: HashMap<u32, [u32; 3]> = HashMap::new(); // [body, master, table_cell]
    for (si, sec) in doc.sections.iter().enumerate() {
        // body paragraphs
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            for cs in &para.char_shapes {
                if target_ids.contains(&cs.char_shape_id) {
                    let entry = counts.entry(cs.char_shape_id).or_insert([0; 3]);
                    entry[0] += 1;
                }
            }
            for ctrl in &para.controls {
                if let rhwp::model::control::Control::Table(t) = ctrl {
                    for cell in &t.cells {
                        for p in &cell.paragraphs {
                            for cs in &p.char_shapes {
                                if target_ids.contains(&cs.char_shape_id) {
                                    let entry = counts.entry(cs.char_shape_id).or_insert([0; 3]);
                                    entry[2] += 1;
                                }
                            }
                        }
                    }
                }
            }
            let _ = (si, pi);
        }
        // master pages
        for mp in &sec.section_def.master_pages {
            for para in &mp.paragraphs {
                for cs in &para.char_shapes {
                    if target_ids.contains(&cs.char_shape_id) {
                        let entry = counts.entry(cs.char_shape_id).or_insert([0; 3]);
                        entry[1] += 1;
                    }
                }
            }
        }
    }
    println!("char_shape_id [body, master, table_cell]");
    for id in &target_ids {
        let c = counts.get(id).copied().unwrap_or([0; 3]);
        println!("  cs={id}: body={} master={} cell={}", c[0], c[1], c[2]);
    }
}
