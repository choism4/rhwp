use std::fs;
fn main() {
    let path = std::env::args().nth(1).expect("hwp");
    let data = fs::read(&path).expect("read");
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document();
    for (si, sec) in doc.sections.iter().enumerate() {
        let pd = &sec.section_def.page_def;
        println!("sec{}: w={} h={} L={} R={} T={} B={} hd={} ft={} gut={}",
            si, pd.width, pd.height, pd.margin_left, pd.margin_right, pd.margin_top, pd.margin_bottom, pd.margin_header, pd.margin_footer, pd.margin_gutter);
    }
}
