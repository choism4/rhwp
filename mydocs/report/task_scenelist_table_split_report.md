# 최종보고서 — 씬구성표 표 분할 시 행 클립

브랜치: `local/task-scenelist-table-split`

## 결함

host 문단에 pre-text(표 위 텍스트)가 있는 표가 페이지 분할될 때, 분할 경로가
pre-text 높이를 가용 높이에서 차감하지 않아 표 행이 페이지 하단을 넘쳐 클립.

재현: SBS 편집틀 씬구성표(32행×11열, 문단 텍스트 "1회 씬 구성표" + 표).

## 근본원인

`TypesetEngine::typeset_block_table` 의 표 분할 경로.

- fits 경로(표 통째 배치)는 `place_table_with_text` 가 `pre_height`(host 텍스트
  높이)를 `current_height` 에 누적.
- 분할 경로(`while cursor_row` 루프)는 `place_table_with_text` 를 호출하지 않아
  pre-text 높이를 미반영. 첫 fragment 의 `page_avail = table_available -
  current_height - caption_extra` 가 pre-text 높이를 빼지 않음.
- → `find_break_row` 가 페이지 전체 높이 기준으로 행 수를 산정 → pre-text 가
  점유한 영역만큼 표가 아래로 밀려 마지막 행 클립.

계측 확인: 씬구성표 page2 — `avail_for_rows=517.8`(pre-text 64px 미차감),
`find_break_row → 17행(487.5px)`. 타이틀 64 + 표 487.5 = 551.5 > 517.8 → 클립.

## 수정

`typeset_block_table` 에 `host_pre_height` 산출 (place_table_with_text 의
pre_height 와 동일 공식: voff>0 && host text 비어있지 않음 && is_first_table 일 때
`fmt.line_advances_sum(0..total_lines)`). 분할 루프 첫 fragment(`!is_continuation`)
의 `page_avail` 에서 `host_pre_height` 추가 차감.

`src/renderer/typeset.rs` — `host_pre_height` 블록 신설 + `page_avail` 1행 수정.

## 검증

- SBS 씬구성표: 수정 전 page2 17행(클립) → 수정 후 **16행**, page2 마지막
  가로선 y=579.3 < body 하단 585.8. 클립 해소. export-pdf PNG 확인.
- MBC 회귀: 씬구성표 16×6 은 분할 없이 whole `Table` 배치 — 분할 경로 미진입,
  무영향. 본문 거대 표(host text 없음 → host_pre_height=0) 무영향. 230p 동일.
- `cargo test --release`: svg_snapshot 7건 mismatch — 전부 **font-family 체인
  차이만**(좌표·레이아웃·페이지네이션 0 차이). 본 수정과 무관한 기존 stale
  golden(이전 변경의 'Noto Sans CJK KR' 체인 삽입 미반영). 본 수정의 레이아웃
  회귀 0.

## 회귀 위험

낮음. pre-text 가 있으면서 동시에 분할되는 표만 영향 — 본 결함 대상. pre-text
없는 표는 `host_pre_height=0` 으로 무변동.

## 잔여

- svg_snapshot golden 의 font-family staleness — 본 타스크 범위 밖, 별도 정리
  필요(`UPDATE_GOLDEN=1`).
