# Reference data for engine/modsym/src/general.rs: dimensions and T_q
# charpolys of ModularSymbols(eps, k, sign) for small N, k, Galois-orbit
# representatives of characters.  One JSON object per line.
import json, sys
out = open('/home/user/data/sage/general_ref.jsonl', 'w')
for N in range(1, 41):
    G = DirichletGroup(N)
    e = G.zeta_order()
    z = G.zeta()
    # discrete log table relative to z
    logs = {z**i: i for i in range(e)}
    for chi in G.galois_orbits():
        eps = chi[0]
        exps = [int(logs[eps(m)]) if gcd(m, N) == 1 else None for m in range(N)]
        for k in range(2, 7):
            if (eps(-1) == 1) != (k % 2 == 0):
                continue
            if N * k > 160:
                continue
            for sign in [1, -1, 0]:
                M = ModularSymbols(eps, k, sign)
                K = M.base_ring()
                m = K._n() if K != QQ else 2
                rec = dict(N=int(N), k=int(k), sign=int(sign), e=int(e), exps=exps, dim=int(M.dimension()), m=int(m), polys={})
                for q in primes(2, 6):
                    f = M.hecke_polynomial(q)
                    coeffs = []
                    for c in f.list():
                        c = K(c)
                        if K == QQ:
                            v = [str(c)]
                        else:
                            v = [str(x) for x in c.list()]
                        coeffs.append(v)
                    rec['polys'][str(q)] = coeffs
                out.write(json.dumps(rec) + '\n')
                out.flush()
