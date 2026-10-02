# Compact exact $q$-expansions: Stein's representation, measured

For a newform orbit $A$ of dimension $k$ with Hecke field $K$:
$$a_p=\sum_{r=1}^{k}\beta_r\,c_{p,r},\qquad c_{p,r}=w_r(T_p x)\in\mathbb{Z},\qquad \beta_r\in K\ \text{fixed},$$
where $w_1,\dots,w_k$ are integral functionals spanning the dual of $A$
(written on the Manin-symbol generators) and $x$ is a fixed generator.
Each $c_{p,r}$ is one sum over Heilbronn matrices of integers. No
number-field arithmetic is needed, and everything is parallel over $p$.

## Exactness (all levels $\le 1000$)

`engine/modsym/src/integral.rs` makes the coordinates exact integers:

1. Compute the echelon basis of each orbit's dual on the generators
   modulo two primes near $2^{31}$, combine by CRT, and recover the
   rationals by rational reconstruction; a third prime checks them.
2. Scale each row to a primitive integer vector $w_r$.
3. Compute $c_{p,r}$ by integer sums.

The check needs no number field. Solve $\tau_r=\operatorname{Tr}(\beta_r)$
from $k$ primes, then require
$\operatorname{Tr}(a_p)=\sum_r c_{p,r}\tau_r$ at every prime.
**It holds for all 5,951 orbits at levels $\le 1000$ and every prime
$p<1000$.** The run takes 217 s on 16 threads, with a 282 MB peak.

## Size: bits per $a_p$, $p<1000$, $p\nmid N$

The comparison covers the 3,334 orbits of dimension 2–20 at levels
$\le 1000$ that LMFDB stores (`mf_hecke_nf`, from the read-only mirror),
matched to ours by level, dimension and traces
(`engine/bench/storage/compare.py`). An integer $x$ costs
$\lceil\log_2(|x|+1)\rceil+1$ bits.

| dim | orbits | ours, raw | + LLL | **+ HNF + LLL** | LMFDB Hecke ring | power basis | best / LMFDB | best / power |
|---|---|---|---|---|---|---|---|---|
| 2 | 1195 | 12.1 | 11.1 | **8.1** | 8.3 | 10.4 | 0.97 | 0.78 |
| 4 | 373 | 25.0 | 21.0 | **13.7** | 14.0 | 19.2 | 0.98 | 0.72 |
| 6 | 182 | 37.3 | 30.1 | **18.8** | 19.2 | 33.3 | 0.98 | 0.56 |
| 8 | 95 | 50.6 | 40.5 | **24.2** | 24.8 | 57.2 | 0.98 | 0.42 |
| 10 | 72 | 59.5 | 45.9 | **28.7** | 29.3 | 113.3 | 0.98 | 0.25 |
| 12 | 50 | 70.7 | 52.2 | **32.5** | 33.6 | 177.4 | 0.97 | 0.18 |
| 14 | 39 | 96.1 | 72.5 | **36.8** | 38.2 | 281.3 | 0.96 | 0.13 |
| 16 | 33 | 97.0 | 72.3 | **41.4** | 43.7 | 396.1 | 0.95 | 0.11 |
| 18 | 32 | 111.8 | 78.2 | **44.0** | 47.2 | 643.5 | 0.93 | 0.07 |
| 20 | 19 | 110.3 | 80.4 | **48.9** | 53.1 | 877.4 | 0.92 | 0.06 |
| **all 3,334** | | 16.4M | 13.3M | **8.54M** | 8.80M (+1.68M basis matrices) | 31.4M | | |

- **HNF + LLL.** The $c_p$ lie in a sublattice of $\mathbb{Z}^k$ (the
  image of the order the $a_p$ generate). Writing $c_p=B\,y_p$ in a basis
  $B$ of that sublattice (from the HNF of the columns), and then applying
  LLL to the coordinate sequences $(y_{p,r})_p$, is a unimodular change of
  basis: $a_p=\beta'\cdot y_p$. LLL alone (without the HNF) gives only
  a 20% reduction.
- **Against the power basis,** the advantage grows with the dimension:
  1.3× at dimension 2, 18× at dimension 20.
- **Against LMFDB's LLL-reduced Hecke-ring basis,** the coordinates are
  2–8% smaller at every dimension, and LMFDB additionally stores $k^2$
  rational basis entries per orbit (1.68M bits in total). The analogous
  one-time cost for us is the $k$ elements $\beta'_r\in K$. **We have not
  computed or measured those yet**, so the totals compare per-$a_p$ data
  only.

## Orbits LMFDB does not store

LMFDB stores $q$-expansions only up to dimension 20. At levels $\le 1000$
there are 154 larger orbits, and all of them have exact coordinates here:

| dimension | orbits | bits per $a_p$ (raw → HNF+LLL) | per coordinate |
|---|---|---|---|
| 21–30 | 89 | 132.9 → 61.0 | 2.39 |
| 31–40 | 40 | 177.6 → 80.2 | 2.26 |
| 41–50 | 20 | 234.0 → 91.7 | 2.02 |
| 51–60 | 5 | 283.6 → 98.6 | 1.78 |

971.2.a.b (dimension 55): 106 bits per $a_p$.

## Next

- Move the HNF+LLL step from the Python prototype (python-flint) into the
  engine, through `sagebrush-flint` (FLINT's `fmpz_mat_hnf`, `fmpz_lll`).
- Compute $\beta'_r\in K$ (one eigenvector over $K$ per orbit), report its
  size, and produce power-basis $a_p$ on demand.
- $a_n$ for composite $n$ by multiplicativity, and $a_p$ for $p\mid N$.
