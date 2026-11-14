#!/usr/bin/env bash
# Builds the eval programs (accuracy, speed, interval) for both inari commits, in three
# configurations each, into $S/bins/<commit>-<config>/ (commit: main or ca6):
#   gen-default: the default x86-64 target with force-generic, what solidcore's CI would build
#                (on this target, the portable backend has no FMA)
#   native:      -Ctarget-cpu=native, inari's default backend (fma)
#   gen-native:  -Ctarget-cpu=native with force-generic (the portable backend, with FMA)
set -eu
. "$(dirname "$0")/env.sh"
for commit in main ca6; do
  crate=$E/cr-eval; [ "$commit" = ca6 ] && crate=$E/cr-eval-ca6
  for cfg in gen-default native gen-native; do
    t=$S/t/$commit-$cfg
    case $cfg in
      gen-default) flags=""; feat="--features force-generic" ;;
      native) flags="-Ctarget-cpu=native"; feat="" ;;
      gen-native) flags="-Ctarget-cpu=native"; feat="--features force-generic" ;;
    esac
    echo "=== $commit $cfg"
    (cd "$crate" && RUSTFLAGS="$flags" CARGO_BUILD_BUILD_DIR=$t/build nice -n 10 taskset -c "$CPUS" \
      cargo +1.99.1 build --release --locked --target-dir "$t" $feat)
    mkdir -p "$S/bins/$commit-$cfg"
    for b in accuracy speed interval; do cp "$t/release/$b" "$S/bins/$commit-$cfg/"; done
  done
done
echo "build-eval done"
