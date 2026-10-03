# An LMFDB for agents: design

Status: draft, 2026-10-03.  Working name: **the atlas** (to be chosen).

## 1. Why

The LMFDB is the reference database of $L$-functions, modular forms,
elliptic curves, number fields and related objects. It is built for
people: web pages, search forms, a Postgres back end. The next large group
of users will be AI agents doing research mathematics, and they need
something different:

- **Bulk, programmatic access** without CAPTCHAs or scraping. Today
  LMFDB's web API starts serving a reCAPTCHA page after a handful of
  requests; the read-only Postgres mirror works but is one server.
- **Trust that can be checked**: how each value was computed and how to
  recompute it.
- **Data that grows on demand**: if an object is not stored, compute it,
  certify it and keep it.
- **Global, cheap distribution**: R2 stores data at about \$0.015 per
  GB-month and charges nothing for egress, so free bulk access is
  affordable.

The atlas is complementary to the LMFDB. It mirrors LMFDB content (the
LMFDB community welcomes this), keeps LMFDB labels, independently
recomputes as much as possible, extends tables beyond what LMFDB stores
(for example newform orbits of dimension $>20$, whose $q$-expansions
LMFDB does not store), and reports any disagreements back.

**Non-goals for now:** a human-facing website (the LMFDB has one), and
replacing LMFDB governance.

## 2. Principles

1. **A cache of certified computations.** Every stored object carries
   provenance (source, engine, version, commit, parameters), its
   verification status, and a recipe to recompute it.
2. **Bulk first.** Immutable, content-addressed, columnar shards
   (Parquet) that standard tools (DuckDB, Polars, Arrow) read directly over
   HTTPS with range requests. An API is a convenience, not a gate.
3. **Compute when missing.** A request for an object that is not stored
   is answered by computing it (small jobs at the edge in WASM, larger ones
   on compute nodes), and the certified result is written back.
4. **Compact exact representations.** For example, newform coefficients
   in Stein's representation $a_p=\sum_r\beta_r c_{p,r}$, $c_{p,r}\in\mathbb{Z}$,
   after HNF+LLL: about 2 bits per coordinate, 2–8% below LMFDB's
   Hecke-ring coordinates per $a_p$, and 18× below the power basis at
   dimension 20 (see `results/compact.md`; the one-time $\beta_r$ are
   not yet measured).
5. **Agent-native interfaces.** Typed schemas with explicit conventions,
   an OpenAPI description and an MCP server, stable LMFDB-compatible labels,
   and machine-checkable links between objects.
6. **Claims, not only data.** Conjectures and observations stored as
   executable assertions over the data, with evidence and status.

## 3. Data model

### Objects and labels

Every object has a **kind** (`ec`, `mf_newform`, `nf_field`, `g2c`,
`lfunc`, …) and a **label**. Labels are LMFDB's wherever LMFDB has one
(`11.a2`, `389.2.a.e`), so the two systems interoperate. New kinds or
extensions get labels in the same grammar.

### Shards

A table is a set of Parquet shards (zstd), partitioned by a natural key
(for example by level range for modular forms, by conductor range for
elliptic curves). Shards are **immutable and content-addressed**: the
path contains the SHA-256 of the content. A table's current state is a
small **manifest** (JSON): the shard list with hashes, row counts, key
ranges, schema version and per-shard provenance summary. Updating a table
writes new shards and a new manifest, and old manifests remain
retrievable, so every query can cite an exact version.

### Columns: values, provenance, status

Each row has its mathematical columns plus three standard ones:

- `source`: `lmfdb-import` (with the mirror table and date),
  `sagebrush` (engine, version, commit, parameters), or a list when
  several sources agree.
- `status`, a small vocabulary ordered by strength:
  - `proven`: a certificate exists, e.g. a CRT bound, or a
    verification that is itself a proof;
  - `checked`: independent sources agree, e.g. LMFDB = Sagebrush = Sage;
  - `computed`: one source, a deterministic method;
  - `imported`: one external source, not yet recomputed;
  - `disputed`: sources disagree, kept with all values.
- `certificate`: a reference to a certificate object (below), when one
  exists.

### Certificates and recipes

A certificate is a JSON object stored once and referenced by hash. It
records:

- what is claimed;
- the method, e.g. "charpoly over $\mathbb{Z}$ by CRT over 45 primes,
  coefficient bound 1380 bits from the sum of squares of the eigenvalues";
- the checks performed;
- a **recipe**: the exact command, e.g.
  `sagebrush modsym 2310 13 --exact` at commit `…`, inputs and expected
  output hash, so any agent can recompute and compare.

### Compact representations

Large exact objects get a representation chosen for size and for
computing on them, recorded in the schema. For newform orbits: the
Hecke field polynomial, the one-time $\beta_r\in K$, and the integer
vectors $c_p$ (HNF+LLL reduced). Power-basis or Hecke-ring coordinates
are derived on demand.

## 4. Storage layout and cost

```
r2://atlas/
  manifests/<kind>/<table>/current.json      # pointer to the latest manifest
  manifests/<kind>/<table>/<sha256>.json     # immutable manifests
  shards/<sha256>.parquet                    # immutable data
  certs/<sha256>.json                        # certificates
  schemas/<kind>/<table>/<version>.json
```

Size, measured on the LMFDB mirror on 2026-10-03: 605 tables,
**1.92 TB with indexes and TOAST, 794 GB of table data**. The largest
tables:

| table | size | rows |
|---|---|---|
| `lfunc_lfunctions` | 304 GB | 24M |
| `char_dirichlet` | 157 GB | 562M |
| `gps_subgroup_data` | 150 GB | 275M |
| `nf_fields_extra` | 95 GB | 23M |

(`nf_fields_extra_old1` and `_old2` are 95 GB copies each.)

As compressed Parquet without Postgres indexes, the whole mirror is
plausibly a few hundred GB: a few dollars per month on R2, and under
\$30 per month even at the full 1.92 TB. Operation charges (per million
writes and reads, at R2's list prices) are negligible at this scale.
Prices should be rechecked before committing to a budget.

## 5. Ingest

1. **Import the LMFDB mirror.** Read each table through the mirror's
   Postgres, write Parquet shards with `source = lmfdb-import` and
   `status = imported`, and keep the LMFDB schema and knowl documentation
   alongside. This runs once in full, then incrementally (track
   `pg_stat` or table checksums).
2. **Recompute.** Sagebrush engines recompute what they can and set
   `checked` or `disputed`. Today's coverage, as the first targets:
   - elliptic curves of conductor $\le 9999$ (rational newforms, a_p),
     done and matching Cremona;
   - weight-2 newform orbits for $N\le 1000$ (dimensions, traces and
     exact compact $a_p$), done and matching LMFDB;
   - exact Hecke charpolys.

   Disagreements go into a `disputed` report and are sent to LMFDB.
3. **Extend.** Add what LMFDB lacks: $q$-expansions of all orbits (no
   dimension cap), larger ranges as compute allows, and new kinds of
   objects.

## 6. Access

- **Bulk.** Shards over HTTPS, cached by Cloudflare's CDN and readable
  with range requests:

  ```sql
  -- DuckDB, straight from the bucket (illustrative)
  select label, dim from read_parquet('https://atlas.example/shards/<sha>.parquet') where dim > 20;
  ```

  A small client (Python and JS, on Sagebrush's bindings) resolves
  manifests to shards and verifies hashes.
- **API (Workers).** Label lookup, small queries over manifest metadata,
  certificate retrieval, and compute requests. It is stateless; indexes
  live in D1 or KV.
- **MCP server and OpenAPI.** The same operations as tools for agents,
  with schemas, units and conventions in the tool descriptions.
- **No CAPTCHA.** Anonymous bulk reads are free. Compute requests
  above a small free tier need an API key, so compute cost stays
  bounded.

## 7. Compute on demand

- **Edge.** Small, bounded computations run in Workers via Sagebrush's
  WASM build, e.g. a_p for $p\le 10^5$ of a given curve, or a mod-$p$
  Hecke charpoly at small level. Worker size and CPU limits mean only
  lean engines go here.
- **Nodes.** Larger jobs go into a queue (Cloudflare Queues or Durable
  Objects) and run on compute nodes (CoCalc projects, dedicated
  machines) with Sagebrush's Rust engines. Results are written as new
  shards with certificates, and the manifest updates.
- **Estimates first.** Engines already predict cost (`estimate`), so the
  API can quote time and memory before accepting a job.

## 8. Claims

A claim is a stored, executable statement over the data, for example
"for every rational newform of level $\le 10^4$, $|a_p|\le 2\sqrt p$
for $p<1000$", or a murmuration statistic, or a conjectured identity. It
records the query or program that evaluates it, the data versions it ran
on, its result, and who or what proposed it (human or agent). Claims are
re-evaluated when the data changes. This is where agents doing research
leave reusable evidence instead of chat transcripts.

## 9. Trust and verification

- Statuses form a ladder (`imported` < `computed` < `checked` <
  `proven`), and queries can filter on them.
- Every certificate has a recipe, and a verification service
  periodically recomputes a random sample and records the outcome.
- Independent implementations are the strongest check: LMFDB, Sage,
  Magma and Sagebrush each agreeing on a value is recorded as such.
- Later: certificates checkable by a proof assistant (Lean) for the
  statements where that is realistic, e.g. exact charpolys via a
  verified CRT bound.

## 10. People and licensing

- The LMFDB collaboration (Andrew Sutherland is central organizationally)
  is interested in mirrors of its content. Agree on attribution, labels
  and how disagreements are reported, and confirm the data license terms
  before republishing.
- Imported data keeps its license and attribution. Recomputed data is
  ours to license, and should be permissive, e.g. CC BY or CC0.

## 11. Milestones

| milestone | contents | needs |
|---|---|---|
| **M0: local prototype** | schemas, Parquet layout, manifests, certificates; elliptic curves ($N\le 9999$) and weight-2 newforms ($N\le 1000$) from Sagebrush, cross-referenced with the LMFDB import; DuckDB queries in a notebook | nothing new |
| **M1: R2 and read API** | bucket, CDN, Workers lookup API, MCP server, Python and JS clients | Cloudflare account, R2 bucket, API token as a CoCalc project secret, a domain |
| **M2: full LMFDB mirror** | all 605 tables imported with provenance, incremental updates | about 1 TB of transfer from the mirror (to be scheduled politely with LMFDB) |
| **M3: compute on demand** | edge WASM jobs, node queue, write-back with certificates | compute nodes |
| **M4: claims and verification** | claims store, sampling verifier, disagreement reports to LMFDB | |

M0 could be demonstrated at the October DARPA workshop: an agent queries
the atlas, checks a certificate by running its recipe, and extends a
table.

## 12. Open questions

- **The name.**
- **Search beyond labels:** full-text and semantic search over knowls
  and claims, and how much query capability to offer beyond bulk Parquet.
- **Mutable data:** LMFDB's tables change over time; how closely to
  track them.
- **Abuse and cost controls** for compute on demand.
- **Representations for other object kinds:** which compact forms they
  should use, and where they differ from LMFDB's.
