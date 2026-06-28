use std::fs;
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document();
  println!("sections={}", doc.sections.len());
  for (si,sec) in doc.sections.iter().enumerate(){
    println!("--- section {} (paras={}) ---", si, sec.paragraphs.len());
    for (pi,p) in sec.paragraphs.iter().enumerate().take(12){
      let txt:String=p.text.chars().take(20).collect();
      let tables:Vec<String>=p.controls.iter().filter_map(|c| if let rhwp::model::control::Control::Table(t)=c { Some(format!("표{}x{}",t.cells.iter().map(|x|x.row+1).max().unwrap_or(0),t.cells.iter().map(|x|x.col+1).max().unwrap_or(0))) } else {None}).collect();
      // 씬구성표 키워드
      let scene = if p.text.contains("씬")&&p.text.contains("구성") {" <<씬구성표제목"} else {""};
      println!("  s{}p{} <{}> {:?}{}", si, pi, txt, tables, scene);
    }
  }
}
