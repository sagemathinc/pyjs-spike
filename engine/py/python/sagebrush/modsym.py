"""Weight-2 modular symbols for Gamma0(N), sign +1 (the Rust engine).

Every function releases the GIL and uses `threads` worker threads (0 means
all cores). Invalid arguments raise ValueError.
"""

from ._native import modsym as _native

charpoly_exact = _native.charpoly_exact
batch_exact = _native.batch_exact
hecke_charpoly = _native.hecke_charpoly
level_data = _native.level_data
estimate = _native.estimate
commute = _native.commute

__all__ = ["charpoly_exact", "batch_exact", "hecke_charpoly", "level_data", "estimate", "commute"]
