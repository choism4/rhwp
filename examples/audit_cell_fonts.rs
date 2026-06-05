use std::fs;
fn face_name(doc:&rhwp::model::document::Document, fid:u16)->String{
  doc.doc_info.font_faces.get(0)
    .and_then(|v|v.get(fid as usize))
    .map(|f|f.name.clone()).unwrap_or_else(||format!("?id{}",fid))
}
fn main(){
  let path=std::env::args().nth(1).unwrap();
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(&path).unwrap()).unwrap();
  let doc=core.document(); let cs=&doc.doc_info.char_shapes;
  println!("== {} ==", path.rsplit('/').next().unwrap());
  for s in &doc.sections { for p in &s.paragraphs {
    for c in &p.controls { if let rhwp::model::control::Control::Table(t)=c {
      for (ci,cell) in t.cells.iter().enumerate() { for cp in &cell.paragraphs {
        let txt:String=cp.text.chars().take(18).collect();
        if txt.trim().is_empty() {continue;}
        if let Some(r)=cp.char_shapes.first(){
          if let Some(c0)=cs.get(r.char_shape_id as usize){
            let fid=c0.font_ids[0];
            println!("cs{:>3}{} font{} '{}' {}pt bold={} <{}>",
              r.char_shape_id, if c0.raw_data.is_some(){"o"}else{"X생성"},
              fid, face_name(doc,fid), c0.base_size/100, c0.bold, txt);
          }
        }
      }}
    }}
  }}
}
