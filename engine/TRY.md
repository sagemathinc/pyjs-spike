# Trying modsym-engine interactively

Two local, unpublished modules built from the same Rust core
(`engine/core`): a CPython extension (PyO3) and a Node.js native addon
(napi-rs).  Both are multithreaded (`threads=0` means all cores), and
exact coefficients are arbitrary-precision (Python `int`, JS `BigInt`).

## Start a REPL

    ~/pyjs-spike/engine/try-python     # IPython, module bound to m
    ~/pyjs-spike/engine/try-node       # Node REPL, module bound to m

Or import it yourself:

    ~/pyjs-spike/engine/.venv/bin/python -c "import modsym_engine as m; print(m.charpoly_exact(37, 2))"
    node -e 'const m = require(process.env.HOME + "/pyjs-spike/engine/node"); console.log(m.charpolyExact(37, 2))'

## Functions (Python name / JS name)

| Python | JS | returns |
|---|---|---|
| `charpoly_exact(N, q, threads=0)` | `charpolyExact(N, q, threads?)` | proven charpoly of T_q over Z (constant term first), status, checks |
| `batch_exact(levels, q, threads=0)` | `batchExact(levels, q, threads?)` | the above for many levels in parallel; bad levels get `error` |
| `hecke_charpoly(N, q, p=67108859, threads=0)` | `heckeCharpoly(N, q, p?, threads?)` | charpoly mod p, dim, timings |
| `level_data(N)` | `levelData(N)` | psi, genus, cusps, Eisenstein dim, dim |
| `estimate(N, q)` | `estimate(N, q)` | predicted dim, primes, bytes, one-thread seconds |
| `commute(N, q, r, p=..., threads=0)` | `commute(N, q, r, p?, threads?)` | T_q T_r == T_r T_q mod p |

Space: weight 2, Gamma0(N), sign +1 modular symbols.  q must be a prime
not dividing N (otherwise ValueError / a thrown Error).

## Things to try

    m.charpoly_exact(37, 2)["charpoly"]      # [0, -6, -1, 1] = x (x - 3)(x + 2)
    m.estimate(20011, 2)                     # what would a big one cost?
    r = m.batch_exact(range(11, 500, 2), 2)  # 245 levels, well under a second
    m.charpoly_exact(2310, 13)["dim"]        # 592, coefficients up to ~1000 bits

    m.charpolyExact(389, 2).charpoly         // BigInts
    m.batchExact([33, 35, 37], 3)            // 33 fails (3 | 33), others proven

The calls are synchronous: in Node a big one blocks the REPL until it is
done, and in Python the GIL is released (other threads keep running).

## Rebuilding after changing the Rust code

    cd ~/pyjs-spike/engine
    (cd py && ../.venv/bin/maturin develop --release)   # Python
    node/build.sh                                       # Node
