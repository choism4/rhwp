// 진단: 바탕쪽(master page) footer 글상자들의 위치·크기·폰트 덤프.
// 쪽수 글상자 vs 작품명 글상자 수직정렬 불일치 원인 확인용.
// 사용: cargo run --release --example inspect_footer -- <파일.hwp>
use std::fs;
use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;
use rhwp::model::document::Document;

fn font_sizes(doc: &Document, para: &rhwp::model::paragraph::Paragraph) -> Vec<i32> {
    let mut v: Vec<i32> = para.char_shapes.iter()
        .filter_map(|r| doc.doc_info.char_shapes.get(r.char_shape_id as usize))
        .map(|cs| cs.base_size)
        .collect();
    v.dedup();
    v
}

fn dump_shape(doc: &Document, loc: &str, shape: &ShapeObject) {
    let c = shape.common();
    println!(
        "{loc} {} v_off={} h_off={} w={} h={} z={} vert_rel={:?}",
        shape.shape_name(), c.vertical_offset, c.horizontal_offset,
        c.width, c.height, c.z_order, c.vert_rel_to,
    );
    if let Some(d) = shape.drawing() {
        if let Some(tb) = &d.text_box {
            println!(
                "{loc}   TextBox valign={:?} margin(t/b)={}/{} paras={}",
                tb.vertical_align, tb.margin_top, tb.margin_bottom, tb.paragraphs.len(),
            );
            for (pi, p) in tb.paragraphs.iter().enumerate() {
                let txt: String = p.text.chars().take(40).collect();
                println!(
                    "{loc}     tb_p[{pi}] font_size={:?} text=\"{}\"",
                    font_sizes(doc, p), txt.replace('\n', "/"),
                );
            }
        }
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("사용: inspect_footer <파일.hwp>");
    let data = fs::read(&path).expect("읽기 실패");
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("파싱 실패");
    let doc = core.document();
    for (si, section) in doc.sections.iter().enumerate() {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                if let Control::SectionDef(sd) = ctrl {
                    for (mi, mp) in sd.master_pages.iter().enumerate() {
                        for (mpi, mpara) in mp.paragraphs.iter().enumerate() {
                            for (xi, c) in mpara.controls.iter().enumerate() {
                                let loc = format!(
                                    "s{si} 바탕쪽[{mi}]{:?} 문단{mpi} ctrl[{xi}]",
                                    mp.apply_to,
                                );
                                if let Control::Shape(shape) = c {
                                    dump_shape(doc, &loc, shape);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
