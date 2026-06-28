use std::fs;
fn hex(d:&[u8])->String{ d.iter().take(14).map(|b|format!("{:02x}",b)).collect::<Vec<_>>().join(" ") }
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document(); let faces=&doc.doc_info.font_faces[0];
  let nm=|id:u16| faces.get(id as usize).map(|f|f.name.clone()).unwrap_or("?".into());
  // 본문 표 셀별 run cs + 그 cs langs
  for (si,sec) in doc.sections.iter().enumerate(){ for (pi,p) in sec.paragraphs.iter().enumerate(){
    for ctrl in &p.controls { if let rhwp::model::control::Control::Table(t)=ctrl {
      for (ci,cell) in t.cells.iter().enumerate(){ for cp in &cell.paragraphs {
        let txt:String=cp.text.chars().take(10).collect();
        if txt.contains("지문")||txt.contains("순덕")||txt.contains("대사입") {
          if let Some(r)=cp.char_shapes.first(){ let id=r.char_shape_id as usize;
            if let Some(cs)=doc.doc_info.char_shapes.get(id){
              let langs:String = cs.font_ids.iter().map(|f|format!("{}",f)).collect::<Vec<_>>().join(",");
              let raw = cs.raw_data.as_ref().map(|d|hex(d)).unwrap_or("none".into());
              println!("s{si}p{pi}c{ci} <{}> cs{} langs=[{}] h={} raw14=[{}]", txt, id, langs, nm(cs.font_ids[0]), raw);
            }}
        }
      }}
    }}
  }}
}
