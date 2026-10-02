"""compare.py OURS.jsonl LMFDB_NEWFORMS.jsonl LMFDB_HECKE_NF.jsonl

Bits per a_p (p < 1000, p not dividing N) for newform orbits stored in
LMFDB (mf_hecke_nf, dimension 2..20), in three representations:
  ours   -- the integer vector c_p of Stein's representation a_p = sum beta_r c_{p,r}
  hecke  -- LMFDB's integer coordinates in its LLL-reduced Hecke-ring basis
  power  -- the power basis of Q(alpha): integer numerators over a common denominator
An integer x costs bitlen(|x|) + 1 (sign).  One-time costs (our beta_r,
LMFDB's basis matrix) are reported separately, not per a_p.
"""
import json, sys
try:
    import flint  # python-flint, for the LLL-reduced variant of our basis
except ImportError:
    flint = None
from collections import defaultdict
from fractions import Fraction
from math import lcm

def bits(x):
    return abs(int(x)).bit_length() + 1

primes = [p for p in range(2, 1000) if all(p % d for d in range(2, int(p**0.5) + 1))]
ours = defaultdict(list)
for line in open(sys.argv[1]):
    r = json.loads(line)
    ours[(r["level"], r["dim"], tuple(r["traces"]))].append(r)
label_key = {}
for line in open(sys.argv[2]):
    r = json.loads(line)
    n = r["level"]
    key = (n, r["dim"], tuple(r["traces"][p - 1] for p in primes if n % p))
    label_key[r["label"]] = key

rows = []
for line in open(sys.argv[3]):
    h = json.loads(line)
    n, k = h["level"], h["dim"]
    match = ours.get(label_key[h["label"]])
    if not match:
        print("unmatched", h["label"]); continue
    o = match[0]
    good = [i for i, p in enumerate(primes) if n % p]
    c = {p: v for p, v in o["c"]}
    ours_bits = sum(bits(x) for p in primes if n % p for x in c[p])
    # LLL on the lattice spanned by the k coordinate sequences (c_{p,r})_p:
    # a unimodular change c'_p = V c_p (beta' = beta V^-1) with small entries.
    rowsk = [[c[p][r] for p in primes if n % p] for r in range(k)]
    lll_bits = sum(bits(int(x)) for row in flint.fmpz_mat(rowsk).lll().tolist() for x in row) if flint else 0
    # The c_p lie in a sublattice L of Z^k (the image of the order the a_p
    # generate).  Coordinates in a basis B of L (from the HNF of the columns),
    # y_p = B^-1 c_p, then LLL on the coordinate sequences.
    C = flint.fmpz_mat(rowsk)
    B = C.transpose().hnf()
    B = flint.fmpz_mat([B.tolist()[i] for i in range(k)]).transpose()
    Y = B.solve(C)
    Y = flint.fmpz_mat([[int(x) for x in row] for row in Y.tolist()])
    hnf_bits = sum(bits(int(x)) for row in Y.lll().tolist() for x in row)
    hecke_bits = sum(bits(x) for i in good for x in h["ap"][i])
    if h["num"] is None:  # hecke_ring_power_basis: the Hecke-ring basis is the power basis
        h["num"], h["den"] = [[int(i == j) for j in range(k)] for i in range(k)], [1] * k
    basis = [[Fraction(a, d) for a in num] for num, d in zip(h["num"], h["den"])]
    power_bits = 0
    for i in good:
        v = [sum(Fraction(x) * b[j] for x, b in zip(h["ap"][i], basis)) for j in range(k)]
        dd = lcm(*[q.denominator for q in v])
        power_bits += sum(bits(q * dd) for q in v) + bits(dd)
    basis_bits = sum(bits(x) for num in h["num"] for x in num) + sum(bits(d) for d in h["den"])
    rows.append((h["label"], k, len(good), ours_bits, hecke_bits, power_bits, basis_bits, lll_bits, hnf_bits))

by_dim = defaultdict(lambda: [0, 0, 0, 0, 0, 0, 0])
for label, k, np_, o, hk, pw, bb, ll, hn in rows:
    t = by_dim[k]
    t[0] += 1; t[1] += o / np_; t[2] += hk / np_; t[3] += pw / np_; t[4] += bb; t[5] += ll / np_; t[6] += hn / np_
print(f"{len(rows)} newform orbits compared (bits per a_p, averaged over orbits)")
print(f"{'dim':>4} {'forms':>6} {'ours':>7} {'+LLL':>7} {'+HNF+LLL':>9} {'LMFDB':>7} {'power':>7} {'best/LMFDB':>11} {'best/power':>11}")
for k in sorted(by_dim):
    n_, o, hk, pw, bb, ll, hn = by_dim[k]
    print(f"{k:>4} {n_:>6} {o/n_:>7.1f} {ll/n_:>7.1f} {hn/n_:>9.1f} {hk/n_:>7.1f} {pw/n_:>7.1f} {hn/hk:>11.2f} {hn/pw:>11.3f}")
tot = [sum(r[i] for r in rows) for i in (3, 7, 8, 4, 5, 6)]
print(f"total bits for all a_p, p < 1000: ours {tot[0]:,}; +LLL {tot[1]:,}; +HNF+LLL {tot[2]:,}; LMFDB Hecke {tot[3]:,} (+ {tot[5]:,} for its basis matrices); power basis {tot[4]:,}")
