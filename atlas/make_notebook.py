"""Writes and executes demo_m0.ipynb (the atlas M0 demo):  python make_notebook.py ROOT"""
import sys
import nbformat
from nbclient import NotebookClient

ROOT = sys.argv[1]
md = nbformat.v4.new_markdown_cell
code = nbformat.v4.new_code_cell
cells = [
    md("# The atlas, milestone M0\n\nA local prototype of the agent-first LMFDB described in `design/lmfdb-for-agents.md`: "
       "immutable Parquet shards, manifests, certificates, and a status for every row. "
       "Everything below is what an agent would do: discover tables, query them with DuckDB, check a certificate, "
       "recompute a row, and evaluate a claim over a whole table."),
    code(f"import json, os, sys, duckdb\nsys.path.insert(0, os.path.expanduser('~/sagebrush/atlas'))\nimport atlas\nROOT = {ROOT!r}\n"
         "tables = [('mf', 'rational_newforms_wt2'), ('mf', 'newform_orbits_wt2'), ('ec', 'curves'), ('lmfdb', 'mf_newforms_wt2')]\n"
         "for kind, table in tables:\n"
         "    m = atlas.manifest(ROOT, kind, table)\n"
         "    size = sum(s['bytes'] for s in m['shards'])\n"
         "    print(f\"{kind}/{table}: {m['rows']:,} rows, {len(m['shards'])} shards, {size/1e6:.1f} MB; \"\n"
         "          + ', '.join(f'{k} {v}' for k, v in m['status_counts'].items() if v))"),
    md("## Query with DuckDB\n\nShards are verified against their SHA-256 before use. Every elliptic curve is linked to "
       "its newform by point counts (modularity, checked row by row)."),
    code("con = duckdb.connect()\nfor kind, table in tables:\n    atlas.duckdb_view(con, ROOT, kind, table, f'{kind}_{table}')\n"
         "con.sql(\"\"\"select c.label as curve, c.ainvs, c.rank, n.label as newform, n.ap[1:6] as a2_to_a13\n"
         "          from ec_curves c join mf_rational_newforms_wt2 n on c.newform = n.label\n"
         "          where c.conductor in (11, 37, 389, 5077) and c.label like '%1' order by c.conductor\"\"\").show()"),
    md("## Certificates\n\nEach row points to a certificate: what is claimed, the method, the checks, and a recipe to recompute."),
    code("sha = con.sql(\"select certificate from mf_rational_newforms_wt2 where label = '5077.2.a.a'\").fetchone()[0]\n"
         "print(json.dumps(atlas.certificate(ROOT, sha), indent=2)[:1500])"),
    md("## Recompute a row and compare\n\nAn agent that doubts a value recomputes it. Here, from modular symbols "
       "and from point counting, with Sagebrush."),
    code("from sagebrush import modsym, ap\n"
         "row = con.sql(\"select label, level, ap from mf_rational_newforms_wt2 where label = '5077.2.a.a'\").fetchone()\n"
         "primes = [p for p in range(2, 1000) if all(p % d for d in range(2, int(p**0.5) + 1))]\n"
         "stored = {p: a for p, a in zip(primes, row[2]) if a is not None}\n"
         "fresh = dict(modsym.rational_newforms(row[1], 999)[0])\n"
         "curve = con.sql(\"select ainvs from ec_curves where newform = '5077.2.a.a' limit 1\").fetchone()[0]\n"
         "counted = dict(ap.aplist(curve, 999))\n"
         "print('stored == recomputed by modular symbols:', stored == fresh)\n"
         "print('stored == point counts:', all(counted[p] == a for p, a in stored.items()))"),
    md("## A claim over a whole table\n\nThe Hasse bound $|a_p|\\le 2\\sqrt p$ for every rational newform of level $\\le 9999$ "
       "and every $p<1000$, as one query."),
    code("total = con.sql(\"select count(*) from (select unnest(ap) as a from mf_rational_newforms_wt2) where a is not null\").fetchone()[0]\n"
         "viol = con.execute(\"\"\"select count(*) from (select unnest(ap) as a, unnest(?) as p from mf_rational_newforms_wt2)\n"
         "                     where a is not null and a * a > 4 * p\"\"\", [primes]).fetchone()[0]\n"
         "print(f'{total:,} values of a_p checked; violations of the Hasse bound: {viol}')"),
    md("## Murmurations\n\nThe average of $a_p$ over rational newforms with level in $[5000, 10000)$, by parity of the "
       "analytic rank (He, Lee, Oliver and Pozdnyakov). Computed from the table in one query."),
    code("import matplotlib.pyplot as plt\n"
         "df = con.execute(\"\"\"select analytic_rank % 2 as parity, p, avg(a) as mean_ap, count(*) as n\n"
         "    from (select analytic_rank, unnest(ap) as a, unnest(?) as p from mf_rational_newforms_wt2 where level >= 5000)\n"
         "    where a is not null group by all order by p\"\"\", [primes]).df()\n"
         "fig, ax = plt.subplots(figsize=(9, 3.5))\n"
         "for parity, colour in [(0, 'tab:blue'), (1, 'tab:red')]:\n"
         "    d = df[df.parity == parity]\n"
         "    ax.scatter(d.p, d.mean_ap, s=6, c=colour, label=f'analytic rank {\"even\" if parity == 0 else \"odd\"}')\n"
         "ax.axhline(0, c='grey', lw=0.5); ax.set_xlabel('p'); ax.set_ylabel('mean a_p'); ax.legend()\n"
         "ax.set_title('Murmurations: rational newforms of level 5000-9999')\nplt.savefig('murmurations_m0.png', dpi=120, bbox_inches='tight'); plt.show()"),
    md("## Compact exact coefficients for orbits LMFDB does not store\n\n"
       "LMFDB stores $q$-expansions only up to dimension 20. Here every orbit of level $\\le 1000$ has exact integer "
       "coordinates of $a_p$ (Stein's representation after HNF + LLL)."),
    code("con.sql(\"\"\"select label, dim, len(ap_coordinates) as primes, trace_ap[1:4] as tr_a2_to_a7\n"
         "   from mf_newform_orbits_wt2 where dim >= 50 order by dim desc\"\"\").show()\n"
         "row = con.sql(\"select ap_coordinates from mf_newform_orbits_wt2 where label = '971.2.a.b'\").fetchone()[0]\n"
         "bits = sum(abs(int(x)).bit_length() + 1 for v in row for x in v)\n"
         "print(f'971.2.a.b (dim 55): {bits / len(row):.0f} bits per a_p; a_2 coordinates: {row[0]}')"),
]
nb = nbformat.v4.new_notebook(cells=cells, metadata={"kernelspec": {"name": "python3", "display_name": "Python 3", "language": "python"}})
NotebookClient(nb, timeout=600, kernel_name="python3").execute()
nbformat.write(nb, "demo_m0.ipynb")
print("wrote demo_m0.ipynb")
