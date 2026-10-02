//! Mod-p characteristic polynomial hashes printed by the pure-Python
//! reference implementation, bench/modsym/modsym.py (p = 67108859).

use sagebrush_modsym::hecke_charpoly;

#[test]
fn hashes_match_the_python_reference() {
    let cases = [
        (11, 2, 239875495584181673u128),
        (37, 3, 710763449808712194),
        (100, 3, 1635890953669117275),
        (389, 2, 689472025024149617),
        (997, 3, 1822777395048363841),
        (1001, 2, 1977030913831521789),
        (2003, 5, 1395610721400322499),
    ];
    for (n, q, hash) in cases {
        let r = hecke_charpoly(n, q, 67108859).unwrap();
        assert_eq!(r.hash(), hash, "N={} q={}", n, q);
        assert!(r.eisenstein_root(), "N={} q={}", n, q);
    }
}
