#!/usr/bin/env bash
# Builds the goldens binary with every libm call routed to inari::cr (portable backend, the
# default x86-64 target, as solidcore's CI would build it), in three variants:
#   cr-count  inari main (174092e), counting calls (for the pass/fail run and the call counts)
#   cr-main   inari main (174092e)
#   cr-ca6    inari ca6faf9 (the macOS run's commit)
set -eu
S=/tmp/claude-1000/-home-mattias-git-tangency-labs-solidcore/a56740ec-5566-4006-b816-32aa7cfb9983/scratchpad
W=/home/mattias/git/tangency-labs/solidcore-inari-eval
export CARGO_BUILD_JOBS=2
b() { # feature, inari dir, output name
  sed -i "s#^inari = { path = \"../inari-[a-z0-9]*\"#inari = { path = \"../$2\"#" $S/cr-libm-shim/Cargo.toml
  grep '^inari' $S/cr-libm-shim/Cargo.toml
  (cd $W && nice -n 10 taskset -c 0-10,12-22 cargo build --profile goldens -p goldens --features "$1" 2>&1 | grep -E "^(error|warning: unused)|Finished|Compiling (inari|cr-libm|goldens)")
  cp $W/bin/goldens/goldens $S/bins/goldens-$3
  echo "built $3"
}
b cr-libm-count inari-main cr-count
b cr-libm-generic inari-main cr-main
b cr-libm-generic inari-ca6 cr-ca6
sed -i "s#^inari = { path = \"../inari-[a-z0-9]*\"#inari = { path = \"../inari-main\"#" $S/cr-libm-shim/Cargo.toml
echo done
