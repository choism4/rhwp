use std::fs;
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  for (i,c) in core.document().doc_info.char_shapes.iter().enumerate(){
    if c.font_ids[0]==1 {println!("cs{} size={}pt bold={}",i,c.base_size/100,c.bold);}
  }
}
