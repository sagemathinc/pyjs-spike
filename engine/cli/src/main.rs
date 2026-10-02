//! modsym-engine N q [p] [--threads T]   (T = 0: all cores)

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut threads = 0usize;
    if let Some(i) = args.iter().position(|a| a == "--threads") {
        threads = args[i + 1].parse().unwrap();
        args.drain(i..i + 2);
    }
    let n: u64 = args.first().map_or(389, |s| s.parse().unwrap());
    let q: u64 = args.get(1).map_or(2, |s| s.parse().unwrap());
    let p: u64 = args.get(2).map_or(67108859, |s| s.parse().unwrap());
    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build().unwrap();
    let r = pool.install(|| modsym_core::hecke_charpoly(n, q, p));
    println!(
        "N={} q={} p={} symbols={} dim={} eisenstein_root={} charpoly_hash={}",
        r.n, r.q, r.p, r.symbols, r.dim, if r.eisenstein_root() { "True" } else { "False" }, r.hash()
    );
    println!(
        "times: symbols {:.0} ms, T_{} {:.0} ms, charpoly {:.0} ms, total {:.0} ms ({} threads)",
        r.ms[0], q, r.ms[1], r.ms[2], r.ms.iter().sum::<f64>(), pool.current_num_threads()
    );
}
