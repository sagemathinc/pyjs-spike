//! Times linalg::charpoly on a random n x n matrix mod p:
//! cargo run --release --example charpoly [n] [p]   (RAYON_NUM_THREADS=1 for one thread)
fn main() {
    let a: Vec<u64> = std::env::args().skip(1).map(|s| s.parse().unwrap()).collect();
    let (n, p) = (*a.first().unwrap_or(&835) as usize, *a.get(1).unwrap_or(&2147483647));
    let mut x: u64 = 12345;
    let mut rnd = || { x ^= x << 13; x ^= x >> 7; x ^= x << 17; x % p };
    let h: Vec<Vec<u64>> = (0..n).map(|_| (0..n).map(|_| rnd()).collect()).collect();
    let t = std::time::Instant::now();
    let f = modsym_core::linalg::charpoly(h, p);
    println!("n={} charpoly {:.3}s f[0]={}", n, t.elapsed().as_secs_f64(), f[0]);
}
