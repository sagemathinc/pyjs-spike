# pyjs-spike

A two-week experiment to decide whether Python semantics compiled to
JavaScript can be competitive with CPython. Read [PLAN.md](PLAN.md).

```sh
python3 bench/bench.py      # CPython baseline
node target/bench.js        # hand-written ideal compiler output
node --test target/         # semantics checks for the runtime fast paths
```
