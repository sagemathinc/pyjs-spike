//! Galois orbits of newforms and their Hecke data, at prime level (for now).
//!
//! At prime N there are no oldforms and the Eisenstein part of the sign +1
//! space is one line, with T_q eigenvalue q + 1.  So the cuspidal
//! charpoly chi(T_q) / (x - q - 1) in Z[x] factors as prod f_j, and when
//! it is squarefree each irreducible f_j is one Galois orbit A_j of
//! newforms, of dimension deg f_j.  Factoring is passed in by the caller
//! (e.g. FLINT via sagebrush-flint), so this crate stays free of it.
//!
//! Each orbit is found in the dual space mod ell without forming f(T):
//! with h = chi / f coprime to f, A^dual = image of h(T), and it is cyclic,
//! so it is spanned by u, Tu, ..., T^(k-1) u for u = h(T) v.  Then, from
//! k functionals w_1..w_k on the Manin-symbol generators (row reduced):
//!   tr(T_p | A) = sum_r (T_p w_r)[pivot_r]: k Heilbronn sums per prime;
//!   c_{p,r} = w_r(T_p x) for a fixed generator x: the coordinates of a_p
//!   in Stein's basis (a_p = sum_r beta_r c_{p,r}, beta_r fixed in K).

use crate::exact::{exact_charpoly, is_prime};
use crate::linalg;
use crate::newforms::{heilbronn_pairing, rref, ELL};
use crate::par;
use crate::presentation::Presentation;
use crate::space::Space;
use num_bigint::BigInt;
use num_traits::{ToPrimitive, Zero};

/// Factors a polynomial in Z[x] (constant term first) into irreducible
/// factors with multiplicities.
pub type Factorer<'a> = &'a (dyn Fn(&[BigInt]) -> Vec<(Vec<BigInt>, u32)> + Sync);

#[derive(Debug, Clone)]
pub struct Orbit {
    pub dim: usize,
    /// The prime q and the irreducible f in Z[x] with A = ker f(T_q).
    pub q: u64,
    pub f: Vec<BigInt>,
    /// tr(T_p | A) for primes p <= bound, p != N.
    pub traces: Vec<(u64, i64)>,
    /// c_{p,r} = w_r(T_p x) mod ell (r = 1..dim), for the same primes.
    pub c: Vec<(u64, Vec<u64>)>,
}

fn mulmod(a: u64, b: u64, p: u64) -> u64 {
    (a as u128 * b as u128 % p as u128) as u64
}

/// (T v)_i = sum_j T[i][j] v_j mod p: the action on functionals.
fn matvec(t: &[Vec<u64>], v: &[u64], p: u64) -> Vec<u64> {
    par::map_slice(t, |row| (row.iter().zip(v).fold(0u128, |s, (&a, &b)| s + a as u128 * b as u128) % p as u128) as u64)
}

/// Quotient of a by b mod p (b monic), constant term first.
fn poly_div(a: &[u64], b: &[u64], p: u64) -> Vec<u64> {
    let (n, m) = (a.len() - 1, b.len() - 1);
    let mut r = a.to_vec();
    let mut q = vec![0u64; n - m + 1];
    for i in (0..=n - m).rev() {
        let c = r[i + m];
        q[i] = c;
        if c != 0 {
            for j in 0..=m {
                r[i + j] = (r[i + j] + p - mulmod(c, b[j], p)) % p;
            }
        }
    }
    debug_assert!(r.iter().all(|&x| x == 0), "inexact division");
    q
}

fn reduce(f: &[BigInt], p: u64) -> Vec<u64> {
    let pb = BigInt::from(p);
    f.iter().map(|c| ((c % &pb + &pb) % &pb).to_u64().unwrap()).collect()
}

/// The Galois orbits of newforms of prime level N, with tr(T_p | A) and
/// the coordinates c_p of a_p for primes p <= bound, p != N.
pub fn prime_level_orbits(n: u64, bound: u64, factor: Factorer) -> Result<Vec<Orbit>, String> {
    if !is_prime(n) {
        return Err(format!("N = {} is not prime (only prime levels so far)", n));
    }
    let p = ELL;
    // A prime q whose cuspidal charpoly is squarefree.
    let mut chosen = None;
    for q in (2..60u64).filter(|&q| is_prime(q) && q != n) {
        let chi = exact_charpoly(n, q)?.coeffs;
        // Divide out the Eisenstein factor x - (q + 1).
        let e = BigInt::from(q + 1);
        let mut cusp = vec![BigInt::zero(); chi.len() - 1];
        let mut acc = BigInt::zero();
        for i in (0..chi.len()).rev() {
            acc = acc * &e + &chi[i];
            if i > 0 {
                cusp[i - 1] = acc.clone();
            }
        }
        if !acc.is_zero() {
            return Err(format!("q + 1 is not a root of chi(T_{}) at N = {}", q, n));
        }
        let fs = if cusp.len() > 1 { factor(&cusp) } else { vec![] };
        if fs.iter().all(|f| f.1 == 1) {
            chosen = Some((q, chi, fs));
            break;
        }
    }
    let (q, chi, fs) = chosen.ok_or("no prime q < 60 with a squarefree cuspidal charpoly")?;
    let pres = Presentation::new(n);
    let sp = Space::new(&pres, p);
    let d = sp.dimension();
    let t = sp.hecke_matrix(&pres, q);
    let chi_p = reduce(&chi, p);
    let primes: Vec<u64> = (2..=bound).filter(|&l| is_prime(l) && l != n).collect();
    let mut out = vec![];
    for (f, _) in fs {
        let k = f.len() - 1;
        let h = poly_div(&chi_p, &reduce(&f, p), p);
        // u = h(T) v by Horner, then the Krylov basis u, Tu, ..., T^(k-1) u.
        let mut rows = vec![];
        for seed in 1..=4u64 {
            let v: Vec<u64> = (0..d as u64).map(|i| (i * 2654435761 + seed * 40503) % p).collect();
            let mut u = vec![0u64; d];
            for &c in h.iter().rev() {
                u = matvec(&t, &u, p);
                for (x, &vi) in u.iter_mut().zip(&v) {
                    *x = (*x + mulmod(c, vi, p)) % p;
                }
            }
            rows = vec![u];
            for _ in 1..k {
                let next = matvec(&t, rows.last().unwrap(), p);
                rows.push(next);
            }
            let rank = linalg::rref_mod(rows.clone(), p).0.len();
            if rank == k {
                break;
            }
        }
        let pivots = rref(&mut rows, p);
        if rows.len() != k {
            return Err(format!("N = {}: an orbit of degree {} has a degenerate Krylov basis", n, k));
        }
        let psis: Vec<Vec<u64>> = rows.iter().map(|w| sp.extend(w)).collect();
        let x = sp.basis_gen[pivots[0]];
        let data = par::map_slice(&primes, |&l| {
            let hl = linalg::heilbronn(l as i64);
            let mut tr = 0u64;
            let mut c = Vec::with_capacity(k);
            for (r, psi) in psis.iter().enumerate() {
                tr = (tr + heilbronn_pairing(&pres, &hl, psi, sp.basis_gen[pivots[r]], p)) % p;
                c.push(heilbronn_pairing(&pres, &hl, psi, x, p));
            }
            let tr = if tr > p / 2 { tr as i64 - p as i64 } else { tr as i64 };
            ((l, tr), (l, c))
        });
        let (traces, c) = data.into_iter().unzip();
        out.push(Orbit { dim: k, q, f, traces, c });
    }
    out.sort_by(|a, b| (a.dim, &a.traces).cmp(&(b.dim, &b.traces)));
    Ok(out)
}
