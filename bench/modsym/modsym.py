"""Weight-2 modular symbols for Gamma0(N), sign +1, over GF(p).

Plain Python (ints, lists, dicts) so the same source runs under CPython,
pyjs and Sage's Python.  Algorithms as in Stein, "Modular Forms: A
Computational Approach": Manin symbols indexed by P^1(Z/NZ), the 2- and
3-term relations plus the star involution, and Hecke operators T_q via
Cremona's Heilbronn matrices.  Usage: modsym.py N q [p]
"""

import sys
import time
from math import gcd


def p1_list(N):
    """Representatives of P^1(Z/NZ) and a table (c*N + d) -> index."""
    units = [u for u in range(1, N) if gcd(u, N) == 1] if N > 1 else [0]
    index = [-1] * (N * N)
    reps = []
    for c in range(N):
        for d in range(N):
            if index[c * N + d] != -1 or gcd(gcd(c, d), N) != 1:
                continue
            i = len(reps)
            reps.append((c, d))
            for u in units:
                index[(u * c % N) * N + (u * d % N)] = i
    return reps, index


class ManinSymbols:
    def __init__(self, N, p):
        self.N, self.p = N, p
        self.reps, self.index = p1_list(N)
        self._quotient()

    def idx(self, c, d):
        N = self.N
        return self.index[(c % N) * N + (d % N)]

    def _quotient(self):
        """Reduce modulo 2-term relations (x + xS = 0, x = x*eta for sign
        +1) with a union-find carrying signs, then the 3-term relations by
        Gaussian elimination mod p."""
        n = len(self.reps)
        p = self.p
        parent = list(range(n))
        sign = [1] * n  # x_i = sign[i] * x_parent[i]
        zero = [False] * n

        def find(i):
            s = 1
            while parent[i] != i:
                s *= sign[i]
                i = parent[i]
            return i, s

        def union(i, j, s):
            # impose x_i = s * x_j
            ri, si = find(i)
            rj, sj = find(j)
            if ri == rj:
                if si != s * sj:
                    zero[ri] = True  # x = -x forces x = 0 (p odd)
                return
            parent[ri] = rj
            sign[ri] = s * sj * si
            if zero[ri]:
                zero[rj] = True

        for i, (c, d) in enumerate(self.reps):
            union(i, self.idx(d, -c), -1)  # x + xS = 0
            union(i, self.idx(-c, d), 1)  # sign +1: x = x*eta
        free = {}
        rep_of = [None] * n  # (free index or -1 for zero, sign)
        for i in range(n):
            r, s = find(i)
            if zero[r]:
                rep_of[i] = (-1, 0)
                continue
            if r not in free:
                free[r] = len(free)
            rep_of[i] = (free[r], s)
        m = len(free)
        # 3-term relations x + xT + xT^2 = 0 as dense rows over the free gens.
        rows = []
        for (c, d) in self.reps:
            row = [0] * m
            for (a, b) in ((c, d), (d, -c - d), (-c - d, c)):
                k, s = rep_of[self.idx(a, b)]
                if k >= 0:
                    row[k] = (row[k] + s) % p
            if any(row):
                rows.append(row)
        pivots = echelon(rows, m, p)
        nonpivot = [j for j in range(m) if j not in pivots]
        self.dim = len(nonpivot)
        col = {j: t for t, j in enumerate(nonpivot)}
        # Each free generator as a vector in the quotient basis.
        coords = []
        for j in range(m):
            v = [0] * self.dim
            if j in col:
                v[col[j]] = 1
            else:
                row = rows[pivots[j]]
                for k in nonpivot:
                    if row[k]:
                        v[col[k]] = (-row[k]) % p
            coords.append(v)
        # Each Manin symbol as a sparse vector in the quotient basis.
        self.vec = []
        for i in range(n):
            k, s = rep_of[i]
            if k < 0:
                self.vec.append(())
            else:
                self.vec.append(tuple((t, (s * x) % p) for t, x in enumerate(coords[k]) if x))
        # A Manin symbol representing each basis element.
        self.basis_symbol = []
        for t, j in enumerate(nonpivot):
            for i in range(n):
                k, s = rep_of[i]
                if k == j:
                    self.basis_symbol.append((i, s))
                    break

    def hecke(self, q):
        """Matrix of T_q (q prime, q not dividing N) on the quotient."""
        H = heilbronn(q)
        p, N = self.p, self.N
        M = []
        for (i, s) in self.basis_symbol:
            c, d = self.reps[i]
            v = [0] * self.dim
            for (a, b, cc, dd) in H:
                for (t, x) in self.vec[self.idx(c * a + d * cc, c * b + d * dd)]:
                    v[t] = (v[t] + s * x) % p
            M.append(v)
        return M


def echelon(rows, ncols, p):
    """Reduced row echelon form mod p, in place.  Returns {col: row}."""
    pivots = {}
    r = 0
    nrows = len(rows)
    for c in range(ncols):
        piv = -1
        for i in range(r, nrows):
            if rows[i][c]:
                piv = i
                break
        if piv < 0:
            continue
        rows[r], rows[piv] = rows[piv], rows[r]
        row = rows[r]
        inv = pow(row[c], p - 2, p)
        for k in range(c, ncols):
            if row[k]:
                row[k] = row[k] * inv % p
        for i in range(nrows):
            if i != r:
                other = rows[i]
                f = other[c]
                if f:
                    for k in range(c, ncols):
                        if row[k]:
                            other[k] = (other[k] - f * row[k]) % p
        pivots[c] = r
        r += 1
        if r == nrows:
            break
    del rows[r:]
    return pivots


def heilbronn(q):
    """Cremona's Heilbronn matrices of determinant q, as (a, b, c, d)."""
    if q == 2:
        return [(1, 0, 0, 2), (2, 0, 0, 1), (2, 1, 0, 1), (1, 0, 1, 2)]
    out = [(1, 0, 0, q)]
    for r in range(-(q // 2), q // 2 + 1):
        x1, x2, y1, y2, a, b = q, -r, 0, 1, -q, r
        out.append((x1, x2, y1, y2))
        while b:
            # round half away from zero
            qq = (abs(a) * 2 + abs(b)) // (2 * abs(b))
            if (a < 0) != (b < 0):
                qq = -qq
            a, b = -b, a - b * qq
            x1, x2 = x2, qq * x2 - x1
            y1, y2 = y2, qq * y2 - y1
            out.append((x1, x2, y1, y2))
    return out


def charpoly(M, p):
    """Characteristic polynomial mod p via the Hessenberg form."""
    n = len(M)
    H = [row[:] for row in M]
    for m in range(1, n - 1):
        i = m
        while i < n and H[i][m - 1] == 0:
            i += 1
        if i == n:
            continue
        if i != m:
            H[i], H[m] = H[m], H[i]
            for r in range(n):
                H[r][i], H[r][m] = H[r][m], H[r][i]
        inv = pow(H[m][m - 1], p - 2, p)
        for i in range(m + 1, n):
            u = H[i][m - 1] * inv % p
            if u:
                for k in range(n):
                    H[i][k] = (H[i][k] - u * H[m][k]) % p
                for r in range(n):
                    H[r][m] = (H[r][m] + u * H[r][i]) % p
    # Recurrence for the characteristic polynomials of leading submatrices.
    polys = [[1]]
    for m in range(1, n + 1):
        prev = polys[m - 1]
        cur = [0] + prev[:]  # x * prev
        h = H[m - 1][m - 1]
        for k in range(len(prev)):
            cur[k] = (cur[k] - h * prev[k]) % p
        t = 1
        for i in range(1, m):
            t = t * H[m - i][m - i - 1] % p
            coef = t * H[m - i - 1][m - 1] % p
            sub = polys[m - i - 1]
            for k in range(len(sub)):
                cur[k] = (cur[k] - coef * sub[k]) % p
        polys.append(cur)
    return polys[n]


def evalpoly(f, x, p):
    r = 0
    for c in reversed(f):
        r = (r * x + c) % p
    return r


def main():
    N = int(sys.argv[1]) if len(sys.argv) > 1 else 389
    q = int(sys.argv[2]) if len(sys.argv) > 2 else 2
    p = int(sys.argv[3]) if len(sys.argv) > 3 else 67108859
    t0 = time.perf_counter()
    M = ManinSymbols(N, p)
    t1 = time.perf_counter()
    T = M.hecke(q)
    t2 = time.perf_counter()
    f = charpoly(T, p)
    t3 = time.perf_counter()
    h = 0
    for c in f:
        h = (h * 1000003 + c) % 2305843009213693951
    print(f"N={N} q={q} p={p} symbols={len(M.reps)} dim={M.dim} eisenstein_root={evalpoly(f, q + 1, p) == 0} charpoly_hash={h}")
    print(f"times: symbols {1000 * (t1 - t0):.0f} ms, T_{q} {1000 * (t2 - t1):.0f} ms, charpoly {1000 * (t3 - t2):.0f} ms, total {1000 * (t3 - t0):.0f} ms")


main()
