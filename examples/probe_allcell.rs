use std::fs;
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document(); let cs=&doc.doc_info.char_shapes;
  let mut seen=std::collections::HashSet::new();
  for sec in &doc.sections { for p in &sec.paragraphs {
    for ctrl in &p.controls { if let rhwp::model::control::Control::Table(t)=ctrl {
      for cell in &t.cells { for cp in &cell.paragraphs {
        let tc=cp.text.chars().count(); if tc<2 {continue;}
        if let Some(r)=cp.char_shapes.first(){ let id=r.char_shape_id as usize;
          if !seen.insert(id){continue;}
          let l=cs.get(id).and_then(|c|c.raw_data.as_ref()).map(|d|d.len()).unwrap_or(99);
          println!("cs{} rawlen={} <{}>", id, l, cp.text.chars().take(10).collect::<String>());
        }}
      }}
    }}
  }}
}
