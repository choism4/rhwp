use std::fs;
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document(); let cs=&doc.doc_info.char_shapes;
  // 재사용=원본은 raw_data 보유(byte보존). 생성=raw_data None.
  let mut created=Vec::new(); let mut origu=std::collections::BTreeSet::new();
  for s in &doc.sections { for p in &s.paragraphs {
    for c in &p.controls { if let rhwp::model::control::Control::Table(t)=c {
      for cell in &t.cells { for cp in &cell.paragraphs {
        if cp.text.chars().count()==0 {continue;}
        for r in &cp.char_shapes {
          let has_raw=cs.get(r.char_shape_id as usize).map(|c|c.raw_data.is_some()).unwrap_or(false);
          if !has_raw {created.push((r.char_shape_id,cp.text.chars().take(12).collect::<String>()));}
          else {origu.insert(r.char_shape_id);}
        }
      }}
    }}
  }}
  println!("생성(raw_data없음=한컴빈폰트위험) 셀: {}건",created.len());
  for (id,t) in created.iter().take(8){println!("  cs{} <{}>",id,t);}
  println!("원본재사용 set: {:?}",origu);
}
