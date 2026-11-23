#!/usr/bin/env bash
# Runs the timed phases, one after another (default: all, timing.sh's three), with a load
# sampler beside it:
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
# The phases of timing.sh, and:
#   perf        perf-stat.sh (needs kernel.perf_event_paranoid <= 2)
#   wasm-micro  wasm-micro.sh speed (needs $ACOS_ARGS)
#   wasm-whole  wasm-goldens.sh whole for each <variant>:<engine> of $WASM_WHOLE, in order
#   wasm-heavy  wasm-goldens.sh heavy of the variants $WASM_HEAVY, once per engine of
#               $WASM_HEAVY_ENGINES
WASM_WHOLE=${WASM_WHOLE:-det-libm-wasm:wasmtime det-cr-010-wasm:wasmtime det-libm-wasm:wasmtime}
WASM_HEAVY=${WASM_HEAVY:-det-libm-wasm det-cr-010-wasm}
WASM_HEAVY_ENGINES=${WASM_HEAVY_ENGINES:-wasmtime}
for phase in "$@"; do
  case $phase in
    perf) bash "$E/scripts/perf-stat.sh" || code=$? ;;
    wasm-micro) bash "$E/scripts/wasm-micro.sh" speed || code=$? ;;
    wasm-whole)
      for ve in $WASM_WHOLE; do
        bash "$E/scripts/wasm-goldens.sh" whole "${ve%:*}" "${ve#*:}" || code=$?
      done ;;
    wasm-heavy)
      for eng in $WASM_HEAVY_ENGINES; do
        ENGINE=$eng bash "$E/scripts/wasm-goldens.sh" heavy $WASM_HEAVY || code=$?
      done ;;
    *) bash "$E/scripts/timing.sh" "$phase" || code=$? ;;
  esac
done
kill $sampler
echo "timed-run $* exit=$code"
