# 수행계획서 — 본문 표 분할 시 대사 중복 렌더

> 로컬 타스크 (외부 repo `edwardkim/rhwp` 에 issue 미생성 — script-ai 야간 무인 cycle 의
> 하위 작업. 작업지시자가 상위 계획 승인함). 브랜치 `local/task-dup-render` (off cb5678a6).

## 결함

방송 대본 본문은 하나의 거대 표로 렌더된다. 표 행/셀 문단이 페이지 경계를 넘을 때,
대사 텍스트가 페이지 N 하단과 페이지 N+1 상단에 **중복 출력**된다. 간헐적 ~2%.
작업지시자 요구: 단 1건도 허용 불가.

## 목표

1. 프로그래매틱 디텍터로 중복을 결정적으로 재현·계측한다.
2. 근본원인을 데이터로 1개 확정한다 (추정 금지).
3. 좁게 수정한다.
4. 전체 코퍼스(601 편집본 HWP)에서 DUPLICATE 0 검증.

## 접근

`compute_cell_line_ranges` 출력이 가장 깨끗한 신호다. 페이지 fragment 마다 셀 문단별
`(start_line, end_line)` 범위가 산정된다. 동일 셀 문단의 page N 범위 `[a,b)` 와 page N+1
범위 `[c,d)` 가 겹치면(`c<b && d>a`) = 중복. 정수 비교 — PDF/OCR 불필요.

## 의심 위치 (사전 조사)

1. `typeset.rs` 표 분할 cursor 전진 (~2210–2271) — `end_row` 과포함 시 다음 행 재방문.
2. `table_layout.rs:2582` `compute_cell_line_ranges` — `line_break_pos > abs_limit`
   epsilon 없음 → 경계 줄 page N 제외/page N+1 포함.
3. `table_partial.rs:346–347` — 한 row 가 split-start·end 동시 매칭 시 이중 계산.
4. `height_measurer.rs find_break_row` 과포함.

## 회귀 가드

- `typeset.rs` host_pre_height (cb5678a6) 미변경.
- svg_snapshot golden — 사전 baseline 캡처, font-family staleness 와 실회귀 구분.

## 산출물

- `mydocs/tools/scan_dup.py` — 디텍터 스캐너.
- `mydocs/report/dup_scan_<date>.txt` — 코퍼스 스캔 결과.
- `mydocs/report/task_dup_render_report.md` — 최종 보고.
