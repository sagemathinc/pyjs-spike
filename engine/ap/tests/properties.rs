//! Internal consistency: the baby-step giant-step answer equals direct
//! counting, a_p obeys the Hasse bound, and bad input is an error.

use sagebrush_ap::{isqrt, primes_up_to, EllipticCurve};

#[test]
fn bsgs_agrees_with_counting_points() {
    let curves = [[0, -1, 1, -10, -20], [1, 1, 1, -10, -10], [0, 0, 0, -1, 0], [0, 0, 1, 0, 0], [1, -1, 0, -4, 4], [0, 0, 0, 3, 7]];
    for a in curves {
        let e = EllipticCurve::new(a).unwrap();
        for p in primes_up_to(12000).into_iter().filter(|&p| p >= 1000) {
            assert_eq!(e.ap(p), if e.bad(p) { None } else { Some(e.ap_naive(p)) }, "{:?} p={}", a, p);
        }
    }
}

#[test]
fn hasse_bound() {
    let e = EllipticCurve::new([0, 1, 1, -2, 0]).unwrap();
    for (p, a) in sagebrush_ap::aplist(&e, 200000) {
        let a = a.unwrap_or(0);
        assert!(a * a <= 4 * p as i64, "p={} a={}", p, a);
    }
}

#[test]
fn invalid_curves_are_errors() {
    assert!(EllipticCurve::new([0, 0, 0, 0, 0]).is_err()); // singular
    assert!(EllipticCurve::new([0, 0, 0, -3, 2]).is_err()); // node at (1, 0)
    assert!(EllipticCurve::new([0, 0, 0, i64::MAX, i64::MAX]).is_err()); // overflow
}

#[test]
fn primes_and_isqrt() {
    assert_eq!(primes_up_to(30), vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
    assert_eq!(primes_up_to(10_000_000).len(), 664579);
    for n in [0u64, 1, 3, 4, 15, 16, 17, (1 << 40) + 7] {
        let r = isqrt(n);
        assert!(r * r <= n && (r + 1) * (r + 1) > n);
    }
}
