use std::fs;
fn walk<'a>(ps:&'a [rhwp::model::paragraph::Paragraph], out:&mut Vec<&'a rhwp::model::paragraph::Paragraph>){
  for p in ps { out.push(p); for c in &p.controls { if let rhwp::model::control::Control::Table(t)=c {
    for cell in &t.cells { walk(&cell.paragraphs,out);} } } }
}
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document(); let cs=&doc.doc_info.char_shapes;
  let mut all=Vec::new(); for s in &doc.sections{walk(&s.paragraphs,&mut all);}
  let mut nb=0; let mut bd=0;
  for p in &all { if p.text.chars().count()<6 {continue;}
    for r in &p.char_shapes { if let Some(c)=cs.get(r.char_shape_id as usize){
      if c.font_ids[0]==5 { if c.attr&1==1 {bd+=1} else {nb+=1} } }}}
  println!("기준본 MBC body font5(-윤명조130) run: bold={} non-bold={}",bd,nb);
}
