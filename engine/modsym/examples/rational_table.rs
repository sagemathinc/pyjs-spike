//! rational_table OUT.jsonl FROM TO: every rational newform of each level in
//! [FROM, TO] with a_p for primes p < 1000 not dividing N (in parallel over
//! levels), one JSON object per newform.
use rayon::prelude::*;
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (from, to): (u64, u64) = (args[1].parse().unwrap(), args[2].parse().unwrap());
    let t = std::time::Instant::now();
    let levels: Vec<u64> = (from..=to).collect();
    let all: Vec<(u64, Vec<Vec<(u64, i64)>>)> = levels
        .par_iter()
        .map(|&n| (n, sagebrush_modsym::newforms::rational_newforms(n, 999, 40).unwrap().forms.into_iter().map(|f| f.ap).collect()))
        .collect();
    let mut out = std::io::BufWriter::new(std::fs::File::create(&args[0]).unwrap());
    let mut count = 0;
    for (n, forms) in all {
        for ap in forms {
            count += 1;
            writeln!(out, "{}", serde_json::json!({"level": n, "ap": ap})).unwrap();
        }
    }
    eprintln!("levels {}..={}: {} rational newforms, {:.1} s", from, to, count, t.elapsed().as_secs_f64());
}
