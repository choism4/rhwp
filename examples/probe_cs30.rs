use std::fs;
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document(); let f=&doc.doc_info.font_faces[0];
  for id in [29usize,30] { if let Some(c)=doc.doc_info.char_shapes.get(id){
    let nm=f.get(c.font_ids[0] as usize).map(|x|x.name.clone()).unwrap_or_default();
    println!("cs{} hangulFont={} ({}) size={}", id, c.font_ids[0], nm, c.base_size);
  }}
}
