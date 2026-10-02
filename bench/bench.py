"""Microbenchmarks plus pyperformance's nbody, timed in-process after one warmup."""

import time


def timeit(name, f):
    """Print the first (cold) run and the median of five further runs, in ms."""
    t = time.perf_counter()
    r = f()
    cold = time.perf_counter() - t
    warm = []
    for _ in range(5):
        t = time.perf_counter()
        f()
        warm.append(time.perf_counter() - t)
    warm.sort()
    print(f"{name:12s} {cold * 1000:9.1f} {warm[2] * 1000:9.1f}  {r!r}")


def fib(n):
    return n if n < 2 else fib(n - 1) + fib(n - 2)


def b_fib():
    return fib(25)


def b_loop():
    s = 0
    for i in range(2000000):
        s += i * i % 7
    return s


def b_float():
    x = 0.0
    for i in range(1000000):
        x = x * 0.999 + 1.5
    return round(x, 3)


class P:
    def __init__(self, x):
        self.x = x

    def inc(self, d):
        self.x += d
        return self


def b_attr():
    p = P(0)
    for i in range(1000000):
        p.inc(1)
    return p.x


def b_list():
    return sum([i * 2 for i in range(1000000) if i % 3])


def b_dict():
    d = {}
    for i in range(200000):
        d[str(i)] = i
    return sum(d[k] for k in d)


def b_str():
    return len(",".join(str(i) for i in range(200000)).split(","))


def b_sieve():
    n = 2000000
    s = bytearray([1]) * (n + 1)
    s[0] = s[1] = 0
    i = 2
    while i * i <= n:
        if s[i]:
            s[i * i :: i] = bytearray(len(range(i * i, n + 1, i)))
        i += 1
    return sum(s)


def b_bigint():
    a, b = 0, 1
    for i in range(20000):
        a, b = b, a + b
    return a % 1000000007


class Sq:
    def __init__(self, s):
        self.s = s

    def area(self):
        return self.s * self.s


class Rect:
    def __init__(self, w, h):
        self.w = w
        self.h = h

    def area(self):
        return self.w * self.h


class Circ:
    def __init__(self, r):
        self.r = r

    def area(self):
        return 3 * self.r * self.r


class Tri:
    def __init__(self, b, h):
        self.b = b
        self.h = h

    def area(self):
        return self.b * self.h // 2


def b_poly():
    shapes = [Sq(3), Rect(2, 5), Circ(4), Tri(6, 7)] * 1000
    t = 0
    for i in range(250):
        for s in shapes:
            t += s.area()
    return t


# --- nbody, from pyperformance (bm_nbody) ---
PI = 3.14159265358979323
SOLAR_MASS = 4 * PI * PI
DAYS_PER_YEAR = 365.24

BODIES = {
    "sun": ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], SOLAR_MASS),
    "jupiter": (
        [4.84143144246472090e00, -1.16032004402742839e00, -1.03622044471123109e-01],
        [
            1.66007664274403694e-03 * DAYS_PER_YEAR,
            7.69901118419740425e-03 * DAYS_PER_YEAR,
            -6.90460016972063023e-05 * DAYS_PER_YEAR,
        ],
        9.54791938424326609e-04 * SOLAR_MASS,
    ),
    "saturn": (
        [8.34336671824457987e00, 4.12479856412430479e00, -4.03523417114321381e-01],
        [
            -2.76742510726862411e-03 * DAYS_PER_YEAR,
            4.99852801234917238e-03 * DAYS_PER_YEAR,
            2.30417297573763929e-05 * DAYS_PER_YEAR,
        ],
        2.85885980666130812e-04 * SOLAR_MASS,
    ),
    "uranus": (
        [1.28943695621391310e01, -1.51111514016986312e01, -2.23307578892655734e-01],
        [
            2.96460137564761618e-03 * DAYS_PER_YEAR,
            2.37847173959480950e-03 * DAYS_PER_YEAR,
            -2.96589568540237556e-05 * DAYS_PER_YEAR,
        ],
        4.36624404335156298e-05 * SOLAR_MASS,
    ),
    "neptune": (
        [1.53796971148509165e01, -2.59193146099879641e01, 1.79258772950371181e-01],
        [
            2.68067772490389322e-03 * DAYS_PER_YEAR,
            1.62824170038242295e-03 * DAYS_PER_YEAR,
            -9.51592254519715870e-05 * DAYS_PER_YEAR,
        ],
        5.15138902046611451e-05 * SOLAR_MASS,
    ),
}


def combinations(l):
    result = []
    for x in range(len(l) - 1):
        ls = l[x + 1 :]
        for y in ls:
            result.append((l[x], y))
    return result


SYSTEM = list(BODIES.values())
PAIRS = combinations(SYSTEM)


def advance(dt, n, bodies=SYSTEM, pairs=PAIRS):
    for i in range(n):
        for ([x1, y1, z1], v1, m1), ([x2, y2, z2], v2, m2) in pairs:
            dx = x1 - x2
            dy = y1 - y2
            dz = z1 - z2
            mag = dt * ((dx * dx + dy * dy + dz * dz) ** (-1.5))
            b1m = m1 * mag
            b2m = m2 * mag
            v1[0] -= dx * b2m
            v1[1] -= dy * b2m
            v1[2] -= dz * b2m
            v2[0] += dx * b1m
            v2[1] += dy * b1m
            v2[2] += dz * b1m
        for r, [vx, vy, vz], m in bodies:
            r[0] += dt * vx
            r[1] += dt * vy
            r[2] += dt * vz


def report_energy(bodies=SYSTEM, pairs=PAIRS, e=0.0):
    for ((x1, y1, z1), v1, m1), ((x2, y2, z2), v2, m2) in pairs:
        dx = x1 - x2
        dy = y1 - y2
        dz = z1 - z2
        e -= (m1 * m2) / ((dx * dx + dy * dy + dz * dz) ** 0.5)
    for r, [vx, vy, vz], m in bodies:
        e += m * (vx * vx + vy * vy + vz * vz) / 2.0
    return e


def offset_momentum(ref, bodies=SYSTEM, px=0.0, py=0.0, pz=0.0):
    for r, [vx, vy, vz], m in bodies:
        px -= vx * m
        py -= vy * m
        pz -= vz * m
    (r, v, m) = ref
    v[0] = px / m
    v[1] = py / m
    v[2] = pz / m


def b_nbody():
    offset_momentum(BODIES["sun"])
    advance(0.01, 20000)
    return report_energy()


print(f"{'':12s} {'cold ms':>9s} {'warm ms':>9s}  result")
for name, f in [
    ("fib25", b_fib),
    ("int loop", b_loop),
    ("float loop", b_float),
    ("method call", b_attr),
    ("poly method", b_poly),
    ("listcomp", b_list),
    ("dict str", b_dict),
    ("str join", b_str),
    ("sieve", b_sieve),
    ("bigint fib", b_bigint),
    ("nbody", b_nbody),
]:
    timeit(name, f)
