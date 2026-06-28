// 무변경 라운드트립: 기준본 load → 아무 수정 없이 export → 한컴 export 충실도 격리 테스트.
use std::fs;
fn main(){
  let inp=std::env::args().nth(1).unwrap(); let out=std::env::args().nth(2).unwrap();
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(&inp).unwrap()).unwrap();
  let bytes=core.export_hwp_native().expect("export");
  fs::write(&out,&bytes).unwrap();
  println!("wrote {} ({} bytes)", out, bytes.len());
}
