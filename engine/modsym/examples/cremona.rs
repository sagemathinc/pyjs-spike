//! cremona APLIST FROM TO [MAX_SPLIT]: the rational newforms of every level in
//! [FROM, TO], compared with Cremona's isogeny classes (a_p for good p < 100,
//! from ecdata's aplist file).  Prints mismatches and a summary.
use sagebrush_modsym::newforms::newform_aps;
use std::collections::HashMap;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (from, to): (u64, u64) = (args[1].parse().unwrap(), args[2].parse().unwrap());
    let max_split: usize = args.get(3).map_or(40, |s| s.parse().unwrap());
    let primes: Vec<u64> = (2..100).filter(|&q| (2..q).all(|d| q % d != 0)).collect();
    let mut cremona: HashMap<u64, Vec<Vec<i64>>> = HashMap::new();
    for line in std::fs::read_to_string(&args[0]).unwrap().lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        let n: u64 = w[0].parse().unwrap();
        if n < from || n > to {
            continue;
        }
        let aps: Vec<i64> = primes.iter().zip(&w[2..]).filter(|(q, _)| n % **q != 0).map(|(_, a)| a.parse().unwrap()).collect();
        cremona.entry(n).or_default().push(aps);
    }
    let levels: Vec<u64> = (from..=to).collect();
    let t = std::time::Instant::now();
    use rayon::prelude::*;
    let results: Vec<(u64, usize, usize, bool, bool)> = levels
        .par_iter()
        .map(|&n| {
            let forms = newform_aps(n, max_split).unwrap();
            let mut ours: Vec<Vec<i64>> = forms.iter().map(|f| f.iter().filter(|x| x.0 < 100).map(|x| x.1).collect()).collect();
            let mut theirs = cremona.get(&n).cloned().unwrap_or_default();
            ours.sort();
            theirs.sort();
            let hasse = forms.iter().all(|f| f.iter().all(|&(p, a)| a * a <= 4 * p as i64));
            (n, ours.len(), theirs.len(), ours == theirs, hasse)
        })
        .collect();
    let secs = t.elapsed().as_secs_f64();
    let bad: Vec<_> = results.iter().filter(|r| !r.3 || !r.4).collect();
    for r in bad.iter().take(20) {
        println!("MISMATCH N={} ours={} cremona={} hasse_ok={}", r.0, r.1, r.2, r.4);
    }
    let forms: usize = results.iter().map(|r| r.1).sum();
    println!("levels {}..={}: {} rational newforms, {} levels disagree, {:.2} s", from, to, forms, bad.len(), secs);
}
