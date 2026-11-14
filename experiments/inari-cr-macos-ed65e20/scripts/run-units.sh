#!/bin/bash
# Units alone, alternating variants within each round; wall time from bash's `time`.
cd "$(dirname "$0")/wt-cr2"
R=../r-ed65e20
TIMEFORMAT=%R
run() { # out-file, variants, units...
  local out=$1 vs=$2; shift 2
  for round in 1 2; do for u in "$@"; do for v in $vs; do
    t=$( { time nice -n 10 bin/variants/$v --unit $u --goldens > /dev/null 2>&1; } 2>&1 )
    printf "%s\t%s\t%s\t%s\n" $round $u $v $t | tee -a $R/$out
  done; done; done
}
uptime
run heavy-units.tsv "base cr-main cr-ca6" corpus.parity offset.offset_executive_p2 blend.fillet_executive_p3 blend.feature_executive surface.surface_p7
uptime
run ivl-units.tsv "base ivl-inari ivl-generic" topology.brep_local_operation offset.offset_executive_p2 blend.feature_executive curve.curve_p9 intersect.topology_solver_p2
uptime
