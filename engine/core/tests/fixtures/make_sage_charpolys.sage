# Regenerates sage_charpolys.txt:  sage make_sage_charpolys.sage
# One line per case: "N q c_0 c_1 ... c_d" (charpoly of T_q on weight-2
# sign +1 modular symbols for Gamma0(N), over QQ, constant term first).
cases = [(1, 2), (2, 3), (11, 2), (11, 3), (25, 2), (27, 2), (37, 2), (37, 11),
         (49, 3), (64, 3), (100, 3), (105, 2), (121, 2), (125, 3), (243, 2),
         (360, 7), (389, 2), (389, 97), (512, 3), (997, 2), (1001, 2),
         (1155, 2), (1728, 5), (2003, 2)]
with open("sage_charpolys.txt", "w") as f:
    for N, q in cases:
        M = ModularSymbols(N, 2, sign=1)
        cp = M.hecke_matrix(q).charpoly()
        assert all(c.denominator() == 1 for c in cp.list())
        f.write(" ".join(map(str, [N, q] + cp.list())) + "\n")
        print(N, q, M.dimension(), flush=True)
