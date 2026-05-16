# 수행계획서 — 씬구성표 표 분할 시 행 클립

브랜치: `local/task-scenelist-table-split` (from `local/task-textbox-clip` fe5fee75)
마일스톤: M (typeset 표 분할 정합)

## 1. 문제

SBS 편집틀의 씬구성표(32행×11열 단일 표)가 2페이지에 걸쳐 분할될 때, 페이지
경계의 행이 잘린다(클립). 페이지 하단 밖으로 행이 삐져나가 윗부분만 보임.

`dump-pages` (script-ai 측 SBS 생성 HWP):
```
page2: PartialTable rows=0..17  used=0.0px    ← 17행 배치
page3: PartialTable rows=17..32 used=480.0px
```

- 씬구성표 구조: 헤더행 → S#1~30(15행) → 2번째 헤더행(row 16) → S#31~48.
- page2 에 행 0~16(17행)을 배치 → 마지막 행 16(2번째 헤더 "S# 장소 PAGE")이
  body_area 하단(h=517.8px)을 넘쳐 클립.
- 올바른 동작: page2 = 행 0~15, 행 16 은 page3 로.

## 2. 진단 단서

- `used=0.0px` — page2 의 PartialTable 이 컬럼 누적 높이에 0 을 보고. 행 높이가
  `used` 에 누적되지 않음 → 엔진이 페이지 잔여 공간을 오판해 한 행 더 packing
  하는 것으로 의심.
- LAYOUT_OVERFLOW 미발생 — overflow 감지 경로가 이 케이스를 못 잡음.
- 표 `쪽나눔=RowBreak (attr=0x04000006)` — 행 경계 분할 표.
- 관련 코드: `src/renderer/typeset.rs` 표 분할 (`find_break_row` + 인트라-로우
  분할, 약 line 2036~2130). 직전 작업 fe5fee75(Task #713 류)가 본문 표 분할의
  유사 결함(`find_break_row` 행높이 과대/과소추정)을 손봄 — 씬구성표 케이스는
  미커버.

## 3. 수행 단계 (구현계획서는 승인 후 별도 상세화)

1. **근본원인 계측** — `typeset.rs` 표 분할 경로에 eprintln 삽입. 씬구성표
   32×11 분할 시 `find_break_row` 입력/출력, `avail_for_rows`, 각 행 height,
   `used` 누적값을 덤프. page2 가 17행을 택하는 정확한 분기 확정.
   재현: script-ai 가 생성한 SBS HWP (`.artifacts/mbc-footer-2b/sbs-test.hwp`)
   또는 SBS 편집본 직접. `rhwp dump-pages` / `export-pdf`.
2. **수정** — 확정된 분기만 좁게 교정. find_break_row 가 페이지에 들어가는
   행 수를 정확히 산정하도록 (행 높이 누적 / avail 비교). `used=0` 누적 누락이
   원인이면 누적 경로 수정.
3. **회귀 검증** — 골든: 씬구성표 클립 해소 확인. 본문 표(fe5fee75 가 고친
   케이스) 회귀 없음 확인. `pdf/` 권위 자료 대비 page count drift 확인.
   script-ai MBC/SBS 매트릭스 재렌더.

## 4. 비-목표 / 범위

- 표 분할 일반 로직만. 씬구성표 외 다른 표 구조 변경 금지.
- 외부 API/출력 계약 유지. 내부 typeset 분할 산정만 좁게 수정.
- "좋아 보임" 금지 — 행 클립 해소 + 회귀 0 을 dump-pages/PDF 로 입증.

## 5. 검증 자료

- 재현: SBS 씬구성표 32×11 (script-ai 생성 HWP).
- 회귀: 본문 거대 표(MBC/KBS/SBS 편집본), `pdf/` 한컴 2022 PDF.
- 산출: 수정 전/후 dump-pages, export-pdf PNG.
