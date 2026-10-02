# Regenerates sage_aplists.txt:  sage make_sage_aplists.sage
# One line per curve: "name a1,a2,a3,a4,a6 bad_primes p:a_p ..." for good p < 20000,
# where bad primes are those dividing the discriminant of the given model.
curves = [
    ("11a1", [0, -1, 1, -10, -20]),
    ("37a1", [0, 0, 1, -1, 0]),          # rank 1
    ("389a1", [0, 1, 1, -2, 0]),         # rank 2
    ("5077a1", [0, 0, 1, -7, 6]),        # rank 3
    ("14a1", [1, 0, 1, 4, -6]),          # bad at 2 and 7, torsion Z/6
    ("15a1", [1, 1, 1, -10, -10]),       # torsion Z/4 x Z/2: full 2-torsion
    ("27a3", [0, 0, 1, 0, 0]),           # CM by Z[zeta_3], j = 0
    ("32a2", [0, 0, 0, -1, 0]),          # CM by Z[i], j = 1728, full 2-torsion
    ("36a1", [0, 0, 0, 0, 1]),           # CM, j = 0, torsion Z/6
    ("big", [1, -1, 1, -1394, 19431]),
    ("bigger", [0, 0, 0, -123456, 7891011]),
]
with open("sage_aplists.txt", "w") as f:
    for name, a in curves:
        E = EllipticCurve(a)
        D = E.discriminant()
        bad = [p for p in prime_range(20000) if D % p == 0]
        aps = [f"{p}:{E.ap(p)}" for p in prime_range(20000) if p not in bad]
        f.write(f"{name} {','.join(map(str, a))} {','.join(map(str, bad)) or '-'} {' '.join(aps)}\n")
        print(name, E.torsion_subgroup().invariants(), E.has_cm(), bad[:6], flush=True)
