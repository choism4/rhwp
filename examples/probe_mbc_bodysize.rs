use std::fs;
fn walk<'a>(ps:&'a [rhwp::model::paragraph::Paragraph], o:&mut Vec<&'a rhwp::model::paragraph::Paragraph>){for p in ps{o.push(p);for c in &p.controls{if let rhwp::model::control::Control::Table(t)=c{for cell in &t.cells{walk(&cell.paragraphs,o);}}}}}
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document(); let cs=&doc.doc_info.char_shapes;
  let mut m=std::collections::BTreeMap::new();
  let mut all=Vec::new(); for s in &doc.sections{walk(&s.paragraphs,&mut all);}
  for p in &all{ if p.text.chars().count()<8{continue;}
    for r in &p.char_shapes{if let Some(c)=cs.get(r.char_shape_id as usize){if c.font_ids[0]==5 && !c.bold{*m.entry((r.char_shape_id,c.base_size/100)).or_insert(0)+=1;}}}}
  println!("본문 비볼드 font5 (cs,size)->run: {:?}",m);
}
