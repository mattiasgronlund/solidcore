"""Every case whose output differs from its golden, bit for bit, in the format of the macOS
run's changed-cases-*.jsonl.

usage: python3 -I changed-cases.py <expected dir> <S> [<key>=<variant> <name>=<variant>...]

The first pair is the unmodified libm, each further pair a cr variant. The default is the
Nov 14 run: glibc=baseline cr=cr-main cr-ca6=cr-ca6. The rerun of 2026-11-22 used
  glibc=baseline cr-new=cr-new        against solidcore ed65e20's goldens
  libm=det-libm det-cr=det-cr         against the goldens committed in solidcraft#15405

Reads the blessed outputs $S/out/<variant>/<unit>.jsonl, written by check-run.sh, and the
logs $S/results/check-<variant>.log. Writes $S/results/changed-cases-<key or name>.jsonl:
the cases where that variant differs from the golden. Each line holds the unit, the case,
the records `golden`, `<key>` (the unmodified libm's) and `cr` (the file's own variant's; in
the unmodified libm's file, the first cr variant's), and two flags for the file's own variant:
  goldens_changed        the goldens tool listed the case as changed (or added, removed or
                         broken) in that variant's log
  beyond_1e-9_relative   some number differs by more than 1e-9 relative, or anything else
                         differs; without the units' abs_floors(), as in the macOS files
Two records are equal when their lines are equal as text, so a sign of zero counts too.

It prints, per file, the cases, the units, the cases beyond 1e-9 relative and the cases the
tool listed, then the same counts split in two:
  float-only   both records exist and differ only in numbers inside `values`
  other        an integer, the topology, the status (`deferred`, `error`) or the shape of
               `values` differs, or the case exists on one side only
"""

import json
import os
import re
import sys

NOV14 = ["glibc=baseline", "cr=cr-main", "cr-ca6=cr-ca6"]


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
            m = re.match(r"(PASS|FAIL|BLESSED) (\S+) cases=", line)
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


def same_shape(a, b):
    """The same structure, with a number wherever the other has one."""
    num = (int, float)
    if isinstance(a, num) and isinstance(b, num) and not isinstance(a, bool) and not isinstance(b, bool):
        return True
    if isinstance(a, dict) and isinstance(b, dict):
        return a.keys() == b.keys() and all(same_shape(a[k], b[k]) for k in a)
    if isinstance(a, list) and isinstance(b, list):
        return len(a) == len(b) and all(same_shape(x, y) for x, y in zip(a, b))
    return a == b


def float_only(g, o):
    if g is None or o is None:
        return False
    rest = lambda r: {k: v for k, v in r.items() if k != "values"}
    return rest(g) == rest(o) and same_shape(g.get("values"), o.get("values"))


def main():
    expected, s = sys.argv[1], sys.argv[2]
    pairs = [a.split("=", 1) for a in (sys.argv[3:] or NOV14)]
    (ref_key, ref), crs = pairs[0], pairs[1:]
    first_cr = crs[0][1]
    units = sorted(f[: -len(".jsonl")] for f in os.listdir(expected) if f.endswith(".jsonl"))
    logs = {v: listed_changes(f"{s}/results/check-{v}.log") for _, v in pairs}
    files = {name: open(f"{s}/results/changed-cases-{name}.jsonl", "w") for name, _ in pairs}
    # cases, units, beyond, listed; then the same four for float-only and for the others
    counts = {name: {"all": [0, set(), 0, 0], "float-only": [0, set(), 0, 0], "other": [0, set(), 0, 0]} for name, _ in pairs}
    for unit in units:
        golden = read_unit(f"{expected}/{unit}.jsonl")
        outs = {v: read_unit(f"{s}/out/{v}/{unit}.jsonl") for _, v in pairs}
        names = sorted(set(golden).union(*outs.values()))
        for case in names:
            g = golden.get(case)
            for name, v in pairs:
                o = outs[v].get(case)
                if (g and g[0]) == (o and o[0]):
                    continue
                rec = lambda x: x[1] if x else None
                cr = outs[first_cr if v == ref else v].get(case)
                far = g is None or o is None or beyond(g[1], o[1])
                listed = case in logs[v].get(unit, set())
                row = {
                    "unit": unit,
                    "case": case,
                    "goldens_changed": listed,
                    "beyond_1e-9_relative": far,
                    "golden": rec(g),
                    ref_key: rec(outs[ref].get(case)),
                    "cr": rec(cr),
                }
                files[name].write(json.dumps(row, separators=(",", ":")) + "\n")
                kind = "float-only" if float_only(rec(g), rec(o)) else "other"
                for c in (counts[name]["all"], counts[name][kind]):
                    c[0] += 1
                    c[1].add(unit)
                    c[2] += far
                    c[3] += listed
    for name, v in pairs:
        files[name].close()
        for kind, (n, u, far, listed) in counts[name].items():
            print(
                f"{name} ({v}) {kind}: {n} cases in {len(u)} units differ from the golden bit for bit;"
                f" {far} beyond 1e-9 relative; {listed} listed by the goldens tool"
            )


main()
