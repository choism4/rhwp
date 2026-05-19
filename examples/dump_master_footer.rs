// 바탕쪽 footer 글상자 위치·내용·char_shape 진단 — 섹션·master·shape 재귀 출력.
// 사용: dump_master_footer <입력.hwp>
use std::fs;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::ShapeObject;

fn dump_char_shapes(para: &Paragraph, indent: &str, doc: &Document) {
    for cs in &para.char_shapes {
        let id = cs.char_shape_id as usize;
        if let Some(shape) = doc.doc_info.char_shapes.get(id) {
            let font = doc.doc_info.font_faces.first()
                .and_then(|faces| faces.get(shape.font_ids[0] as usize))
                .map(|f| f.name.as_str())
                .unwrap_or("?");
            println!(
                "{}  charshape@{} id={} bold={} italic={} base_size={} font={:?}",
                indent, cs.start_pos, id, shape.bold, shape.italic, shape.base_size, font
            );
        }
    }
}

fn walk_shape(s: &ShapeObject, indent: &str, doc: &Document) {
    let c = s.common();
    let kind = match s {
        ShapeObject::Picture(_) => "PICTURE",
        ShapeObject::Group(_) => "GROUP",
        ShapeObject::Rectangle(_) => "RECT",
        ShapeObject::Line(_) => "LINE",
        ShapeObject::Ole(_) => "OLE",
        _ => "SHAPE",
    };
    println!("{}{} v_off={} h_off={} h={} w={}",
        indent, kind, c.vertical_offset, c.horizontal_offset, c.height, c.width);
    if let Some(d) = s.drawing() {
        let sa = &d.shape_attr;
        println!(
            "{}  xform rot={} flip=({},{}) matrix=[sx={} b={} tx={} c={} sy={} ty={}]",
            indent, sa.rotation_angle, sa.horz_flip, sa.vert_flip,
            sa.render_sx, sa.render_b, sa.render_tx, sa.render_c, sa.render_sy, sa.render_ty
        );
    }
    if let ShapeObject::Group(g) = s {
        for child in &g.children {
            walk_shape(child, &format!("{}  ", indent), doc);
        }
    }
    if let Some(d) = s.drawing() {
        if let Some(tb) = &d.text_box {
            for (pi, para) in tb.paragraphs.iter().enumerate() {
                let txt: String = para.text.chars().take(40).collect();
                if !txt.trim().is_empty() || para.text.chars().any(|c| (c as u32) < 0x20) {
                    let codes: Vec<String> = para.text.chars().take(24)
                        .map(|c| format!("{:04X}", c as u32)).collect();
                    println!("{}  tb.p{} text={:?} codes=[{}]", indent, pi, txt, codes.join(" "));
                    dump_char_shapes(para, indent, doc);
                }
                for ctrl in &para.controls {
                    match ctrl {
                        Control::Shape(inner) => walk_shape(inner, &format!("{}  tb.", indent), doc),
                        Control::Picture(p) => println!(
                            "{}  tb.PICTURE-CTRL bin_data_id={} h={} w={}",
                            indent, p.image_attr.bin_data_id, p.common.height, p.common.width),
                        Control::AutoNumber(an) => println!(
                            "{}  tb.AUTONUMBER type={:?} format={}", indent, an.number_type, an.format),
                        other => println!("{}  tb.CTRL {:?}", indent, std::mem::discriminant(other)),
                    }
                }
            }
        }
    }
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() != 1 {
        eprintln!("사용: dump_master_footer <입력.hwp>");
        std::process::exit(2);
    }
    let data = fs::read(&a[0]).expect("입력 읽기 실패");
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("파싱 실패");
    let doc = core.document();
    for (si, sec) in doc.sections.iter().enumerate() {
        let mps = &sec.section_def.master_pages;
        println!("=== sec{} master_pages={} ===", si, mps.len());
        for (mi, mp) in mps.iter().enumerate() {
            println!("  mp{} apply={:?} is_ext={} overlap={}",
                mi, mp.apply_to, mp.is_extension, mp.overlap);
            for (pi, para) in mp.paragraphs.iter().enumerate() {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if let Control::Shape(s) = ctrl {
                        walk_shape(s, &format!("    p{}c{} ", pi, ci), doc);
                    }
                }
            }
        }
    }
}
