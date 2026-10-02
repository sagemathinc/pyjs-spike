# sagebrush-ap: a Rust port of smalljac's genus-1 strategy

`engine/ap` computes the traces of Frobenius a_p = p + 1 - #E(F_p) of an
elliptic curve over Q at every prime up to a bound. It follows the strategy
of Drew Sutherland's smalljac (Kedlaya-Sutherland), written from scratch
in about 600 lines of Rust, in an afternoon:

- arithmetic mod p < 2^62 in Montgomery form, with branchless add/sub
  and an interleaved batch inversion;
- for each prime, a point on E or on its quadratic twist from one
  Legendre symbol (x0 with d = f(x0) gives (d x0, d^2) on
  y^2 = x^3 + a d^2 x + b d^3), so no square roots are needed;
- baby-step giant-step over the Hasse interval for every t with
  (p + 1 - t) P = O, with Jacobian coordinates and batched normalization;
- the search is halved when one Legendre symbol (the cubic's
  discriminant) proves a_p is even;
- more points until exactly one a_p is consistent with all of them, so
  every answer is unique, never probabilistic; direct point counting
  below p = 1000;
- parallel over chunks of primes (one curve) or over curves (many).

## Correctness

| check | result |
|---|---|
| every a_p of 11a for p < 10^6 vs smalljac's lpdata output | identical, all 78,497 primes (also kept as a digest in `cargo test`) |
| number of good primes and sum of a_p vs smalljac: 11a to 10^8, 37a to 10^7, three more curves to 10^6 | identical everywhere (11a to 10^8: 5,761,454 primes, sum 5,982,199) |
| a_p and bad primes vs Sage for p < 20000: 11 curves of rank 0-3, torsion up to Z/4 x Z/2, CM with j = 0 and 1728, large coefficients | identical |
| baby-step giant-step vs direct counting, 1000 < p < 12000, 6 curves | identical |
| Sato-Tate moments of a_p^2/p to 10^7 | 11a: 1.000, 1.999, 4.998, 13.992 (theory 1, 2, 5, 14); 27a3 (CM): 0.999, 2.997, 9.988, 34.955 (theory 1, 3, 10, 35) |

`cargo test -p sagebrush-ap`: 9 tests, about 5 s.

## Speed against smalljac 4.1.3

Same machine (16-core AMD EPYC 7B13 dev host), curve 11a, all good primes
p <= N, run one after another. The cache probe read 25 GB/s before and after.
smalljac was timed with a small C harness around `smalljac_Lpolys`
(serial) and `smalljac_parallel_Lpolys` (its built-in fork-based
parallelism), with flags A1_ONLY | GOOD_ONLY.

| N | smalljac, 1 core | Rust, 1 thread | smalljac, 16 processes | Rust, 16 threads |
|---|---|---|---|---|
| 10^7 | 3.99 s | 4.93 s (1.23x) | 0.58 s | **0.45 s** |
| 10^8 | 42.1 s | 67.4 s (1.60x) | **4.27 s** | 6.10 s |

Many curves: a_p for 1000 curves y^2 = x^3 + a x + b at all 9592 primes
below 10^5 takes 4.24 s from Python on 16 threads, including building the
9.6 million Python tuples.

## What the gap is, and what did not work

- The gap grows with p: from 1.23x at 10^7 to 1.6x at 10^8. The
  baby-step giant-step work grows like p^(1/4) and the fixed per-prime
  work like log p, so smalljac's advantage is per group operation.
- Profiling: about 45% of the time is Montgomery multiplication, and it
  is latency-bound. Scalar multiplication and batch inversion are serial
  chains of ~12-cycle multiplications.
- Tried: affine progressions with shared inversions (6 multiplications
  per point instead of ~16). That was 20% slower at 10^7, because each extra
  inversion is a serial chain of ~33 multiplications. A progression that
  adds kB to [B..kB] also hits a doubling, which made every call fall back
  at first. Removed, but it should win at larger p if the inversions are
  amortized better.
- Next steps: interleave several primes so the serial chains overlap
  (the main lever on a latency-bound kernel); revisit affine batching for
  large p; #E mod 3 and mod 4 information to shrink the interval further;
  and p up to 2^62 is already supported by the arithmetic but is untested
  beyond 10^8.

## Interfaces

    sagebrush ap "[0,-1,1,-10,-20]" 10000000          # count, sum, moments, time
    from sagebrush import ap; ap.aplist([0,0,1,-1,0], 10**6); ap.moments(a, n)
    require("sagebrush").ap.aplist([0,0,1,-1,0], 1e6)
