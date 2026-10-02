# bench-1: 8x  AMD EPYC 7B13, Python 3.14.8, node v26.5.1, commit 354dc76
| benchmark | CPython warm ms | pyjs warm ms | warm ratio | cold ratio |
| --- | ---: | ---: | ---: | ---: |
| chaos | 54.1 | 56.7 | 1.05 | 1.62 |
| deltablue | 3.13 | 2.23 | 0.71 | 8.71 |
| fannkuch | 359 | 471 | 1.31 | 1.36 |
| float | 72.2 | 50.6 | 0.70 | 0.98 |
| generators | 38.2 | 46.2 | 1.21 | 1.71 |
| go | 110 | 115 | 1.05 | 1.68 |
| hexiom | 5.63 | 4.25 | 0.76 | 4.34 |
| meteor_contest | 98.5 | 205 | 2.08 | 2.28 |
| nbody | 95.9 | 101 | 1.06 | 1.24 |
| nqueens | 79.0 | 75.5 | 0.96 | 1.26 |
| pidigits | 203 | 361 | 1.77 | 1.89 |
| raytrace | 254 | 257 | 1.01 | 1.30 |
| richards | 38.6 | 59.2 | 1.53 | 2.25 |
| richards_super | 44.2 | 77.3 | 1.75 | 2.88 |
| scimark_fft | 297 | 186 | 0.63 | 0.77 |
| scimark_lu | 114 | 128 | 1.12 | 1.49 |
| scimark_monte_carlo | 62.5 | 61.7 | 0.99 | 1.12 |
| scimark_sor | 105 | 93.9 | 0.89 | 1.01 |
| scimark_sparse_mat_mult | 4.70 | 5.58 | 1.19 | 2.53 |
| spectral_norm | 91.5 | 43.5 | 0.48 | 0.66 |
| unpack_sequence | 0.02 | 0.01 | 0.33 | 4.38 |

geometric mean warm ratio over 21: 0.99
