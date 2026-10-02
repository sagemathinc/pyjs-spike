//! newforms_one N [MAX_SPLIT]: timing and structure for one level.
fn main() {
    let a: Vec<u64> = std::env::args().skip(1).map(|s| s.parse().unwrap()).collect();
    let t = std::time::Instant::now();
    let nf = sagebrush_modsym::newforms::rational_newforms(a[0], 97, a.get(1).map_or(40, |&x| x as usize)).unwrap();
    println!("N={} dim={} forms={} old={:?} split primes={} {:.3} s", nf.n, nf.dim, nf.forms.len(), nf.old.iter().map(|o| o.0).collect::<Vec<_>>(), nf.split_primes.len(), t.elapsed().as_secs_f64());
}
