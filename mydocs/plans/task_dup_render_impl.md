# 구현계획서 — 본문 표 분할 시 대사 중복 렌더

## Stage 1 — 디텍터 계측 (env-gated)

- `paragraph_layout.rs` 에 `line_range_dump_enabled()` 추가 — `OnceLock<bool>`,
  `RHWP_LINE_RANGE_DUMP` env. unset 시 무동작.
- `table_partial.rs:572` `(start_line,end_line)` 바인딩 직후 stderr 1줄:
  `LRD\t{section}\t{para}\t{control}\t{cell_idx}\t{cp_idx}\t{start_line}\t{end_line}\t{start_row}\t{end_row}\t{is_continuation}\t{is_split_start_row}\t{is_split_end_row}`
- svg_snapshot baseline 캡처 → `mydocs/working/baseline_stale_goldens.txt`.
- `cargo build` 성공 + 신규 테스트 실패 0.

## Stage 2 — 코퍼스 스캔 + 재현

- `mydocs/tools/scan_dup.py` 작성 — dump-pages 실행 → LRD 파싱 → 구간 겹침 검출.
- smoke subset(방송사별 최장 1) → 전체 601 HWP.
- `dup_scan_<date>.txt` 산출. DUPLICATE ≥1 → Stage 3.

## Stage 3 — 근본원인 + fix

- 재현 fragment 로 용의자 판별 (Stage1 dump 의 row/bool 필드 활용).
- 정확히 1개 확정 → 좁은 fix. host_pre_height 미변경 diff 확인.
- svg_snapshot 신규 회귀 0.

## Stage 4 — 검증

- 수정 binary 전체 코퍼스 재스캔 → DUPLICATE 0 + GAP-BUG 미증가.
- smoke 3회 결정성.
- `task_dup_render_report.md`.
