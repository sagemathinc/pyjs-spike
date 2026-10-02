//! CPython bindings: the `sagebrush._native` extension.  Computations
//! release the GIL, so Python threads can run several in parallel; each call
//! also uses `threads` worker threads (0 = all cores).  Invalid arguments
//! raise ValueError.

use sagebrush_modsym::exact::Exact;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

fn run<T: Send>(py: Python<'_>, threads: usize, f: impl FnOnce() -> T + Send) -> T {
    py.detach(|| rayon::ThreadPoolBuilder::new().num_threads(threads).build().unwrap().install(f))
}

fn err(e: String) -> PyErr {
    PyValueError::new_err(e)
}

/// T_q's characteristic polynomial mod p (constant term first).
#[pyfunction]
#[pyo3(signature = (n, q, p=67108859, threads=0))]
fn hecke_charpoly<'py>(py: Python<'py>, n: u64, q: u64, p: u64, threads: usize) -> PyResult<Bound<'py, PyDict>> {
    let r = run(py, threads, || sagebrush_modsym::hecke_charpoly(n, q, p)).map_err(err)?;
    let d = PyDict::new(py);
    d.set_item("symbols", r.symbols)?;
    d.set_item("gens", r.gens)?;
    d.set_item("dim", r.dim)?;
    d.set_item("charpoly", r.charpoly.clone())?;
    d.set_item("hash", r.hash())?;
    d.set_item("eisenstein_root", r.eisenstein_root())?;
    d.set_item("ms", r.ms.to_vec())?;
    Ok(d)
}

fn exact_dict<'py>(py: Python<'py>, e: &Exact) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("n", e.n)?;
    d.set_item("q", e.q)?;
    d.set_item("genus", e.genus)?;
    d.set_item("cusps", e.cusps)?;
    d.set_item("eisenstein", e.eis)?;
    d.set_item("dim", e.dim)?;
    d.set_item("charpoly", e.coeffs.clone())?;
    d.set_item("primes_used", e.primes_used.len())?;
    d.set_item("bound_bits", e.bound_bits)?;
    d.set_item("status", e.status)?;
    d.set_item("checks", e.checks.clone())?;
    Ok(d)
}

/// T_q's characteristic polynomial over Z, proven by CRT with a coefficient bound.
#[pyfunction]
#[pyo3(signature = (n, q, threads=0))]
fn charpoly_exact<'py>(py: Python<'py>, n: u64, q: u64, threads: usize) -> PyResult<Bound<'py, PyDict>> {
    let e = run(py, threads, || sagebrush_modsym::exact::exact_charpoly(n, q)).map_err(err)?;
    exact_dict(py, &e)
}

/// charpoly_exact for many levels in parallel; failures are {"n", "error"} dicts.
#[pyfunction]
#[pyo3(signature = (levels, q, threads=0))]
fn batch_exact<'py>(py: Python<'py>, levels: Vec<u64>, q: u64, threads: usize) -> PyResult<Vec<Bound<'py, PyDict>>> {
    let rs = run(py, threads, || sagebrush_modsym::exact::batch_exact(&levels, q));
    levels
        .iter()
        .zip(rs)
        .map(|(&n, r)| match r {
            Ok(e) => exact_dict(py, &e),
            Err(e) => {
                let d = PyDict::new(py);
                d.set_item("n", n)?;
                d.set_item("error", e)?;
                Ok(d)
            }
        })
        .collect()
}

/// Psi(N), genus, cusps, Eisenstein dimension and dimension of the sign +1 space.
#[pyfunction]
fn level_data<'py>(py: Python<'py>, n: u64) -> PyResult<Bound<'py, PyDict>> {
    let (psi, g, c, e, dim) = sagebrush_modsym::exact::level_data(n);
    let d = PyDict::new(py);
    d.set_item("psi", psi)?;
    d.set_item("genus", g)?;
    d.set_item("cusps", c)?;
    d.set_item("eisenstein", e)?;
    d.set_item("dim", dim)?;
    Ok(d)
}

/// Whether T_q and T_r commute mod p.
#[pyfunction]
#[pyo3(signature = (n, q, r, p=67108859, threads=0))]
fn commute(py: Python<'_>, n: u64, q: u64, r: u64, p: u64, threads: usize) -> PyResult<bool> {
    run(py, threads, || sagebrush_modsym::hecke_commute(n, q, r, p)).map_err(err)
}

/// Predicted dimension, bytes and single-thread seconds, without computing.
#[pyfunction]
fn estimate<'py>(py: Python<'py>, n: u64, q: u64) -> PyResult<Bound<'py, PyDict>> {
    sagebrush_modsym::validate(n, q, None).map_err(err)?;
    let e = sagebrush_modsym::estimate::estimate(n, q);
    let d = PyDict::new(py);
    d.set_item("symbols", e.symbols)?;
    d.set_item("dim", e.dim)?;
    d.set_item("genus", e.genus)?;
    d.set_item("primes", e.primes)?;
    d.set_item("primes_max", e.primes_max)?;
    d.set_item("bytes_modp", e.bytes_modp)?;
    d.set_item("bytes_exact", e.bytes_exact)?;
    d.set_item("seconds_modp", e.seconds_modp)?;
    d.set_item("seconds_exact", e.seconds_exact)?;
    Ok(d)
}

/// The native extension, `sagebrush._native`; each engine is a submodule,
/// re-exported by the pure-Python package (python/sagebrush).
#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let modsym = PyModule::new(m.py(), "modsym")?;
    modsym.add_function(wrap_pyfunction!(hecke_charpoly, &modsym)?)?;
    modsym.add_function(wrap_pyfunction!(charpoly_exact, &modsym)?)?;
    modsym.add_function(wrap_pyfunction!(batch_exact, &modsym)?)?;
    modsym.add_function(wrap_pyfunction!(level_data, &modsym)?)?;
    modsym.add_function(wrap_pyfunction!(commute, &modsym)?)?;
    modsym.add_function(wrap_pyfunction!(estimate, &modsym)?)?;
    m.add_submodule(&modsym)
}
