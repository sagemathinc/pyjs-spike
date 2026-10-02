"""modsym-engine from CPython: run_py.py N q [threads]"""

import sys
import modsym_engine

n, q = int(sys.argv[1]), int(sys.argv[2])
threads = int(sys.argv[3]) if len(sys.argv) > 3 else 0
r = modsym_engine.hecke_charpoly(n, q, threads=threads)
print(f"N={n} q={q} dim={r['dim']} eisenstein_root={r['eisenstein_root']} charpoly_hash={r['hash']}")
print(f"times: total {sum(r['ms']):.0f} ms")
