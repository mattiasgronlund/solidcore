#!/usr/bin/env bash
# The timed part of the Linux run of inari issue #2. Only on a quiet host (after the inari
# queue's row 115). Everything off CPUs 11 and 23; single-threaded runs pinned to CPU 5.
#   phase micro:  speed and interval programs, every configuration, one at a time
#   phase whole:  full goldens runs, --jobs 4: baseline, cr-main, cr-ca6, baseline again
#   phase heavy:  five heavy units back to back, two rounds, variants interleaved
set -u
S=/tmp/claude-1000/-home-mattias-git-tangency-labs-solidcore/a56740ec-5566-4006-b816-32aa7cfb9983/scratchpad
R=$S/results/timing
mkdir -p $R
log() { echo "$(date +%T) $*" | tee -a $R/timing.log; }
snap() { log "load: $(cut -d' ' -f1-3 /proc/loadavg)  governor: $(cat /sys/devices/system/cpu/cpufreq/policy5/scaling_governor)"; }

cd /home/mattias/git/tangency-labs/solidcore-inari-eval
phase=${1:-all}

if [ "$phase" = micro ] || [ "$phase" = all ]; then
  snap
  for c in main-gen-default main-native main-gen-native ca6-gen-default ca6-native ca6-gen-native; do
    log "speed $c"
    taskset -c 5 $S/bins/$c/speed > $R/speed-$c.tsv
    log "interval $c"
    taskset -c 5 $S/bins/$c/interval > $R/interval-$c.txt
  done
  snap
fi

if [ "$phase" = whole ] || [ "$phase" = all ]; then
  for v in baseline cr-main cr-ca6 baseline; do
    n=$(ls $R | grep -c "^whole-$v-") ; n=$((n + 1))
    mkdir -p $R/whole-$v-$n
    snap
    log "whole $v round $n"
    start=$(date +%s.%N)
    taskset -c 0-10,12-22 $S/bins/goldens-$v --all --goldens --jobs 4 \
      --report-dir $R/whole-$v-$n > $R/whole-$v-$n/stdout.log 2>&1
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
        taskset -c 5 $S/bins/goldens-$v --unit $u --goldens > $R/heavy-$u-$v-$round.log 2>&1
        code=$?
        end=$(date +%s.%N)
        log "heavy $u $v round $round exit=$code wall=$(echo "$end - $start" | bc)"
      done
    done
  done
  snap
fi
log "timing $phase done"
