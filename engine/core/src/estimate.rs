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
    /// Primes needed for the exact characteristic polynomial.
    pub primes: u64,
    /// Peak bytes for one prime, and for the exact computation.
    pub bytes_modp: f64,
    pub bytes_exact: f64,
    /// Single-thread seconds for one prime, and for the exact computation.
    pub seconds_modp: f64,
    pub seconds_exact: f64,
}

// Fitted constants (seconds per unit of work, free generators per symbol).
const GENS_PER_SYMBOL: f64 = 0.5;
const ELIM: f64 = 2e-9;
const HECKE: f64 = 2e-9;
const CHARPOLY: f64 = 2e-9;

pub fn estimate(n: u64, q: u64) -> Estimate {
    let (psi, genus, _, eis, dim) = level_data(n);
    let (m, d) = (GENS_PER_SYMBOL * psi as f64, dim as f64);
    let h = heilbronn(q as i64).len() as f64;
    let primes = (bound_bits(q, genus, eis) / 30.99).ceil();
    let bytes_modp = 8.0 * (m * d + 2.0 * d * d) + 64.0 * psi as f64;
    let seconds_modp = ELIM * m * d + HECKE * h * d * d + CHARPOLY * d * d * d;
    Estimate {
        n,
        q,
        symbols: psi,
        dim,
        genus,
        primes: primes as u64,
        bytes_modp,
        bytes_exact: bytes_modp * primes.min(8.0),
        seconds_modp,
        seconds_exact: seconds_modp * primes,
    }
}
