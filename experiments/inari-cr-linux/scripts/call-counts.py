"""Sums the shim's per-process call counts ($S/counts/<pid>.tsv, from check-run.sh cr-count)
into one table: TOTAL first, then each function with a non-zero count, most calls first.

usage: python3 -I call-counts.py <S>/counts > call-counts.tsv
"""

import collections
import os
import sys

total = collections.Counter()
for name in os.listdir(sys.argv[1]):
    with open(os.path.join(sys.argv[1], name)) as f:
        for line in f:
            fn, n = line.split("\t")
            if fn != "unused":
                total[fn] += int(n)
print(f"TOTAL\t{sum(total.values())}")
for fn, n in total.most_common():
    if n:
        print(f"{fn}\t{n}")
