//! Exact characteristic polynomials over Z, with a proof sketch.
//!
//! Let L be the integral plus-quotient of weight-2 modular symbols for
//! Gamma0(N) modulo torsion.  Its rank is g + e, where g is the genus of
//! X0(N) and e + 1 is the number of cusps up to the involution x -> -x
//! (the Eisenstein part of the plus space is the degree-zero part of the
//! cusp divisors fixed by that involution).  T_q acts on L integrally.
//! Over GF(p) the Manin-symbol presentation gives M(Z) (x) GF(p) (right
//! exactness); if its dimension equals g + e the torsion vanishes at p, so
//! the mod-p characteristic polynomial is the reduction of the integral
//! one.  Eigenvalues satisfy |a| <= 2 sqrt(q) on cusp forms (Eichler-Shimura
//! + Weil) and |a| <= 1 + q on Eisenstein series, which bounds the
//! coefficients; CRT over enough good primes then determines the
//! polynomial exactly.

use crate::linalg;
use crate::par;
use crate::presentation::Presentation;
use crate::space::Space;
use num_bigint::{BigInt, BigUint};
use num_traits::{One, Signed, ToPrimitive, Zero};

pub fn factor(mut n: u64) -> Vec<(u64, u32)> {
    let mut out = vec![];
    let mut p = 2;
    while p * p <= n {
        let mut e = 0;
        while n % p == 0 {
            n /= p;
            e += 1;
        }
        if e > 0 {
            out.push((p, e));
        }
        p += 1;
    }
    if n > 1 {
        out.push((n, 1));
    }
    out
}

fn kronecker_minus(d: i64, p: u64) -> i64 {
    // (d/p) for d in {-1, -3}, p prime
    let p = p as i64;
    if d == -1 {
        return if p == 2 { 0 } else if p % 4 == 1 { 1 } else { -1 };
    }
    if p == 3 {
        0
    } else if p % 3 == 1 {
        1
    } else {
        -1
    }
}

/// Level data for X0(N): (psi(N), genus, cusps, Eisenstein dimension of the
/// sign +1 space, dimension of the sign +1 space).
pub fn level_data(n: u64) -> (u64, u64, u64, u64, u64) {
    let f = factor(n);
    let psi = f.iter().fold(n, |acc, &(p, _)| acc / p * (p + 1));
    let nu2: i64 = if n % 4 == 0 { 0 } else { f.iter().map(|&(p, _)| 1 + kronecker_minus(-1, p)).product() };
    let nu3: i64 = if n % 9 == 0 { 0 } else { f.iter().map(|&(p, _)| 1 + kronecker_minus(-3, p)).product() };
    // Cusps over d | N: phi(gcd(d, N/d)) of them, paired by x -> -x when
    // gcd(d, N/d) > 2.
    let (mut cusps, mut orbits) = (0u64, 0u64);
    for d in 1..=n {
        if n % d == 0 {
            let g = crate::p1::gcd(d, n / d);
            let phi = factor(g).iter().fold(g, |acc, &(p, _)| acc / p * (p - 1));
            cusps += phi;
            orbits += if g > 2 { phi / 2 } else { phi };
        }
    }
    // 12 g = 12 + psi - 3 nu2 - 4 nu3 - 6 c
    let g12 = 12 + psi as i64 - 3 * nu2 - 4 * nu3 - 6 * cusps as i64;
    let genus = (g12 / 12) as u64;
    (psi, genus, cusps, orbits - 1, genus + orbits - 1)
}

pub fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    for p in [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        if n % p == 0 {
            return n == p;
        }
    }
    let (mut d, mut s) = (n - 1, 0);
    while d % 2 == 0 {
        d /= 2;
        s += 1;
    }
    let mulmod = |a: u64, b: u64| ((a as u128 * b as u128) % n as u128) as u64;
    'outer: for a in [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        let mut x = 1u64;
        let (mut b, mut e) = (a, d);
        while e > 0 {
            if e & 1 == 1 {
                x = mulmod(x, b);
            }
            b = mulmod(b, b);
            e >>= 1;
        }
        if x == 1 || x == n - 1 {
            continue;
        }
        for _ in 1..s {
            x = mulmod(x, x);
            if x == n - 1 {
                continue 'outer;
            }
        }
        return false;
    }
    true
}

pub struct Exact {
    pub n: u64,
    pub q: u64,
    pub genus: u64,
    pub cusps: u64,
    /// Dimension of the Eisenstein part of the sign +1 space.
    pub eis: u64,
    pub dim: u64,
    /// Coefficients, constant term first; monic of degree dim.
    pub coeffs: Vec<BigInt>,
    pub primes_used: Vec<u64>,
    pub primes_rejected: Vec<u64>,
    pub bound_bits: f64,
    pub status: &'static str,
    pub checks: Vec<String>,
}

/// Bits of a bound on |coefficient| (plus sign and margin).
pub fn bound_bits(q: u64, genus: u64, eis: u64) -> f64 {
    let qf = q as f64;
    genus as f64 * (1.0 + 2.0 * qf.sqrt()).log2() + eis as f64 * (2.0 + qf).log2() + 2.0
}

pub fn exact_charpoly(n: u64, q: u64) -> Result<Exact, String> {
    if !is_prime(q) || n % q == 0 {
        return Err(format!("q = {} must be a prime not dividing N = {}", q, n));
    }
    let (_, genus, cusps, eis, dim) = level_data(n);
    let need = bound_bits(q, genus, eis);
    let pres = Presentation::new(n);
    let (mut used, mut rejected) = (vec![], vec![]);
    let mut residues: Vec<(u64, Vec<u64>)> = vec![];
    let mut bits = 0.0;
    let mut next = 1u64 << 31;
    while bits < need {
        // A batch of primes, computed in parallel.
        let mut batch = vec![];
        while batch.len() < 8 {
            next -= 1;
            if is_prime(next) {
                batch.push(next);
            }
        }
        let results = par::map_slice(&batch, |&p| {
            let sp = Space::new(&pres, p);
            if sp.dimension() as u64 != dim {
                return (p, None);
            }
            (p, Some(linalg::charpoly(sp.hecke_matrix(&pres, q), p)))
        });
        for (p, r) in results {
            match r {
                Some(f) if bits < need => {
                    bits += (p as f64).log2();
                    used.push(p);
                    residues.push((p, f));
                }
                Some(_) => {}
                None => rejected.push(p),
            }
        }
        if rejected.len() > 64 {
            return Err(format!("too many primes with the wrong dimension (expected {})", dim));
        }
    }
    let coeffs = crt(&residues, dim as usize);
    let mut checks = vec![];
    let monic = coeffs.last().map_or(false, |c| c.is_one());
    checks.push(format!("monic of degree {}: {}", dim, monic));
    let x = BigInt::from(q + 1);
    let at = coeffs.iter().rev().fold(BigInt::zero(), |acc, c| acc * &x + c);
    let eis_ok = eis == 0 || at.is_zero();
    checks.push(format!("q + 1 is a root (an Eisenstein eigenvalue): {}", eis_ok));
    let max_bits = coeffs.iter().map(|c| c.abs().bits()).max().unwrap_or(0);
    checks.push(format!("largest coefficient has {} bits, bound {:.0}", max_bits, need));
    let status = if monic && eis_ok && (max_bits as f64) < need { "proven" } else { "inconsistent" };
    Ok(Exact { n, q, genus, cusps, eis, dim, coeffs, primes_used: used, primes_rejected: rejected, bound_bits: need, status, checks })
}

/// Chinese remaindering to symmetric representatives.
fn crt(residues: &[(u64, Vec<u64>)], dim: usize) -> Vec<BigInt> {
    let mut modulus = BigUint::one();
    let mut acc = vec![BigUint::zero(); dim + 1];
    for (p, f) in residues {
        let pb = BigUint::from(*p);
        let m_mod_p = (&modulus % &pb).to_u64().unwrap();
        let inv = modpow(m_mod_p, p - 2, *p);
        for (k, a) in acc.iter_mut().enumerate() {
            let r = (&*a % &pb).to_u64().unwrap();
            let t = ((f[k] + p - r) % p) as u128 * inv as u128 % *p as u128;
            *a += &modulus * BigUint::from(t as u64);
        }
        modulus *= pb;
    }
    let half = &modulus >> 1;
    acc.into_iter().map(|a| if a > half { BigInt::from(a) - BigInt::from(modulus.clone()) } else { BigInt::from(a) }).collect()
}

fn modpow(mut b: u64, mut e: u64, m: u64) -> u64 {
    let mut r = 1u128;
    let mut bb = (b % m) as u128;
    while e > 0 {
        if e & 1 == 1 {
            r = r * bb % m as u128;
        }
        bb = bb * bb % m as u128;
        e >>= 1;
    }
    b = r as u64;
    b
}
