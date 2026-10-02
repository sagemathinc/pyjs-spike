//! Agreement with Drew Sutherland's smalljac 4.1.3 (built separately; the
//! numbers below are its output): the number of good primes and the sum of
//! a_p up to 10^6 for four curves, and a digest of smalljac's complete
//! per-prime output ("p,-a_p" lines from lpdata) for 11a up to 10^6.

use sagebrush_ap::{aplist, EllipticCurve};

fn good(a: [i64; 5], n: u64) -> Vec<(u64, i64)> {
    aplist(&EllipticCurve::new(a).unwrap(), n).into_iter().filter_map(|(p, a)| a.map(|a| (p, a))).collect()
}

#[test]
fn counts_and_trace_sums_match_smalljac() {
    for (a, count, sum) in [
        ([0, -1, 1, -10, -20], 78497, 10334),
        ([0, 0, 1, -1, 0], 78497, -13432),
        ([0, 0, 1, 0, 0], 78497, 94317),
        ([1, -1, 1, -1394, 19431], 78495, 65973),
    ] {
        let g = good(a, 1_000_000);
        assert_eq!((g.len(), g.iter().map(|x| x.1).sum::<i64>()), (count, sum), "{:?}", a);
    }
}

#[test]
fn every_a_p_of_11a_below_a_million_matches_smalljac() {
    let mut h = 0xcbf29ce484222325u64;
    for (p, a) in good([0, -1, 1, -10, -20], 1_000_000) {
        for b in format!("{},{}\n", p, -a).bytes() {
            h = (h ^ b as u64).wrapping_mul(0x100000001b3);
        }
    }
    assert_eq!(format!("{:016x}", h), "4427589f51a01af4");
}
