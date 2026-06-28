use std::fs;
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document();
  for sec in &doc.sections { for p in &sec.paragraphs {
    for ctrl in &p.controls { if let rhwp::model::control::Control::Table(t)=ctrl {
      for cell in &t.cells { for cp in &cell.paragraphs {
        let txt:String=cp.text.chars().take(12).collect();
        if txt.contains("지문")||txt.contains("대사입")||txt.contains("능청") {
          let textlen = cp.text.chars().count();
          let segs:Vec<String>=cp.char_shapes.iter().map(|r|format!("@{}:cs{}",r.start_pos,r.char_shape_id)).collect();
          println!("<{}> textChars={} segs({})=[{}]", txt, textlen, cp.char_shapes.len(), segs.join(" "));
        }
      }}
    }}
  }}
}
