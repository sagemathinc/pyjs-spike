"use strict";
// Hand-written stand-in for what the planned compiler would emit for
// bench/bench.py.  Rules: every Python operation goes through the generic
// runtime helper (add, getitem, truth, ...) or a per-site cache; the only
// specializations are ones a compiler knows statically:
//   * `for i in range(n)` becomes a counting loop, guarded on `range` still
//     being the builtin and `n` being a small int (else a generic loop);
//   * float literals with integral value are pre-boxed module constants;
//   * a freshly built comprehension list is appended with push().
// Module globals are JS `let` bindings; names never bound in the module
// resolve to the builtins object B.
const R = require("./rt.js");
const { add, sub, mul, truediv, floordiv, mod, pow, iadd, isub, lt, le, truth, getitem, setitem, pyiter, unpack, tuple, B, RANGE, argcError, callSlow } = R;
const F0 = new R.FloatBox(0); // the literal 0.0
const F2 = new R.FloatBox(2); // the literal 2.0
const genericLoop = () => {
  throw new Error("spike: generic for-loop path not written by hand");
};

// def fib(n): return n if n < 2 else fib(n - 1) + fib(n - 2)
let fib = R.pyfn("fib", function (n) {
  if (arguments.length !== 1) argcError("fib", 1, arguments.length);
  let t0, t1;
  return truth(lt(n, 2))
    ? n
    : add(
        typeof (t0 = fib) === "function" ? t0(sub(n, 1)) : callSlow(t0, [sub(n, 1)]),
        typeof (t1 = fib) === "function" ? t1(sub(n, 2)) : callSlow(t1, [sub(n, 2)])
      );
});
let b_fib = R.pyfn("b_fib", function () {
  if (arguments.length !== 0) argcError("b_fib", 0, arguments.length);
  let t0;
  return typeof (t0 = fib) === "function" ? t0(25) : callSlow(t0, [25]);
});

let b_loop = R.pyfn("b_loop", function () {
  if (arguments.length !== 0) argcError("b_loop", 0, arguments.length);
  let s, i;
  s = 0;
  const n = 2000000;
  if (B.range === RANGE) {
    for (let k = 0; k < n; k++) {
      i = k;
      s = iadd(s, mod(mul(i, i), 7));
    }
  } else genericLoop();
  return s;
});

let b_float = R.pyfn("b_float", function () {
  if (arguments.length !== 0) argcError("b_float", 0, arguments.length);
  let x, i, t0;
  x = F0;
  if (B.range === RANGE) {
    for (let k = 0; k < 1000000; k++) {
      i = k;
      x = add(mul(x, 0.999), 1.5);
    }
  } else genericLoop();
  return typeof (t0 = B.round) === "function" ? t0(x, 3) : callSlow(t0, [x, 3]);
});

const set_x_1 = R.siteSet("x"), get_x_2 = R.siteGet("x"), set_x_2 = R.siteSet("x"), get_x_3 = R.siteGet("x");
let P = R.makeClass("P", {
  __init__: R.pyfn("__init__", function (self, x) {
    if (arguments.length !== 2) argcError("__init__", 2, arguments.length);
    set_x_1(self, x);
    return null;
  }),
  inc: R.pyfn("inc", function (self, d) {
    if (arguments.length !== 2) argcError("inc", 2, arguments.length);
    set_x_2(self, iadd(get_x_2(self), d));
    return self;
  }),
});
const call_inc_1 = R.siteCall("inc", 1);
let b_attr = R.pyfn("b_attr", function () {
  if (arguments.length !== 0) argcError("b_attr", 0, arguments.length);
  let p, i, t0;
  p = typeof (t0 = P) === "function" ? t0(0) : callSlow(t0, [0]);
  if (B.range === RANGE) {
    for (let k = 0; k < 1000000; k++) {
      i = k;
      call_inc_1(p, 1);
    }
  } else genericLoop();
  return get_x_3(p);
});

// Four unrelated classes behind one `s.area()` call site.
const shapeClass = (name, fields, area) => {
  const sets = fields.map((f) => R.siteSet(f));
  return R.makeClass(name, {
    __init__: R.pyfn("__init__", function (self, ...args) {
      if (args.length !== fields.length) argcError("__init__", fields.length + 1, args.length + 1);
      sets.forEach((set, i) => set(self, args[i]));
      return null;
    }),
    area: R.pyfn("area", area),
  });
};
const get_s_1 = R.siteGet("s"), get_s_2 = R.siteGet("s");
const get_w_1 = R.siteGet("w"), get_h_1 = R.siteGet("h");
const get_r_1 = R.siteGet("r"), get_r_2 = R.siteGet("r");
const get_b_1 = R.siteGet("b"), get_h_2 = R.siteGet("h");
let Sq = shapeClass("Sq", ["s"], function (self) {
  if (arguments.length !== 1) argcError("area", 1, arguments.length);
  return mul(get_s_1(self), get_s_2(self));
});
let Rect = shapeClass("Rect", ["w", "h"], function (self) {
  if (arguments.length !== 1) argcError("area", 1, arguments.length);
  return mul(get_w_1(self), get_h_1(self));
});
let Circ = shapeClass("Circ", ["r"], function (self) {
  if (arguments.length !== 1) argcError("area", 1, arguments.length);
  return mul(mul(3, get_r_1(self)), get_r_2(self));
});
let Tri = shapeClass("Tri", ["b", "h"], function (self) {
  if (arguments.length !== 1) argcError("area", 1, arguments.length);
  return floordiv(mul(get_b_1(self), get_h_2(self)), 2);
});
const call_area_1 = R.siteCall("area", 0);
let b_poly = R.pyfn("b_poly", function () {
  if (arguments.length !== 0) argcError("b_poly", 0, arguments.length);
  let shapes, t, i, t0, t1, t2, t3;
  shapes = mul(
    [
      typeof (t0 = Sq) === "function" ? t0(3) : callSlow(t0, [3]),
      typeof (t1 = Rect) === "function" ? t1(2, 5) : callSlow(t1, [2, 5]),
      typeof (t2 = Circ) === "function" ? t2(4) : callSlow(t2, [4]),
      typeof (t3 = Tri) === "function" ? t3(6, 7) : callSlow(t3, [6, 7]),
    ],
    1000
  );
  t = 0;
  if (B.range === RANGE) {
    for (let k = 0; k < 250; k++) {
      i = k;
      for (const s of pyiter(shapes)) t = iadd(t, call_area_1(s));
    }
  } else genericLoop();
  return t;
});

let b_list = R.pyfn("b_list", function () {
  if (arguments.length !== 0) argcError("b_list", 0, arguments.length);
  let t0;
  const L = [];
  if (B.range === RANGE) {
    for (let k = 0; k < 1000000; k++) {
      const i = k;
      if (truth(mod(i, 3))) L.push(mul(i, 2));
    }
  } else genericLoop();
  return typeof (t0 = B.sum) === "function" ? t0(L) : callSlow(t0, [L]);
});

let b_dict = R.pyfn("b_dict", function () {
  if (arguments.length !== 0) argcError("b_dict", 0, arguments.length);
  let d, i, t0, t1;
  d = new R.PyDict();
  if (B.range === RANGE) {
    for (let k = 0; k < 200000; k++) {
      i = k;
      setitem(d, typeof (t0 = B.str) === "function" ? t0(i) : callSlow(t0, [i]), i);
    }
  } else genericLoop();
  // (d[k] for k in d): the outermost iterable is evaluated eagerly.
  const it = pyiter(d);
  const gen = (function* () {
    for (const k of it) yield getitem(d, k);
  })();
  return typeof (t1 = B.sum) === "function" ? t1(gen) : callSlow(t1, [gen]);
});

const call_join_1 = R.siteCall("join", 1), call_split_1 = R.siteCall("split", 1);
let b_str = R.pyfn("b_str", function () {
  if (arguments.length !== 0) argcError("b_str", 0, arguments.length);
  let t0;
  const gen = (function* () {
    let t1;
    if (B.range === RANGE) {
      for (let k = 0; k < 200000; k++) yield typeof (t1 = B.str) === "function" ? t1(k) : callSlow(t1, [k]);
    } else genericLoop();
  })();
  return typeof (t0 = B.len) === "function"
    ? t0(call_split_1(call_join_1(",", gen), ","))
    : callSlow(t0, [call_split_1(call_join_1(",", gen), ",")]);
});

let b_sieve = R.pyfn("b_sieve", function () {
  if (arguments.length !== 0) argcError("b_sieve", 0, arguments.length);
  let n, s, i, t0, t1, t2, t3, t4;
  n = 2000000;
  s = mul(typeof (t0 = B.bytearray) === "function" ? t0([1]) : callSlow(t0, [[1]]), add(n, 1));
  setitem(s, 0, 0);
  setitem(s, 1, 0);
  i = 2;
  while (truth(le(mul(i, i), n))) {
    if (truth(getitem(s, i))) {
      const r = typeof (t1 = B.range) === "function" ? t1(mul(i, i), add(n, 1), i) : callSlow(t1, [mul(i, i), add(n, 1), i]);
      const m = typeof (t2 = B.len) === "function" ? t2(r) : callSlow(t2, [r]);
      const ba = typeof (t3 = B.bytearray) === "function" ? t3(m) : callSlow(t3, [m]);
      setitem(s, new R.PySlice(mul(i, i), null, i), ba);
    }
    i = iadd(i, 1);
  }
  return typeof (t4 = B.sum) === "function" ? t4(s) : callSlow(t4, [s]);
});

let b_bigint = R.pyfn("b_bigint", function () {
  if (arguments.length !== 0) argcError("b_bigint", 0, arguments.length);
  let a, b, i;
  a = 0;
  b = 1;
  if (B.range === RANGE) {
    for (let k = 0; k < 20000; k++) {
      i = k;
      const t0 = b, t1 = add(a, b);
      a = t0;
      b = t1;
    }
  } else genericLoop();
  return mod(a, 1000000007);
});

// ---------------------------------------------------------------- nbody
let PI = 3.14159265358979323;
let SOLAR_MASS = mul(mul(4, PI), PI);
let DAYS_PER_YEAR = 365.24;
const body = (pos, vel, mass) => tuple([pos, vel, mass]);
let BODIES = R.dictFrom([
  ["sun", body([F0, F0, F0], [F0, F0, F0], SOLAR_MASS)],
  ["jupiter", body(
    [4.8414314424647209, -1.16032004402742839, -1.03622044471123109e-1],
    [mul(1.66007664274403694e-3, DAYS_PER_YEAR), mul(7.69901118419740425e-3, DAYS_PER_YEAR), mul(-6.90460016972063023e-5, DAYS_PER_YEAR)],
    mul(9.54791938424326609e-4, SOLAR_MASS))],
  ["saturn", body(
    [8.34336671824457987, 4.12479856412430479, -4.03523417114321381e-1],
    [mul(-2.76742510726862411e-3, DAYS_PER_YEAR), mul(4.99852801234917238e-3, DAYS_PER_YEAR), mul(2.30417297573763929e-5, DAYS_PER_YEAR)],
    mul(2.85885980666130812e-4, SOLAR_MASS))],
  ["uranus", body(
    [1.2894369562139131e1, -1.51111514016986312e1, -2.23307578892655734e-1],
    [mul(2.96460137564761618e-3, DAYS_PER_YEAR), mul(2.3784717395948095e-3, DAYS_PER_YEAR), mul(-2.96589568540237556e-5, DAYS_PER_YEAR)],
    mul(4.36624404335156298e-5, SOLAR_MASS))],
  ["neptune", body(
    [1.53796971148509165e1, -2.59193146099879641e1, 1.79258772950371181e-1],
    [mul(2.68067772490389322e-3, DAYS_PER_YEAR), mul(1.62824170038242295e-3, DAYS_PER_YEAR), mul(-9.5159225451971587e-5, DAYS_PER_YEAR)],
    mul(5.15138902046611451e-5, SOLAR_MASS))],
]);

const call_append_2 = R.siteCall("append", 1), call_values_0 = R.siteCall("values", 0);
let combinations = R.pyfn("combinations", function (l) {
  if (arguments.length !== 1) argcError("combinations", 1, arguments.length);
  let result, x, ls, t0;
  result = [];
  const n = sub(typeof (t0 = B.len) === "function" ? t0(l) : callSlow(t0, [l]), 1);
  if (B.range === RANGE && typeof n === "number") {
    for (let k = 0; k < n; k++) {
      x = k;
      ls = getitem(l, new R.PySlice(add(x, 1), null, null));
      for (const y of pyiter(ls)) call_append_2(result, tuple([getitem(l, x), y]));
    }
  } else genericLoop();
  return result;
});
let t_;
let SYSTEM = typeof (t_ = B.list) === "function" ? t_(call_values_0(BODIES)) : callSlow(t_, [call_values_0(BODIES)]);
let PAIRS = typeof (t_ = combinations) === "function" ? t_(SYSTEM) : callSlow(t_, [SYSTEM]);

const advance$defaults = [SYSTEM, PAIRS];
let advance = R.pyfn("advance", function (dt, n, bodies, pairs) {
  const na = arguments.length;
  if (na < 2 || na > 4) argcError("advance", 4, na);
  if (na < 3) bodies = advance$defaults[0];
  if (na < 4) pairs = advance$defaults[1];
  let i, x1, y1, z1, v1, m1, x2, y2, z2, v2, m2, dx, dy, dz, mag, b1m, b2m, r, vx, vy, vz, m;
  if (B.range === RANGE && typeof n === "number") {
    for (let k = 0; k < n; k++) {
      i = k;
      for (const item of pyiter(pairs)) {
        const u = unpack(item, 2);
        const u0 = unpack(u[0], 3), u1 = unpack(u[1], 3);
        const p0 = unpack(u0[0], 3), p1 = unpack(u1[0], 3);
        x1 = p0[0]; y1 = p0[1]; z1 = p0[2]; v1 = u0[1]; m1 = u0[2];
        x2 = p1[0]; y2 = p1[1]; z2 = p1[2]; v2 = u1[1]; m2 = u1[2];
        dx = sub(x1, x2);
        dy = sub(y1, y2);
        dz = sub(z1, z2);
        mag = mul(dt, pow(add(add(mul(dx, dx), mul(dy, dy)), mul(dz, dz)), -1.5));
        b1m = mul(m1, mag);
        b2m = mul(m2, mag);
        setitem(v1, 0, isub(getitem(v1, 0), mul(dx, b2m)));
        setitem(v1, 1, isub(getitem(v1, 1), mul(dy, b2m)));
        setitem(v1, 2, isub(getitem(v1, 2), mul(dz, b2m)));
        setitem(v2, 0, iadd(getitem(v2, 0), mul(dx, b1m)));
        setitem(v2, 1, iadd(getitem(v2, 1), mul(dy, b1m)));
        setitem(v2, 2, iadd(getitem(v2, 2), mul(dz, b1m)));
      }
      for (const item of pyiter(bodies)) {
        const u = unpack(item, 3);
        r = u[0];
        const vv = unpack(u[1], 3);
        vx = vv[0]; vy = vv[1]; vz = vv[2]; m = u[2];
        setitem(r, 0, iadd(getitem(r, 0), mul(dt, vx)));
        setitem(r, 1, iadd(getitem(r, 1), mul(dt, vy)));
        setitem(r, 2, iadd(getitem(r, 2), mul(dt, vz)));
      }
    }
  } else genericLoop();
  return null;
});

const report_energy$defaults = [SYSTEM, PAIRS, F0];
let report_energy = R.pyfn("report_energy", function (bodies, pairs, e) {
  const na = arguments.length;
  if (na > 3) argcError("report_energy", 3, na);
  if (na < 1) bodies = report_energy$defaults[0];
  if (na < 2) pairs = report_energy$defaults[1];
  if (na < 3) e = report_energy$defaults[2];
  let x1, y1, z1, v1, m1, x2, y2, z2, v2, m2, dx, dy, dz, r, vx, vy, vz, m;
  for (const item of pyiter(pairs)) {
    const u = unpack(item, 2);
    const u0 = unpack(u[0], 3), u1 = unpack(u[1], 3);
    const p0 = unpack(u0[0], 3), p1 = unpack(u1[0], 3);
    x1 = p0[0]; y1 = p0[1]; z1 = p0[2]; v1 = u0[1]; m1 = u0[2];
    x2 = p1[0]; y2 = p1[1]; z2 = p1[2]; v2 = u1[1]; m2 = u1[2];
    dx = sub(x1, x2);
    dy = sub(y1, y2);
    dz = sub(z1, z2);
    e = isub(e, truediv(mul(m1, m2), pow(add(add(mul(dx, dx), mul(dy, dy)), mul(dz, dz)), 0.5)));
  }
  for (const item of pyiter(bodies)) {
    const u = unpack(item, 3);
    r = u[0];
    const vv = unpack(u[1], 3);
    vx = vv[0]; vy = vv[1]; vz = vv[2]; m = u[2];
    e = iadd(e, truediv(mul(m, add(add(mul(vx, vx), mul(vy, vy)), mul(vz, vz))), F2));
  }
  return e;
});

const offset_momentum$defaults = [SYSTEM, F0, F0, F0];
let offset_momentum = R.pyfn("offset_momentum", function (ref, bodies, px, py, pz) {
  const na = arguments.length;
  if (na < 1 || na > 5) argcError("offset_momentum", 5, na);
  if (na < 2) bodies = offset_momentum$defaults[0];
  if (na < 3) px = offset_momentum$defaults[1];
  if (na < 4) py = offset_momentum$defaults[2];
  if (na < 5) pz = offset_momentum$defaults[3];
  let r, vx, vy, vz, m, v;
  for (const item of pyiter(bodies)) {
    const u = unpack(item, 3);
    r = u[0];
    const vv = unpack(u[1], 3);
    vx = vv[0]; vy = vv[1]; vz = vv[2]; m = u[2];
    px = isub(px, mul(vx, m));
    py = isub(py, mul(vy, m));
    pz = isub(pz, mul(vz, m));
  }
  const u = unpack(ref, 3);
  r = u[0]; v = u[1]; m = u[2];
  setitem(v, 0, truediv(px, m));
  setitem(v, 1, truediv(py, m));
  setitem(v, 2, truediv(pz, m));
  return null;
});

let b_nbody = R.pyfn("b_nbody", function () {
  if (arguments.length !== 0) argcError("b_nbody", 0, arguments.length);
  let t0, t1, t2;
  const ref = getitem(BODIES, "sun");
  typeof (t0 = offset_momentum) === "function" ? t0(ref) : callSlow(t0, [ref]);
  typeof (t1 = advance) === "function" ? t1(0.01, 20000) : callSlow(t1, [0.01, 20000]);
  return typeof (t2 = report_energy) === "function" ? t2() : callSlow(t2, []);
});

// ---------------------------------------------------------------- harness
// Same protocol as bench.py: first (cold, includes JIT warmup) run, then the
// median of five further runs.
function timeit(name, f) {
  let t = performance.now();
  const r = f();
  const cold = performance.now() - t;
  const warm = [];
  for (let i = 0; i < 5; i++) {
    t = performance.now();
    f();
    warm.push(performance.now() - t);
  }
  warm.sort((a, b) => a - b);
  console.log(`${name.padEnd(12)} ${cold.toFixed(1).padStart(9)} ${warm[2].toFixed(1).padStart(9)}  ${R.repr(r)}`);
}
console.log(`${"".padEnd(12)} ${"cold ms".padStart(9)} ${"warm ms".padStart(9)}  result`);
for (const [name, f] of [
  ["fib25", b_fib],
  ["int loop", b_loop],
  ["float loop", b_float],
  ["method call", b_attr],
  ["poly method", b_poly],
  ["listcomp", b_list],
  ["dict str", b_dict],
  ["str join", b_str],
  ["sieve", b_sieve],
  ["bigint fib", b_bigint],
  ["nbody", b_nbody],
]) {
  timeit(name, f);
}
