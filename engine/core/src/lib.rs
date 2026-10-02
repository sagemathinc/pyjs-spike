//! Weight-2 modular symbols for Gamma0(N), sign +1, over GF(p): the same
//! mathematics as bench/modsym/modsym.py, engineered for speed and memory.
//!
//! * P^1(Z/NZ) via Sage's p1_normalize, O(N d(N)) memory (the Python
//!   reference uses an N^2 table).
//! * 3-term relations by sparse elimination over GF(p).
//! * Hecke operators via Cremona's Heilbronn matrices, parallel over basis
//!   elements; characteristic polynomial via a parallel Hessenberg form.
//!
//! With the `parallel` feature, work runs on the current rayon pool.

mod par;
pub mod linalg;
pub mod p1;

use p1::P1List;
use std::time::Instant;

pub struct ModularSymbols {
    pub n: u64,
    pub p: u64,
    pub p1: P1List,
    /// For each Manin symbol: (free generator, sign) or None if it is zero.
    rep_of: Vec<Option<(u32, bool)>>,
    /// Each free generator as a dense vector in the quotient basis.
    coords: Vec<Vec<u64>>,
    /// A Manin symbol (index, negated) representing each basis element.
    basis_symbol: Vec<(u32, bool)>,
}

impl ModularSymbols {
    pub fn new(n: u64, p: u64) -> Self {
        let p1 = P1List::new(n);
        let ns = p1.len();
        // Signed union-find for x + xS = 0 and x = x*eta (sign +1).
        let mut parent: Vec<u32> = (0..ns as u32).collect();
        let mut neg = vec![false; ns]; // x_i = (-1)^neg[i] x_parent[i]
        let mut zero = vec![false; ns];
        fn find(parent: &mut [u32], neg: &mut [bool], i: usize) -> (usize, bool) {
            let mut path = vec![];
            let mut r = i;
            let mut s = false;
            while parent[r] as usize != r {
                path.push(r);
                s ^= neg[r];
                r = parent[r] as usize;
            }
            // Path compression, keeping signs relative to the root.
            let mut acc = s;
            for &k in &path {
                let next = neg[k];
                parent[k] = r as u32;
                neg[k] = acc;
                acc ^= next;
            }
            (r, s)
        }
        let mut union = |parent: &mut Vec<u32>, neg: &mut Vec<bool>, i: usize, j: usize, minus: bool| {
            let (ri, si) = find(parent, neg, i);
            let (rj, sj) = find(parent, neg, j);
            if ri == rj {
                if si != (sj ^ minus) {
                    zero[ri] = true;
                }
                return;
            }
            parent[ri] = rj as u32;
            neg[ri] = si ^ sj ^ minus;
            if zero[ri] {
                zero[rj] = true;
            }
        };
        for i in 0..ns {
            let (c, d) = p1.get(i);
            let s = p1.index(d as i64, -(c as i64));
            union(&mut parent, &mut neg, i, s, true);
            let e = p1.index(-(c as i64), d as i64);
            union(&mut parent, &mut neg, i, e, false);
        }
        let mut free_of_root = vec![u32::MAX; ns];
        let mut m = 0u32;
        let mut rep_of = vec![None; ns];
        for i in 0..ns {
            let (r, s) = find(&mut parent, &mut neg, i);
            if zero[r] {
                continue;
            }
            if free_of_root[r] == u32::MAX {
                free_of_root[r] = m;
                m += 1;
            }
            rep_of[i] = Some((free_of_root[r], s));
        }
        let m = m as usize;
        // 3-term relations x + xT + xT^2 = 0 as sparse rows over free gens.
        let rows: Vec<Vec<(u32, u64)>> = par::map_range(ns, |i| {
            let (c, d) = p1.get(i);
            let (c, d) = (c as i64, d as i64);
            let mut row: Vec<(u32, u64)> = Vec::with_capacity(3);
            for (a, b) in [(c, d), (d, -c - d), (-c - d, c)] {
                if let Some((g, s)) = rep_of[p1.index(a, b)] {
                    let v = if s { p - 1 } else { 1 };
                    match row.iter_mut().find(|e| e.0 == g) {
                        Some(e) => e.1 = (e.1 + v) % p,
                        None => row.push((g, v)),
                    }
                }
            }
            row.retain(|e| e.1 != 0);
            row
        });
        let (pivot_rows, pivot_col) = linalg::sparse_echelon(&rows, m, p);
        let coords = linalg::back_substitute(&pivot_rows, &pivot_col, m, p);
        let dim = coords.first().map_or(0, |v| v.len());
        // A representative symbol for each basis element (a non-pivot gen).
        let mut basis_gen = vec![u32::MAX; dim];
        for (g, v) in coords.iter().enumerate() {
            if pivot_col[g] == u32::MAX {
                let t = v.iter().position(|&x| x == 1).unwrap();
                basis_gen[t] = g as u32;
            }
        }
        let mut sym_of_gen = vec![None; m];
        for (i, r) in rep_of.iter().enumerate() {
            if let Some((g, s)) = r {
                if sym_of_gen[*g as usize].is_none() {
                    sym_of_gen[*g as usize] = Some((i as u32, *s));
                }
            }
        }
        let basis_symbol = basis_gen.iter().map(|&g| sym_of_gen[g as usize].unwrap()).collect();
        ModularSymbols { n, p, p1, rep_of, coords, basis_symbol }
    }

    /// Coordinates of Manin symbol i in the quotient basis (debugging).
    pub fn symbol_vector(&self, i: usize) -> Vec<u64> {
        match self.rep_of[i] {
            None => vec![0; self.dimension()],
            Some((g, s)) => self.coords[g as usize].iter().map(|&x| if s { (self.p - x) % self.p } else { x }).collect(),
        }
    }

    /// Check the 2- and 3-term relations hold in the quotient (debugging).
    pub fn check_relations(&self) -> Vec<String> {
        let p = self.p;
        let mut bad = vec![];
        let add = |a: &[u64], b: &[u64]| a.iter().zip(b).map(|(x, y)| (x + y) % p).collect::<Vec<u64>>();
        for i in 0..self.p1.len() {
            let (c, d) = self.p1.get(i);
            let (c, d) = (c as i64, d as i64);
            let x = self.symbol_vector(i);
            let s = self.symbol_vector(self.p1.index(d, -c));
            if add(&x, &s).iter().any(|&v| v != 0) {
                bad.push(format!("S fails at {:?}", (c, d)));
            }
            let t1 = self.symbol_vector(self.p1.index(d, -c - d));
            let t2 = self.symbol_vector(self.p1.index(-c - d, c));
            if add(&add(&x, &t1), &t2).iter().any(|&v| v != 0) {
                bad.push(format!("T fails at {:?}", (c, d)));
            }
        }
        bad
    }

    pub fn basis_symbols(&self) -> &[(u32, bool)] {
        &self.basis_symbol
    }

    pub fn dimension(&self) -> usize {
        self.basis_symbol.len()
    }

    /// Matrix of T_q (q prime, not dividing N), rows indexed by basis.
    pub fn hecke_matrix(&self, q: u64) -> Vec<Vec<u64>> {
        let h = linalg::heilbronn(q as i64);
        let p = self.p;
        let m = self.coords.len();
        par::map_slice(&self.basis_symbol, |&(i, s)| {
            let (c, d) = self.p1.get(i as usize);
            let (c, d) = (c as i64, d as i64);
            // Count hits per free generator, then combine coordinate vectors.
            let mut count = vec![0i64; m];
            let mut touched = vec![];
            for &(a, b, cc, dd) in &h {
                if let Some((g, gs)) = self.rep_of[self.p1.index(c * a + d * cc, c * b + d * dd)] {
                    let g = g as usize;
                    if count[g] == 0 {
                        touched.push(g);
                    }
                    count[g] += if gs ^ s { -1 } else { 1 };
                }
            }
            let mut row = vec![0u64; self.dimension()];
            for g in touched {
                let k = count[g].rem_euclid(p as i64) as u64;
                if k != 0 {
                    for (r, &x) in row.iter_mut().zip(&self.coords[g]) {
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

pub struct Result {
    pub n: u64,
    pub q: u64,
    pub p: u64,
    pub symbols: usize,
    pub dim: usize,
    pub charpoly: Vec<u64>,
    pub ms: [f64; 3],
}

impl Result {
    /// The same hash modsym.py prints.
    pub fn hash(&self) -> u128 {
        self.charpoly.iter().fold(0u128, |h, &c| (h * 1000003 + c as u128) % 2305843009213693951)
    }
    pub fn eisenstein_root(&self) -> bool {
        let x = (self.q + 1) % self.p;
        self.charpoly.iter().rev().fold(0u64, |r, &c| (r * x + c) % self.p) == 0
    }
}

pub fn hecke_charpoly(n: u64, q: u64, p: u64) -> Result {
    let t0 = Instant::now();
    let ms = ModularSymbols::new(n, p);
    let t1 = Instant::now();
    let t = ms.hecke_matrix(q);
    let t2 = Instant::now();
    let f = linalg::charpoly(t, p);
    let t3 = Instant::now();
    let d = |a: Instant, b: Instant| (b - a).as_secs_f64() * 1000.0;
    Result { n, q, p, symbols: ms.p1.len(), dim: ms.dimension(), charpoly: f, ms: [d(t0, t1), d(t1, t2), d(t2, t3)] }
}
