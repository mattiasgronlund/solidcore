"""Compares the Linux changed-case files with the macOS run's, case by case.

usage: python3 -I compare-macos.py <S>/results <macOS results dir>

Both runs list each case whose output differs from the same golden (solidcore ed65e20), with
the golden's, the platform libm's and cr's records. A case missing from a file equals the
golden there. So cr gives the same output on both platforms, for every case, exactly when
both files name the same cases with equal cr records. Records are compared re-serialised
with sorted keys, so a sign of zero counts. Also reports how the platform libms compare
with each other and with cr.
"""

import json
import sys


def load(path, libm):
    out = {}
    with open(path) as f:
        for line in f:
            r = json.loads(line)
            out[(r["unit"], r["case"])] = {k: json.dumps(r[k], sort_keys=True) for k in ("golden", libm, "cr")}
    return out


lin_dir, mac_dir = sys.argv[1], sys.argv[2]
lin_cr = load(f"{lin_dir}/changed-cases-cr.jsonl", "glibc")
mac_cr = load(f"{mac_dir}/changed-cases-cr-main.jsonl", "apple_libm")
lin_libm = load(f"{lin_dir}/changed-cases-glibc.jsonl", "glibc")
mac_libm = load(f"{mac_dir}/changed-cases-apple-libm.jsonl", "apple_libm")

only_lin = sorted(set(lin_cr) - set(mac_cr))
only_mac = sorted(set(mac_cr) - set(lin_cr))
both = sorted(set(lin_cr) & set(mac_cr))
golden_differs = [k for k in both if lin_cr[k]["golden"] != mac_cr[k]["golden"]]
cr_differs = [k for k in both if lin_cr[k]["cr"] != mac_cr[k]["cr"]]
print(f"cr: Linux {len(lin_cr)} cases, macOS {len(mac_cr)}; in both {len(both)}")
print(f"cr: only on Linux {len(only_lin)}, only on macOS {len(only_mac)}")
print(f"cr: golden records differ in {len(golden_differs)} shared cases (should be 0)")
print(f"cr: cr records differ in {len(cr_differs)} shared cases")
for k in (only_lin + only_mac + cr_differs)[:20]:
    print("  ", *k)


def output(libm_file, cr_file, libm, k):
    """The platform libm's output for case k, or None when it equals the golden."""
    for d in (libm_file, cr_file):
        if k in d:
            return None if d[k][libm] == d[k]["golden"] else d[k][libm]
    return None


keys = set(lin_libm) | set(mac_libm) | set(lin_cr) | set(mac_cr)
libms_differ = [k for k in keys if output(lin_libm, lin_cr, "glibc", k) != output(mac_libm, mac_cr, "apple_libm", k)]
print(f"libm: glibc differs from the golden in {len(lin_libm)} cases, Apple's libm in {len(mac_libm)}")
print(f"libm: glibc and Apple's libm give different outputs in {len(libms_differ)} cases")
