#!/usr/bin/env bash
# One full goldens run of a variant (all units, --jobs 4), not timed. It runs from the
# variant's tree (see tree() in env.sh) and --blesses into a fresh copy of that tree's
# crates/goldens/expected at $S/out/<variant>, so every case's complete output is kept.
#   check-run.sh <variant>   *-count variants also write each process's libm call counts
#                            to $S/counts/<variant>/<pid>.tsv
# The log, $S/results/check-<variant>.log, lists every changed case (GOLDENS_MAX_LINES).
set -u
. "$(dirname "$0")/env.sh"
v=$1
t=$(tree "$v")
mkdir -p "$S/results" "$S/reports/check-$v"
export GOLDENS_MAX_LINES=1000000
case $v in
  *-count)
    [ -e "$S/counts/$v" ] && { echo "$S/counts/$v exists; move it away first"; exit 1; }
    mkdir -p "$S/counts/$v"; export CR_COUNT_DIR=$S/counts/$v ;;
esac
[ -e "$S/out/$v" ] && { echo "$S/out/$v exists; move it away first"; exit 1; }
mkdir -p "$S/out"; cp -r "$t/crates/goldens/expected" "$S/out/$v"
# From the tree's root: some units read upstream/TestFiles relative to it.
cd "$t"
start=$(date +%s.%N)
nice -n 10 taskset -c "$CPUS" "$S/bins/goldens-$v" --all --jobs 4 --bless --goldens-dir "$S/out/$v" \
  --report-dir "$S/reports/check-$v" > "$S/results/check-$v.log" 2>&1
code=$?
end=$(date +%s.%N)
echo "exit=$code wall=$(echo "$end - $start" | bc)s" >> "$S/results/check-$v.log"
tail -2 "$S/results/check-$v.log"
