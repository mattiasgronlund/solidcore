"""The wasm and perf runs' tables, from $S/results/timing/ (written by wasm-goldens.sh,
wasm-micro.sh and perf-stat.sh under timed-run.sh).

usage: python3 -I summarize-wasm.py <S>/results/timing

Prints tab-separated tables:
  wasm-whole   each wasm whole run, in the order it ran: wall, the sum of unit times, each
               one's change against the mean of det-libm-wasm's runs on the same engine, the
               units that did not exit 0, and the sum of unit times against the native twin's
               (the variant without -wasm, the same units, mean of its whole runs' times.tsv)
  wasm-unit    per unit and variant, the mean over runs of each engine and of the native twin,
               and each engine's ratio to native (one row per unit; for a chart or a sort)
  wasm-heavy   each heavy unit: wall per round per variant and engine, the change of the mean
               against det-libm-wasm on that engine, and the ratio to the native twin's heavy
               runs (timing.sh heavy)
  wasm-micro   for acos asin atan atan2 sin cos tan exp ln pow (sin and cos on [-7, 7]): ns per
               call of std, the libm crate and cr, and cr's ratio to each, per run
  wasm-acos    the `all` row of each run's acos-args table
  perf         each perf-stat.sh counter per variant (mean of three runs), and the
               instructions per cycle
Only the steps after the last NOTE line of timing.log count.
"""

import os
import re
import sys
from statistics import mean

r = sys.argv[1]
with open(os.path.join(r, "timing.log")) as f:
    log = f.read().split("NOTE:")[-1]


def change(x, base):
    return f"{100 * (x / base - 1):+.1f}%"


def native_twin(v):
    return v.removesuffix("-wasm")


def units_tsv(path):
    out = {}
    with open(path) as f:
        for line in f:
            u, code, wall = line.rstrip("\n").split("\t")
            out[u] = (int(code), float(wall))
    return out


# Native whole runs: variant -> [unit -> seconds]
native_runs = {}
for m in re.finditer(r"(?<!wasm )whole (\S+) round (\d+) exit=\d+ wall=", log):
    path = os.path.join(r, f"whole-{m.group(1)}-{m.group(2)}", "times.tsv")
    if os.path.exists(path):
        with open(path) as f:
            t = {u: int(ms) / 1000 for u, ms in (line.split("\t") for line in f if line.strip())}
        native_runs.setdefault(m.group(1), []).append(t)

whole = []  # (variant, engine, round, wall, units)
for m in re.finditer(r"wasm whole (\S+) (\S+) round (\d+) wall=([\d.]+)", log):
    v, eng, n, wall = m.group(1), m.group(2), m.group(3), float(m.group(4))
    whole.append((v, eng, n, wall, units_tsv(os.path.join(r, f"wasm-whole-{v}-{eng}-{n}", "units.tsv"))))

print("wasm-whole\tvariant\tengine\twall_s\tunit_times_s\twall_change\tunit_times_change\tunits_not_0\tnative_unit_times_s\tratio_to_native")
for v, eng, n, wall, units in whole:
    s = sum(w for _, w in units.values())
    ref = [x for x in whole if x[0] == "det-libm-wasm" and x[1] == eng]
    bw = mean(x[3] for x in ref)
    bs = mean(sum(w for _, w in x[4].values()) for x in ref)
    bad = sum(1 for c, _ in units.values() if c != 0)
    nat = native_runs.get(native_twin(v))
    ns = mean(sum(t[u] for u in units) for t in nat) if nat else float("nan")
    print(f"wasm-whole\t{v}\t{eng}\t{wall:.1f}\t{s:.1f}\t{change(wall, bw)}\t{change(s, bs)}\t{bad}\t{ns:.1f}\t{s / ns:.2f}")
print()

engines = sorted({x[1] for x in whole})
print("wasm-unit\tunit\tvariant\tnative_s\t" + "\t".join(f"{e}_s\t{e}_ratio" for e in engines))
for v in sorted({x[0] for x in whole}):
    nat = native_runs.get(native_twin(v), [])
    for u in sorted({u for x in whole if x[0] == v for u in x[4]}):
        ns = mean(t[u] for t in nat) if nat else float("nan")
        cols = []
        for e in engines:
            ws = [x[4][u][1] for x in whole if x[0] == v and x[1] == e and u in x[4]]
            w = mean(ws) if ws else float("nan")
            cols.append(f"{w:.2f}\t{w / ns:.2f}")
        print(f"wasm-unit\t{u}\t{v}\t{ns:.2f}\t" + "\t".join(cols))
print()

native_heavy = {}  # (unit, variant) -> [wall]
for m in re.finditer(r"(?<!wasm )heavy (\S+) (\S+) round (\d) exit=\d+ wall=([\d.]+)", log):
    native_heavy.setdefault((m.group(1), m.group(2)), []).append(float(m.group(4)))
heavy = {}  # (unit, engine) -> variant -> [wall]
for m in re.finditer(r"wasm heavy (\S+) (\S+) (\S+) round (\d) exit=\d+ wall=([\d.]+)", log):
    heavy.setdefault((m.group(2), m.group(1)), {}).setdefault(m.group(3), []).append(float(m.group(5)))
print("wasm-heavy\tunit\tengine\tvariant\tround_1_s\tround_2_s\tchange_of_mean\tnative_mean_s\tratio_to_native")
for (u, eng), vs in heavy.items():
    for v, t in vs.items():
        base = mean(vs["det-libm-wasm"]) if "det-libm-wasm" in vs else float("nan")
        nat = native_heavy.get((u, native_twin(v)))
        nm = mean(nat) if nat else float("nan")
        print(f"wasm-heavy\t{u}\t{eng}\t{v}\t" + "\t".join(f"{x:.2f}" for x in t)
              + f"\t{change(mean(t), base)}\t{nm:.2f}\t{mean(t) / nm:.2f}")
print()

FUNCTIONS = ["acos", "asin", "atan", "atan2", "sin", "cos", "tan", "exp", "ln", "pow"]
SKIP_RANGE = {"sin": "[-1e4, 1e4]", "cos": "[-1e4, 1e4]"}
RUNS = ["native", "wasip1-wasmtime", "wasip1-simd-wasmtime", "wasip1-node", "wasip1-simd-node",
        "unknown-node", "unknown-simd-node"]
m_dir = os.path.join(r, "wasm-micro")


def rows(path):
    with open(path) as f:
        head = f.readline().rstrip("\n").split("\t")
        for line in f:
            yield dict(zip(head, line.rstrip("\n").split("\t")))


cols = ["std_ns", "libm_ns", "cr_ns", "ratio", "cr_vs_libm"]
runs = [x for x in RUNS if os.path.exists(os.path.join(m_dir, f"speed-{x}.tsv"))]
print("wasm-micro\trun\tfunction\trange\t" + "\t".join(cols))
for x in runs:
    got = {}
    for row in rows(os.path.join(m_dir, f"speed-{x}.tsv")):
        if row["function"] in FUNCTIONS and SKIP_RANGE.get(row["function"]) != row["range"]:
            got[row["function"]] = row
    for fn in FUNCTIONS:
        row = got[fn]
        print(f"wasm-micro\t{x}\t{fn}\t{row['range']}\t" + "\t".join(row[c] for c in cols))
print()
print("wasm-acos\trun\targs\t" + "\t".join(cols))
for x in runs:
    path = os.path.join(m_dir, f"speed-acos-args-{x}.tsv")
    if os.path.exists(path):
        row = next(y for y in rows(path) if y["bucket"] == "all")
        print(f"wasm-acos\t{x}\targs={row['args']}\t" + "\t".join(row[c] for c in cols))
print()

EVENTS = ["cycles:u", "instructions:u", "branches:u", "branch-misses:u", "L1-icache-load-misses:u",
          "stalled-cycles-frontend:u"]
perf = sorted(f for f in os.listdir(r) if f.startswith("perf-") and f.endswith(".txt"))
if perf:
    print("perf\tunit_variant\t" + "\t".join(EVENTS) + "\tipc\tseconds")
for name in perf:
    with open(os.path.join(r, name)) as f:
        text = f.read()
    got = {}
    for e in EVENTS:
        m = re.search(r"^\s*([\d,]+|<not supported>|<not counted>)\s+" + re.escape(e) + r"\b", text, re.M)
        got[e] = m.group(1).replace(",", "") if m else ""
    sec = re.search(r"([\d.]+) \+- [\d.]+ seconds time elapsed", text)
    ipc = (f"{int(got['instructions:u']) / int(got['cycles:u']):.3f}"
           if got["instructions:u"].isdigit() and got["cycles:u"].isdigit() else "")
    print(f"perf\t{name[5:-4]}\t" + "\t".join(got[e] for e in EVENTS) + f"\t{ipc}\t{sec.group(1) if sec else ''}")
