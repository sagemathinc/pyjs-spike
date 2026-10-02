# modsym-engine: Rust core, CPython and WASM bindings, threads, memory

`engine/` is a Rust workspace: `core` (the mathematics), `cli`, `py` (PyO3,
releases the GIL) and `wasm` (wasm-bindgen).  Same mathematics as
`bench/modsym/modsym.py`, engineered rather than ported:

* P^1(Z/NZ) via Sage's p1_normalize with per-divisor tables: O(N d(N))
  memory instead of the reference's N^2 lookup table;
* the 3-term relations by sparse elimination over GF(p);
* T_q by counting Heilbronn images per generator, parallel over basis rows;
* the characteristic polynomial by a Hessenberg reduction whose row and
  column updates run in parallel (rayon).

Every run below printed the same characteristic-polynomial hash as the
plain-Python reference (and Sage, checked earlier).  `bench-1`: 8 x AMD
EPYC 7B13, 31 GB; wall time includes process startup; RSS is peak resident
memory from /usr/bin/time.  Raw output: `results/engine-raw.txt`.

## N = 10007, q = 97 (dim 835)

| system | wall | peak RSS |
| --- | ---: | ---: |
| CPython 3.14, plain Python | 240 s | 1013 MB |
| pyjs, plain Python | 164 s | 1558 MB |
| PyPy 3.11, plain Python | 16.5 s | 1060 MB |
| Rust line-by-line port | 4.9 s | 957 MB |
| engine, 1 thread | 2.9 s | 26 MB |
| engine, 8 threads | **0.53 s** | 26 MB |
| engine called from CPython (8 threads) | 0.55 s | 38 MB |
| engine as WASM in Node (1 thread) | 3.7 s | 77 MB |

## All sizes (wall s / peak RSS MB)

| system | N=2003 | N=5077 | N=10007 | N=20011 |
| --- | ---: | ---: | ---: | ---: |
| CPython 3.14 | 2.64 / 54 | 30.4 / 272 | 240 / 1013 | -- |
| pyjs | 2.12 / 145 | 19.5 / 387 | 164 / 1558 | -- |
| PyPy 3.11 | 0.54 / 100 | 2.63 / 318 | 16.5 / 1060 | -- |
| Rust port (dense) | 0.13 / 40 | 1.13 / 248 | 4.90 / 957 | -- |
| engine, 1 thread | 0.03 / 3 | 0.43 / 8 | 2.90 / 26 | 41-71 / 89 |
| engine, 8 threads | 0.02 / 3 | 0.14 / 8 | 0.53 / 26 | 7.3-9.7 / 90 |
| engine from CPython | 0.05 / 15 | 0.17 / 20 | 0.55 / 38 | 7.2 / 101 |
| engine WASM in Node | 0.11 / 58 | 0.95 / 61 | 3.72 / 77 | 114 / 149 |

N = 20011 (dim 1668) shows run-to-run variance on this VM: two 1-thread
runs took 41 s and 71 s.

## Threads

* Inside one computation (N = 20011, q = 97, one sweep): 1 thread 71.4 s,
  2: 37.3 s, 4: 17.6 s, 8: 9.7 s -- 7.3x on 8 cores.
* Across computations, from Python: T_2 charpolys for 16 prime levels near
  5000 take 5.5 s from one Python thread and 1.15 s from a pool of 8 Python
  threads (4.8x), because the engine releases the GIL.

## Reading

* The engine at N = 10007 is 450x faster than plain Python on CPython, 31x
  faster than PyPy and 9x faster than the line-by-line Rust port, while
  using 40x less memory.  Algorithms (sparse elimination, O(N) P^1 data)
  and threads account for most of it; the language for the rest.
* Calling it from CPython costs nothing measurable: the boundary is one
  call per computation.
* WebAssembly (single-threaded) is 1.3x slower than native single-threaded
  at N = 10007 and about 2-3x at N = 20011, with a 72 KB module; it would
  gain from WASM threads (SharedArrayBuffer + wasm-bindgen-rayon), not done
  here.
* Memory is now dominated by the dense coordinate vectors of free
  generators (O(m * dim)); storing them sparsely is the next step for
  larger N.
