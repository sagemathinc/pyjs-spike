# Galois orbits of newforms of weight 2

`engine/modsym/src/orbits.rs` decomposes the weight-2 newforms on
$\Gamma_0(N)$, for every level $N$, into Galois orbits, and computes
$\operatorname{tr}(a_p)$ and the coordinates $c_p$ of $a_p$ for each orbit.

## Composite levels (second milestone)

| check | result |
|---|---|
| **every level $1\le N\le 1000$ vs LMFDB** `mf_newforms` | **5,951 Galois orbits (largest dimension 55): dimensions and $\operatorname{Tr}(a_p)$ for all primes $p<1000$, $p\nmid N$, identical at every level.** 101.8 s on 16 threads, 94 MB |
| `cargo test`: every level $\le 200$ vs Sage | 470 orbits identical (Sage took 33 s on one core to produce them) |
| cusp classes from Cremona's criterion, $N\le 400$ | the formula $\sum_{d\mid N}\varphi(\gcd(d,N/d))$, and with $x\sim -x$ the Eisenstein dimension $+1$, at every level |

How composite levels work. Take $T=\sum_i r_iT_{q_i}$ and factor
$\chi(T)\in\mathbb{Z}[x]$. A new orbit occurs once in the sign $+1$
space, while an old orbit from level $M\mid N$ occurs $d(N/M)\ge 2$
times. The Eisenstein factors are exactly those dividing $\chi_E$, the
charpoly of $T$ on the boundary image $\delta(M)$ (cusp classes via
Cremona's criterion, computed mod $\ell$). So the new orbits are the
non-Eisenstein irreducible factors of exponent one, with no recursion over
lower levels. The coefficients are super-increasing: $r_1=1$ and
$r_{i+1}=1+\sum_{j\le i}r_j\lceil 4\sqrt{q_j}\rceil$. Then
$\sum_i r_i(a_{q_i}-b_{q_i})=0$ forces $a_{q_i}=b_{q_i}$, since
$|a_q-b_q|\le 4\sqrt q$. With small coefficients, rational forms
collided often (at $N=200$ and $214$). The degrees must add up to
$\dim S_2^{\rm new}(N)=\sum_{M\mid N}\beta(N/M)\,g(M)$, and otherwise
more primes are tried. One bug on the way: the "random" vectors for the
Krylov basis were affine in the seed, so all seeds spanned only two fixed
vectors, and an eigen-functional orthogonal to both defeated every seed.

## Prime levels (first milestone)

## Method

- $\chi(T_q)\in\mathbb{Z}[x]$ comes exactly from the multimodular charpoly.
  The Eisenstein factor $x-(q+1)$ is divided out, and the rest is factored
  with FLINT (`sagebrush-flint`, LGPL, a dev-dependency only; the library
  takes the factoring routine as a parameter and stays MIT). The first
  prime $q$ that gives a squarefree cuspidal charpoly is used.
- Each irreducible factor $f$ of degree $k$ is one orbit $A$. Its dual is
  found mod $\ell$ without forming $f(T)$: $A^\vee=\operatorname{im} h(T)$
  for $h=\chi/f$, which is cyclic, so it is spanned by
  $u, Tu,\dots,T^{k-1}u$ with $u=h(T)v$. That is matrix-vector products only.
- From a row-reduced basis $w_1,\dots,w_k$ of $A^\vee$, extended to the
  Manin-symbol generators through the sparse relations:
  $\operatorname{tr}(T_p\mid A)=\sum_r (T_p w_r)[\text{pivot}_r]$, which is
  $k$ Heilbronn sums per prime; and $c_{p,r}=w_r(T_p x)$, so that
  $a_p=\sum_r\beta_r c_{p,r}$ with $\beta_r\in K$ fixed (Stein's
  representation). Everything is parallel over primes.

## Results

| check | result |
|---|---|
| every prime level $11\le N\le 1000$ vs LMFDB `mf_newforms` (read-only Postgres mirror `devmirror.lmfdb.xyz`) | **164 levels, 450 Galois orbits (largest dimension 55): dimensions and $\operatorname{Tr}(a_p)$ for all 167 primes $p<1000$, $p\ne N$, identical**. 17.6 s on 16 threads, 354 MB |
| `cargo test`: prime levels $\le 300$ vs Sage (`ModularSymbols(N,2,1).cuspidal_subspace().new_subspace().decomposition()`, traces for $p\le 200$) | 129 orbits identical, 0.34 s |

Sage, generating the same reference data with traces only to $p\le 200$,
was still running after 4 minutes on one core, at level 739.

## Next

1. ~~Composite levels~~ (done, above).
2. Integral $c_p\in\mathbb{Z}^k$: an integral basis of $A^\vee$ (rational
   reconstruction from several $\ell$, or an integral presentation à la
   PARI's well-formed fundamental domain, implemented from the mathematics
   to keep it MIT). Then measure the size against power-basis and LMFDB
   Hecke-ring storage.
3. $a_n$ for composite $n$, the $\beta_r$, and power-basis output on demand.
