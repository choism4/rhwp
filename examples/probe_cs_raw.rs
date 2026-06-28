// cs raw_data 의 hangul font id(byte0-1 LE) vs model font_ids[0] 비교 — raw stale 진단.
use std::fs;
fn main(){
  let p=std::env::args().nth(1).unwrap();
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(&p).unwrap()).unwrap();
  let doc=core.document();
  for i in [0usize,1,51,52,53] {
    if let Some(cs)=doc.doc_info.char_shapes.get(i){
      let model_font = cs.font_ids.get(0).copied().unwrap_or(0);
      let raw_font = cs.raw_data.as_ref().filter(|d| d.len()>=2).map(|d| u16::from_le_bytes([d[0],d[1]]));
      let rawlen = cs.raw_data.as_ref().map(|d| d.len()).unwrap_or(0);
      println!("cs[{i}] model_hangulFont={} raw_hangulFont={:?} rawlen={}", model_font, raw_font, rawlen);
    }
  }
}
