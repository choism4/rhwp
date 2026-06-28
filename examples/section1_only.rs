// section 0 제거 → body(section1)를 page1 로. 한컴 뷰어 캡처용.
use std::fs;
fn main(){
  let inp=std::env::args().nth(1).unwrap(); let out=std::env::args().nth(2).unwrap();
  let mut core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(&inp).unwrap()).unwrap();
  let d=core.document_mut();
  if d.sections.len()>1 { d.sections.remove(0); }
  d.sections[0].raw_stream=None;
  let bytes=core.export_hwp_native().expect("export");
  fs::write(&out,&bytes).unwrap();
  println!("wrote pages={}", core.page_count());
}
