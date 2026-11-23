#!/usr/bin/env bash
# usage: perf-stat.sh [variant...]   (default: baseline cr-010 cr-010-gen det-libm det-cr-010)
# Hardware counters of one unit (default surface.surface_p7; $UNIT), each variant run alone
# from its tree, pinned to CPU $PIN, three times (perf stat -r 3), user space only. Only on a
# quiet host, and with kernel.perf_event_paranoid at 2 or lower (the user sets it).
# Results go to $S/results/timing/perf-<unit>-<variant>.txt.
set -u
. "$(dirname "$0")/env.sh"
UNIT=${UNIT:-surface.surface_p7}
EVENTS=cycles:u,instructions:u,branches:u,branch-misses:u,L1-icache-load-misses:u,stalled-cycles-frontend:u
R=$S/results/timing
mkdir -p "$R"
[ "$(cat /proc/sys/kernel/perf_event_paranoid)" -le 2 ] \
  || { echo "kernel.perf_event_paranoid is $(cat /proc/sys/kernel/perf_event_paranoid); it must be 2 or lower"; exit 1; }
log() { echo "$(date +%T) $*" | tee -a "$R/timing.log"; }
[ $# -eq 0 ] && set -- baseline cr-010 cr-010-gen det-libm det-cr-010
for v in "$@"; do
  cd "$(tree "$v")"
  log "perf stat $UNIT $v"
  taskset -c "$PIN" perf stat -r 3 -e "$EVENTS" -o "$R/perf-$UNIT-$v.txt" -- \
    "$S/bins/goldens-$v" --unit "$UNIT" --goldens > "$R/perf-$UNIT-$v.log" 2>&1
  log "perf stat $UNIT $v exit=$?"
done
