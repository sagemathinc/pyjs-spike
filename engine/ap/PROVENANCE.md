# Provenance of sagebrush-ap

`sagebrush-ap` computes a_p of elliptic curves over Q following the
genus-1 strategy of smalljac, by Andrew V. Sutherland (with Kiran
Kedlaya; see "Computing L-series of hyperelliptic curves", ANTS 2008).
The Rust code was written from scratch: no smalljac source was
translated or copied. smalljac 4.1.3 was used only as an oracle, for
exact agreement tests and benchmarks (see `tests/smalljac.rs` and
`results/ap.md`).

On 2026-10-02 Andrew Sutherland gave explicit permission, conveyed to
and recorded by William Stein, for this port to be licensed however the
Sagebrush project chooses (for example, under the MIT license).
