//! WebAssembly bindings (single-threaded): returns a JSON string.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn hecke_charpoly(n: u32, q: u32, p: u32) -> String {
    let r = match modsym_core::hecke_charpoly(n as u64, q as u64, p as u64) {
        Ok(r) => r,
        Err(e) => return format!("{{\"error\":{:?}}}", e),
    };
    format!(
        "{{\"symbols\":{},\"dim\":{},\"hash\":\"{}\",\"eisenstein_root\":{},\"ms\":[{:.1},{:.1},{:.1}]}}",
        r.symbols, r.dim, r.hash(), r.eisenstein_root(), r.ms[0], r.ms[1], r.ms[2]
    )
}
