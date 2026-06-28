use std::fs;
fn walk_paras<'a>(paras:&'a [rhwp::model::paragraph::Paragraph], out:&mut Vec<&'a rhwp::model::paragraph::Paragraph>){
  for p in paras { out.push(p);
    for c in &p.controls { if let rhwp::model::control::Control::Table(t)=c {
      for cell in &t.cells { walk_paras(&cell.paragraphs, out); }
    }}
  }
}
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document(); let cs=&doc.doc_info.char_shapes;
  let mut all=Vec::new();
  for sec in &doc.sections { walk_paras(&sec.paragraphs, &mut all); }
  // 본문 텍스트(대사/지문) 쓰는 문단의 char_shape 길이
  let mut dist=std::collections::BTreeMap::new();
  let mut samples=Vec::new();
  for p in &all {
    let tc=p.text.chars().count(); if tc<4 {continue;}
    for r in &p.char_shapes {
      let l=cs.get(r.char_shape_id as usize).and_then(|c|c.raw_data.as_ref()).map(|d|d.len()).unwrap_or(99);
      *dist.entry(l).or_insert(0)+=1;
      if samples.len()<6 && (p.text.contains("대사")||p.text.contains("지문")) {
        samples.push(format!("cs{} len{} <{}>", r.char_shape_id, l, p.text.chars().take(8).collect::<String>()));
      }
    }
  }
  println!("본문문단 char_shape 길이분포: {:?}", dist);
  for s in samples { println!("  {}", s); }
}
