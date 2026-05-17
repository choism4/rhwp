#!/usr/bin/env python3
"""대사 중복 디텍터 (task_dup_render).

본문 거대 표가 페이지 분할될 때 셀 문단의 줄 범위가 페이지 N 하단과 페이지 N+1
상단에 겹쳐 출력되는(=대사 중복) 결함을 프로그래매틱하게 검출한다.

rhwp 를 `RHWP_LINE_RANGE_DUMP=1 export-svg` 로 실행하면 분할 행 셀 문단마다
stderr 에 LRD 라인을 찍는다 (table_partial.rs). 그 라인을 파싱해서, 동일 셀
문단의 연속 페이지 fragment 줄 범위가 겹치는지 검사한다.

LRD 필드 (탭 구분, "LRD" 다음 13개):
  section para control cell_idx cp_idx start_line end_line
  start_row end_row is_continuation is_split_start is_split_end is_repeated_header

키 = (section, para, control, cell_idx, cp_idx) — 표 하나의 셀 문단을 유일 식별.
emission 순서 = pagination(페이지) 순서.

판정 (연속 fragment 쌍 A[a,b) B[c,d), 빈 범위 제외):
  DUPLICATE   : c < b and d > a   (구간 겹침 — 결함)
  ADJACENT-OK : c == b            (정상 분할)
  GAP-BUG     : c > b             (줄 누락 — 별도 경고)

is_repeated_header=true 레코드는 제목행 반복 출력이므로 제외.

사용:
  scan_dup.py <rhwp_bin> <scratch_dir> <report_path> <list_file>
  list_file = HWP 경로 한 줄에 하나 (공백 포함 경로 안전).
exit 0 iff DUPLICATE count == 0.
"""
import os
import subprocess
import sys
import shutil
from collections import defaultdict


def render_lrd(rhwp_bin, scratch, hwp_path, timeout=300):
    """한 HWP 를 export-svg 렌더하고 LRD 라인 리스트를 emission 순서로 반환.

    반환: (records, status)  status in {"ok", "render-fail", "timeout"}
    record = dict(section,para,control,cell,cp,sl,el,srow,erow,cont,sstart,send,rhdr,emit)
    """
    for f in os.listdir(scratch):
        p = os.path.join(scratch, f)
        try:
            os.remove(p) if os.path.isfile(p) else shutil.rmtree(p)
        except OSError:
            pass
    env = dict(os.environ, RHWP_LINE_RANGE_DUMP="1")
    try:
        proc = subprocess.run(
            [rhwp_bin, "export-svg", hwp_path, "-o", scratch],
            env=env, capture_output=True, timeout=timeout,
        )
    except subprocess.TimeoutExpired:
        return [], "timeout"
    if proc.returncode != 0:
        return [], "render-fail"
    records = []
    emit = 0
    for line in proc.stderr.decode("utf-8", "replace").splitlines():
        if not line.startswith("LRD\t"):
            continue
        parts = line.split("\t")
        if len(parts) != 18:
            continue
        try:
            vals = parts[1:]
            rec = {
                "section": int(vals[0]), "para": int(vals[1]),
                "control": int(vals[2]), "cell": int(vals[3]),
                "cp": int(vals[4]), "sl": int(vals[5]), "el": int(vals[6]),
                "srow": int(vals[7]), "erow": int(vals[8]),
                "cont": vals[9] == "true", "sstart": vals[10] == "true",
                "send": vals[11] == "true", "rhdr": vals[12] == "true",
                "cellrow": int(vals[13]), "rowspan": int(vals[14]),
                "soff": float(vals[15]), "slim": float(vals[16]),
                "emit": emit,
            }
        except (ValueError, IndexError):
            continue
        records.append(rec)
        emit += 1
    return records, "ok"


def scan_records(records):
    """LRD 레코드에서 중복/갭 검출. 반환: (duplicates, gaps)."""
    groups = defaultdict(list)
    for r in records:
        if r["rhdr"]:
            continue  # 제목행 반복 출력 — 정상, 제외
        key = (r["section"], r["para"], r["control"], r["cell"], r["cp"])
        groups[key].append(r)
    duplicates = []
    gaps = []
    for key, recs in groups.items():
        recs.sort(key=lambda x: x["emit"])
        for i in range(len(recs) - 1):
            a, b = recs[i], recs[i + 1]
            ar, br = (a["sl"], a["el"]), (b["sl"], b["el"])
            if ar[0] >= ar[1] or br[0] >= br[1]:
                continue  # 빈 범위
            c, d = br
            lo, hi = ar
            if c < hi and d > lo:
                duplicates.append((key, a, b, max(c, lo), min(hi, d)))
            elif c > hi:
                gaps.append((key, a, b))
    return duplicates, gaps


def main():
    if len(sys.argv) != 5:
        print(__doc__)
        sys.exit(2)
    rhwp_bin, scratch, report_path, list_file = sys.argv[1:5]
    with open(list_file, encoding="utf-8") as fh:
        files = [ln.rstrip("\n") for ln in fh if ln.strip()]
    missing = [f for f in files if not os.path.isfile(f)]
    if missing:
        print(f"ERROR: {len(missing)} 파일 없음, 예: {missing[0]}")
        sys.exit(2)
    os.makedirs(scratch, exist_ok=True)

    total_dup = 0
    total_gap = 0
    total_pairs = 0
    render_fail = []
    lines = []
    lines.append("=== 대사 중복 스캔 (task_dup_render) ===")
    lines.append(f"rhwp: {rhwp_bin}")
    lines.append(f"파일 수: {len(files)}")
    lines.append("")

    for idx, hwp in enumerate(files):
        name = os.path.basename(hwp)
        records, status = render_lrd(rhwp_bin, scratch, hwp)
        if status != "ok":
            render_fail.append((name, status))
            lines.append(f"[{idx+1}/{len(files)}] RENDER-FAIL({status}): {name}")
            print(f"[{idx+1}/{len(files)}] RENDER-FAIL({status}): {name}", flush=True)
            continue
        dups, gaps = scan_records(records)
        total_dup += len(dups)
        total_gap += len(gaps)
        total_pairs += len(records)
        flag = f"DUP={len(dups)}" if dups else "ok"
        if gaps:
            flag += f" GAP={len(gaps)}"
        lines.append(f"[{idx+1}/{len(files)}] {flag}: {name} ({len(records)} LRD)")
        print(f"[{idx+1}/{len(files)}] {flag}: {name}", flush=True)
        for key, a, b, ov_lo, ov_hi in dups:
            lines.append(
                f"    DUPLICATE key=s{key[0]}/p{key[1]}/c{key[2]}/cell{key[3]}/cp{key[4]}"
                f"  cell(row={a['cellrow']},span={a['rowspan']})"
                f"  pageA[{a['sl']},{a['el']}) frag{a['srow']}-{a['erow']}"
                f" soff={a['soff']:.1f} slim={a['slim']:.1f}"
                f"  pageB[{b['sl']},{b['el']}) frag{b['srow']}-{b['erow']}"
                f" soff={b['soff']:.1f} slim={b['slim']:.1f}"
                f"  overlap_lines[{ov_lo},{ov_hi})"
            )
        for key, a, b in gaps:
            lines.append(
                f"    GAP-BUG key=s{key[0]}/p{key[1]}/c{key[2]}/cell{key[3]}/cp{key[4]}"
                f"  pageA[{a['sl']},{a['el']}) pageB[{b['sl']},{b['el']})"
            )

    lines.append("")
    lines.append("=== 요약 ===")
    lines.append(f"스캔 파일: {len(files)}")
    lines.append(f"RENDER-FAIL: {len(render_fail)}")
    for n, s in render_fail:
        lines.append(f"  - {n}: {s}")
    lines.append(f"검사 LRD 레코드: {total_pairs}")
    lines.append(f"GAP-BUG: {total_gap}")
    lines.append(f"DUPLICATE: {total_dup}")
    lines.append("게이트: " + ("PASS (DUPLICATE 0)" if total_dup == 0 else "FAIL"))

    with open(report_path, "w", encoding="utf-8") as fh:
        fh.write("\n".join(lines) + "\n")
    print("\n".join(lines[-8:]))
    print(f"\n리포트: {report_path}")
    sys.exit(0 if total_dup == 0 else 1)


if __name__ == "__main__":
    main()
