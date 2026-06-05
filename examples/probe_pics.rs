use std::fs; use rhwp::model::control::Control;
fn walk(ps:&[rhwp::model::paragraph::Paragraph],pics:&mut usize,tbls:&mut usize){
  for p in ps { for c in &p.controls { match c {
    Control::Picture(_)=>*pics+=1,
    Control::Table(t)=>{*tbls+=1; for cell in &t.cells {walk(&cell.paragraphs,pics,tbls);}},
    _=>{} } } }
}
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document();
  let mut pics=0; let mut tbls=0;
  for s in &doc.sections { walk(&s.paragraphs,&mut pics,&mut tbls);
    // master page / header-footer
  }
  let bins=0usize;
  println!("Picture control={} Table={} BinData={}",pics,tbls,bins);
}
