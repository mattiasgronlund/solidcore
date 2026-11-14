#!/usr/bin/env bash
# The timed runs; only on a quiet host. Results go to $S/results/timing/.
# Full goldens runs use $CPUS; single-threaded runs are pinned to CPU $PIN.
#   phase micro:  speed and interval, every configuration, one at a time; and, when
#                 $ACOS_ARGS names an acos-args.tsv, `speed --acos-args` on it
#   phase whole:  full goldens runs, --jobs 4: baseline, cr-main, cr-ca6, baseline again
#   phase heavy:  five heavy units back to back, two rounds, variants interleaved
set -u
. "$(dirname "$0")/env.sh"
R=$S/results/timing
mkdir -p "$R"
log() { echo "$(date +%T) $*" | tee -a "$R/timing.log"; }
snap() { log "load: $(cut -d' ' -f1-3 /proc/loadavg)  governor: $(cat /sys/devices/system/cpu/cpufreq/policy$PIN/scaling_governor)"; }

cd "$W"
phase=${1:-all}

if [ "$phase" = micro ] || [ "$phase" = all ]; then
  snap
  for c in main-gen-default main-native main-gen-native ca6-gen-default ca6-native ca6-gen-native; do
    log "speed $c"
    taskset -c "$PIN" "$S/bins/$c/speed" > "$R/speed-$c.tsv"
    if [ -n "${ACOS_ARGS:-}" ]; then
      log "speed --acos-args $c"
      taskset -c "$PIN" "$S/bins/$c/speed" --acos-args "$ACOS_ARGS" > "$R/speed-acos-args-$c.tsv"
    fi
    log "interval $c"
    taskset -c "$PIN" "$S/bins/$c/interval" > "$R/interval-$c.txt"
  done
  snap
fi

if [ "$phase" = whole ] || [ "$phase" = all ]; then
  for v in baseline cr-main cr-ca6 baseline; do
    n=$(ls "$R" | grep -c "^whole-$v-") ; n=$((n + 1))
    mkdir -p "$R/whole-$v-$n"
    snap
    log "whole $v round $n"
    start=$(date +%s.%N)
    taskset -c "$CPUS" "$S/bins/goldens-$v" --all --goldens --jobs 4 \
      --report-dir "$R/whole-$v-$n" > "$R/whole-$v-$n/stdout.log" 2>&1
    code=$?
    end=$(date +%s.%N)
    log "whole $v round $n exit=$code wall=$(echo "$end - $start" | bc)"
  done
fi

if [ "$phase" = heavy ] || [ "$phase" = all ]; then
  snap
  for round in 1 2; do
    for u in corpus.parity offset.offset_executive_p2 blend.fillet_executive_p3 blend.feature_executive surface.surface_p7; do
      for v in baseline cr-main cr-ca6; do
        start=$(date +%s.%N)
        taskset -c "$PIN" "$S/bins/goldens-$v" --unit "$u" --goldens > "$R/heavy-$u-$v-$round.log" 2>&1
        code=$?
        end=$(date +%s.%N)
        log "heavy $u $v round $round exit=$code wall=$(echo "$end - $start" | bc)"
      done
    done
  done
  snap
fi
log "timing $phase done"
