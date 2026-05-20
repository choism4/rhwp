// char_shape 의 한글·영문 font_id 를 변경 (raw_data 무효화 후 model 직렬화).
// 향후 폰트 업로드/변경 기능 백엔드 패치 기반.
//
// 사용: patch_char_shape_font <in.hwp> <out.hwp> <cs_id> <new_font_id_korean> <new_font_id_english>
//
// 변경되는 char_shape:
//   font_ids[0] = korean, font_ids[1] = english.
//   raw_data = None → export 시 model 필드 사용.
use std::fs;

fn main() {
    let mut args = std::env::args().skip(1);
    let input = args.next().expect("input.hwp");
    let output = args.next().expect("output.hwp");
    let cs_id: usize = args.next().expect("cs_id").parse().expect("int");
    let kor_id: u16 = args.next().expect("kor font id").parse().expect("u16");
    let eng_id: u16 = args.next().expect("eng font id").parse().expect("u16");

    let data = fs::read(&input).expect("read");
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document_mut();
    if cs_id >= doc.doc_info.char_shapes.len() {
        eprintln!("cs_id {} 범위 초과 (총 {})", cs_id, doc.doc_info.char_shapes.len());
        std::process::exit(1);
    }
    let cs = &mut doc.doc_info.char_shapes[cs_id];
    let old_font_ids = cs.font_ids;
    cs.raw_data = None;
    cs.font_ids = [kor_id, eng_id, eng_id, eng_id, eng_id, eng_id, eng_id];
    println!("char_shape[{cs_id}] font_ids {:?} → {:?}", old_font_ids, cs.font_ids);
    // DocInfo raw_stream 캐시 무효화 — 캐시 우선이라 표시 안 함.
    doc.doc_info.raw_stream_dirty = true;

    let out_bytes = core.export_hwp_with_adapter().expect("export");
    fs::write(&output, &out_bytes).expect("write");
    println!("출력: {output} ({} bytes)", out_bytes.len());
}
