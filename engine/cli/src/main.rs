//! modsym-engine N q [p] [--threads T] [--exact] [--commute r] [--level]
//!
//! Default: characteristic polynomial of T_q mod p (prints a hash).
//! --exact: the characteristic polynomial over Z, with its status.
//! --commute r: check T_q T_r == T_r T_q mod p.
//! --level: genus, cusps and dimension from the formulas only.

fn take_flag(args: &mut Vec<String>, name: &str) -> bool {
    match args.iter().position(|a| a == name) {
        Some(i) => {
            args.remove(i);
            true
        }
        None => false,
    }
}

fn take_value(args: &mut Vec<String>, name: &str) -> Option<String> {
    let i = args.iter().position(|a| a == name)?;
    let v = args[i + 1].clone();
    args.drain(i..i + 2);
    Some(v)
}

fn fail(e: &str) -> ! {
    eprintln!("error: {}", e);
    std::process::exit(2)
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let threads: usize = take_value(&mut args, "--threads").map_or(0, |s| s.parse().unwrap());
    let commute: Option<u64> = take_value(&mut args, "--commute").map(|s| s.parse().unwrap());
    let exact = take_flag(&mut args, "--exact");
    let level = take_flag(&mut args, "--level");
    let n: u64 = args.first().map_or(389, |s| s.parse().unwrap());
    let q: u64 = args.get(1).map_or(2, |s| s.parse().unwrap());
    let p: u64 = args.get(2).map_or(67108859, |s| s.parse().unwrap());
    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build().unwrap();
    let t = std::time::Instant::now();
    if level {
        let (psi, g, c, e, d) = modsym_core::exact::level_data(n);
        println!("N={} psi={} genus={} cusps={} eisenstein+={} dim={}", n, psi, g, c, e, d);
        return;
    }
    if let Some(r) = commute {
        let ok = pool.install(|| modsym_core::hecke_commute(n, q, r, p)).unwrap_or_else(|e| fail(&e));
        println!("N={} T_{} T_{} commute mod {}: {} ({:.0} ms)", n, q, r, p, ok, t.elapsed().as_secs_f64() * 1000.0);
        return;
    }
    if exact {
        match pool.install(|| modsym_core::exact::exact_charpoly(n, q)) {
            Err(e) => fail(&e),
            Ok(e) => {
                let shown: Vec<String> = e.coeffs.iter().map(|c| c.to_string()).collect();
                let poly = if shown.len() <= 12 { shown.join(", ") } else { format!("{}, ..., {}", shown[..4].join(", "), shown[shown.len() - 4..].join(", ")) };
                println!("N={} q={} genus={} cusps={} dim={} status={}", e.n, e.q, e.genus, e.cusps, e.dim, e.status);
                println!("charpoly (constant term first): [{}]", poly);
                // FNV-1a of the comma-joined decimal coefficients, to compare runs.
                let h = shown.join(",").bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3));
                println!("coefficients digest {:016x}", h);
                println!("primes used {} (rejected {:?}), bound {:.0} bits; {:.0} ms on {} threads", e.primes_used.len(), e.primes_rejected, e.bound_bits, t.elapsed().as_secs_f64() * 1000.0, pool.current_num_threads());
                for c in &e.checks {
                    println!("  check: {}", c);
                }
            }
        }
        return;
    }
    let r = pool.install(|| modsym_core::hecke_charpoly(n, q, p)).unwrap_or_else(|e| fail(&e));
    println!(
        "N={} q={} p={} symbols={} gens={} dim={} eisenstein_root={} charpoly_hash={}",
        r.n, r.q, r.p, r.symbols, r.gens, r.dim, if r.eisenstein_root() { "True" } else { "False" }, r.hash()
    );
    println!(
        "times: symbols {:.0} ms, T_{} {:.0} ms, charpoly {:.0} ms, total {:.0} ms ({} threads)",
        r.ms[0], q, r.ms[1], r.ms[2], r.ms.iter().sum::<f64>(), pool.current_num_threads()
    );
}
