#!/bin/bash
# Complete outputs: --bless into a copy of expected/ (ed65e20).
cd "$(dirname "$0")/wt-cr2"
R=../r-ed65e20; mkdir -p $R/full
for v in base cr-main cr-ca6; do
  echo "== bless $v $(date +%T)"
  mkdir -p $R/full/$v; cp -R crates/goldens/expected/. $R/full/$v/
  nice -n 10 bin/variants/$v --all --jobs 4 --goldens --bless --goldens-dir $PWD/$R/full/$v > $R/bless-$v.log 2>&1
  tail -1 $R/bless-$v.log
done
echo "== done $(date +%T)"
