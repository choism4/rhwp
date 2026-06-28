use std::fs; use rhwp::model::control::Control;
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document();
  for s in &doc.sections { for p in &s.paragraphs { for c in &p.controls { if let Control::Table(t)=c {
    if t.row_count<500 {continue;}
    let rc=t.row_count;
    for r in [rc-2,rc-1]{
      for cell in t.cells.iter().filter(|cl|cl.row==r){
        println!("row{} col{} ({}문단):",r,cell.col,cell.paragraphs.len());
        for (i,pp) in cell.paragraphs.iter().enumerate(){
          println!("  p{}: len{} '{}'",i,pp.text.chars().count(),pp.text.chars().take(30).collect::<String>());
        }
      }
    }
  }}}}
}
