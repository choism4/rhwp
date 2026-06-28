// 각 언어 category 별 font face 목록 dump.
use std::fs;
fn main() {
    let path = std::env::args().nth(1).expect("hwp");
    let data = fs::read(&path).expect("read");
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document();
    let cats = ["한글", "영문", "한자", "일본", "기타", "기호", "사용자"];
    for (lang, faces) in doc.doc_info.font_faces.iter().enumerate() {
        let cat = cats.get(lang).copied().unwrap_or("?");
        println!("== category {lang} ({cat}) — {} faces ==", faces.len());
        for (i, f) in faces.iter().enumerate() {
            if i > 20 { println!("  ... 생략"); break; }
            println!("  [{i}] name='{}'", f.name);
        }
    }
}
