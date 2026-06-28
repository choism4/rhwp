use std::fs;
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document(); let cs=&doc.doc_info.char_shapes; let faces=&doc.doc_info.font_faces[0];
  let nm=|id:u16| faces.get(id as usize).map(|f|f.name.clone()).unwrap_or(format!("?{}",id));
  for sec in &doc.sections { for p in &sec.paragraphs {
    for ctrl in &p.controls { if let rhwp::model::control::Control::Table(t)=ctrl {
      for cell in &t.cells { for cp in &cell.paragraphs {
        if (cp.text.contains("대사입")||cp.text.contains("지문")||cp.text.contains("순덕")) && cp.text.chars().count()>3 {
          if let Some(r)=cp.char_shapes.first(){ let id=r.char_shape_id as usize;
            let l=cs.get(id).and_then(|c|c.raw_data.as_ref()).map(|d|d.len()).unwrap_or(0);
            let f=cs.get(id).map(|c|nm(c.font_ids[0])).unwrap_or_default();
            println!("<{}> cs{} rawlen={} font={}", cp.text.chars().take(8).collect::<String>(), id, l, f);
          }}
        }
      }}
    }}
  }}
}
