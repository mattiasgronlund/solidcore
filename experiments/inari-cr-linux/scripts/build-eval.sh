#!/usr/bin/env bash
# Builds the eval programs (accuracy, speed, interval) for each cr commit into
# $S/bins/<commit>-<config>/. Commits: main (inari 6a9fa7a), ca6 (inari ca6faf9) and
# new (fushimi $NEW_COMMIT); give a subset as arguments. Configurations:
#   gen-default: the default x86-64 target with force-generic, what solidcore's CI would build
#                (on this target, the portable backend has no FMA)
#   native:      -Ctarget-cpu=native, the crate's default backend (fma)
#   gen-native:  -Ctarget-cpu=native with force-generic (the portable backend, with FMA)
#   default:     new only: the default x86-64 target, no force-generic. main and ca6 do not
#                build so; at new, the kernel picks its FMA copy at run time. This is the
#                build solidcore would ship.
set -eu
. "$(dirname "$0")/env.sh"
[ $# -eq 0 ] && set -- main ca6 new
for commit in "$@"; do
  case $commit in
    main) crate=$E/cr-eval; cfgs="gen-default native gen-native" ;;
    ca6) crate=$E/cr-eval-ca6; cfgs="gen-default native gen-native" ;;
    new) crate=$E/cr-eval-new; cfgs="gen-default native gen-native default" ;;
    *) echo "unknown commit $commit"; exit 1 ;;
  esac
  for cfg in $cfgs; do
    t=$S/t/$commit-$cfg
    case $cfg in
      gen-default) flags=""; feat="--features force-generic" ;;
      native) flags="-Ctarget-cpu=native"; feat="" ;;
      gen-native) flags="-Ctarget-cpu=native"; feat="--features force-generic" ;;
      default) flags=""; feat="" ;;
    esac
    echo "=== $commit $cfg"
    (cd "$crate" && RUSTFLAGS="$flags" CARGO_BUILD_BUILD_DIR=$t/build nice -n 10 taskset -c "$CPUS" \
      cargo +1.99.1 build --release --locked --target-dir "$t" $feat)
    mkdir -p "$S/bins/$commit-$cfg"
    for b in accuracy speed interval; do cp "$t/release/$b" "$S/bins/$commit-$cfg/"; done
  done
done
echo "build-eval done"
