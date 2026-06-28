use std::fs;
fn main(){
  for path in std::env::args().skip(1){
    let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(&path).unwrap()).unwrap();
    let cs=&core.document().doc_info.char_shapes;
    let mut lens=std::collections::BTreeMap::new();
    for c in cs { let l=c.raw_data.as_ref().map(|d|d.len()).unwrap_or(0); *lens.entry(l).or_insert(0)+=1; }
    let name=path.rsplit('/').next().unwrap();
    println!("{} : char_shapes={} lengths={:?}", name, cs.len(), lens);
  }
}
