# Can Python semantics on V8 be competitive with CPython?

This repository answers one question, because it decides what Sage.js is:

> Can a Python→JavaScript compiler that keeps **real Python semantics** run
> ordinary Python code **within 2× of CPython on every benchmark, with no
> cliffs**?

If yes, the JavaScript runtime is a genuine differentiator: instant startup,
browser-native, V8's JIT on numeric loops, direct JS interop and one runtime
for many front-end languages. If no, CPython (native, or Pyodide in the browser)
should be the primary Python host, and Sage.js's value moves to its libraries,
language front ends, graphics and Rust kernels.

"Within 2×" applies to both a first (cold) run and a warm median, on every
benchmark individually. Averages hide exactly the cliffs that make the current
runtime unusable for agents and people.

## Step 0: the ceiling experiment (done)

Before writing a compiler, write its intended output by hand and measure it.
If even ideal output cannot reach the goal, no compiler will.

* `bench/bench.py` holds ten microbenchmarks plus pyperformance's `nbody`.
* `target/rt.js` is an ~800-line runtime implementing the representation and
  caching design below.
* `target/bench.js` is `bench.py` translated *mechanically*, the way the
  compiler would emit it:
  * every Python operation goes through a generic runtime helper (`add`,
    `getitem`, `truth`, ...) or a per-site inline cache;
  * the only specializations are ones a compiler knows statically: counting
    loops for `range` (guarded on `range` being the builtin), pre-boxed
    integral float literals, and `push` onto a freshly built comprehension
    list.
* `target/semantics.test.js` checks that the fast paths keep Python semantics:
  * BigInt overflow and its return to numbers;
  * int/float identity, `-0.0`, and float `repr`;
  * floor-division and modulo signs;
  * cache invalidation when a class is mutated;
  * instance attributes shadowing methods;
  * data descriptors winning over the instance dict;
  * call sites seeing more than four classes;
  * `1`/`1.0`/`True` being the same dict key;
  * slice index adjustment;
  * CPython-worded error messages.

  Run them with `node --test target/`.

Results on a 16-CPU Linux x64 host (`results/run-1.txt`): CPython 3.14.4, Node
26.10.0, and Sage.js HEAD `b52dbdb73` running the identical `bench.py`. All three
produce identical results, including nbody's energy to the last bit.

| benchmark | CPython warm ms | target warm ms | target / CPython (warm, cold) | Sage.js HEAD / CPython (warm) |
| --- | ---: | ---: | ---: | ---: |
| `fib(25)` | 9.0 | 1.2 | 0.13, 1.09 | 4.8 |
| int loop | 121.9 | 47.7 | 0.39, 0.42 | 1.2 |
| float loop | 30.6 | 9.3 | 0.30, 0.37 | 11 |
| method call | 52.0 | 5.0 | 0.10, 0.15 | 45 |
| polymorphic method (4 classes) | 72.3 | 25.5 | 0.35, 0.44 | 5.8 |
| list comprehension | 54.9 | 26.8 | 0.49, 0.78 | 1.7 |
| dict with str keys | 50.1 | 62.8 | **1.25, 1.53** | 19 |
| `str.join`/`split` | 27.8 | 22.8 | 0.82, 0.72 | 18 |
| `bytearray` sieve | 9.6 | 10.1 | 1.05, 1.48 | 368 |
| bigint Fibonacci | 4.0 | 2.6 | 0.65, 0.95 | 0.9 |
| pyperformance `nbody` | 96.0 | 80.4 | 0.84, 0.93 | 28 |

What this shows:

* **The ceiling clears the goal by a wide margin.** The worst case is 1.25× warm
  and 1.53× cold, and most cases beat CPython. The gap to current Sage.js
  (up to 368×) comes from code generation and runtime design, not from
  JavaScript.
* **Two cliffs were library bugs, not compiler problems.** The sieve started at
  13× because `bytearray * n` copied one subarray per element and
  `sum(bytearray)` iterated generically. Fixing both brought it to 1.05×. The
  benchmark suite has to cover library breadth, not just language speed.
* **Polymorphism needs polymorphic caches.** With single-entry caches the
  four-class call site ran at 0.8×; four-way caches brought it to 0.35×.

What it does **not** show:

* that a compiler emits this code automatically. Body duplication for
  guarded loops, temporaries, and default/keyword argument handling are
  mechanical, but they are not yet mechanized;
* anything about the long tail: generators beyond the simple cases,
  exceptions and tracebacks, closures and cells, `super()` and multiple
  inheritance, `property`/`classmethod`, keyword-argument and `*args` calls,
  `str` code-point semantics, dicts with tuple keys, or `globals()`;
* code size, compile latency, or startup.

## Status (day 1 of the two-week compiler)

The compiler exists and runs the pyperformance subset.  `src/` has the
tree-sitter frontend, scope analysis, emitter and runtime (about 10k lines
of TypeScript); `lib/` has stdlib modules written in Python and compiled by
pyjs.  Measured on `bench-1` (8x AMD EPYC 7B13, dedicated) against CPython
3.14.8 (`results/pyperformance-bench1.md`; one cold call, ~1 s warmup,
median of five):

* **Warm: geometric mean 0.99x CPython over 21 benchmarks.**  Faster than
  CPython on 11 (spectral_norm 0.48, scimark_fft 0.63, float 0.70, deltablue
  0.71, hexiom 0.76, scimark_sor 0.89, nqueens 0.96, ...).  Since that run,
  meteor_contest went from 2.08x to 1.49x (set algebra on Map keys), so
  every benchmark is now within 2x warm; the slowest are pidigits (1.8-2.0x,
  V8 BigInt division), richards_super 1.75x and generators 1.2-1.6x.
* **Cold (first call): misses 2x on short benchmarks**: deltablue 8.7x (its
  whole run is 3 ms), unpack_sequence 4.4x, hexiom 4.3x, richards_super 2.9x,
  sparse_mat_mult 2.5x.  The first call executes our generated JavaScript in
  V8's interpreter before TurboFan optimizes it; profiling shows the time
  spread over the generated code itself, not compilation or cache misses.
  In absolute terms this is a one-time cost of tens of milliseconds per hot
  function.  End to end the scripts remain competitive (deltablue: 190 ms
  total vs CPython's 276 ms), but process startup is 81 ms vs 24 ms (Node
  29 ms, runtime init 24 ms, tree-sitter WASM 20 ms).
* **Correctness:** every benchmark matches CPython's results; the MicroPython
  corpus passes 363 of 511.  Most remaining failures are library breadth
  (memoryview, struct, io.BytesIO, collections, weakref) or out-of-scope
  features (async), not the language core.  Uncaught exceptions print CPython
  tracebacks with file, line and source.

What made the difference, in order of impact: the representation and
caches from step 0; a TypeScript `array` with typed storage; per-class
caches (dunders, construction, `==` reducing to identity, class-receiver
call sites) keyed on class version; native delegation for `yield from`;
eager loops for generator expressions consumed by builtins; set algebra on
Map keys; and statement-scoped temporaries.

Known gaps: set iteration order differs from CPython's hash-table order;
`str` uses UTF-16 indexing; no `async`, metaclasses, or int/str/float
subclasses; generator close()/throw() edge cases; `__dict__` is a snapshot.

* **Real number-theory code** (`results/modsym.md`): a plain-Python
  modular-symbols program (Gamma0(N), Hecke operators, charpoly mod p) runs
  1.25-1.5x faster on pyjs than on CPython, 9x faster than on the old
  sagejs runtime, and on par with Sage's own native implementation up to
  N ~ 5000; all agree exactly with Sage.  Sage pulls ahead at N = 10007
  through native linear algebra, which is where FLINT belongs in the stack.

**Provisional answer to the question:** yes on throughput -- compiled Python
on V8 is at parity with CPython 3.14 in steady state, and faster on numeric
and object-heavy code.  The open risk is first-run latency on very short
workloads, which is a property of V8's tiering rather than of this design.

## Design validated by step 0

**Value representation.**

* `int`: a primitive JS number when it is an integer, and then always within
  ±(2^53−1). Outside that range it is a BigInt, so there is exactly one
  representation per value.
* `float`: a primitive number that is *not* an integer (including NaN and
  ±Infinity), or a `FloatBox` for integral values and −0.0.
* `bool` is a JS boolean, `None` is `null`, `str` is a JS string.
* `list` is a plain JS Array and `tuple` is an Array marked `$t`.
* `dict` wraps a `Map`, with key normalization so that `1`, `1.0` and `True`
  are the same key.

So `typeof x === "number" && Number.isInteger(x)` decides int vs float with
no allocation. Integer results are checked with `Number.isSafeInteger` and
fall back to BigInt. Float results are boxed only when integral, which is rare
inside real floating-point work.

**Instances and attribute access.**

* Instances are ordinary objects whose prototype chain contains only `$cls`.
  A missing attribute therefore reads `undefined`, and V8 hidden classes stay
  monomorphic.
* Each class has a version number, bumped (including in subclasses) whenever
  the class dict changes.
* Each attribute site caches (class, version, kind):
  * kind 0 means "instance dict, and the class has no data descriptor of this
    name";
  * kind 1 means "class attribute not shadowed by the instance".
* Method-call sites cache up to four (class, version, function) entries. On a
  hit they call the plain function with `self` directly, so no bound-method
  object is created.
* Builtin receivers (`str`, `list`, `dict`) use a separate kind slot.

**Calls.** Every Python callable that is a JS function accepts positional
arguments directly, and arity is checked with `arguments.length`. Keyword
calls will use a second generated entry point, so positional calls never scan
for a kwargs sentinel. Call sites emit
`typeof f === "function" ? f(a) : callSlow(f, [a])`.

**Module globals.** Module globals are JS `let` bindings. A name never bound in
the module resolves to the builtins object.

**Per-site code.** Every cache site owns its own code and feedback. In the
experiment this is a generated closure; the compiler emits it inline. Shared
helpers such as `add` only perform JS operations on already-checked types, so
their V8 feedback stays clean when inlined.

## The compiler to build

* **One compiler, written in TypeScript.** No self-hosting and no bootstrap
  stage.
* **Pipeline:**
  1. Parse with the tree-sitter Python grammar (WASM), reusing Sage.js's
     vendored grammar.
  2. Lower to a Python AST with source spans. Sage.js's
     `tools/python/lowerer.ts` and `frontend.ts` are a starting point.
  3. Run scope analysis: locals, cells, globals, comprehension scopes.
  4. Build a small explicit IR: three-address operations on named temporaries,
     with an explicit site id for each cacheable operation.
  5. Run a handful of passes:
     * statically known types for loop counters and literals;
     * guarded `range` loops;
     * constant boxing;
     * cache site allocation.
  6. Emit JavaScript plus a source map.
* **Tracebacks are a compiler invariant, not a feature.** Every uncaught
  exception prints a CPython-style traceback with the Python file, line and
  source text, mapped through the source map. Internal control-flow exceptions
  (StopIteration, KeyError on a `.get` miss) must not capture JS stacks.
* **The runtime stays small:** `rt.js` grows into a set of modules covering
  builtins, `str`, `list`, `dict`, `set`, `bytes`, exceptions and generators.
  The runtime library is written in TypeScript, not in compiled Python, so
  its own hot paths are plain JS.

## Pass/fail criteria for the two-week compiler

1. **Speed.**
   * Compiled output of `bench/bench.py` stays within 1.2× of `target/bench.js`.
   * A pure-Python pyperformance subset runs within 2× of CPython, cold and
     warm, on *every* benchmark: `richards`, `deltablue`, `raytrace`, `chaos`,
     `go`, `hexiom`, `nqueens`, `fannkuch`, `spectral_norm`, `float`, `nbody`,
     `pystone`, `unpack_sequence`, `generators`, `pidigits`. A benchmark
     needing an unimplemented feature counts as a failure, not an exclusion.
2. **Correctness.** Use differential execution against CPython: identical
   stdout and exception type for every applicable program in the MicroPython
   corpus that Sage.js vendors (`upstream-tests/micropython`, 508 programs).
   Every benchmark's result must match CPython exactly.
3. **Errors.** CPython-worded messages for the common exceptions, and Python
   tracebacks for uncaught exceptions, tested.
4. **Budgets.**
   * Startup to the first `print` under 50 ms in Node with a warm code cache.
   * Runtime plus compiler under 2 MB minified.
   * Compile time under 1 ms per 100 source lines once warm.

**Decision rule.** If 1–3 pass, the JavaScript runtime is the foundation of
Sage.js. If speed fails on a benchmark, write that benchmark's ideal output by
hand, as in step 0. If the hand-written version also fails, the gap is
fundamental and CPython becomes the primary host; otherwise the compiler needs
more work, not a different direction.

## Milestones

| days | deliverable |
| --- | --- |
| 1–3 | Compiler skeleton. Compiles `bench/bench.py` to output equivalent to `target/bench.js`; differential test and benchmark harness in CI. |
| 4–7 | Language core: functions with defaults, keywords, `*args`/`**kwargs`; closures; classes with inheritance, `super()`, `property`, `classmethod`, `staticmethod`; exceptions; generators; comprehensions; `with`; `global`/`nonlocal`. Tracebacks. MicroPython differential corpus. |
| 8–11 | pyperformance subset. For every benchmark over 2×, profile and classify the cause (code generation, cache design, or runtime library) and fix it. |
| 12–14 | Cold start, code size, megamorphic and deoptimization stress, browser run. Write the decision report into this file. |

## Out of scope for this experiment

Sage mathematics; the Magma, Mathematica, Maple, MATLAB and Macaulay2 front
ends; C-extension compatibility; numpy; packaging; Sage preparser syntax.
These all sit above the runtime and do not affect the answer.

## Known hard problems

* **`str` semantics.** Python strings index by code point, JS strings by UTF-16
  unit. Candidate design: strings carry no flag, and the runtime checks
  `/[\uD800-\uDFFF]/` lazily per string, caching the result in a side
  `WeakMap` only for non-BMP strings. This needs measuring.
* **Dicts with arbitrary hashable keys** (tuples, user `__hash__`): a hash →
  bucket fallback alongside the primitive-key `Map`, preserving insertion
  order across both.
* **Guarded loop specialization** duplicates loop bodies. Body size limits, or
  outlining the generic path into a closure, will be needed.
* **Representation for JS interop.** Python instances are null-prototype
  objects, so exposing them to JavaScript needs a wrapper or proxy layer.
* **Deoptimization behavior.** Programs that mutate classes or rebind builtins
  in hot loops must stay correct and degrade gracefully, never crash or loop
  forever in cache refills.
