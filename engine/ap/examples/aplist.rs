//! aplist "[a1,a2,a3,a4,a6]" N [--dump]   (RAYON_NUM_THREADS=k to limit threads)
//! Prints the number of good primes p <= N, the sum of a_p over them and the
//! time; --dump also prints "p,-a_p" lines in the format of smalljac's lpdata.
use sagebrush_ap::{aplist, EllipticCurve};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let a: Vec<i64> = args[0].trim_matches(|c| c == '[' || c == ']').split(',').map(|s| s.trim().parse().unwrap()).collect();
    let n: u64 = args[1].parse().unwrap();
    let e = EllipticCurve::new([a[0], a[1], a[2], a[3], a[4]]).unwrap();
    let t = std::time::Instant::now();
    let r = aplist(&e, n);
    let secs = t.elapsed().as_secs_f64();
    let good: Vec<(u64, i64)> = r.iter().filter_map(|&(p, a)| a.map(|a| (p, a))).collect();
    let sum: i64 = good.iter().map(|g| g.1).sum();
    if args.iter().any(|s| s == "--dump") {
        for (p, a) in &good {
            println!("{},{}", p, -a);
        }
    }
    eprintln!("{} N={} {:.3} s, good primes {}, sum a_p {}", args[0], n, secs, good.len(), sum);
}
