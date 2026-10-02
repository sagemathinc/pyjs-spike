# Modular symbols: one plain-Python program under four systems

`bench/modsym/modsym.py`: weight-2 modular symbols for Gamma0(N), sign +1,
over GF(67108859): Manin symbols on P^1(Z/NZ), 2- and 3-term relations,
T_q via Cremona's Heilbronn matrices, characteristic polynomial via
Hessenberg form.  Only ints, lists and dicts.  Every system printed the same
characteristic-polynomial hash; the Sage column is Sage's own
`ModularSymbols(N, 2, sign=1, base_ring=GF(p)).hecke_matrix(q).charpoly()`
(not this program), which also matches.

16-CPU development host; CPython 3.14.4, Node 26.10, Sage (system install).
Times are the program's own total (ms, excluding process startup) except
Sage, which is wall time of the computation.

| N | q | dim | CPython | pyjs | old sagejs (HEAD b52dbdb73) | Sage native |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 389 | 2 | 33 | 28 | 51 | 327 | 99 |
| 2003 | 2 | 168 | 1362 | 946 | 8711 | 1310 |
| 2003 | 97 | 168 | 1501 | 1034 | | 1705 |
| 5077 | 2 | 423 | 15276 | 10057 | | 11250 |
| 5077 | 97 | 423 | 16040 | 10596 | | 14575 |
| 10007 | 2 | 835 | 128776 | 103177 | | 48154 |

Reading: pyjs runs this number-theory code 1.25-1.5x faster than CPython
and 9x faster than the old sagejs runtime, and is on par with Sage's native
implementation up to N ~ 5000.  At N = 10007 Sage wins because its
characteristic polynomial and elimination use native linear algebra over
GF(p), while this program does dense linear algebra in Python loops -- the
layering argument: Python for the structure, FLINT for the kernels.
