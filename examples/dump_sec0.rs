use std::fs; use rhwp::model::control::Control;
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document();
  for (si,s) in doc.sections.iter().enumerate(){
    println!("== section{} ({} 문단) ==",si,s.paragraphs.len());
    for (pi,p) in s.paragraphs.iter().enumerate(){
      let ctrls:Vec<String>=p.controls.iter().map(|c|match c{Control::Table(t)=>format!("Table[{}cells]",t.cells.len()),Control::Picture(_)=>"Pic".into(),Control::Equation(_)=>"Eqn".into(),_=>"ctrl".into()}).collect();
      println!("  p{}: text='{}' controls={:?}",pi,p.text.chars().take(30).collect::<String>(),ctrls);
    }
    if si>=1 {break;}
  }
}
