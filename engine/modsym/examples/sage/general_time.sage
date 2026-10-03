import json, time
cases = []
def chi_of(N, order, k):
    G = DirichletGroup(N)
    for c in G.galois_orbits():
        if c[0].order() == order and c[0](-1) == (-1)**k:
            return G, c[0]
specs = [(1009, 2, 1, 1), (50, 12, 1, 1), (91, 2, 0, 6), (200, 4, 1, 2), (131, 3, 1, 2), (61, 6, 0, 10), (400, 4, 1, 4), (151, 2, 1, 15)]
for N, k, sign, order in specs:
    G, eps = chi_of(N, order, k)
    e = G.zeta_order(); z = G.zeta(); logs = {z**i: i for i in range(e)}
    exps = [int(logs[eps(m)]) if gcd(m, N) == 1 else None for m in range(N)]
    t = time.time(); M = ModularSymbols(eps, k, sign); d = M.dimension(); t1 = time.time() - t
    t = time.time(); T = M.hecke_matrix(2); t2 = time.time() - t
    t = time.time(); f = T.charpoly(); t3 = time.time() - t
    cases.append(dict(N=int(N), k=int(k), sign=int(sign), e=int(e), exps=exps, dim=int(d), order=int(order), space=t1, t2=t2, charpoly=t3))
    print(N, k, sign, order, d, t1, t2, t3, flush=True)
json.dump(cases, open('/home/user/data/sage/general_time.json', 'w'))
