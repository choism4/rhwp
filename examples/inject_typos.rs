// AI prompt 보정 방지 테스트용. 본문 단어를 깨진 자모·오타로 치환한다.
// 사용: inject_typos <입력 hwp> <출력 hwp>

use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("사용: inject_typos <입력 hwp> <출력 hwp>");
        std::process::exit(2);
    }
    let (input, output) = (&args[0], &args[1]);
    let data = fs::read(input).expect("입력 읽기 실패");
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("파싱 실패");

    let pairs: &[(&str, &str)] = &[
        // 깨진 한글 자모 ("ㅘ이키키" 패턴)
        ("놀이터", "놀ㅇㅣ터"),
        // 모음 오타
        ("청소", "쳥소"),
        // 자음 오타
        ("선결", "섯결"),
        // 종성 오타
        ("로봇", "로봍"),
        // 자모 누락
        ("샤워", "샤어"),
        // 띄어쓰기 변형
        ("아이들", "아 이들"),
    ];

    for (from, to) in pairs {
        match core.replace_all_native(from, to, true) {
            Ok(json) => println!("'{from}' -> '{to}': {json}"),
            Err(e) => eprintln!("'{from}' 치환 실패: {e}"),
        }
    }

    // replace_all_native 가 raw_stream 을 무효화하지 않으므로 직렬화 시 원본 raw 가 그대로
    // 사용되어 본문 텍스트 수정이 출력에 반영되지 않는다. 모든 섹션 raw_stream 을 명시적
    // 으로 비워 재직렬화를 강제한다.
    let section_count = core.document().sections.len();
    for sec_idx in 0..section_count {
        core.document_mut().sections[sec_idx].raw_stream = None;
    }

    let bytes = core.export_hwp_with_adapter().expect("직렬화 실패");
    fs::write(output, &bytes).expect("출력 쓰기 실패");
    println!("저장: {output} ({} bytes)", bytes.len());
}
