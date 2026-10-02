# Galois orbits of newforms (prime level): first milestone

`engine/modsym/src/orbits.rs` decomposes the weight-2 newforms on
$\Gamma_0(N)$ into Galois orbits and computes $\operatorname{tr}(a_p)$ and
the coordinates $c_p$ of $a_p$ for each orbit, at prime level $N$ for now.

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

1. Composite levels: the new subspace. Old orbits from level $M\mid N$
   appear $d(N/M)$ times, so a factor whose multiplicity exceeds 1 is
   separated or attributed using the lower levels, as for rational newforms.
   The cuspidal projection can use the Eisenstein eigenvalue bound for $q\ge 7$.
2. Integral $c_p\in\mathbb{Z}^k$: an integral basis of $A^\vee$ (rational
   reconstruction from several $\ell$, or an integral presentation à la
   PARI's well-formed fundamental domain, implemented from the mathematics
   to keep it MIT). Then measure the size against power-basis and LMFDB
   Hecke-ring storage.
3. $a_n$ for composite $n$, the $\beta_r$, and power-basis output on demand.
