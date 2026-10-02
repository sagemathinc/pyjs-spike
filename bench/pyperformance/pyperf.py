"""A minimal stand-in for pyperf, shared by CPython and pyjs runs.

bench_time_func(name, func, *args) times func(loops, *args): one cold call
with loops=1, about one second of warmup, then five samples with loops
calibrated to ~50 ms each.  It
prints `name cold_ms warm_ms_per_loop`.
"""

import time

perf_counter = time.perf_counter


class _Namespace:
    pass


class _ArgParser:
    def __init__(self):
        self.defaults = {}

    def add_argument(self, *names, **kw):
        dest = kw.get("dest")
        if dest is None:
            dest = [n for n in names if n.startswith("--")][0][2:].replace("-", "_") if any(n.startswith("--") for n in names) else names[0]
        default = kw.get("default")
        if kw.get("action") == "store_true":
            default = False
        self.defaults[dest] = default


class Runner:
    def __init__(self, *args, **kw):
        self.metadata = {}
        self.argparser = _ArgParser()
        self.args = None

    def parse_args(self, args=None):
        ns = _Namespace()
        for k, v in self.argparser.defaults.items():
            setattr(ns, k, v)
        self.args = ns
        return ns

    def bench_func(self, name, func, *args, inner_loops=None):
        def timed(loops):
            t0 = perf_counter()
            for _ in range(loops):
                func(*args)
            return perf_counter() - t0

        return self.bench_time_func(name, timed, inner_loops=inner_loops)

    def bench_time_func(self, name, func, *args, inner_loops=None):
        cold = func(1, *args)
        loops = 1
        if cold > 0:
            loops = max(1, min(10000, int(0.05 / cold)))
        # Warm up for about a second (JIT compilers need it; CPython does not)
        # before taking five samples.
        t_end = perf_counter() + 1.0
        while perf_counter() < t_end:
            func(loops, *args)
        samples = sorted(func(loops, *args) / loops for _ in range(5))
        print(f"{name} {cold * 1000:.3f} {samples[2] * 1000:.3f}")
