"""Every case whose output differs from its golden, bit for bit, in the format of the macOS
run's changed-cases-*.jsonl.

usage: python3 -I changed-cases.py <expected dir> <S>

Reads the blessed outputs $S/out/{baseline,cr-main,cr-ca6}/<unit>.jsonl, written by
check-run.sh, and the logs $S/results/check-<variant>.log. Writes to $S/results/:
  changed-cases-glibc.jsonl    the cases where the unmodified binary differs from the golden
  changed-cases-cr.jsonl       the same for cr at inari main
  changed-cases-cr-ca6.jsonl   the same for cr at inari ca6faf9
Each line holds the unit, the case, the records `golden`, `glibc` and `cr` (in the ca6 file,
`cr` is the ca6faf9 record), and two flags for the file's own variant:
  goldens_changed        the goldens tool listed the case as changed (or added, removed or
                         broken) in that variant's log
  beyond_1e-9_relative   some number differs by more than 1e-9 relative, or anything else
                         differs; without the units' abs_floors(), as in the macOS files
Two records are equal when their lines are equal as text, so a sign of zero counts too.
"""

import json
import os
import re
import sys

VARIANTS = [("glibc", "baseline"), ("cr", "cr-main"), ("cr-ca6", "cr-ca6")]


def read_unit(path):
    """case name -> (line text, record); the first line is the file's header."""
    cases = {}
    with open(path) as f:
        for i, line in enumerate(f):
            line = line.rstrip("\n")
            if i == 0 or not line:
                continue
            rec = json.loads(line)
            cases[rec["case"]] = (line, rec)
    return cases


def listed_changes(log):
    """unit -> names the goldens tool listed as changed, added, removed or broken."""
    out, unit = {}, None
    with open(log) as f:
        for line in f:
            m = re.match(r"(PASS|FAIL) (\S+) cases=", line)
            if m:
                unit = m.group(2)
                continue
            m = re.match(r"  (?:changed (.*) \(\d+ differences\)|added (.*)|removed (.*)|broken ([^:]*):)", line)
            if m and unit:
                out.setdefault(unit, set()).add(next(g for g in m.groups() if g is not None))
    return out


def beyond(a, b):
    if isinstance(a, float) or isinstance(b, float):
        if isinstance(a, (int, float)) and isinstance(b, (int, float)):
            return abs(a - b) > 1e-9 * max(abs(a), abs(b))
        return True
    if isinstance(a, dict) and isinstance(b, dict):
        return a.keys() != b.keys() or any(beyond(a[k], b[k]) for k in a)
    if isinstance(a, list) and isinstance(b, list):
        return len(a) != len(b) or any(beyond(x, y) for x, y in zip(a, b))
    return a != b


def main():
    expected, s = sys.argv[1], sys.argv[2]
    units = sorted(f[: -len(".jsonl")] for f in os.listdir(expected) if f.endswith(".jsonl"))
    logs = {v: listed_changes(f"{s}/results/check-{v}.log") for _, v in VARIANTS}
    files = {name: open(f"{s}/results/changed-cases-{name}.jsonl", "w") for name, _ in VARIANTS}
    counts = {name: [0, set(), 0] for name, _ in VARIANTS}  # cases, units, beyond
    for unit in units:
        golden = read_unit(f"{expected}/{unit}.jsonl")
        outs = {v: read_unit(f"{s}/out/{v}/{unit}.jsonl") for _, v in VARIANTS}
        names = sorted(set(golden).union(*outs.values()))
        for case in names:
            g = golden.get(case)
            for name, v in VARIANTS:
                o = outs[v].get(case)
                if (g and g[0]) == (o and o[0]):
                    continue
                rec = lambda x: x[1] if x else None
                cr = outs["cr-ca6" if name == "cr-ca6" else "cr-main"].get(case)
                far = g is None or o is None or beyond(g[1], o[1])
                row = {
                    "unit": unit,
                    "case": case,
                    "goldens_changed": case in logs[v].get(unit, set()),
                    "beyond_1e-9_relative": far,
                    "golden": rec(g),
                    "glibc": rec(outs["baseline"].get(case)),
                    "cr": rec(cr),
                }
                files[name].write(json.dumps(row, separators=(",", ":")) + "\n")
                c = counts[name]
                c[0] += 1
                c[1].add(unit)
                c[2] += far
    for name, _ in VARIANTS:
        files[name].close()
        n, u, far = counts[name]
        print(f"{name}: {n} cases in {len(u)} units differ from the golden bit for bit; {far} beyond 1e-9 relative")


main()
