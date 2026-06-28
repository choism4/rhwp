use std::fs;
fn main(){
  let data=fs::read(std::env::args().nth(1).unwrap()).unwrap();
  // crude: cfb 파싱 대신 rhwp 파서로 BinData 개수/크기 추정
  let core=rhwp::document_core::DocumentCore::from_bytes(&data).unwrap();
  let doc=core.document();
  println!("총 파일 {:.2}MB", data.len() as f64/1048576.0);
  println!("char_shapes={} faces={} sections={}", doc.doc_info.char_shapes.len(), doc.doc_info.font_faces.get(0).map(|v|v.len()).unwrap_or(0), doc.sections.len());
  for (si,s) in doc.sections.iter().enumerate(){
    let paras=s.paragraphs.len();
    println!("  section{}: {} 문단", si, paras);
  }
}
