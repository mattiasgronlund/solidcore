#!/bin/bash
# Untimed runs: Ivl operation counts and acos arguments (ed65e20).
cd "$(dirname "$0")/wt-cr2"
R=../r-ed65e20; mkdir -p $R/ivl-counts $R/acos-args
echo "== ivl-count $(date +%T)"
IVL_COUNTS=$PWD/$R/ivl-counts nice -n 10 bin/variants/ivl-count --all --jobs 4 --goldens > $R/goldens-ivl-count.log 2>&1
tail -2 $R/goldens-ivl-count.log
echo "== acos-args $(date +%T)"
CR_COUNTS=$PWD/$R/acos-args nice -n 10 bin/variants/acos-args --all --jobs 4 --goldens > $R/goldens-acos-args.log 2>&1
tail -2 $R/goldens-acos-args.log
echo "== done $(date +%T)"
