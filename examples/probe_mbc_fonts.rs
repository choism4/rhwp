use std::fs; use std::collections::BTreeMap;
fn walk<'a>(ps:&'a [rhwp::model::paragraph::Paragraph], out:&mut Vec<&'a rhwp::model::paragraph::Paragraph>){
  for p in ps { out.push(p);
    for c in &p.controls { if let rhwp::model::control::Control::Table(t)=c {
      for cell in &t.cells { walk(&cell.paragraphs, out);} } } }
}
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document(); let cs=&doc.doc_info.char_shapes;
  // font-id histogram in body runs (text>=4)
  let mut all=Vec::new(); for s in &doc.sections {walk(&s.paragraphs,&mut all);}
  let mut hist:BTreeMap<u16,u32>=BTreeMap::new();
  for p in &all { if p.text.chars().count()<4 {continue;}
    for r in &p.char_shapes { if let Some(c)=cs.get(r.char_shape_id as usize){*hist.entry(c.font_ids[0]).or_insert(0)+=1;} } }
  println!("MBC 본문 font-id 히스토그램(run수): {:?}",hist);
  // which char_shapes use font-id 1 (윤고딕140) + bold?
  for (i,c) in cs.iter().enumerate(){
    if c.font_ids[0]==1 { let bold=c.attr&1; println!("  cs{} font1(윤고딕140) base_size={} bold={}",i,c.base_size,bold);}
  }
}
