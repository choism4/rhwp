use std::fs;
fn dist(name:&str, lens: impl Iterator<Item=usize>){
  let mut m=std::collections::BTreeMap::new();
  for l in lens { *m.entry(l).or_insert(0)+=1; }
  println!("  {}: {:?}", name, m);
}
fn main(){ for p in std::env::args().skip(1){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(&p).unwrap()).unwrap();
  let d=core.document(); let di=&d.doc_info;
  println!("{} (v{}.{}.{}.{})", p.rsplit('/').next().unwrap(), d.header.version.major,d.header.version.minor,d.header.version.build,d.header.version.revision);
  dist("char_shape", di.char_shapes.iter().filter_map(|c|c.raw_data.as_ref().map(|r|r.len())));
  dist("para_shape", di.para_shapes.iter().filter_map(|c|c.raw_data.as_ref().map(|r|r.len())));
  dist("border_fill", di.border_fills.iter().filter_map(|c|c.raw_data.as_ref().map(|r|r.len())));
  dist("face_name", di.font_faces[0].iter().filter_map(|c|c.raw_data.as_ref().map(|r|r.len())));
}}
