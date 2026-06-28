// 표 셀 텍스트 run 의 char_shape 폰트 인코딩 덤프 (기준본 vs 우리 비교용).
use std::fs;
fn hex(d:&[u8])->String{ d.iter().map(|b|format!("{:02x}",b)).collect::<Vec<_>>().join(" ") }
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document(); let faces=&doc.doc_info.font_faces[0];
  let nm=|id:u16| faces.get(id as usize).map(|f|f.name.clone()).unwrap_or(format!("?{}",id));
  println!("총 char_shapes={} fonts={}", doc.doc_info.char_shapes.len(), faces.len());
  let mut seen=std::collections::HashSet::new();
  for sec in &doc.sections { for p in &sec.paragraphs {
    for ctrl in &p.controls { if let rhwp::model::control::Control::Table(t)=ctrl {
      for cell in &t.cells { for cp in &cell.paragraphs {
        let txt:String=cp.text.chars().take(8).collect();
        if cp.text.chars().count()<3 { continue; }
        if let Some(r)=cp.char_shapes.first(){ let id=r.char_shape_id as usize;
          if !seen.insert(id) { continue; }
          if let Some(cs)=doc.doc_info.char_shapes.get(id){
            let langs:Vec<String>=cs.font_ids.iter().map(|f|nm(*f)).collect();
            let raw=cs.raw_data.as_ref().map(|d|format!("len{} [{}]",d.len(),hex(d))).unwrap_or("NONE".into());
            println!("cs{} <{}> langs=[{}]\n   raw={}", id, txt, langs.join("|"), raw);
          }}
      }}
    }}
  }}
}
