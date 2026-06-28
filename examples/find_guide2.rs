use std::fs;
fn scan_text(s:&str,tag:&str){ if s.contains("성희롱")||s.contains("Guide")||s.contains("예방")||s.contains("제작진")||s.contains("KBS2TV"){ println!("{}: {}", tag, s.chars().take(40).collect::<String>()); } }
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document();
  for (si,sec) in doc.sections.iter().enumerate(){
    // 바탕쪽
    for (mi,mp) in sec.master_pages.iter().enumerate(){ for p in &mp.paragraphs { scan_text(&p.text, &format!("s{} master{} para",si,mi)); }}
    for (pi,p) in sec.paragraphs.iter().enumerate(){
      // 모든 control 종류
      for c in &p.controls {
        let cn=format!("{:?}", c); let short:String=cn.chars().take(30).collect();
        if cn.contains("성희롱")||cn.contains("Guide")||cn.contains("예방")||cn.contains("KBS2TV"){ println!("s{}p{} CTRL has guide ({})", si,pi,short); }
      }
      scan_text(&p.text, &format!("s{}p{}",si,pi));
    }
  }
}
