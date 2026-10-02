# Regenerates sage_orbits_le200.jsonl:  sage make_sage_orbits.sage 1 200 sage_orbits_le200.jsonl
# For every level N in [a, b]: the Galois orbits of weight-2 newforms on
# Gamma0(N) (factors of the new cuspidal sign +1 modular symbols), with
# their dimension and tr(T_p | A) for primes p <= 100 not dividing N.
import json, sys
a, b = int(sys.argv[1]), int(sys.argv[2])
out = open(sys.argv[3], "w")
for N in range(a, b + 1):
    S = ModularSymbols(N, 2, sign=1).cuspidal_subspace().new_subspace()
    for A in S.decomposition():
        tr = {int(p): int(A.hecke_matrix(p).trace()) for p in prime_range(101) if N % p != 0}
        out.write(json.dumps({"level": int(N), "dim": int(A.dimension()), "traces": tr}) + "\n")
