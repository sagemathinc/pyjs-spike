//! Compare general.rs (weight k, character, sign) with Sage's
//! ModularSymbols(eps, k, sign): dimensions and T_q charpolys mod ell.
//! Reference data: ~/data/sage/general_ref.jsonl from examples/sage/general.sage.
use sagebrush_modsym::general::{prime_field, Character, GeneralSpace};
use serde_json::Value;

fn powmod(mut b: u64, mut e: u64, p: u64) -> u64 {
    let mut r = 1u64;
    b %= p;
    while e > 0 {
        if e & 1 == 1 {
            r = (r as u128 * b as u128 % p as u128) as u64;
        }
        b = (b as u128 * b as u128 % p as u128) as u64;
        e >>= 1;
    }
    r
}

fn rat(s: &str, p: u64) -> u64 {
    let red = |t: &str| {
        let neg = t.starts_with('-');
        let t = t.trim_start_matches('-');
        let mut r = 0u64;
        for ch in t.bytes() {
            r = ((r as u128 * 10 + (ch - b'0') as u128) % p as u128) as u64;
        }
        if neg { (p - r) % p } else { r }
    };
    match s.split_once('/') {
        Some((a, b)) => (red(a) as u128 * powmod(red(b), p - 2, p) as u128 % p as u128) as u64,
        None => red(s),
    }
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| format!("{}/data/sage/general_ref.jsonl", std::env::var("HOME").unwrap()));
    let text = std::fs::read_to_string(path).unwrap();
    let (mut ok, mut bad) = (0, 0);
    for line in text.lines() {
        let r: Value = serde_json::from_str(line).unwrap();
        let n = r["N"].as_u64().unwrap();
        let k = r["k"].as_u64().unwrap() as usize;
        let sign = r["sign"].as_i64().unwrap() as i32;
        let e = r["e"].as_u64().unwrap().max(1);
        let m = r["m"].as_u64().unwrap();
        let exps: Vec<u32> = r["exps"].as_array().unwrap().iter().map(|x| x.as_u64().map_or(u32::MAX, |v| v as u32)).collect();
        let chi = Character::from_exponents(n, e, exps).unwrap();
        let (p, zeta) = prime_field(e, 1 << 31);
        let sp = GeneralSpace::new_mod(n, k, &chi, sign, p, zeta).unwrap();
        let want = r["dim"].as_u64().unwrap() as usize;
        let mut msg = vec![];
        if sp.dimension() != want {
            msg.push(format!("dim {} vs sage {}", sp.dimension(), want));
        } else {
            let zm = if e % m == 0 { powmod(zeta, e / m, p) } else { 0 };
            for (q, coeffs) in r["polys"].as_object().unwrap() {
                let q: u64 = q.parse().unwrap();
                let sage: Vec<u64> = coeffs.as_array().unwrap().iter().map(|c| {
                    let c = c.as_array().unwrap();
                    c.iter().enumerate().fold(0u64, |acc, (i, x)| {
                        let t = rat(x.as_str().unwrap(), p) as u128 * powmod(zm, i as u64, p) as u128 % p as u128;
                        (acc + t as u64) % p
                    })
                }).collect();
                let ours = sp.hecke_charpoly(q).unwrap();
                if ours != sage {
                    msg.push(format!("T_{} differs", q));
                }
            }
        }
        if msg.is_empty() {
            ok += 1;
        } else {
            bad += 1;
            if bad <= 25 {
                println!("N={} k={} sign={} e={} even={}: {}", n, k, sign, e, chi.is_even(), msg.join("; "));
            }
        }
    }
    println!("{} agree, {} differ", ok, bad);
}
