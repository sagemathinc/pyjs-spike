#!/bin/bash
# Full comparison on one machine: wall time and peak RSS for every system.
export PATH=$HOME/.cargo/bin:$HOME/.local/bin:$PATH
cd "$(dirname "$0")"
M=./measure.sh
MS=../../bench/modsym
PYPY=$(uv python find pypy3.11)
PY=/tmp/menv/bin/python
echo "# $(nproc) x $(grep -m1 'model name' /proc/cpuinfo | cut -d: -f2) | commit $(git rev-parse --short HEAD)"
for a in "2003 97" "5077 97" "10007 97"; do
  echo "== N q = $a"
  $M "CPython 3.14" python3.14 $MS/modsym.py $a
  $M "PyPy 3.11" $PYPY $MS/modsym.py $a
  $M "pyjs" node ../../dist/src/cli.js $MS/modsym.py $a
  $M "Rust port (dense)" $MS/rust/target/release/modsym $a
  $M "engine CLI 1 thread" ../target/release/modsym-engine $a --threads 1
  $M "engine CLI 8 threads" ../target/release/modsym-engine $a --threads 8
  $M "engine via CPython" $PY run_py.py $a 0
  $M "engine WASM (Node)" node run_wasm.cjs $a
done
for a in "20011 97"; do
  echo "== N q = $a (engine only)"
  $M "engine CLI 1 thread" ../target/release/modsym-engine $a --threads 1
  $M "engine CLI 8 threads" ../target/release/modsym-engine $a --threads 8
  $M "engine via CPython" $PY run_py.py $a 0
  $M "engine WASM (Node)" node run_wasm.cjs $a
done
echo "== thread scaling, N=20011 q=97"
for t in 1 2 4 8; do $M "engine CLI $t threads" ../target/release/modsym-engine 20011 97 --threads $t; done
echo "== exploration: 16 prime levels near 5000, T_2, from Python threads"
LEVELS="4999 5003 5009 5011 5021 5023 5039 5051 5059 5077 5081 5087 5099 5101 5107 5113"
for w in 1 8; do /usr/bin/time -f "  wall %e s, maxrss %M KB" $PY explore_py.py $w $LEVELS; done
