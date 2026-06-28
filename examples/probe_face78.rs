use std::fs;
fn hex(d:&[u8])->String{ d.iter().map(|b|format!("{:02x}",b)).collect::<Vec<_>>().join(" ") }
fn main(){
  for p in std::env::args().skip(1){
    let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(&p).unwrap()).unwrap();
    let faces=&core.document().doc_info.font_faces[0];
    println!("### {} (faces={})", p.rsplit('/').next().unwrap(), faces.len());
    for i in 0..faces.len().min(12){ let f=&faces[i];
      let raw=f.raw_data.as_ref().map(|d|format!("len{} {}",d.len(),hex(d))).unwrap_or("NONE".into());
      println!("  [{i}] '{}' alt={} raw={}", f.name, f.alt_type, raw);
    }
  }
}
