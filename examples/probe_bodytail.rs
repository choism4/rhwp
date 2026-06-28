use std::fs; use rhwp::model::control::Control;
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document();
  for s in &doc.sections { for p in &s.paragraphs {
    for c in &p.controls { if let Control::Table(t)=c {
      if t.row_count<50 {continue;}
      let rc=t.row_count;
      println!("본문표 {}행x{}열. 마지막 8행 셀 내용:",rc,t.col_count);
      for r in (rc.saturating_sub(8))..rc {
        let cells:Vec<String>=t.cells.iter().filter(|cl|cl.row==r).map(|cl|{
          let txt:String=cl.paragraphs.iter().map(|pp|pp.text.clone()).collect::<Vec<_>>().join("|");
          let np=cl.paragraphs.len();
          format!("c{}[{}p]'{}'",cl.col,np,txt.chars().take(18).collect::<String>())
        }).collect();
        println!("  row{}: {}",r,cells.join("  "));
      }
    }}
  }}
}
