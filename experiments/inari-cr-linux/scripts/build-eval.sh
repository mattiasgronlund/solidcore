#!/usr/bin/env bash
# Builds the eval crate for both inari commits in three configurations, on disk (not ZRAM).
#   gen-default: default x86-64 target, force-generic (what solidcore's CI would have to use)
#   native:      -Ctarget-cpu=native, inari's default backend (fma)
#   gen-native:  -Ctarget-cpu=native, force-generic (portable backend, with FMA)
set -u
S=/tmp/claude-1000/-home-mattias-git-tangency-labs-solidcore/a56740ec-5566-4006-b816-32aa7cfb9983/scratchpad
export CARGO_BUILD_JOBS=2
for commit in main ca6; do
  crate=$S/cr-eval; [ "$commit" = ca6 ] && crate=$S/cr-eval-ca6
  for cfg in gen-default native gen-native; do
    t=$S/t/$commit-$cfg
    case $cfg in
      gen-default) flags=""; feat="--features force-generic" ;;
      native) flags="-Ctarget-cpu=native"; feat="" ;;
      gen-native) flags="-Ctarget-cpu=native"; feat="--features force-generic" ;;
    esac
    echo "=== $commit $cfg"
    (cd "$crate" && RUSTFLAGS="$flags" CARGO_BUILD_BUILD_DIR=$t/build nice -n 10 taskset -c 0-10,12-22 \
      cargo +1.99.1 build --release --target-dir "$t" $feat 2>&1 | grep -E "^(error|warning)|Finished" )
    mkdir -p $S/bins/$commit-$cfg
    for b in accuracy speed interval; do cp "$t/release/$b" $S/bins/$commit-$cfg/ || echo "missing $b"; done
  done
done
echo done
