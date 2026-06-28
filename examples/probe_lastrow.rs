use std::fs; use rhwp::model::control::Control;
fn main(){
  let core=rhwp::document_core::DocumentCore::from_bytes(&fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
  let doc=core.document();
  // 본문 표(가장 큰 표) 찾기 → 마지막 몇 행 셀의 문단/텍스트
  for s in &doc.sections { for p in &s.paragraphs {
    for c in &p.controls { if let Control::Table(t)=c {
      if t.row_count<10 {continue;}
      println!("본문표 {}행 {}열, 셀 {}", t.row_count, t.col_count, t.cells.len());
      // 마지막 행 인덱스
      let last=t.row_count-1;
      for cell in &t.cells {
        if cell.row+cell.row_span> last as u16 && cell.row<=last as u16 {
          let paras=cell.paragraphs.len();
          let empties=cell.paragraphs.iter().filter(|pp|pp.text.trim().is_empty()).count();
          let texts:Vec<String>=cell.paragraphs.iter().map(|pp|pp.text.chars().take(15).collect()).collect();
          println!("  마지막행 셀(row{} span{} col{}): 문단{} 빈{} {:?}",cell.row,cell.row_span,cell.col,paras,empties,texts);
        }
      }
    }}
  }}
}
