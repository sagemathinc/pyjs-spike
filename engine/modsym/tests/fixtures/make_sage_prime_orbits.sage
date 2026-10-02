# For prime N in [a, b]: the Galois orbits of newforms of weight 2 on Gamma0(N)
# (factors of the new cuspidal sign +1 modular symbols), with their dimension
# and tr(T_p | A) for primes p <= 200, p != N.  One JSON object per line.
import json, sys
a, b = int(sys.argv[1]), int(sys.argv[2])
out = open(sys.argv[3], "w")
for N in prime_range(a, b + 1):
    S = ModularSymbols(N, 2, sign=1).cuspidal_subspace().new_subspace()
    for A in S.decomposition():
        tr = {int(p): int(A.hecke_matrix(p).trace()) for p in prime_range(201) if p != N}
        out.write(json.dumps({"level": int(N), "dim": int(A.dimension()), "traces": tr}) + "\n")
    out.flush()
