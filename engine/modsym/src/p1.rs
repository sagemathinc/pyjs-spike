//! P^1(Z/NZ) with Sage's normalization (sage/modular/modsym/p1list.pyx).
//! Normalized representatives are (0:1) and (g:v) with g a proper divisor
//! of N, so lookups use one table of length N per divisor.

pub fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

fn xgcd(a: i64, b: i64) -> (i64, i64) {
    // returns (g, s) with s*a == g (mod b)
    let (mut r0, mut r1, mut s0, mut s1) = (a, b, 1i64, 0i64);
    while r1 != 0 {
        let q = r0 / r1;
        (r0, r1) = (r1, r0 - q * r1);
        (s0, s1) = (s1, s0 - q * s1);
    }
    (r0, s0)
}

/// Canonical representative of (u:v), or None if gcd(u, v, N) != 1.
pub fn normalize(n: u64, u: u64, v: u64) -> Option<(u64, u64)> {
    let (u, v) = (u % n, v % n);
    if u == 0 {
        return if gcd(v, n) == 1 { Some((0, 1)) } else { None };
    }
    let (g, s) = xgcd(u as i64, n as i64);
    let g = g as u64;
    let mut s = s.rem_euclid(n as i64) as u64;
    if gcd(g, v) != 1 {
        return None;
    }
    if g != 1 {
        let d = n / g;
        while gcd(s, n) != 1 {
            s = (s + d) % n;
        }
    }
    let mut v = (s * v) % n;
    if g != 1 {
        let ng = n / g;
        let vng = (v * ng) % n;
        let (mut t, mut min_v) = (1u64, v);
        for _ in 2..=g {
            v = (v + vng) % n;
            t = (t + ng) % n;
            if v < min_v && gcd(t, n) == 1 {
                min_v = v;
            }
        }
        v = min_v;
    }
    Some((g, v))
}

pub struct P1List {
    n: u64,
    list: Vec<(u32, u32)>,
    zero_index: u32,       // index of (0:1)
    div_pos: Vec<u32>,     // g -> position in tables, for g | N
    tables: Vec<Vec<u32>>, // per divisor g: v -> index of (g:v)
}

impl P1List {
    pub fn new(n: u64) -> Self {
        assert!(n >= 1, "N must be at least 1");
        let mut list = vec![(0u32, 1u32)];
        let mut div_pos = vec![u32::MAX; n as usize + 1];
        let mut tables = vec![];
        for g in 1..n {
            if n % g != 0 {
                continue;
            }
            div_pos[g as usize] = tables.len() as u32;
            let mut table = vec![u32::MAX; n as usize];
            for v in 0..n {
                if gcd(g, v) == 1 && normalize(n, g, v) == Some((g, v)) {
                    table[v as usize] = list.len() as u32;
                    list.push((g as u32, v as u32));
                }
            }
            tables.push(table);
        }
        P1List { n, list, zero_index: 0, div_pos, tables }
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn get(&self, i: usize) -> (u64, u64) {
        let (c, d) = self.list[i];
        (c as u64, d as u64)
    }

    /// Index of (c:d) for any integers with gcd(c, d, N) == 1.
    pub fn index(&self, c: i64, d: i64) -> usize {
        let n = self.n as i64;
        let (u, v) = normalize(self.n, c.rem_euclid(n) as u64, d.rem_euclid(n) as u64).expect("not in P^1(Z/NZ)");
        if u == 0 {
            return self.zero_index as usize;
        }
        self.tables[self.div_pos[u as usize] as usize][v as usize] as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn psi(n: u64) -> u64 {
        let mut r = n;
        let mut m = n;
        let mut p = 2;
        while p * p <= m {
            if m % p == 0 {
                r = r / p * (p + 1);
                while m % p == 0 {
                    m /= p;
                }
            }
            p += 1;
        }
        if m > 1 {
            r = r / m * (m + 1);
        }
        r
    }

    #[test]
    fn size_is_psi() {
        for n in 1..300 {
            assert_eq!(P1List::new(n).len() as u64, psi(n), "N={}", n);
        }
    }

    /// Every (c, d) with gcd(c, d, N) = 1 indexes an element (u : v) with
    /// (c, d) = l (u, v) mod N for a unit l (checked by brute force).
    #[test]
    fn index_finds_an_equivalent_element() {
        for n in 1..40u64 {
            let p1 = P1List::new(n);
            for c in 0..n {
                for d in 0..n {
                    if gcd(gcd(c, d), n) != 1 {
                        continue;
                    }
                    let (u, v) = p1.get(p1.index(c as i64, d as i64));
                    let ok = (0..n.max(1)).any(|l| gcd(l, n) == 1 && (l * u) % n == c % n && (l * v) % n == d % n);
                    assert!(ok, "N={} (c,d)=({},{}) -> ({},{})", n, c, d, u, v);
                }
            }
        }
    }

    #[test]
    fn index_accepts_negative_and_large_inputs() {
        let p1 = P1List::new(30);
        assert_eq!(p1.index(-1, 7), p1.index(29, 7));
        assert_eq!(p1.index(31, -23), p1.index(1, 7));
    }
}
