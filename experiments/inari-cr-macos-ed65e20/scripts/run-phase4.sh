#!/bin/bash
# Ivl through inari::Interval (ed65e20): the units that use Ivl, alone, alternating; then
# complete outputs of the two inari variants.
cd "$(dirname "$0")/wt-cr2"
R=../r-ed65e20; mkdir -p $R/full
for round in 1 2; do
  for u in topology.brep_local_operation offset.offset_executive_p2 blend.feature_executive curve.curve_p9 intersect.topology_solver_p2; do
    for v in base ivl-inari ivl-generic; do
      t=$( { /usr/bin/time -p nice -n 10 bin/variants/$v --unit $u --goldens > /dev/null 2>&1; } 2>&1 | awk '/^real/{print $2}')
      printf "%s\t%s\t%s\t%s\n" $round $u $v $t | tee -a $R/ivl-units.tsv
    done
  done
done
for v in ivl-inari ivl-generic; do
  echo "== bless $v $(date +%T)"
  mkdir -p $R/full/$v; cp -R crates/goldens/expected/. $R/full/$v/
  nice -n 10 bin/variants/$v --all --jobs 4 --goldens --bless --goldens-dir $PWD/$R/full/$v > $R/bless-$v.log 2>&1
  tail -1 $R/bless-$v.log
done
echo "== done $(date +%T)"
