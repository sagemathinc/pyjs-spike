//! modularity ALLCURVES FROM TO: for every level N in [FROM, TO], a_p for
//! p < 400 (p not dividing N) of the rational newforms from modular symbols
//! must equal, as multisets, a_p of Cremona's curves of conductor N counted
//! by point counting (sagebrush-ap): two independent computations that agree
//! only because elliptic curves over Q are modular.
use sagebrush_ap::EllipticCurve;
use sagebrush_modsym::newforms::newform_aps;
use std::collections::HashMap;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (from, to): (u64, u64) = (args[1].parse().unwrap(), args[2].parse().unwrap());
    let mut curves: HashMap<u64, Vec<[i64; 5]>> = HashMap::new();
    for line in std::fs::read_to_string(&args[0]).unwrap().lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        let n: u64 = w[0].parse().unwrap();
        if n < from || n > to || w[2] != "1" {
            continue; // one curve per isogeny class (isogenous curves share a_p)
        }
        let a: Vec<i64> = w[3].trim_matches(|c| c == '[' || c == ']').split(',').map(|s| s.parse().unwrap()).collect();
        curves.entry(n).or_default().push([a[0], a[1], a[2], a[3], a[4]]);
    }
    let t = std::time::Instant::now();
    use rayon::prelude::*;
    let levels: Vec<u64> = (from..=to).collect();
    let res: Vec<(u64, usize, usize, bool)> = levels
        .par_iter()
        .map(|&n| {
            let mut ours: Vec<Vec<(u64, i64)>> = newform_aps(n, 40).unwrap().iter().cloned().collect();
            let mut theirs: Vec<Vec<(u64, i64)>> = curves
                .get(&n)
                .map(|cs| {
                    cs.iter()
                        .map(|&a| {
                            let e = EllipticCurve::new(a).unwrap();
                            ours.first().map_or(vec![], |f| f.iter().map(|&(p, _)| (p, e.ap(p).expect("bad prime not dividing N"))).collect())
                        })
                        .collect()
                })
                .unwrap_or_default();
            ours.sort();
            theirs.sort();
            let pairs = ours.iter().map(|f| f.len()).sum();
            (n, ours.len(), pairs, ours == theirs && ours.len() == curves.get(&n).map_or(0, |c| c.len()))
        })
        .collect();
    let bad: Vec<_> = res.iter().filter(|r| !r.3).collect();
    for r in bad.iter().take(10) {
        println!("DISAGREE N={} newforms={}", r.0, r.1);
    }
    println!(
        "levels {}..={}: {} newforms, {} (newform, p) pairs compared with point counts, {} levels disagree, {:.1} s",
        from, to, res.iter().map(|r| r.1).sum::<usize>(), res.iter().map(|r| r.2).sum::<usize>(), bad.len(), t.elapsed().as_secs_f64()
    );
}
