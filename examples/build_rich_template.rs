// rich 편집틀 생성: 1_우선 기준본에서 모든 Picture control 제거(제작진 사진·doodle 잔상)
// → DocInfo(char_shape/face) 전부 보존한 폰트 완전 + 이미지 클린 템플릿.
// 본문 표 내용은 fill 단계가 비우고 재채우므로 그대로 둔다.
// 사용: build_rich_template <입력 기준본> <출력>
use rhwp::model::control::Control;
use std::fs;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() != 2 {
        eprintln!("사용: build_rich_template <입력> <출력>");
        std::process::exit(2);
    }
    let mut core =
        rhwp::document_core::DocumentCore::from_bytes(&fs::read(&a[0]).expect("입력 읽기"))
            .expect("파싱");

    // 모든 섹션/문단의 Picture control 수집 후 (섹션, 문단, 제어) 역순 삭제.
    let mut targets: Vec<(usize, usize, usize)> = Vec::new();
    {
        let doc = core.document();
        for (si, s) in doc.sections.iter().enumerate() {
            for (pi, p) in s.paragraphs.iter().enumerate() {
                for (ci, c) in p.controls.iter().enumerate() {
                    if matches!(c, Control::Picture(_)) {
                        targets.push((si, pi, ci));
                    }
                }
            }
        }
    }
    // 같은 문단 내에서는 control_idx 큰 것부터 지워야 인덱스가 안 밀린다.
    targets.sort_unstable_by(|x, y| y.cmp(x));
    let mut removed = 0;
    for (si, pi, ci) in targets {
        match core.delete_picture_control_native(si, pi, ci) {
            Ok(_) => removed += 1,
            Err(e) => eprintln!("  경고: s{si} p{pi} ctrl{ci} 삭제 실패: {e:?}"),
        }
    }

    // section0 의 씬구성표 문단('… 씬 구성표') 앞에 있는 빈 front-matter 문단 제거.
    // 안 지우면 빈 첫 페이지가 생긴다(스켈레톤은 씬구성표부터 시작). 역순 삭제.
    let scene_idx = {
        let doc = core.document();
        doc.sections.first().and_then(|s| {
            s.paragraphs
                .iter()
                .position(|p| p.text.contains("씬 구성표") || p.text.contains("씬구성표"))
        })
    };
    if let Some(scene) = scene_idx {
        let mut blanks: Vec<usize> = {
            let doc = core.document();
            let s = &doc.sections[0];
            (0..scene)
                .filter(|&pi| {
                    s.paragraphs[pi].controls.is_empty()
                        && s.paragraphs[pi].text.trim().is_empty()
                })
                .collect()
        };
        blanks.sort_unstable_by(|a, b| b.cmp(a));
        for pi in blanks {
            match core.delete_paragraph_native(0, pi) {
                Ok(_) => println!("  section0 빈 문단 p{pi} 삭제"),
                Err(e) => eprintln!("  경고: p{pi} 삭제 실패: {e:?}"),
            }
        }
        // 씬구성표 문단의 page_break_before 제거 — 안 그러면 씬구성표가 2페이지로
        // 밀려 빈 첫 페이지가 남는다. 삭제 후 재탐색(인덱스 이동).
        let new_scene = {
            let doc = core.document();
            doc.sections[0]
                .paragraphs
                .iter()
                .position(|p| p.text.contains("씬 구성표") || p.text.contains("씬구성표"))
        };
        if let Some(sc) = new_scene {
            match core.apply_para_format_native(0, sc, "{\"pageBreakBefore\":false}") {
                Ok(_) => println!("  씬구성표 p{sc} page_break_before 제거"),
                Err(e) => eprintln!("  경고: page_break 제거 실패: {e:?}"),
            }
        }
    }

    // Picture control 을 지워도 이미지 BinData 블롭(실제 바이트)은 남아 파일이
    // 비대해진다(KBS 기준본 제작진 사진 2.3MB). 전 BinData 이미지를 비워 dead
    // weight 제거 — 출력에는 어차피 안 쓰이는(control 삭제됨) 이미지들.
    let bin_count = core.document().doc_info.bin_data_list.len();
    let mut blanked = 0;
    for i in 0..bin_count {
        if core.blank_bin_data_image(i as u16).is_ok() {
            blanked += 1;
        }
    }
    println!("  BinData 이미지 {}/{} 개 비움", blanked, bin_count);

    let out = core.export_hwp_with_adapter().expect("직렬화");
    fs::write(&a[1], &out).expect("출력 쓰기");
    println!(
        "Picture 제거 {} 개 → {} ({:.2}MB)",
        removed,
        a[1],
        out.len() as f64 / 1048576.0
    );
}
