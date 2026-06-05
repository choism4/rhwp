use std::fs;
fn main() {
    let core = rhwp::document_core::DocumentCore::from_bytes(
        &fs::read(std::env::args().nth(1).unwrap()).unwrap(),
    )
    .unwrap();
    let doc = core.document();
    let target = std::env::args().nth(2).unwrap();
    let faces = &doc.doc_info.font_faces[0];
    for (fid, f) in faces.iter().enumerate() {
        if f.name.contains(&target) {
            let mut sizes: Vec<(usize, i32, bool)> = doc
                .doc_info
                .char_shapes
                .iter()
                .enumerate()
                .filter(|(_, c)| c.font_ids[0] == fid as u16)
                .map(|(i, c)| (i, c.base_size / 100, c.bold))
                .collect();
            sizes.sort_by_key(|x| x.1);
            println!("  font{} '{}': {:?}", fid, f.name, sizes);
        }
    }
}
