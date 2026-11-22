"""The timed runs' tables, from $S/results/timing/ (written by timing.sh).

usage: python3 -I summarize-timing.py <S>/results/timing

Prints two tab-separated tables:
  whole   each full run, in the order it ran: wall time and the sum of unit times, both as
          the goldens tool's summary line gives them; each one's change against the mean of
          its reference's runs; the reference; and the number of units that failed
  heavy   each heavy unit: wall time per round per variant, the change of the mean against
          the reference's mean, and the reference
The reference of a det-* variant (the solidcraft#15405 tree) is det-libm, of any other
variant the unmodified baseline. Only the steps after the last NOTE line of timing.log count
(an earlier pass was invalid).
"""

import os
import re
import sys
from statistics import mean

r = sys.argv[1]
with open(os.path.join(r, "timing.log")) as f:
    log = f.read().split("NOTE:")[-1]


def reference(v):
    return "det-libm" if v.startswith("det-") else "baseline"


whole = []  # (variant, wall, sum, failed), in the order they ran
for m in re.finditer(r"whole (\S+) round (\d+) exit=\d+ wall=", log):
    with open(os.path.join(r, f"whole-{m.group(1)}-{m.group(2)}", "stdout.log")) as f:
        out = f.read()
    s = re.search(r"wall ([\d.]+) s, sum of unit times ([\d.]+) s", out)
    failed = re.search(r"goldens: \d+ unit\(s\), \d+ passed, (\d+) failed", out)
    whole.append((m.group(1), float(s.group(1)), float(s.group(2)), int(failed.group(1))))

heavy = {}  # unit -> variant -> [wall per round]
for m in re.finditer(r"heavy (\S+) (\S+) round (\d) exit=\d+ wall=([\d.]+)", log):
    heavy.setdefault(m.group(1), {}).setdefault(m.group(2), []).append(float(m.group(4)))


def change(x, base):
    return f"{100 * (x / base - 1):+.1f}%"


print("whole\tvariant\twall_s\tunit_times_s\twall_change\tunit_times_change\tagainst\tunits_failed")
for v, w, s, failed in whole:
    ref = reference(v)
    bw = mean(x[1] for x in whole if x[0] == ref)
    bs = mean(x[2] for x in whole if x[0] == ref)
    print(f"whole\t{v}\t{w:.1f}\t{s:.1f}\t{change(w, bw)}\t{change(s, bs)}\t{ref}\t{failed}")
print()
print("heavy\tunit\tvariant\tround_1_s\tround_2_s\tchange_of_mean\tagainst")
for u, vs in heavy.items():
    for v, t in vs.items():
        ref = reference(v)
        base = mean(vs[ref])
        print(f"heavy\t{u}\t{v}\t" + "\t".join(f"{x:.2f}" for x in t) + f"\t{change(mean(t), base)}\t{ref}")
