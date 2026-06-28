use std::fs;
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document();
  for (si,sec) in doc.sections.iter().enumerate(){ for (pi,p) in sec.paragraphs.iter().enumerate(){
    // 문단 본문
    if p.text.contains("성희롱")||p.text.contains("Guide")||p.text.contains("예방")||p.text.contains("제작"){ println!("PARA s{}p{}: {}", si,pi, p.text.chars().take(40).collect::<String>()); }
    for c in &p.controls { if let rhwp::model::control::Control::Table(t)=c {
      for cell in &t.cells { for cp in &cell.paragraphs {
        if cp.text.contains("성희롱")||cp.text.contains("Guide")||cp.text.contains("예방"){ println!("TABLE s{}p{} r{}c{}: {}", si,pi,cell.row,cell.col, cp.text.chars().take(40).collect::<String>()); }
      }}
    }}
  }}
}
