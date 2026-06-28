use std::fs;
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document(); let faces=&doc.doc_info.font_faces[0];
  let nm=|id:u16| faces.get(id as usize).map(|f|f.name.clone()).unwrap_or(format!("?{}",id));
  for (i,cs) in doc.doc_info.char_shapes.iter().enumerate(){
    let l=cs.raw_data.as_ref().map(|d|d.len()).unwrap_or(0);
    if l!=70 { println!("cs[{i}] rawlen={} hangulFont={} size={}", l, nm(cs.font_ids[0]), cs.base_size); }
  }
}
