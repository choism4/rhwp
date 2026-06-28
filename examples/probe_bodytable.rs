use std::fs;
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document();
  for (si,sec) in doc.sections.iter().enumerate(){ for (pi,p) in sec.paragraphs.iter().enumerate(){
    for c in &p.controls { if let rhwp::model::control::Control::Table(t)=c {
      let rows=t.cells.iter().map(|x|x.row+1).max().unwrap_or(0);
      if rows>20 {
        println!("s{}p{} 본문표 {}행", si,pi,rows);
        for cell in t.cells.iter().take(20){ for cp in &cell.paragraphs { let tx:String=cp.text.chars().take(24).collect(); if !tx.trim().is_empty(){ println!("   r{}c{}: {}", cell.row, cell.col, tx); }}}
      }
    }}
  }}
}
