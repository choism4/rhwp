// TEST5: 인물명(작동) vs 대사(빈칸) char_shape 바이트 비교 — 한컴 수용 차이 격리.
use std::fs;
fn hex(d:&[u8])->String{ d.iter().map(|b|format!("{:02x}",b)).collect::<Vec<_>>().join(" ") }
fn walk<'a>(ps:&'a [rhwp::model::paragraph::Paragraph], o:&mut Vec<&'a rhwp::model::paragraph::Paragraph>){
  for p in ps { o.push(p); for c in &p.controls { if let rhwp::model::control::Control::Table(t)=c { for cell in &t.cells { walk(&cell.paragraphs,o);}}}}
}
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document(); let cs=&doc.doc_info.char_shapes;
  let mut all=Vec::new(); for s in &doc.sections{ walk(&s.paragraphs,&mut all);}
  let mut char_cs=None; let mut dlg_cs=None;
  for p in &all { let t=&p.text;
    if char_cs.is_none() && (t=="정우"||t=="순덕") { char_cs=p.char_shapes.first().map(|r|r.char_shape_id); }
    if dlg_cs.is_none() && t.contains("대사입니다") { dlg_cs=p.char_shapes.first().map(|r|r.char_shape_id); }
  }
  println!("인물명 cs={:?} 대사 cs={:?}", char_cs, dlg_cs);
  for (tag,id) in [("CHAR",char_cs),("DLG",dlg_cs)] { if let Some(i)=id { if let Some(c)=cs.get(i as usize){
    if let Some(r)=&c.raw_data { println!("{} cs{} len{}: {}", tag, i, r.len(), hex(r)); }
  }}}
  if let (Some(a),Some(b))=(char_cs.and_then(|i|cs.get(i as usize)).and_then(|c|c.raw_data.as_ref()),
                            dlg_cs.and_then(|i|cs.get(i as usize)).and_then(|c|c.raw_data.as_ref())){
    let n=a.len().min(b.len());
    let d:Vec<String>=(0..n).filter(|&k|a[k]!=b[k]).map(|k|format!("@{}:{:02x}/{:02x}",k,a[k],b[k])).collect();
    println!("DIFF char vs dlg: {}", d.join(" "));
  }
}
