#!/usr/bin/env bash
# One full goldens check (all units, --jobs 4), not timed for the report: stdout to
# results/check-<variant>.log, unit times to reports/check-<variant>/times.tsv.
# With the cr-count variant, call counts land in counts/<pid>.tsv (one per process).
set -u
S=/tmp/claude-1000/-home-mattias-git-tangency-labs-solidcore/a56740ec-5566-4006-b816-32aa7cfb9983/scratchpad
v=$1
mkdir -p $S/results $S/reports/check-$v
if [ "$v" = cr-count ]; then mkdir -p $S/counts; export CR_COUNT_DIR=$S/counts; fi
cd /home/mattias/git/tangency-labs/solidcore-inari-eval
start=$(date +%s.%N)
nice -n 10 taskset -c 0-10,12-22 $S/bins/goldens-$v --all --goldens --jobs 4 \
  --report-dir $S/reports/check-$v > $S/results/check-$v.log 2>&1
code=$?
end=$(date +%s.%N)
echo "exit=$code wall=$(echo "$end - $start" | bc)s" >> $S/results/check-$v.log
