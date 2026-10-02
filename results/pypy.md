# PyPy comparison

PyPy 7.3.23 (Python 3.11) on the 16-CPU development host.

## Modular symbols (`bench/modsym/modsym.py`, same source everywhere)

Total compute time; all systems produce the same characteristic polynomial.

| N | q | CPython 3.14 | pyjs | PyPy | Sage native |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 2003 | 2 | 1.36 s | 0.95 s | 0.22 s | 1.31 s |
| 5077 | 2 | 15.3 s | 10.1 s | 1.29 s | 11.3 s |
| 5077 | 97 | 16.0 s | 10.6 s | 1.36 s | 14.6 s |
| 10007 | 2 | 129 s | 103 s | 6.6 s | 48 s |

PyPy runs this plain-Python program 7-15x faster than pyjs and 7x faster
than Sage's native implementation at N = 10007.

## pyperformance subset (warm = after ~1 s warmup, median of five)

pyjs time / PyPy time; geometric mean over 20 benchmarks about 12.7x
(unpack_sequence excluded: below the timer resolution under PyPy).

| benchmark | warm | cold |
| --- | ---: | ---: |
| go | 1.01 | 0.35 |
| pidigits | 2.33 | 2.06 |
| generators | 2.44 | 0.30 |
| meteor_contest | 2.92 | 2.66 |
| hexiom | 3.22 | 0.52 |
| nqueens | 3.64 | 1.83 |
| float | 4.81 | 4.08 |
| fannkuch | 6.49 | 5.49 |
| nbody | 8.49 | 5.18 |
| deltablue | 9.52 | 1.95 |
| spectral_norm | 13.4 | 4.48 |
| chaos | 13.7 | 0.87 |
| monte_carlo | 25.2 | 1.20 |
| raytrace | 26.5 | 1.36 |
| scimark_fft | 39.4 | 8.26 |
| richards | 44.1 | 1.20 |
| richards_super | 54.7 | 1.34 |
| sparse_mat_mult | 65.2 | 5.15 |
| scimark_sor | 142 | 10.8 |
| scimark_lu | 164 | 16.2 |

## Where pyjs's time goes on integer loops

Inner loop `A[k] = (A[k] - 12345 * B[k]) % p` (values < 2^26), ns per
iteration in steady state:

| variant | ns/op |
| --- | ---: |
| PyPy | 2.4 |
| hand-written JS, `a - floor(a/p)*p` | 4.7-7.1 |
| hand-written JS, `%` on doubles (fmod) | 16.4 |
| pyjs generated code | 46-51 |

Each generic helper costs 2-7 ns for its type and overflow checks (mul 4,
sub 2, mod 7.5, subscripting ~12), and V8 cannot hoist them because they
depend on each value.  PyPy's tracing JIT specializes the loop to machine
integers and checks types once.  Closing that gap needs type
specialization in the pyjs compiler (guarded int/float code with a
deoptimizing fallback), i.e. a Python-level JIT on top of V8's.
