// 원본 vs 생성 char_shape raw 바이트 diff — 한컴 빈폰트 깨진필드 탐지.
use std::fs;
fn hex(d:&[u8])->String{ d.iter().map(|b|format!("{:02x}",b)).collect::<Vec<_>>().join(" ") }
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let cs=&core.document().doc_info.char_shapes;
  // 원본 -윤명조150 후보 = cs[0] (size1500), 생성 = cs[53] (size1400)
  for i in [0usize,11,53,19] {
    if let Some(c)=cs.get(i){ if let Some(d)=&c.raw_data {
      println!("cs[{i}] len={} font0={} size={}", d.len(), u16::from_le_bytes([d[0],d[1]]), c.base_size);
      println!("   {}", hex(d));
    }}
  }
  // byte-diff cs[0] vs cs[53]
  if let (Some(a),Some(b))=(cs.get(0).and_then(|c|c.raw_data.as_ref()), cs.get(53).and_then(|c|c.raw_data.as_ref())){
    let n=a.len().min(b.len());
    let diffs:Vec<String>=(0..n).filter(|&i|a[i]!=b[i]).map(|i|format!("@{}:{:02x}->{:02x}",i,a[i],b[i])).collect();
    println!("DIFF cs0 vs cs53 ({} bytes): {}", diffs.len(), diffs.join(" "));
  }
}
