# 최종보고서 — 본문 표 분할 시 대사 중복 렌더

브랜치: `local/task-dup-render` (off cb5678a6)

## 결함

방송 대본 본문은 하나의 거대 표로 렌더된다. 표 셀 문단이 페이지 경계를 넘을 때
대사 텍스트가 페이지 N 하단과 페이지 N+1 상단에 중복 출력. 간헐적 ~2%.

## 디텍터

`table_partial.rs` 에 env-gated(`RHWP_LINE_RANGE_DUMP`) LRD 덤프 추가 — 분할 행 셀
문단별 줄 범위 `(start_line,end_line)` + 행/분할 메타. `mydocs/tools/scan_dup.py` 가
`export-svg` 렌더 stderr 를 파싱, 동일 셀 문단의 연속 페이지 fragment 줄 범위가
겹치면(`c<b && d>a`) DUPLICATE 로 검출. 정수 비교 — 결정적, PDF/OCR 불필요.

코퍼스: `전달용_원문 및 편집틀 V2/{MBC,SBS,KBS}/편집본/*.hwp` 601 파일.

## 근본원인 (3건, 데이터로 확정)

### 1. offset/limit 측 경계 기준 불일치 — `table_layout.rs compute_cell_line_ranges`
limit 측 break 비교는 `line_break_pos = cum + h` (trailing line_spacing 제외, #656),
offset 측 skip 비교는 `line_end_pos = cum + line_h` (line_spacing 포함). 경계값이
줄의 trailing-ls 구간에 걸치면 page N 은 그 줄 include, page N+1 은 skip 안 함 →
경계 줄 중복. **수정**: offset skip 도 `line_break_pos` 기준 사용. (109/112 smoke)

### 2. row_span 셀의 페이지 straddle — `table_partial.rs layout_partial_table`
행 경계에서 row_span 으로 페이지를 걸치는 셀은 `is_split_*_row`(셀이 분할 행
자체일 때만 set)에 안 잡혀 `line_ranges=None` → 인접 fragment 양쪽에 전체 렌더.
**수정**: straddle 검출 → 이전 fragment 점유 행 높이를 content_offset, 이후로
넘어가는 분량을 content_limit 으로 환산해 `compute_cell_line_ranges` 적용.

### 3. straddle 가 intra-row 분할 경계를 가로지름
straddle 셀이 intra-row 분할되는 boundary 행을 span 하면 offset/limit 이 그 행의
split 분량을 누락 → 경계 줄 중복. **수정**: 양쪽 fragment 에서 boundary 를 동일
공식 `sum(분할행 이전 셀 행) + split offset` 으로 산정 → content_limit(A) ==
content_offset(B) 보장.

## 검증

- 전체 601 코퍼스: **DUPLICATE 0 / GAP-BUG 0 / RENDER-FAIL 0** (LRD 2,357,318 검사).
  수정 전 동일 코퍼스 DUPLICATE 37+ (smoke 3 파일만 112).
- `cargo test --test svg_snapshot`: 1 passed / 7 failed — 7건 전부 기존 stale golden
  (font-family 체인 차이, 본 수정 무관): form_002_page_0, issue_147_aift_page3,
  issue_157_page_1, issue_267_ktx_toc_page, issue_617_exam_kor_page5,
  issue_677_bokhakwonseo_page1, table_text_page_0. **본 수정 신규 회귀 0.**
- 회귀 가드 `host_pre_height`(cb5678a6) 미변경.

## 잔여

- svg_snapshot golden 7건 staleness — 별도 정리 (`UPDATE_GOLDEN=1`).
- `RHWP_LINE_RANGE_DUMP` 디텍터·`scan_dup.py` 영구 보존 — 회귀 검증 도구.
