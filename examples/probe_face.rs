use std::fs;
fn hex(d:&[u8])->String{ d.iter().map(|b|format!("{:02x}",b)).collect::<Vec<_>>().join(" ") }
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let faces=&core.document().doc_info.font_faces[0];
  for i in [11usize,13,14] {
    if let Some(f)=faces.get(i){
      let raw=f.raw_data.as_ref().map(|d|format!("len{} [{}]",d.len(),hex(d))).unwrap_or("NONE".into());
      println!("face[{i}] name='{}' alt_type={} alt={:?} default={:?}\n   raw={}", f.name, f.alt_type, f.alt_name, f.default_name, raw);
    }
  }
}
