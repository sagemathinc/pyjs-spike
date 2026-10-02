//! The quotient over GF(p): coordinates of every free generator in a basis,
//! and Hecke operators.

use crate::linalg;
use crate::par;
use crate::presentation::Presentation;

pub struct Space {
    pub p: u64,
    /// Each free generator as a dense vector in the quotient basis.
    pub coords: Vec<Vec<u64>>,
    /// For each basis element, the free generator it is.
    pub basis_gen: Vec<u32>,
}

impl Space {
    pub fn new(pres: &Presentation, p: u64) -> Self {
        let rows: Vec<Vec<(u32, u64)>> = pres.rows.iter().map(|r| r.iter().map(|&(g, v)| (g, v.rem_euclid(p as i64) as u64)).filter(|e| e.1 != 0).collect()).collect();
        let (pivot_rows, pivot_of) = linalg::sparse_echelon(&rows, pres.m, p);
        let coords = linalg::back_substitute(&pivot_rows, &pivot_of, pres.m, p);
        let dim = coords.first().map_or(0, |v| v.len());
        let mut basis_gen = vec![u32::MAX; dim];
        for (g, v) in coords.iter().enumerate() {
            if pivot_of[g] == u32::MAX {
                basis_gen[v.iter().position(|&x| x == 1).unwrap()] = g as u32;
            }
        }
        Space { p, coords, basis_gen }
    }

    pub fn dimension(&self) -> usize {
        self.basis_gen.len()
    }

    /// Matrix of T_q (q prime, not dividing N); row i is T_q(basis_i).
    pub fn hecke_matrix(&self, pres: &Presentation, q: u64) -> Vec<Vec<u64>> {
        let h = linalg::heilbronn(q as i64);
        let p = self.p;
        let m = pres.m;
        par::map_slice(&self.basis_gen, |&g| {
            let (i, s) = pres.sym_of_gen[g as usize];
            let (c, d) = pres.p1.get(i as usize);
            let (c, d) = (c as i64, d as i64);
            let mut count = vec![0i64; m];
            let mut seen = vec![false; m];
            let mut touched = vec![];
            for &(a, b, cc, dd) in &h {
                if let Some((g2, s2)) = pres.rep_of[pres.p1.index(c * a + d * cc, c * b + d * dd)] {
                    let g2 = g2 as usize;
                    if !seen[g2] {
                        seen[g2] = true;
                        touched.push(g2);
                    }
                    count[g2] += if s2 ^ s { -1 } else { 1 };
                }
            }
            let mut row = vec![0u64; self.dimension()];
            for g2 in touched {
                let k = count[g2].rem_euclid(p as i64) as u64;
                if k != 0 {
                    for (r, &x) in row.iter_mut().zip(&self.coords[g2]) {
                        if x != 0 {
                            *r = (*r + k * x) % p;
                        }
                    }
                }
            }
            row
        })
    }
}
