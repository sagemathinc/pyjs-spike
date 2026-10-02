//! Line-by-line Rust port of ../modsym.py: weight-2 modular symbols for
//! Gamma0(N), sign +1, over GF(p), Hecke operator T_q, charpoly mod p.
//! Same algorithms and data layout (Vec for Python lists, HashMap where the
//! Python code uses a dict); i64 arithmetic, rem_euclid for Python's `%`.

use std::collections::HashMap;
use std::time::Instant;

fn gcd(mut a: i64, mut b: i64) -> i64 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

fn p1_list(n: i64) -> (Vec<(i64, i64)>, Vec<i64>) {
    let units: Vec<i64> = if n > 1 { (1..n).filter(|&u| gcd(u, n) == 1).collect() } else { vec![0] };
    let mut index = vec![-1i64; (n * n) as usize];
    let mut reps = Vec::new();
    for c in 0..n {
        for d in 0..n {
            if index[(c * n + d) as usize] != -1 || gcd(gcd(c, d), n) != 1 {
                continue;
            }
            let i = reps.len() as i64;
            reps.push((c, d));
            for &u in &units {
                index[((u * c % n) * n + (u * d % n)) as usize] = i;
            }
        }
    }
    (reps, index)
}

struct ManinSymbols {
    n: i64,
    p: i64,
    reps: Vec<(i64, i64)>,
    index: Vec<i64>,
    dim: usize,
    vec: Vec<Vec<(usize, i64)>>,
    basis_symbol: Vec<(usize, i64)>,
}

impl ManinSymbols {
    fn new(n: i64, p: i64) -> Self {
        let (reps, index) = p1_list(n);
        let mut m = ManinSymbols { n, p, reps, index, dim: 0, vec: vec![], basis_symbol: vec![] };
        m.quotient();
        m
    }

    fn idx(&self, c: i64, d: i64) -> usize {
        let n = self.n;
        self.index[(c.rem_euclid(n) * n + d.rem_euclid(n)) as usize] as usize
    }

    fn quotient(&mut self) {
        let n = self.reps.len();
        let p = self.p;
        let mut parent: Vec<usize> = (0..n).collect();
        let mut sign = vec![1i64; n];
        let mut zero = vec![false; n];
        fn find(parent: &[usize], sign: &[i64], mut i: usize) -> (usize, i64) {
            let mut s = 1;
            while parent[i] != i {
                s *= sign[i];
                i = parent[i];
            }
            (i, s)
        }
        let mut union = |parent: &mut Vec<usize>, sign: &mut Vec<i64>, zero: &mut Vec<bool>, i: usize, j: usize, s: i64| {
            let (ri, si) = find(parent, sign, i);
            let (rj, sj) = find(parent, sign, j);
            if ri == rj {
                if si != s * sj {
                    zero[ri] = true;
                }
                return;
            }
            parent[ri] = rj;
            sign[ri] = s * sj * si;
            if zero[ri] {
                zero[rj] = true;
            }
        };
        for i in 0..n {
            let (c, d) = self.reps[i];
            let j = self.idx(d, -c);
            union(&mut parent, &mut sign, &mut zero, i, j, -1);
            let j = self.idx(-c, d);
            union(&mut parent, &mut sign, &mut zero, i, j, 1);
        }
        let mut free: HashMap<usize, usize> = HashMap::new();
        let mut rep_of: Vec<(i64, i64)> = vec![(0, 0); n];
        for i in 0..n {
            let (r, s) = find(&parent, &sign, i);
            if zero[r] {
                rep_of[i] = (-1, 0);
                continue;
            }
            let k = free.len();
            let f = *free.entry(r).or_insert(k);
            rep_of[i] = (f as i64, s);
        }
        let m = free.len();
        let mut rows: Vec<Vec<i64>> = Vec::new();
        for &(c, d) in &self.reps {
            let mut row = vec![0i64; m];
            for (a, b) in [(c, d), (d, -c - d), (-c - d, c)] {
                let (k, s) = rep_of[self.idx(a, b)];
                if k >= 0 {
                    row[k as usize] = (row[k as usize] + s).rem_euclid(p);
                }
            }
            if row.iter().any(|&x| x != 0) {
                rows.push(row);
            }
        }
        let pivots = echelon(&mut rows, m, p);
        let nonpivot: Vec<usize> = (0..m).filter(|j| !pivots.contains_key(j)).collect();
        self.dim = nonpivot.len();
        let col: HashMap<usize, usize> = nonpivot.iter().enumerate().map(|(t, &j)| (j, t)).collect();
        let mut coords = Vec::with_capacity(m);
        for j in 0..m {
            let mut v = vec![0i64; self.dim];
            if let Some(&t) = col.get(&j) {
                v[t] = 1;
            } else {
                let row = &rows[pivots[&j]];
                for &k in &nonpivot {
                    if row[k] != 0 {
                        v[col[&k]] = (-row[k]).rem_euclid(p);
                    }
                }
            }
            coords.push(v);
        }
        self.vec = (0..n)
            .map(|i| {
                let (k, s) = rep_of[i];
                if k < 0 {
                    vec![]
                } else {
                    coords[k as usize].iter().enumerate().filter(|(_, &x)| x != 0).map(|(t, &x)| (t, (s * x).rem_euclid(p))).collect()
                }
            })
            .collect();
        for &j in &nonpivot {
            for i in 0..n {
                let (k, s) = rep_of[i];
                if k == j as i64 {
                    self.basis_symbol.push((i, s));
                    break;
                }
            }
        }
    }

    fn hecke(&self, q: i64) -> Vec<Vec<i64>> {
        let h = heilbronn(q);
        let p = self.p;
        let mut out = Vec::with_capacity(self.dim);
        for &(i, s) in &self.basis_symbol {
            let (c, d) = self.reps[i];
            let mut v = vec![0i64; self.dim];
            for &(a, b, cc, dd) in &h {
                for &(t, x) in &self.vec[self.idx(c * a + d * cc, c * b + d * dd)] {
                    v[t] = (v[t] + s * x).rem_euclid(p);
                }
            }
            out.push(v);
        }
        out
    }
}

fn echelon(rows: &mut Vec<Vec<i64>>, ncols: usize, p: i64) -> HashMap<usize, usize> {
    let mut pivots = HashMap::new();
    let mut r = 0;
    let nrows = rows.len();
    for c in 0..ncols {
        let piv = (r..nrows).find(|&i| rows[i][c] != 0);
        let Some(piv) = piv else { continue };
        rows.swap(r, piv);
        let inv = powmod(rows[r][c], p - 2, p);
        for k in c..ncols {
            if rows[r][k] != 0 {
                rows[r][k] = rows[r][k] * inv % p;
            }
        }
        let row = rows[r].clone();
        for i in 0..nrows {
            if i != r {
                let other = &mut rows[i];
                let f = other[c];
                if f != 0 {
                    for k in c..ncols {
                        if row[k] != 0 {
                            other[k] = (other[k] - f * row[k]).rem_euclid(p);
                        }
                    }
                }
            }
        }
        pivots.insert(c, r);
        r += 1;
        if r == nrows {
            break;
        }
    }
    rows.truncate(r);
    pivots
}

fn powmod(mut b: i64, mut e: i64, m: i64) -> i64 {
    let mut r = 1i64;
    b = b.rem_euclid(m);
    while e > 0 {
        if e & 1 == 1 {
            r = r * b % m;
        }
        b = b * b % m;
        e >>= 1;
    }
    r
}

fn heilbronn(q: i64) -> Vec<(i64, i64, i64, i64)> {
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
            let c = a - b * qq;
            a = -b;
            b = c;
            let x3 = qq * x2 - x1;
            x1 = x2;
            x2 = x3;
            let y3 = qq * y2 - y1;
            y1 = y2;
            y2 = y3;
            out.push((x1, x2, y1, y2));
        }
    }
    out
}

fn charpoly(m: &[Vec<i64>], p: i64) -> Vec<i64> {
    let n = m.len();
    let mut h: Vec<Vec<i64>> = m.to_vec();
    for mm in 1..n.saturating_sub(1) {
        let mut i = mm;
        while i < n && h[i][mm - 1] == 0 {
            i += 1;
        }
        if i == n {
            continue;
        }
        if i != mm {
            h.swap(i, mm);
            for r in 0..n {
                h[r].swap(i, mm);
            }
        }
        let inv = powmod(h[mm][mm - 1], p - 2, p);
        for i in mm + 1..n {
            let u = h[i][mm - 1] * inv % p;
            if u != 0 {
                for k in 0..n {
                    h[i][k] = (h[i][k] - u * h[mm][k]).rem_euclid(p);
                }
                for r in 0..n {
                    h[r][mm] = (h[r][mm] + u * h[r][i]).rem_euclid(p);
                }
            }
        }
    }
    let mut polys: Vec<Vec<i64>> = vec![vec![1]];
    for mm in 1..=n {
        let prev = &polys[mm - 1];
        let mut cur = vec![0i64];
        cur.extend_from_slice(prev);
        let hh = h[mm - 1][mm - 1];
        for k in 0..prev.len() {
            cur[k] = (cur[k] - hh * prev[k]).rem_euclid(p);
        }
        let mut t = 1i64;
        for i in 1..mm {
            t = t * h[mm - i][mm - i - 1] % p;
            let coef = t * h[mm - i - 1][mm - 1] % p;
            let sub = &polys[mm - i - 1];
            for k in 0..sub.len() {
                cur[k] = (cur[k] - coef * sub[k]).rem_euclid(p);
            }
        }
        polys.push(cur);
    }
    polys.pop().unwrap()
}

fn evalpoly(f: &[i64], x: i64, p: i64) -> i64 {
    f.iter().rev().fold(0, |r, &c| (r * x + c).rem_euclid(p))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n: i64 = args.get(1).map_or(389, |s| s.parse().unwrap());
    let q: i64 = args.get(2).map_or(2, |s| s.parse().unwrap());
    let p: i64 = args.get(3).map_or(67108859, |s| s.parse().unwrap());
    let t0 = Instant::now();
    let m = ManinSymbols::new(n, p);
    let t1 = Instant::now();
    let t = m.hecke(q);
    let t2 = Instant::now();
    let f = charpoly(&t, p);
    let t3 = Instant::now();
    let mut hsh: i128 = 0;
    for &c in &f {
        hsh = (hsh * 1000003 + c as i128) % 2305843009213693951;
    }
    let ms = |a: Instant, b: Instant| (b - a).as_secs_f64() * 1000.0;
    println!("N={} q={} p={} symbols={} dim={} eisenstein_root={} charpoly_hash={}", n, q, p, m.reps.len(), m.dim, if evalpoly(&f, q + 1, p) == 0 { "True" } else { "False" }, hsh);
    println!("times: symbols {:.0} ms, T_{} {:.0} ms, charpoly {:.0} ms, total {:.0} ms", ms(t0, t1), q, ms(t1, t2), ms(t2, t3), ms(t0, t3));
}
