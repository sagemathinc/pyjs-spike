//! Time general.rs on the spaces in ~/data/sage/general_time.json (written
//! by examples/sage/general_time.sage, which records Sage's timings for the same spaces).
use sagebrush_modsym::general::{Character, GeneralSpace};
use serde_json::Value;
use std::time::Instant;

fn main() {
    let path = format!("{}/data/sage/general_time.json", std::env::var("HOME").unwrap());
    let cases: Vec<Value> = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    println!("| N | k | sign | ord eps | dim | Sage space | Sage T_2 + charpoly | ours space | ours T_2 + charpoly mod ell |");
    println!("|---|---|---|---|---|---|---|---|---|");
    for r in cases {
        let n = r["N"].as_u64().unwrap();
        let k = r["k"].as_u64().unwrap() as usize;
        let sign = r["sign"].as_i64().unwrap() as i32;
        let e = r["e"].as_u64().unwrap().max(1);
        let exps: Vec<u32> = r["exps"].as_array().unwrap().iter().map(|x| x.as_u64().map_or(u32::MAX, |v| v as u32)).collect();
        let chi = Character::from_exponents(n, e, exps).unwrap();
        let t = Instant::now();
        let sp = GeneralSpace::new(n, k, &chi, sign).unwrap();
        let t1 = t.elapsed().as_secs_f64();
        let t = Instant::now();
        let _f = sp.hecke_charpoly(2).unwrap();
        let t2 = t.elapsed().as_secs_f64();
        assert_eq!(sp.dimension() as u64, r["dim"].as_u64().unwrap());
        let ms = |x: f64| format!("{:.1} ms", 1000.0 * x);
        let sage_h = r["t2"].as_f64().unwrap() + r["charpoly"].as_f64().unwrap();
        println!("| {} | {} | {} | {} | {} | {} | {} | {} | {} |", n, k, sign, r["order"], sp.dimension(), ms(r["space"].as_f64().unwrap()), ms(sage_h), ms(t1), ms(t2));
    }
}
