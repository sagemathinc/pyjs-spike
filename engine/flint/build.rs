// Links FLINT 3 (with MPFR and GMP) statically from SAGEBRUSH_FLINT_PREFIX
// (a prefix containing lib/libflint.a, libmpfr.a, libgmp.a and libopenblas.a,
// since this FLINT build uses BLAS for dense matrix products).
fn main() {
    let prefix = std::env::var("SAGEBRUSH_FLINT_PREFIX").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{}/sagejs/packages/flint/.native/prefix", home)
    });
    println!("cargo:rerun-if-env-changed=SAGEBRUSH_FLINT_PREFIX");
    println!("cargo:rustc-link-search=native={}/lib", prefix);
    for lib in ["flint", "mpfr", "gmp", "openblas"] {
        println!("cargo:rustc-link-lib=static={}", lib);
    }
    println!("cargo:rustc-link-lib=dylib=m");
    println!("cargo:rustc-link-lib=dylib=pthread");
}
