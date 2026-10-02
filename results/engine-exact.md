# modsym-engine, iteration 2: proven results, a fast kernel, batch exploration

Question: is "certified, parallel, batch-first Rust engines with Python/JS
bindings" feasible for research math?  This iteration takes weight-2 modular
symbols for Gamma0(N) (sign +1) from "fast mod p" to "proven over Z, in
batch, with estimates", and checks everything against independent sources.

Engine commit: 41984cb plus the estimator change in this commit.  Machines:
bench-1 (GCP t2d-standard-8, AMD EPYC 7B13) and the CoCalc dev host (same CPU
model, shared).  The timing caveat below matters.

## New this iteration

* `exact_charpoly(N, q)`: the characteristic polynomial of T_q over Z, by CRT
  over primes below 2^31.  Every prime's space has to have the dimension the
  genus and cusp formulas predict, otherwise it is rejected.
* A proven coefficient bound.  Cusp eigenvalues are real with |a| <= 2 sqrt(q)
  (Deligne), and Eisenstein eigenvalues are chi(q) + psi(q) q with
  |a| <= 1 + q.  The bound is then sharpened, still rigorously: the first
  prime gives the exact sum of squares of all eigenvalues (p1^2 - 2 e2, with
  p > 2 d (1+q)^2).  Since log(1 + sqrt x) is concave, Jensen's inequality
  gives prod(1 + |a_i|) <= (1 + r)^g with r^2 = (sum over cusp a^2) / g.
  That is 24-25% fewer primes than the Deligne-only bound.  The status is
  "proven" only if the result is monic, q + 1 is a root, and the largest
  coefficient is within the bound.
* `batch_exact(levels, q)`: parallel over levels; a bad level returns an
  error and the batch keeps going.  `estimate(N, q)` predicts the dimension,
  the primes, the memory and the time before running anything.
* Errors, not panics: q | N, q not prime, or p out of range give `Err` in
  Rust and `ValueError` in Python.
* Python (PyO3): `hecke_charpoly`, `charpoly_exact` (coefficients as Python
  ints), `batch_exact`, `level_data`, `commute`, `estimate`.  All of them
  release the GIL.

## Correctness evidence

| check | result |
|---|---|
| dimension: formula (genus + cusp orbits under x -> -x - 1) vs computed, N = 11..3000 plus 4096, 5000, 9240, 10007 | 2994 / 2994 agree |
| exact charpoly vs Sage `ModularSymbols(N,2,sign=1).hecke_matrix(q).charpoly()`, 11 levels up to N = 2310 (dim 592, 1015-bit coefficients) | 11 / 11 identical (CLI and Python) |
| mod-p hash vs the pure-Python reference (modsym.py) | identical for 389/2 and 997/3 |
| rational roots of T_2 vs a_2 of every elliptic curve in Cremona's tables, all 1210 squarefree odd N < 3000, with oldform multiplicities | no curve missing anywhere.  249 levels have extra roots (e.g. N = 113), all in even multiplicity: Galois-conjugate newforms with rational a_2, as expected |
| 1495 odd levels below 3000, exact T_2 | all "proven" |

## Speed

Kernel: the Hessenberg charpoly mod p was doing 64-bit `%` in every inner
step.  It now stores u32 entries and uses Shoup multiplication (a
precomputed floor(w 2^32 / p)), so the inner loops have no division and
vectorize.  The leaf kernels are compiled twice and AVX2 is picked at run
time: closures run by rayon do not inherit `#[target_feature]`, so dispatch
happens at the leaves.  Result: one prime, one thread, N = 10007 (dim 835),
2501 -> 276 ms.

Same machine against Sage (dev host, one thread; the engine was still using
the Deligne-only bound here, so it now needs about 25% fewer primes):

| N, q | dim | Sage default | Sage, ZZ matrix + LinBox | engine, proven |
|---|---|---|---|---|
| 2310, 13 | 592 | 44.0 s | 12.6 s | 11.2 s |
| 5000, 3 | 754 | 113.4 s | - | 14.5 s |
| 10007, 2 | 835 | 162.1 s | 121.9 s (FLINT 155 s) | 25.8 s |

LinBox's integer charpoly stops early (heuristically), while the engine
proves its bound.  With 8 threads and the sharper bound: 2310 takes 1.49 s
and 10007 takes 2.56 s (dev host).

Larger levels (bench-1, 8 threads, measured before the sharper bound):

| run | wall | peak RSS |
|---|---|---|
| exact 20011/2, dim 1668, 105 primes (now 79) | 37.8 s | 707 MB |
| exact 30011/2, dim 2502, 157 primes (~103 predicted now) | 931 s (see caveat) | 1.59 GB |
| one prime 60013/2, dim 5001 | 82 s | 533 MB |

Exploration demo (`engine/bench/explore_exact.py`): exact T_2 for all 1495
odd N < 3000 in one `batch_exact` call took 17.6 s wall (149 MB).  The
engine's one-thread estimate was 197 s.  Factoring with python-flint and
writing Parquet took another 10 s.

One observation the data suggests at once: at prime levels, the largest
irreducible factor of the cuspidal T_2 charpoly is on average 58% of the
cusp space (never below 38%).  That is consistent with the Atkin-Lehner
split, with each half usually one Galois orbit, a Maeda-type pattern.

## What did not work (kept for the record)

* Interleaving 8 primes in one matrix of [u32; 8] lanes (hand-written AVX2):
  correct, half the memory, but 1.5x slower below dim ~1000 and only equal at
  dim 1668.  Lanes do not reduce bytes per prime, and the single-prime kernel
  is already 8-wide across columns.  Removed.
* Large dims are memory bound: the Hessenberg reduction is matrix-vector
  work, at about 3.6 ns per element once the matrix leaves L3.  A single
  prime at dim 5001 on 8 threads runs only ~1.5x faster than on one.  The fix
  is a blocked (BLAS-3) charpoly, not more SIMD.

## Caveat: benchmark machines drift

Around 16:15 UTC bench-1 became 3-5x slower on the same binary and input,
with no CPU steal.  A small probe showed identical scalar CPU speed, but
reads of a 4 MB cache-resident buffer dropped to 4.5 GB/s (the dev host
reads 16.5 GB/s, itself varying from 6 to 17).  Most likely a noisy neighbor
is thrashing the shared L3.  The 30011 exact run overlapped that window, so
its 931 s is not a clean number.  Every comparison above was run A/B in the
same window.  From now on, run the probe before timing anything.

## Feasibility verdict

Yes.  The Rust core gives proven results, checked four independent ways
(formulas, Sage, the Python reference, Cremona).  On one thread it is
3.9-7.8x faster than Sage's default path and 1.1-4.7x faster than Sage's
fastest (LinBox, heuristic) path; it scales across levels, can be driven from Python with
bigints and Parquet, and says what a job will cost before running it.  Next,
in order of research value:

1. Newform decomposition: split the space by Hecke operators and return a_p
   for each Galois orbit.  This is what researchers actually query, and it
   avoids giant full charpolys.
2. A blocked charpoly or decomposition-first strategy for dim > 2000.
3. Stream or sparsify the generator coordinates: memory is dominated by
   8 * gens * dim bytes per prime.
4. A certificate: export the residues, primes and bound derivation so a
   checker (and later Lean) can verify a "proven" status independently.
5. Weight k > 2 and characters, and the same API from WASM.
