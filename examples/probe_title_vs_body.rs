// 우리 파일: 타이틀(섹션 문단, 한컴서 작동) vs 본문 셀(빈폰트) char_shape 비교.
use std::fs;
fn hex(d:&[u8])->String{ d.iter().map(|b|format!("{:02x}",b)).collect::<Vec<_>>().join(" ") }
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document();
  let dump=|id:usize,tag:&str|{ if let Some(cs)=doc.doc_info.char_shapes.get(id){
    let raw=cs.raw_data.as_ref().map(|d|format!("len{} [{}]",d.len(),hex(d))).unwrap_or("NONE".into());
    println!("{} cs{} raw={}", tag, id, raw);}};
  // 섹션 문단(표 아님) 텍스트 run charShape — 타이틀류
  for sec in &doc.sections { for p in &sec.paragraphs {
    let has_table=p.controls.iter().any(|c|matches!(c,rhwp::model::control::Control::Table(_)));
    if has_table { continue; }
    let txt:String=p.text.chars().take(10).collect();
    if p.text.contains("구성표")||p.text.contains("씬") {
      if let Some(r)=p.char_shapes.first(){ println!("[섹션문단 <{}>]",txt); dump(r.char_shape_id as usize,"  TITLE"); }
    }
  }}
  // 표 셀 본문 첫 텍스트 run
  for sec in &doc.sections { for p in &sec.paragraphs {
    for ctrl in &p.controls { if let rhwp::model::control::Control::Table(t)=ctrl {
      for cell in &t.cells { for cp in &cell.paragraphs {
        if cp.text.contains("지문")&&cp.text.chars().count()>5 {
          if let Some(r)=cp.char_shapes.first(){ println!("[셀 <{}>]",cp.text.chars().take(10).collect::<String>()); dump(r.char_shape_id as usize,"  BODY"); return; }
        }
      }}
    }}
  }}
}
