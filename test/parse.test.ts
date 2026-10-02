import test from "node:test";
import assert from "node:assert/strict";
import { initParser, parse } from "../src/parse";

test("parses the benchmark program and exercises the lowering", async () => {
  await initParser();
  const src = require("fs").readFileSync(__dirname + "/../../bench/bench.py", "utf8");
  const m = parse(src, "bench.py");
  assert.ok(m.body.length > 20);
  const s = parse('x = f"{a!r:>{w}}{{" "b\\n" \nd = {**a, 1: 2}\n', "t.py");
  assert.equal(JSON.stringify((s.body[0] as any).value.parts.map((p: any) => (typeof p === "string" ? p : p.conv))), '["r","{b\\n"]');
});
