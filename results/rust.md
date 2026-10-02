# Modular symbols: Rust vs PyPy vs pyjs vs CPython

`bench/modsym/rust` is a line-by-line port of `bench/modsym/modsym.py`:
same algorithms (P^1 table, signed union-find, dense echelon mod p,
Heilbronn matrices, Hessenberg charpoly) and data layout (Vec for lists,
HashMap where Python uses a dict), i64 arithmetic with rem_euclid for
Python's `%`.  Release build with LTO.  All four print the same
characteristic-polynomial hash.

Measured on `bench-1` (8x AMD EPYC 7B13, dedicated): Rust 1.x stable,
PyPy 7.3 (3.11.16), Node 26.5 (pyjs at commit ebbfd99), CPython
3.14.8.  Compute time only (process startup excluded).

| N | q | Rust | PyPy | pyjs | CPython |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 389 | 2 | 3 ms | 65 ms | 92 ms | 42 ms |
| 2003 | 2 | 124 ms | 483 ms | 1.72 s | 2.33 s |
| 2003 | 97 | 145 ms | 546 ms | 2.04 s | 2.59 s |
| 5077 | 2 | 1.04 s | 2.49 s | 18.7 s | 32.3 s |
| 5077 | 97 | 1.19 s | 2.99 s | 21.0 s | 32.5 s |
| 10007 | 2 | 9.4 s | 20.6 s | 191 s | 232 s |
| 10007 | 97 | 6.5 s | 19.4 s | (not run) | (not run) |

Ratios to Rust at N = 5077, q = 2: PyPy 2.4x, pyjs 18x, CPython 31x.

The same Rust port is still dense-linear-algebra-in-loops; Sage's native
implementation (48 s at N = 10007 on the development host, which is about
2x faster than bench-1 for these programs) and FLINT's nmod_mat would be
the reference for kernel speed.
