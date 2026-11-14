"""The timed runs' tables, from $S/results/timing/ (written by timing.sh).

usage: python3 -I summarize-timing.py <S>/results/timing

Prints two tab-separated tables:
  whole   each full run: wall time and the sum of unit times, both as the goldens tool's
          summary line gives them; then each cr variant's change of the mean against the
          mean of the unmodified runs
  heavy   each heavy unit: wall time per round per variant, and the change of the mean
Only the steps after the last NOTE line of timing.log count (an earlier pass was invalid).
"""

import os
import re
import sys
from statistics import mean

r = sys.argv[1]
with open(os.path.join(r, "timing.log")) as f:
    log = f.read().split("NOTE:")[-1]

whole = {}  # variant -> [(wall, sum)]
for d in sorted(os.listdir(r)):
    m = re.fullmatch(r"whole-(.+)-(\d+)", d)
    if not m:
        continue
    with open(os.path.join(r, d, "stdout.log")) as f:
        s = re.search(r"wall ([\d.]+) s, sum of unit times ([\d.]+) s", f.read())
    whole.setdefault(m.group(1), []).append((float(s.group(1)), float(s.group(2))))

heavy = {}  # unit -> variant -> [wall per round]
for m in re.finditer(r"heavy (\S+) (\S+) round (\d) exit=\d+ wall=([\d.]+)", log):
    heavy.setdefault(m.group(1), {}).setdefault(m.group(2), []).append(float(m.group(4)))


def change(x, base):
    return f"{100 * (x / base - 1):+.1f}%"


print("whole\tvariant\twall_s\tunit_times_s\twall_change\tunit_times_change")
bw = mean(w for w, _ in whole["baseline"])
bs = mean(s for _, s in whole["baseline"])
for v, runs in whole.items():
    for w, s in runs:
        print(f"whole\t{v}\t{w:.1f}\t{s:.1f}\t{change(w, bw)}\t{change(s, bs)}")
print()
print("heavy\tunit\tvariant\tround_1_s\tround_2_s\tchange_of_mean")
for u, vs in heavy.items():
    base = mean(vs["baseline"])
    for v, t in vs.items():
        print(f"heavy\t{u}\t{v}\t" + "\t".join(f"{x:.2f}" for x in t) + f"\t{change(mean(t), base)}")
