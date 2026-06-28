use std::fs;
fn find_body(doc:&rhwp::model::document::Document)->Option<(usize,usize)>{
  for (si,sec) in doc.sections.iter().enumerate(){
    for (pi,p) in sec.paragraphs.iter().enumerate(){
      for c in &p.controls { if let rhwp::model::control::Control::Table(t)=c {
        if t.cells.iter().any(|cell| cell.paragraphs.iter().any(|cp| cp.text.contains("대사입니다"))){ return Some((si,pi)); }
      }}
    }
  }
  None
}
fn main(){
  let inp=std::env::args().nth(1).unwrap(); let out=std::env::args().nth(2).unwrap();
  let mut core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(&inp).unwrap()).unwrap();
  let loc=find_body(core.document());
  println!("body at {:?}", loc);
  if let Some((si,pi))=loc {
    // 그 섹션에서 body 앞 문단 삭제
    for _ in 1..pi { let _=core.delete_paragraph_native(si,1); }
    // 앞 섹션들 비우기 (1개 남기고)
    for s in 0..si { let n=core.document().sections[s].paragraphs.len(); for _ in 1..n { if core.delete_paragraph_native(s,1).is_err(){break;} } }
  }
  let bytes=core.export_hwp_native().expect("export");
  fs::write(&out,&bytes).unwrap();
  println!("wrote pages={}", core.page_count());
}
