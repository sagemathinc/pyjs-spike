//! fillin N...: nonzeros in the sparse pivot rows vs the dense coordinate table.
use sagebrush_modsym::{linalg, presentation::Presentation};
fn main() {
    for n in std::env::args().skip(1).map(|s| s.parse::<u64>().unwrap()) {
        let p = 2147483629u64;
        let pres = Presentation::new(n);
        let rows: Vec<Vec<(u32, u64)>> = pres.rows.iter().map(|r| r.iter().map(|&(g, v)| (g, v.rem_euclid(p as i64) as u64)).filter(|e| e.1 != 0).collect()).collect();
        let (piv, _) = linalg::sparse_echelon(&rows, pres.m, p);
        let nnz: usize = piv.iter().map(|r| r.1.len()).sum();
        let dim = pres.m - piv.len();
        println!("N={} gens m={} dim d={} pivot-row nonzeros {} ({:.1} per row) vs dense m*d = {} ({:.0}x)", n, pres.m, dim, nnz, nnz as f64 / piv.len() as f64, pres.m * dim, (pres.m * dim) as f64 / nnz as f64);
    }
}
