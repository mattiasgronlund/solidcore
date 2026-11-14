#!/bin/bash
# Timed runs, one after another (ed65e20). Whole runs, then the heavy units alone.
cd "$(dirname "$0")/wt-cr2"
R=../r-ed65e20; mkdir -p $R
w() { # label, variant
  echo "== whole $1 $(date +%T)"
  nice -n 10 bin/variants/$2 --all --jobs 4 --goldens > $R/goldens-$1.log 2>&1
  tail -1 $R/goldens-$1.log
}
w base-1 base
w cr-main cr-main
w cr-ca6 cr-ca6
w base-2 base
for round in 1 2; do
  for u in corpus.parity offset.offset_executive_p2 blend.fillet_executive_p3 blend.feature_executive surface.surface_p7; do
    for v in base cr-main cr-ca6; do
      t=$( { /usr/bin/time -p nice -n 10 bin/variants/$v --unit $u --goldens > /dev/null 2>&1; } 2>&1 | awk '/^real/{print $2}')
      printf "%s\t%s\t%s\t%s\n" $round $u $v $t | tee -a $R/heavy-units.tsv
    done
  done
done
echo "== done $(date +%T)"
