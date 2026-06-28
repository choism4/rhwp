use std::fs;
fn main(){ for p in std::env::args().skip(1){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(&p).unwrap()).unwrap();
  let v=&core.document().header.version;
  let name=p.rsplit('/').next().unwrap();
  println!("{} : version={:?}", name, v);
}}
