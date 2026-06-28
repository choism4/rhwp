use std::fs; use rhwp::model::control::Control;
fn walk(ps:&[rhwp::model::paragraph::Paragraph], depth:usize, out:&mut Vec<(usize,u16,u16,usize)>){
  for p in ps { for c in &p.controls { if let Control::Table(t)=c {
    out.push((depth, t.row_count, t.col_count, t.cells.len()));
    for cell in &t.cells { walk(&cell.paragraphs, depth+1, out); }
  }}}
}
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document();
  for (si,s) in doc.sections.iter().enumerate(){
    let mut out=Vec::new(); walk(&s.paragraphs,0,&mut out);
    println!("section{}: 표 {}개", si, out.len());
    for (d,r,c,cells) in &out { println!("  depth{} {}행x{}열 ({}셀)",d,r,c,cells); }
  }
}
