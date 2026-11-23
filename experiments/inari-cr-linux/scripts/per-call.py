"""The micro programs' tables, from $S/results/timing/ (written by timing.sh micro).

usage: python3 -I per-call.py <S>/results/timing

Prints three tab-separated tables, one row per eval build (<commit>-<build>) that has files:
  speed       for acos asin atan atan2 sin cos tan exp ln pow (sin and cos on [-7, 7]): ns
              per call of glibc, the libm crate and cr, and cr's ratio to each (the columns
              glibc_ns, libm_ns, cr_ns, ratio and cr_vs_libm of speed-<build>.tsv)
  acos-args   the `all` row of speed-acos-args-<build>.tsv, the same columns
  interval    interval-<build>.txt: ns per dot product for f64, solidcore's Ivl and
              inari::Interval
"""

import os
import re
import sys

r = sys.argv[1]
FUNCTIONS = ["acos", "asin", "atan", "atan2", "sin", "cos", "tan", "exp", "ln", "pow"]
SKIP_RANGE = {"sin": "[-1e4, 1e4]", "cos": "[-1e4, 1e4]"}
ORDER = ["main", "ca6", "new", "v010"]
CONFIGS = ["default", "gen-default", "native", "gen-native"]
builds = [
    f"{c}-{b}" for c in ORDER for b in CONFIGS if os.path.exists(os.path.join(r, f"speed-{c}-{b}.tsv"))
]


def rows(path):
    with open(path) as f:
        head = f.readline().rstrip("\n").split("\t")
        for line in f:
            yield dict(zip(head, line.rstrip("\n").split("\t")))


cols = ["glibc_ns", "libm_ns", "cr_ns", "ratio", "cr_vs_libm"]
print("speed\tbuild\tfunction\trange\t" + "\t".join(cols))
for b in builds:
    got = {}
    for row in rows(os.path.join(r, f"speed-{b}.tsv")):
        if row["function"] in FUNCTIONS and SKIP_RANGE.get(row["function"]) != row["range"]:
            got[row["function"]] = row
    for fn in FUNCTIONS:
        row = got[fn]
        print(f"speed\t{b}\t{fn}\t{row['range']}\t" + "\t".join(row[c] for c in cols))
print()
print("acos-args\tbuild\targs\t" + "\t".join(cols))
for b in builds:
    path = os.path.join(r, f"speed-acos-args-{b}.tsv")
    if os.path.exists(path):
        row = next(x for x in rows(path) if x["bucket"] == "all")
        print(f"acos-args\t{b}\t{row['args']}\t" + "\t".join(row[c] for c in cols))
print()
print("interval\tbuild\tf64_ns\tivl_ns\tinari_interval_ns")
for b in builds:
    with open(os.path.join(r, f"interval-{b}.txt")) as f:
        ns = re.findall(r"([\d.]+) ns", f.read())
    print(f"interval\t{b}\t" + "\t".join(ns))
