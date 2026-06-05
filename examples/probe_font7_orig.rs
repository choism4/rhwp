use std::fs;
fn walk<'a>(ps:&'a [rhwp::model::paragraph::Paragraph], out:&mut Vec<&'a rhwp::model::paragraph::Paragraph>){
  for p in ps { out.push(p);
    for c in &p.controls { if let rhwp::model::control::Control::Table(t)=c {
      for cell in &t.cells { walk(&cell.paragraphs, out); } } }
  }
}
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document(); let cs=&doc.doc_info.char_shapes;
  let mut all=Vec::new();
  for sec in &doc.sections { walk(&sec.paragraphs, &mut all); }
  let (mut u7,mut u8)=(0,0); let mut s7=Vec::new();
  for p in &all { let tc=p.text.chars().count(); if tc<4 {continue;}
    for r in &p.char_shapes {
      if let Some(c)=cs.get(r.char_shape_id as usize){
        if c.font_ids[0]==7 {u7+=1; if s7.len()<5 {s7.push(format!("cs{} <{}>",r.char_shape_id,p.text.chars().take(12).collect::<String>()));}}
        if c.font_ids[0]==8 {u8+=1;}
      }
    }
  }
  println!("기준본 본문 run: font7(-윤명조130) 사용={}  font8(-윤명조150) 사용={}",u7,u8);
  for s in s7 {println!("  font7 예: {}",s);}
}
