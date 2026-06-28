// 진단: 대형 baseline 페이지 렌더트리 build panic 재현 (v3 #2).
// 사용: repro_strip_crash <hwp>
use std::fs;
fn main() {
    let path = std::env::args().nth(1).expect("usage: repro_strip_crash <hwp>");
    let data = fs::read(&path).expect("read");
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let pages = core.page_count();
    println!("pageCount={pages}");
    for p in 0..pages {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| core.get_page_control_layout_native(p))) {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => { println!("page {p}: ERR {e:?}"); }
            Err(_) => { println!("page {p}: PANIC"); break; }
        }
    }
    println!("DONE");
}
