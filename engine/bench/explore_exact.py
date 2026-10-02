"""Exploration demo: proven characteristic polynomials of T_q for many
levels in one parallel call, factored with python-flint, saved as Parquet.

    python explore_exact.py [max_level] [q] [out.parquet]

Prints the engine's own estimate first, then the measured wall time and peak
memory, then a small summary of the factorization data.
"""
import resource
import sys
import time
from collections import Counter

import flint
import pyarrow as pa
import pyarrow.parquet as pq

import modsym_engine as m

top = int(sys.argv[1]) if len(sys.argv) > 1 else 3000
q = int(sys.argv[2]) if len(sys.argv) > 2 else 2
out = sys.argv[3] if len(sys.argv) > 3 else f"charpolys_T{q}_N{top}.parquet"
levels = [n for n in range(11, top + 1) if n % q]

est = [m.estimate(n, q) for n in levels]
print(f"{len(levels)} levels, estimated {sum(e['seconds_exact'] for e in est):.0f} s on one thread, "
      f"largest dim {max(e['dim'] for e in est)}")

t = time.time()
rows = m.batch_exact(levels, q)
wall = time.time() - t
rss = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024
bad = [r for r in rows if r.get("status") != "proven"]
print(f"computed in {wall:.1f} s wall, peak RSS {rss:.0f} MB; not proven: {len(bad)}")

t = time.time()
x = flint.fmpz_poly([-(q + 1), 1])
def squarefree(n):
    return all(n % (d * d) for d in range(2, int(n ** 0.5) + 1))

# For squarefree N every Eisenstein eigenvalue of T_q is q + 1, so dividing
# by (x - q - 1)^e leaves the cusp part.  Otherwise (e.g. N = 25) Eisenstein
# series with characters have eigenvalues chi(q) + psi(q) q, and the factor
# data below is for the whole space.
table = {k: [] for k in ["n", "q", "dim", "genus", "eisenstein", "primes", "charpoly", "cusp_part", "factor_degrees", "rational_roots"]}
for r in rows:
    f = flint.fmpz_poly(r["charpoly"])
    cusp = squarefree(r["n"])
    if cusp:
        for _ in range(r["eisenstein"]):
            f, rem = divmod(f, x)
            assert rem == 0, r["n"]
    _, facs = f.factor()
    degs = sorted(d for g, e in facs for d in [g.degree()] * e)
    roots = sorted(-int(g[0]) for g, e in facs if g.degree() == 1 for _ in range(e))
    for k, v in [("n", r["n"]), ("q", q), ("dim", r["dim"]), ("genus", r["genus"]), ("eisenstein", r["eisenstein"]),
                 ("primes", r["primes_used"]), ("charpoly", [str(c) for c in r["charpoly"]]),
                 ("cusp_part", cusp), ("factor_degrees", degs), ("rational_roots", roots)]:
        table[k].append(v)
pq.write_table(pa.table(table), out)
print(f"factored and wrote {out} in {time.time() - t:.1f} s")

def is_prime(n):
    return n > 1 and all(n % d for d in range(2, int(n ** 0.5) + 1))

# Prime levels: how much of the cusp space is one Galois orbit?
share = []
for n, g, degs in zip(table["n"], table["genus"], table["factor_degrees"]):
    if is_prime(n) and g >= 10:
        share.append(max(degs) / g)
print(f"prime levels with genus >= 10: {len(share)}; largest irreducible factor is on average "
      f"{100 * sum(share) / len(share):.0f}% of the cusp space (min {100 * min(share):.0f}%)")
hist = Counter(len(r) for n, r in zip(table["n"], table["rational_roots"]) if is_prime(n))
print("prime levels by number of rational T_%d eigenvalues on the cusp space:" % q, dict(sorted(hist.items())))
