# 구현계획서 — 씬구성표 표 분할 시 행 클립

브랜치: `local/task-scenelist-table-split`

## 근본원인 (Stage 1 확정)

`typeset_block_table` 의 표 분할 경로가 host 문단 pre-text 높이를 미반영.

계측:
- `[TBT] text="1회 씬 구성표" cur_h=0.0 total_lines=1 lines_sum=64.0 voff=4340`
- `[SLT] avail_for_rows=517.8 approx=17 partial_h=487.5`

씬구성표 문단 = 타이틀 텍스트("1회 씬 구성표", 1줄 64px) + 표(voff=4340HU). 표는
문단 텍스트 아래 pre-text 높이만큼 내려가 배치. `place_table_with_text`(fits 경로)
는 `pre_height` 를 current_height 에 더하나, **분할 경로엔 그 처리가 없음**.
→ 분할 시 `page_avail = table_available`(517.8, pre-text 미차감) → find_break_row
가 17행(487.5) 선택 → 타이틀(64)+표(487.5)=551 > 517.8 → 마지막 행 클립.

## 수정 (단일 단계)

`typeset_block_table` 분할 경로에서, 첫 fragment(`!is_continuation`)의 `page_avail`
에 host pre-text 높이를 차감.

- `place_table_with_text` 와 동일 공식으로 `host_pre_height` 산출:
  `pre_table_end_line = if voff>0 && !para.text.is_empty() { total_lines } else {0}`,
  `is_first_table` 일 때 `fmt.line_advances_sum(0..pre_table_end_line)`.
- 루프의 `page_avail` (`!is_continuation` 분기)에서 `host_pre_height` 추가 차감.
- pre-text 자체의 렌더는 기존대로(표 voff 배치 + 호스트 텍스트) — 페이지 아이템
  추가 안 함(이미 렌더됨, 중복 방지).

검증 후 계측 eprintln 2건(`[SLT]`/`[TBT]`) 제거.

## 검증

1. SBS 씬구성표: 분할 시 page2 = 16행, 행 17 → page3. 클립 해소. dump-pages +
   export-svg 가로선 측정.
2. 회귀: MBC 씬구성표(16×6, 현재 정상) 변동 없음. 본문 거대 표(host text 없음 →
   host_pre_height=0 → 무영향) 변동 없음. `pdf/` 한컴 2022 대비 page count.
3. script-ai MBC/SBS 재렌더.

## 회귀 위험

낮음. host pre-text 가 있으면서 동시에 분할되는 표만 영향 — 그 케이스는 본 결함
대상. pre-text 없는 표(본문 표 등)는 host_pre_height=0 으로 무변동.
