#!/usr/bin/env bash
# Item 3 of inari#2 (comment 6865494739): the Linux micro programs on macOS.
# Each build runs alone, one program at a time. macOS has no taskset; nothing pins.
set -u
S=/private/tmp/claude-501/-Users-mattias-git-tangency-labs-solidcore/a6592295-3d3d-4c66-bef2-b6510308e33d/scratchpad
ARGS=$S/wt-cr2/experiments/inari-cr-macos-ed65e20/results/acos-args.tsv
R=$S/item3/results${ROUND:+-$ROUND}
mkdir -p "$R"
log() { echo "$(date +%T) $*" | tee -a "$R/timing.log"; }
snap() { log "load: $(sysctl -n vm.loadavg)"; }
snap
for c in main-aarch64 main-generic ca6-aarch64 ca6-generic; do
  b=$S/item3/bins/$c
  log "speed $c"
  "$b/speed" > "$R/speed-$c.tsv"
  log "speed --acos-args $c"
  "$b/speed" --acos-args "$ARGS" > "$R/speed-acos-args-$c.tsv"
  log "interval $c"
  "$b/interval" > "$R/interval-$c.txt"
done
snap
log "item3 done"
