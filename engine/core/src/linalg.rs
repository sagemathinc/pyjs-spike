//! Sparse elimination, Heilbronn matrices, and the characteristic
//! polynomial over GF(p) (p < 2^31 so products fit in u64).

use crate::par;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

fn powmod(mut b: u64, mut e: u64, m: u64) -> u64 {
    let mut r = 1u64;
    b %= m;
    while e > 0 {
        if e & 1 == 1 {
            r = r * b % m;
        }
        b = b * b % m;
        e >>= 1;
    }
    r
}

/// Echelonize sparse rows over m columns.  Returns the pivot rows (pivot
/// coefficient 1, pivot entry omitted) in creation order and, per column,
/// the index of the pivot row that eliminates it (u32::MAX if none).
/// A new row is reduced by eliminating pivot columns in creation order,
/// which only introduces columns of later pivots, so the loop terminates.
pub fn sparse_echelon(rows: &[Vec<(u32, u64)>], m: usize, p: u64) -> (Vec<(u32, Vec<(u32, u64)>)>, Vec<u32>) {
    let mut pivots: Vec<(u32, Vec<(u32, u64)>)> = vec![];
    let mut pivot_of = vec![u32::MAX; m];
    let mut acc = vec![0u64; m];
    let mut touched: Vec<u32> = vec![];
    let mut heap = BinaryHeap::new();
    for row in rows {
        for &(c, v) in row {
            if acc[c as usize] == 0 {
                touched.push(c);
            }
            acc[c as usize] = (acc[c as usize] + v) % p;
            if pivot_of[c as usize] != u32::MAX {
                heap.push(Reverse((pivot_of[c as usize], c)));
            }
        }
        while let Some(Reverse((t, c))) = heap.pop() {
            let f = acc[c as usize];
            if f == 0 {
                continue;
            }
            acc[c as usize] = 0;
            for &(k, v) in &pivots[t as usize].1 {
                let k_us = k as usize;
                if acc[k_us] == 0 {
                    touched.push(k);
                    if pivot_of[k_us] != u32::MAX {
                        heap.push(Reverse((pivot_of[k_us], k)));
                    }
                }
                acc[k_us] = (acc[k_us] + p - f * v % p) % p;
            }
        }
        let mut entries: Vec<(u32, u64)> = vec![];
        touched.sort_unstable();
        touched.dedup();
        for &c in &touched {
            let v = acc[c as usize];
            if v != 0 {
                entries.push((c, v));
            }
            acc[c as usize] = 0;
        }
        touched.clear();
        if entries.is_empty() {
            continue;
        }
        let (pc, pv) = entries[0];
        let inv = powmod(pv, p - 2, p);
        let rest = entries[1..].iter().map(|&(c, v)| (c, v * inv % p)).collect();
        pivot_of[pc as usize] = pivots.len() as u32;
        pivots.push((pc, rest));
    }
    (pivots, pivot_of)
}

/// Each column (free generator) as a dense vector over the non-pivot
/// columns, which form the quotient basis.
pub fn back_substitute(pivots: &[(u32, Vec<(u32, u64)>)], pivot_of: &[u32], m: usize, p: u64) -> Vec<Vec<u64>> {
    let basis: Vec<usize> = (0..m).filter(|&c| pivot_of[c] == u32::MAX).collect();
    let dim = basis.len();
    let mut coords = vec![vec![]; m];
    for (t, &c) in basis.iter().enumerate() {
        let mut v = vec![0u64; dim];
        v[t] = 1;
        coords[c] = v;
    }
    // x_pc + sum v_k x_k = 0; later pivots are already expressed.
    for (pc, rest) in pivots.iter().rev() {
        let mut v = vec![0u64; dim];
        for &(k, x) in rest {
            let neg = p - x;
            for (a, &b) in v.iter_mut().zip(&coords[k as usize]) {
                if b != 0 {
                    *a = (*a + neg * b) % p;
                }
            }
        }
        coords[*pc as usize] = v;
    }
    coords
}

/// Cremona's Heilbronn matrices of determinant q, as (a, b, c, d).
pub fn heilbronn(q: i64) -> Vec<(i64, i64, i64, i64)> {
    if q == 2 {
        return vec![(1, 0, 0, 2), (2, 0, 0, 1), (2, 1, 0, 1), (1, 0, 1, 2)];
    }
    let mut out = vec![(1, 0, 0, q)];
    for r in -(q / 2)..=(q / 2) {
        let (mut x1, mut x2, mut y1, mut y2, mut a, mut b) = (q, -r, 0i64, 1i64, -q, r);
        out.push((x1, x2, y1, y2));
        while b != 0 {
            let mut qq = (a.abs() * 2 + b.abs()) / (2 * b.abs());
            if (a < 0) != (b < 0) {
                qq = -qq;
            }
            (a, b) = (-b, a - b * qq);
            (x1, x2) = (x2, qq * x2 - x1);
            (y1, y2) = (y2, qq * y2 - y1);
            out.push((x1, x2, y1, y2));
        }
    }
    out
}

/// Characteristic polynomial (low degree first) via Hessenberg form.
/// Each step applies all row operations, then the matching column
/// operations; the elementary transforms of one step commute, so this is
/// the same similarity transform as the one-at-a-time version.
pub fn charpoly(mut h: Vec<Vec<u64>>, p: u64) -> Vec<u64> {
    let n = h.len();
    for m in 1..n.saturating_sub(1) {
        let Some(i) = (m..n).find(|&i| h[i][m - 1] != 0) else { continue };
        if i != m {
            h.swap(i, m);
            for row in h.iter_mut() {
                row.swap(i, m);
            }
        }
        let inv = powmod(h[m][m - 1], p - 2, p);
        let u: Vec<u64> = (0..n).map(|i| if i > m { h[i][m - 1] * inv % p } else { 0 }).collect();
        let pivot = h[m].clone();
        par::for_each_mut(&mut h, |i, row| {
            let ui = u[i];
            if ui != 0 {
                for (x, &y) in row.iter_mut().zip(&pivot) {
                    *x = (*x + p - ui * y % p) % p;
                }
            }
        });
        par::for_each_mut(&mut h, |_, row| {
            let mut s = row[m];
            for i in m + 1..n {
                if u[i] != 0 {
                    s = (s + u[i] * row[i]) % p;
                }
            }
            row[m] = s;
        });
    }
    let mut polys: Vec<Vec<u64>> = vec![vec![1]];
    for m in 1..=n {
        // coefficients for prev = polys[m-1] and each polys[m-i-1]
        let mut terms: Vec<(usize, u64)> = vec![(m - 1, h[m - 1][m - 1])];
        let mut t = 1u64;
        for i in 1..m {
            t = t * h[m - i][m - i - 1] % p;
            terms.push((m - i - 1, t * h[m - i - 1][m - 1] % p));
        }
        let mut cur = vec![0u64; m + 1];
        for (k, c) in polys[m - 1].iter().enumerate() {
            cur[k + 1] = *c;
        }
        let polys_ref = &polys;
        par::for_each_mut(&mut cur, |k, x| {
            let mut s = *x;
            for &(j, coef) in &terms {
                if coef != 0 {
                    if let Some(&y) = polys_ref[j].get(k) {
                        s = (s + p - coef * y % p) % p;
                    }
                }
            }
            *x = s;
        });
        polys.push(cur);
    }
    polys.pop().unwrap()
}

pub fn matmul(a: &[Vec<u64>], b: &[Vec<u64>], p: u64) -> Vec<Vec<u64>> {
    let n = b.first().map_or(0, |r| r.len());
    par::map_slice(a, |row| {
        let mut out = vec![0u64; n];
        for (k, &x) in row.iter().enumerate() {
            if x != 0 {
                for (o, &y) in out.iter_mut().zip(&b[k]) {
                    *o = (*o + x * y) % p;
                }
            }
        }
        out
    })
}
