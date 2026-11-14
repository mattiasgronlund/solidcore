#!/usr/bin/env bash
# Runs timing.sh, one phase after another (default: all), with a load sampler beside it:
# every 10 s, the load averages and the number of other build or count processes on the
# host go to $S/results/timing/load.log, so every timed step can be checked for quiet.
# Our own goldens binaries are named goldens-<variant> and are not counted.
set -u
. "$(dirname "$0")/env.sh"
R=$S/results/timing
mkdir -p "$R"
[ $# -eq 0 ] && set -- all
(
  while true; do
    b=$(ps -eo comm= | grep -cE '^(cargo|rustc|cc1|cc1plus|ld|ld\.lld|lld|make|coqc|rocq|rocqworker|valgrind|cachegrind.*|callgrind.*|goldens)$')
    echo "$(date +%T) load $(cut -d' ' -f1-3 /proc/loadavg) builds $b"
    sleep 10
  done
) >> "$R/load.log" 2>&1 &
sampler=$!
echo "sampler pid $sampler, timed-run pid $$, phases $*" >> "$R/load.log"
code=0
for phase in "$@"; do
  bash "$E/scripts/timing.sh" "$phase" || code=$?
done
kill $sampler
echo "timed-run $* exit=$code"
