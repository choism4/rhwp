// font_faces[lang][face_id] 의 name 을 변경. raw_data 무효화 후 model 직렬화.
// 향후 폰트 업로드/변경 기능에서 폰트 face 교체에 사용.
//
// 사용: patch_font_face_name <in.hwp> <out.hwp> <lang_idx> <face_id> <new_name>
//   lang_idx: 0=Korean 1=English 2=Hanja 3=Japanese 4=Other 5=Symbol 6=User
use std::fs;
fn main() {
    let mut args = std::env::args().skip(1);
    let input = args.next().expect("input.hwp");
    let output = args.next().expect("output.hwp");
    let lang_idx: usize = args.next().expect("lang_idx").parse().expect("int");
    let face_id: usize = args.next().expect("face_id").parse().expect("int");
    let new_name = args.next().expect("new_name");

    let data = fs::read(&input).expect("read");
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document_mut();
    let face = &mut doc.doc_info.font_faces[lang_idx][face_id];
    let old = face.name.clone();
    face.raw_data = None;
    face.name = new_name.clone();
    println!("font_faces[{lang_idx}][{face_id}] '{old}' → '{new_name}'");
    doc.doc_info.raw_stream_dirty = true;

    let out = core.export_hwp_with_adapter().expect("export");
    fs::write(&output, &out).expect("write");
    println!("출력: {output} ({} bytes)", out.len());
}
