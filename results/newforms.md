# Rational newforms: Cremona's tables to conductor 9999, from scratch

`engine/modsym/src/newforms.rs` finds every rational newform of weight 2 and
level N (equivalently, by modularity, every isogeny class of elliptic
curves over Q of conductor N) and computes its a_p.  It is pure Rust, uses
no polynomial factoring and no external libraries, and is about 350 lines.

## Method

- Work in the dual of the sign +1 modular symbols space modulo
  ell = 2147483629.
- A rational newform has integer a_q with |a_q| <= 2 sqrt(q) (Hasse), so
  split by T_q eigenvalues using only these finitely many integers. For
  the first prime, take the integer roots of the charpoly (the fast
  Hessenberg kernel) and kernels only for them. Then refine prime by
  prime on the subspaces. Eisenstein eigenvalues (chi(q) + psi(q) q) are
  never in the Hasse range.
- The old part is known exactly: every rational newform of level M | N,
  M < N (memoized) occurs with multiplicity d(N/M). A subspace whose
  dimension equals the old multiplicity of its eigenvalues is dropped at
  once, and so is a charpoly root whose multiplicity equals it (no kernel
  is computed then). Splitting continues only where something is
  unexplained. A 1-dimensional joint eigenspace is a Hecke eigenvector of
  multiplicity one, hence new.
- a_p for many p, from one functional psi on the Manin-symbol generators:
  a_p = psi(T_p x) / psi(x), one sum over Heilbronn matrices per prime.
  |a_p| <= 2 sqrt(p) < ell / 2 makes the lift exact.

## Results

| check | result |
|---|---|
| every level 11 <= N <= 9999 vs Cremona's tables (ecdata aplist: a_p for p < 100 per isogeny class) | **38,042 rational newforms, all 38,042 classes matched, 0 levels disagree** |
| modularity cross-check, levels <= 2000: newform a_p (p < 400) vs point counts on Cremona's curves by `sagebrush-ap` | 5,854 newforms, **440,551 (newform, p) pairs agree**, 7.3 s, 80 MB |
| `cargo test`: every level <= 300 vs a Cremona excerpt, known a_p of 11a and 37a/37b | pass |

The whole table to 9999 takes **854 s on 16 threads (AMD EPYC 7B13), peak
1.3 GB** in one process, with all levels sharing the memo of lower levels.
Levels to 2000 take 7.0 s. Single levels: N = 5077 (dim 423) 0.07 s;
N = 9240 (dim 2336, 63 divisors, 36 newforms) 18 s and 120 MB, including
all its lower levels. From Python, a_p for all 1,228 primes below 10^4 of
the level-5077 newform takes 0.22 s, and they agree with point counts on
5077a1.

## What changed in the engine on the way

- **No dense coordinate table.** Measured fill-in: after sparse
  elimination in creation order the relations keep **2.0 nonzeros per
  row** at every level tried. The dense table of every generator in the
  quotient basis was 154x (N = 960) to 1764x (N = 9240) larger than the
  information in it. `Space` now stores only the sparse relations.
  A functional extends to all generators in O(m) (`Space::extend`), and
  Hecke matrices are assembled 64 columns at a time. This cut peak memory
  at N = 9240 from 235 to 120 MB, and the T_17 split from 23 s to 3 s.
  Memory is now O(m + dim^2) instead of O(m dim), which also helps the
  exact charpolys (where the table was most of the 1.6 GB at N = 30011).
- **Fast dense elimination mod p** (`linalg::rref_mod`): u32 entries,
  Shoup row operations with the AVX2 kernels, rows in parallel. It replaced
  a naive version that did a 128-bit `%` per element (N = 5077: 0.42 to
  0.07 s).
- All 21 earlier modsym tests (Sage charpolys, the dimension formula
  for N <= 1000, the Python reference hashes) pass unchanged on the
  sparse representation.

## Not yet

- No timing comparison with eclib (Cremona's C++) yet: that is the
  honest benchmark for this table and the next thing to measure.
- The tail of the sweep is the few largest levels (dim 1500-2500): a
  full-size charpoly and several full-size kernels each. Peak memory
  (1.3 GB) is up to 16 such levels in flight at once. Ideas: split with
  a smaller first operator space (start from the cuspidal part, or
  restrict to an Atkin-Lehner eigenspace first), and schedule big levels
  first so the tail runs in parallel.
- a_p at primes dividing N (U_p), Atkin-Lehner signs, and the curves
  themselves (periods, then Weierstrass equations), which is the rest of
  Cremona's pipeline.
- Non-rational newforms need Hecke fields and polynomial factoring.
  That is where FLINT would come in.
