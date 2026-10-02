//! The dimension from the genus and cusp formulas equals the dimension of
//! the quotient actually computed, for every level up to 1000 and some
//! larger, highly composite ones.

use sagebrush_modsym::exact::level_data;
use sagebrush_modsym::presentation::Presentation;
use sagebrush_modsym::space::Space;

#[test]
fn formula_matches_computed_dimension() {
    let mut wrong = vec![];
    for n in (1..=1000).chain([1728, 2310, 4096, 5000]) {
        let computed = Space::new(&Presentation::new(n), 1000003).dimension() as u64;
        let formula = level_data(n).4;
        if computed != formula {
            wrong.push((n, formula, computed));
        }
    }
    assert!(wrong.is_empty(), "(N, formula, computed): {:?}", wrong);
}
