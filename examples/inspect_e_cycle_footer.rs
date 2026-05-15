//! E cycle 진단 — MBC baseline footer 의 paragraph + controls 트리 dump
//! 목적: AutoNumber 마커의 정확 위치 (어느 paragraph.controls 인지) 확인
//!
//! 실행: cargo run --release --example inspect_e_cycle_footer

use rhwp::model::control::Control;
use rhwp::model::header_footer::MasterPage;
use rhwp::model::paragraph::Paragraph;
use std::fs;

struct Case {
    label: &'static str,
    path: &'static str,
    sim_from: &'static str,
    sim_to: &'static str,
}

fn main() {
    let cases = [
        Case {
            label: "KBS",
            path: "../전달용_원문 및 편집틀 V2/KBS/편집본/어쩌다 마주친, 그대 1회(편집본).hwp",
            sim_from: "어쩌다 마주친, 그대  제 1 회",
            sim_to: "어쩌다 마주친, 그대 제 1 회",
        },
        Case {
            label: "MBC",
            path: "../전달용_원문 및 편집틀 V2/MBC/편집본/일단 뜨겁게 청소하라 1회(편집본).hwp",
            sim_from: "-   -   일단 뜨겁게 청소하라  제 1 부",
            sim_to: "일단 뜨겁게 청소하라 제 1 부",
        },
        Case {
            label: "SBS",
            path: "../전달용_원문 및 편집틀 V2/SBS/편집본/으라차차 와이키키 1회(편집본).hwp",
            sim_from: "-   -               으라차차 와이키키  제 1 회",
            sim_to: "으라차차 와이키키 제 1 회",
        },
    ];
    // E cycle: 우리 pipeline 변환 결과 edited.hwp 직접 검사
    println!("\n########## 변환 결과 edited.hwp 검사 ##########");
    if let Ok(data) = fs::read("/tmp/mbc-edited.hwp") {
        if let Ok(core) = rhwp::document_core::DocumentCore::from_bytes(&data) {
            dump_section("MBC-EDITED", "EDITED", core.document(), 1);
        } else {
            println!("edited.hwp parse fail");
        }
    } else {
        println!("/tmp/mbc-edited.hwp not found");
    }

    for case in &cases {
        let Case { label, path, sim_from, sim_to } = case;
        println!("\n========== {} {} ==========", label, path);
        let data = match fs::read(path) {
            Ok(d) => d,
            Err(e) => { println!("read fail: {}", e); continue; }
        };
        let mut core = match rhwp::document_core::DocumentCore::from_bytes(&data) {
            Ok(c) => c,
            Err(e) => { println!("parse fail: {}", e); continue; }
        };
        println!("\n>>> BEFORE replace_text_in_master_pages_native <<<");
        dump_section(label, "BEFORE", core.document(), 1);

        // simulate replace_text_in_master_pages_native(1, sim_from, sim_to)
        match core.replace_text_in_master_pages_native(1, sim_from, sim_to) {
            Ok(r) => println!("\nreplace_text_in_master_pages_native(1, \"{}\", \"{}\") -> {}", sim_from, sim_to, r),
            Err(e) => println!("\nreplace error: {}", e),
        }

        println!("\n>>> AFTER replace_text_in_master_pages_native <<<");
        dump_section(label, "AFTER", core.document(), 1);

        // round-trip 검증: export_hwp → re-parse → footer 확인
        println!("\n>>> ROUND-TRIP: export_hwp_native → re-parse <<<");
        match core.export_hwp_native() {
            Ok(bytes) => {
                println!("export_hwp_native -> {} bytes", bytes.len());
                match rhwp::document_core::DocumentCore::from_bytes(&bytes) {
                    Ok(rc) => dump_section(label, "ROUNDTRIP", rc.document(), 1),
                    Err(e) => println!("re-parse fail: {}", e),
                }
            }
            Err(e) => println!("export_hwp_native fail: {:?}", e),
        }
    }
}

fn dump_section(label: &str, tag: &str, doc: &rhwp::model::document::Document, sec_idx: usize) {
    let Some(section) = doc.sections.get(sec_idx) else { println!("no section {}", sec_idx); return };
    let masters = &section.section_def.master_pages;
    println!("--- {} section {} master_pages count={} ---", tag, sec_idx, masters.len());
    for (mi, master) in masters.iter().enumerate() {
        dump_master(label, sec_idx, mi, master);
    }
}

fn dump_master(label: &str, sec: usize, mi: usize, master: &MasterPage) {
    println!("[{}] section[{}].master_pages[{}] paragraphs={}", label, sec, mi, master.paragraphs.len());
    for (pi, para) in master.paragraphs.iter().enumerate() {
        dump_para_recursive(label, &format!("master[{}].p[{}]", mi, pi), para, 0);
    }
}

fn dump_para_recursive(label: &str, path: &str, para: &Paragraph, depth: usize) {
    let indent = "  ".repeat(depth);
    let has_autonum = para.controls.iter().any(|c| matches!(c, Control::AutoNumber(_)));
    let text_esc: String = para.text.chars().map(|c| {
        if c == '\u{0012}' { "\\u{0012}".to_string() }
        else if (c as u32) < 0x20 { format!("\\x{:02x}", c as u32) }
        else { c.to_string() }
    }).collect();
    println!("{}[{}] {} has_autonum={} text=\"{}\" ({} chars, controls={})",
        indent, label, path, has_autonum, text_esc, para.text.chars().count(), para.controls.len());
    // char_shapes dump — CharShapeRef 경계로 run 분리 가설 검증
    if !para.text.is_empty() && para.text.chars().count() < 60 {
        let cs_segs: Vec<String> = para.char_shapes.iter()
            .map(|cs| format!("(start={},id={})", cs.start_pos, cs.char_shape_id))
            .collect();
        println!("{}    char_shapes: [{}] count={}", indent, cs_segs.join(", "), para.char_shapes.len());
        // line_segs dump — wrap 여부 + vertical_pos 확인
        let ls_segs: Vec<String> = para.line_segs.iter()
            .map(|ls| format!("(text_start={},vpos={},lh={},seg_w={})", ls.text_start, ls.vertical_pos, ls.line_height, ls.segment_width))
            .collect();
        println!("{}    line_segs: [{}] count={}", indent, ls_segs.join(", "), para.line_segs.len());
        println!("{}    char_count={} char_offsets.len={}", indent, para.char_count, para.char_offsets.len());
        // run boundary 시뮬레이션: 같은 char_shape_id 의 인접 구간을 grouping
        if para.char_shapes.len() > 1 {
            let chars: Vec<char> = para.text.chars().collect();
            for (i, cs) in para.char_shapes.iter().enumerate() {
                let end = para.char_shapes.get(i + 1).map(|n| n.start_pos as usize).unwrap_or(chars.len());
                let start = cs.start_pos as usize;
                let segment: String = chars[start.min(chars.len())..end.min(chars.len())].iter().collect();
                let seg_esc: String = segment.chars().map(|c| {
                    if c == '\u{0012}' { "\\u{0012}".to_string() }
                    else if (c as u32) < 0x20 { format!("\\x{:02x}", c as u32) }
                    else { c.to_string() }
                }).collect();
                println!("{}      run[{}] cs_id={} chars={}..{}: \"{}\"",
                    indent, i, cs.char_shape_id, start, end, seg_esc);
            }
        }
    }
    for (ci, ctrl) in para.controls.iter().enumerate() {
        let cname = match ctrl {
            Control::AutoNumber(an) => format!("AutoNumber type={:?} format={} num={}", an.number_type, an.format, an.assigned_number),
            Control::Footer(_) => "Footer".to_string(),
            Control::Header(_) => "Header".to_string(),
            Control::Table(_) => "Table".to_string(),
            Control::Shape(_) => "Shape".to_string(),
            Control::SectionDef(_) => "SectionDef".to_string(),
            _ => format!("{:?}", std::mem::discriminant(ctrl)),
        };
        println!("{}  ctrl[{}]: {}", indent, ci, cname);
        match ctrl {
            Control::Shape(shape) => {
                let w = shape.as_ref().common().width;
                let h = shape.as_ref().common().height;
                println!("{}    shape common: width={} height={} hwpunit", "  ".repeat(depth), w, h);
                dump_shape_object_recursive(label, &format!("{}.shape[{}]", path, ci), shape.as_ref(), depth + 2);
            }
            Control::Footer(f) => {
                for (ipi, ipara) in f.paragraphs.iter().enumerate() {
                    dump_para_recursive(label, &format!("{}.footer[{}].p[{}]", path, ci, ipi), ipara, depth + 2);
                }
            }
            Control::Header(h) => {
                for (ipi, ipara) in h.paragraphs.iter().enumerate() {
                    dump_para_recursive(label, &format!("{}.header[{}].p[{}]", path, ci, ipi), ipara, depth + 2);
                }
            }
            Control::Table(t) => {
                for (cell_idx, cell) in t.cells.iter().enumerate() {
                    for (cpi, cp) in cell.paragraphs.iter().enumerate() {
                        dump_para_recursive(label, &format!("{}.tbl[{}].cell[{}].p[{}]", path, ci, cell_idx, cpi), cp, depth + 2);
                    }
                }
            }
            _ => {}
        }
    }
}

fn dump_shape_object_recursive(label: &str, path: &str, obj: &rhwp::model::shape::ShapeObject, depth: usize) {
    use rhwp::model::shape::ShapeObject as SO;
    let indent = "  ".repeat(depth);
    println!("{}[{}] {} {}", indent, label, path, obj.shape_name());
    if let Some(drawing) = obj.drawing() {
        if let Some(tb) = drawing.text_box.as_ref() {
            for (ipi, ipara) in tb.paragraphs.iter().enumerate() {
                dump_para_recursive(label, &format!("{}.tb.p[{}]", path, ipi), ipara, depth + 1);
            }
        }
    }
    if let SO::Group(g) = obj {
        for (i, child) in g.children.iter().enumerate() {
            dump_shape_object_recursive(label, &format!("{}.group.child[{}]", path, i), child, depth + 1);
        }
    }
}
