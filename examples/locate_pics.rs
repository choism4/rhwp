use std::fs; use rhwp::model::control::Control;
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document();
  for (si,s) in doc.sections.iter().enumerate(){
    for (pi,p) in s.paragraphs.iter().enumerate(){
      for (ci,c) in p.controls.iter().enumerate(){
        if let Control::Picture(_)=c { println!("Picture: section{} para{} control{} (text='{}')",si,pi,ci,p.text.chars().take(20).collect::<String>()); }
        if let Control::Table(t)=c { for (cellI,cell) in t.cells.iter().enumerate(){ for (cpi,cp) in cell.paragraphs.iter().enumerate(){ for cc in &cp.controls { if let Control::Picture(_)=cc { println!("Picture in TABLE: s{} p{} ctrl{} cell{} cpara{} (celltext='{}')",si,pi,ci,cellI,cpi,cp.text.chars().take(20).collect::<String>()); } } } } }
      }
    }
  }
  // master page
  println!("--- master pages: {} ---", doc.doc_info.char_shapes.len());
}
