// 기준본(70B) char_shape raw vs serialize_char_shape_versioned(false) round-trip 충실도.
use std::fs;
use rhwp::serializer::doc_info::serialize_char_shape_versioned;
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let cs_all=&core.document().doc_info.char_shapes;
  let mut ok=0; let mut bad=0; let mut first_bad=String::new();
  for (i,cs) in cs_all.iter().enumerate(){
    if let Some(raw)=&cs.raw_data { if raw.len()==70 {
      let ser=serialize_char_shape_versioned(cs,false);
      if ser==*raw { ok+=1; } else {
        bad+=1;
        if first_bad.is_empty(){
          let n=raw.len().min(ser.len());
          let d:Vec<String>=(0..n).filter(|&k|raw[k]!=ser[k]).map(|k|format!("@{}:{:02x}/{:02x}",k,raw[k],ser[k])).collect();
          first_bad=format!("cs[{i}] rawlen={} serlen={} diff={} {}", raw.len(), ser.len(), d.len(), d.join(" "));
        }
      }
    }}
  }
  println!("70B char_shapes: round-trip OK={} BAD={}", ok, bad);
  if !first_bad.is_empty(){ println!("first BAD: {}", first_bad); }
}
