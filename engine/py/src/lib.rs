//! CPython bindings.  Computations release the GIL, so Python threads can
//! run several in parallel; each call also uses `threads` worker threads
//! (0 = all cores).

use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyfunction]
#[pyo3(signature = (n, q, p=67108859, threads=0))]
fn hecke_charpoly<'py>(py: Python<'py>, n: u64, q: u64, p: u64, threads: usize) -> PyResult<Bound<'py, PyDict>> {
    let r = py.detach(|| {
        let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build().unwrap();
        pool.install(|| modsym_core::hecke_charpoly(n, q, p))
    });
    let d = PyDict::new(py);
    d.set_item("symbols", r.symbols)?;
    d.set_item("dim", r.dim)?;
    d.set_item("charpoly", r.charpoly.clone())?;
    d.set_item("hash", r.hash())?;
    d.set_item("eisenstein_root", r.eisenstein_root())?;
    d.set_item("ms", r.ms.to_vec())?;
    Ok(d)
}

#[pymodule]
fn modsym_engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hecke_charpoly, m)?)
}
