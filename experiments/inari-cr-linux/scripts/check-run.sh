#!/usr/bin/env bash
# One full goldens run of a variant (all units, --jobs 4), not timed.
#   check-run.sh <variant>   --bless into a fresh copy of crates/goldens/expected at
#                            $S/out/<variant>, so every case's complete output is kept
#   check-run.sh cr-count    --goldens only, with each process's libm call counts in
#                            $S/counts/<pid>.tsv
# The log, $S/results/check-<variant>.log, lists every changed case (GOLDENS_MAX_LINES).
set -u
. "$(dirname "$0")/env.sh"
v=$1
mkdir -p "$S/results" "$S/reports/check-$v"
export GOLDENS_MAX_LINES=1000000
if [ "$v" = cr-count ]; then
  mode=(--goldens)
  mkdir -p "$S/counts"; export CR_COUNT_DIR=$S/counts
else
  [ -e "$S/out/$v" ] && { echo "$S/out/$v exists; move it away first"; exit 1; }
  mkdir -p "$S/out"; cp -r "$W/crates/goldens/expected" "$S/out/$v"
  mode=(--bless --goldens-dir "$S/out/$v")
fi
# From the tree's root: some units read upstream/TestFiles relative to it.
cd "$W"
start=$(date +%s.%N)
nice -n 10 taskset -c "$CPUS" "$S/bins/goldens-$v" --all --jobs 4 "${mode[@]}" \
  --report-dir "$S/reports/check-$v" > "$S/results/check-$v.log" 2>&1
code=$?
end=$(date +%s.%N)
echo "exit=$code wall=$(echo "$end - $start" | bc)s" >> "$S/results/check-$v.log"
tail -2 "$S/results/check-$v.log"
