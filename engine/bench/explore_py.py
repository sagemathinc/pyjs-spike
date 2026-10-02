"""An agent-style exploration: T_q charpolys for many levels at once.

Python threads call modsym_engine concurrently (it releases the GIL);
compare one thread versus a thread pool.  explore_py.py WORKERS LEVELS...
"""

import sys
import time
from concurrent.futures import ThreadPoolExecutor
import modsym_engine

workers = int(sys.argv[1])
levels = [int(x) for x in sys.argv[2:]]
t = time.perf_counter()
with ThreadPoolExecutor(max_workers=workers) as ex:
    results = list(ex.map(lambda n: modsym_engine.hecke_charpoly(n, 2, threads=1), levels))
dt = time.perf_counter() - t
print(f"{len(levels)} levels, {workers} Python threads: {dt:.2f} s; dims {[r['dim'] for r in results]}")
