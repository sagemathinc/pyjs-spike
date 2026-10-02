//! Predicted dimension, memory and time before running anything, so a
//! caller (or an agent) can decide what is affordable.  The constants are
//! fitted to measurements on an 8-core AMD EPYC (see results/engine-exact.md).

use crate::exact::{bound_bits, level_data};
use crate::linalg::heilbronn;

pub struct Estimate {
    pub n: u64,
    pub q: u64,
    pub symbols: u64,
    pub dim: u64,
    pub genus: u64,
    /// Primes the exact characteristic polynomial typically needs (the
    /// proven bound from the sum of squares, with Sato-Tate's r = sqrt(q)),
    /// and at most (the worst-case Deligne bound).
    pub primes: u64,
    pub primes_max: u64,
    /// Peak bytes for one prime, and for the exact computation (up to 8
    /// primes are in flight at once).
    pub bytes_modp: f64,
    pub bytes_exact: f64,
    /// Single-thread seconds for one prime, and for the exact computation.
    pub seconds_modp: f64,
    pub seconds_exact: f64,
}

// Fitted on one thread for dim 84..3334: free generators are psi/4; the
// elimination constant is an upper bound (highly composite levels have the
// most fill-in, prime levels are ~5x cheaper); the charpoly constant grows
// once the matrix leaves the cache.
const GENS_PER_SYMBOL: f64 = 0.25;
const ELIM: f64 = 4e-8;
const HECKE: f64 = 3e-9;
const CHARPOLY: f64 = 4.4e-10;

pub fn estimate(n: u64, q: u64) -> Estimate {
    let (psi, genus, _, eis, dim) = level_data(n);
    let (m, d) = (GENS_PER_SYMBOL * psi as f64, dim as f64);
    let h = heilbronn(q as i64).len() as f64;
    let primes_max = (bound_bits(q, genus, eis) / 30.99).ceil();
    let typical = genus as f64 * (1.0 + (q as f64).sqrt()).log2() + eis as f64 * (2.0 + q as f64).log2() + 2.0;
    let primes = (typical / 30.99).ceil().min(primes_max);
    // Dense coordinates of every free generator dominate, then the matrix.
    let bytes_modp = 8.0 * m * d + 4.0 * d * d + 3e6;
    let seconds_modp = ELIM * m * d + HECKE * h * d * d + CHARPOLY * (1.0 + d / 4000.0) * d * d * d;
    Estimate {
        n,
        q,
        symbols: psi,
        dim,
        genus,
        primes: primes as u64,
        primes_max: primes_max as u64,
        bytes_modp,
        bytes_exact: bytes_modp * primes.min(8.0),
        seconds_modp,
        seconds_exact: seconds_modp * primes,
    }
}
