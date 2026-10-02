//! Cusps of Gamma0(N) and the boundary map on weight-2 modular symbols.
//!
//! Cusp classes use Cremona's criterion: in lowest terms, u1/v1 ~ u2/v2
//! under Gamma0(N) iff s1 v2 = s2 v1 (mod gcd(v1 v2, N)), where
//! u_j s_j = 1 (mod v_j).  The cusp oo is replaced by 1/N (they are
//! equivalent: [1 0; N 1] oo = 1/N).  For the sign +1 quotient, x and -x
//! are identified as well.
//!
//! The Manin symbol (c : d), lifted to g = [a b; c d] in SL_2(Z), is the
//! modular symbol g{0, oo} = {b/d, a/c}, with boundary [a/c] - [b/d].

use crate::p1::{gcd, xgcd};
use crate::presentation::Presentation;

pub struct Cusps {
    n: i64,
    sign_plus: bool,
    /// Class representatives (u, v) in lowest terms, v > 0.
    reps: Vec<(i64, i64)>,
}

fn normalize(u: i64, v: i64, n: i64) -> (i64, i64) {
    if v == 0 {
        return (1, n); // oo ~ 1/N
    }
    let g = gcd(u.unsigned_abs(), v.unsigned_abs()) as i64;
    let (u, v) = (u / g, v / g);
    if v < 0 { (-u, -v) } else { (u, v) }
}

fn inverse_mod(u: i64, v: i64) -> i64 {
    if v == 1 {
        return 0;
    }
    let (_, s) = xgcd(u.rem_euclid(v), v);
    s.rem_euclid(v)
}

impl Cusps {
    pub fn new(n: u64, sign_plus: bool) -> Self {
        Cusps { n: n as i64, sign_plus, reps: vec![] }
    }

    fn equivalent(&self, (u1, v1): (i64, i64), (u2, v2): (i64, i64)) -> bool {
        let n = self.n as i128;
        let (v1, v2) = (v1 as i128, v2 as i128);
        let m = gcd((v1 * v2 % n) as u64, n as u64) as i128;
        let m = if m == 0 { n } else { m };
        let (s1, s2) = (inverse_mod(u1, v1 as i64) as i128, inverse_mod(u2, v2 as i64) as i128);
        (s1 * v2 - s2 * v1).rem_euclid(m) == 0
    }

    /// The index of the class of u/v (added if new).
    pub fn class(&mut self, u: i64, v: i64) -> usize {
        let c = normalize(u, v, self.n);
        let neg = normalize(-c.0, c.1, self.n);
        if let Some(i) = self.reps.iter().position(|&r| self.equivalent(r, c) || (self.sign_plus && self.equivalent(r, neg))) {
            return i;
        }
        self.reps.push(c);
        self.reps.len() - 1
    }

    pub fn count(&self) -> usize {
        self.reps.len()
    }
}

/// Lifts (c, d) with gcd(c, d, N) = 1 to [a b; c' d'] in SL_2(Z), c' = c, d' = d mod N.
pub fn lift_to_sl2(c: i64, d: i64, n: i64) -> (i64, i64, i64, i64) {
    let (c, mut d) = (c.rem_euclid(n.max(1)), d.rem_euclid(n.max(1)));
    if c == 0 {
        // (0 : d) with d a unit mod N; [1 0; 0 1] up to the scalar.
        d = 1;
        return (1, 0, 0, d);
    }
    while gcd(c as u64, d.unsigned_abs()) != 1 {
        d += n;
    }
    // a d - b c = 1
    let (g, x) = xgcd(d, c); // x d = 1 mod c
    debug_assert_eq!(g, 1);
    let a = x;
    let b = (a * d - 1) / c;
    (a, b, c, d)
}

/// The boundary of each free generator as a sparse vector over cusp
/// classes, and the number of classes.
pub fn boundary(pres: &Presentation, sign_plus: bool) -> (Vec<Vec<(usize, i64)>>, usize) {
    let n = pres.n as i64;
    let mut cusps = Cusps::new(pres.n, sign_plus);
    // Register the cusps of every Manin symbol first (generators alone
    // need not reach every class), so class indices cover all cusps.
    for i in 0..pres.p1.len() {
        let (c, d) = pres.p1.get(i);
        let (a, b, c, d) = lift_to_sl2(c as i64, d as i64, n);
        cusps.class(a, c);
        cusps.class(b, d);
    }
    let mut out = Vec::with_capacity(pres.m);
    for &(i, s) in &pres.sym_of_gen {
        let (c, d) = pres.p1.get(i as usize);
        let (a, b, c, d) = lift_to_sl2(c as i64, d as i64, n);
        let hi = cusps.class(a, c);
        let lo = cusps.class(b, d);
        let sg = if s { -1 } else { 1 };
        let mut v = vec![];
        if hi != lo {
            v.push((hi, sg));
            v.push((lo, -sg));
        }
        out.push(v);
    }
    (out, cusps.count())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exact::level_data;

    /// The boundaries of Manin symbols reach every cusp class: as many as
    /// sum_{d | N} phi(gcd(d, N/d)), and with x ~ -x the orbit count
    /// (Eisenstein dimension + 1) from the dimension formula.
    #[test]
    fn cusp_class_counts() {
        for n in 1..=400u64 {
            let pres = Presentation::new(n);
            let (_, all) = boundary(&pres, false);
            let (_, plus) = boundary(&pres, true);
            let (_, _, cusps, eis, _) = level_data(n);
            assert_eq!(all as u64, cusps, "N={} all={} plus={} cusps={} eis={}", n, all, plus, cusps, eis);
            assert_eq!(plus as u64, eis + 1, "N={}", n);
        }
    }
}
