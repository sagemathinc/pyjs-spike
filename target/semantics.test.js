"use strict";
// The fast paths must not change Python semantics.  Each check is the CPython
// behavior; run with `node --test target/`.
const test = require("node:test");
const assert = require("node:assert/strict");
const R = require("./rt.js");
const { add, sub, mul, truediv, floordiv, mod, pow, lt, truth, repr, B } = R;

const raises = (f, type, msg) =>
  assert.throws(f, (e) => e instanceof R.PyError && e.pytype === type && (msg === undefined || e.pymsg === msg));

test("ints overflow to BigInt and return to numbers", () => {
  const max = 2 ** 53 - 1;
  assert.equal(add(max, 1), 9007199254740992n);
  assert.equal(sub(add(max, 1), 1), max);
  assert.equal(typeof sub(add(max, 1), 1), "number");
  assert.equal(mul(3037000500, 3037000500), 9223372037000250000n);
  assert.equal(repr(pow(2, 100)), "1267650600228229401496703205376");
  assert.equal(lt(2n ** 60n, 1e300), true);
});

test("ints never become -0, floats keep their identity", () => {
  assert.ok(Object.is(mul(0, -5), 0));
  assert.equal(repr(add(1.5, 0.5)), "2.0");
  assert.equal(repr(truediv(4, 2)), "2.0");
  assert.equal(repr(mul(2, 0.5)), "1.0");
  assert.equal(repr(mul(-0.5, 0)), "-0.0");
  assert.equal(repr(add(0.1, 0.2)), "0.30000000000000004");
  assert.equal(repr(add(new R.FloatBox(1e16), 0.5)), "1e+16"); // 1e16 + 0.5 rounds to 1e16
  assert.equal(repr(mul(1e-5, 1.5)), "1.5000000000000002e-05");
  assert.equal(repr(mul(1.5e-5, 1)), "1.5e-05");
  assert.equal(R.typeName(add(1.5, 0.5)), "float");
  assert.equal(R.typeName(add(1, 1)), "int");
});

test("floor division and modulo follow Python signs", () => {
  assert.equal(floordiv(-7, 2), -4);
  assert.equal(mod(-7, 2), 1);
  assert.equal(mod(7, -2), -1);
  assert.equal(floordiv(2n ** 70n, -3), -393530540239137101142n);
  assert.equal(repr(mod(-7.5, 2)), "0.5");
  raises(() => mod(1, 0), "ZeroDivisionError", "integer modulo by zero");
  raises(() => truediv(1, 0), "ZeroDivisionError", "division by zero");
});

test("truthiness", () => {
  assert.equal(truth(NaN), true);
  assert.equal(truth(new R.FloatBox(0)), false);
  assert.equal(truth([]), false);
  assert.equal(truth(""), false);
  assert.equal(truth(0n), false);
  assert.equal(truth(null), false);
});

test("errors carry CPython's wording", () => {
  raises(() => add(1, "a"), "TypeError", "unsupported operand type(s) for +: 'int' and 'str'");
  raises(() => R.siteGet("bar")(3), "AttributeError", "'int' object has no attribute 'bar'");
  const f = R.pyfn("f", function (x) {
    if (arguments.length !== 1) R.argcError("f", 1, arguments.length);
    return x;
  });
  raises(() => f(1, 2), "TypeError", "f() takes 1 positional argument but 2 were given");
  raises(() => R.getitem([1, 2], 5), "IndexError", "list index out of range");
  raises(() => R.getitem(new R.PyDict(), "k"), "KeyError", "'k'");
});

function counterClass() {
  const set_n = R.siteSet("n"), get_n = R.siteGet("n");
  return R.makeClass("C", {
    __init__: R.pyfn("__init__", (self) => (set_n(self, 0), null)),
    bump: R.pyfn("bump", (self) => (set_n(self, add(get_n(self), 1)), get_n(self))),
  });
}

test("class mutation invalidates method caches", () => {
  const C = counterClass();
  const o = C();
  const call = R.siteCall("bump", 0);
  assert.equal(call(o), 1);
  assert.equal(call(o), 2);
  R.setClassAttr(C, "bump", R.pyfn("bump", () => "patched"));
  assert.equal(call(o), "patched");
});

test("an instance attribute shadows a method", () => {
  const C = counterClass();
  const o = C();
  const call = R.siteCall("bump", 0);
  assert.equal(call(o), 1);
  o.bump = () => "instance"; // what `o.bump = lambda: "instance"` compiles to
  assert.equal(call(o), "instance");
});

test("a data descriptor on the class beats the instance dict", () => {
  const Prop = R.makeClass("Prop", {
    __get__: R.pyfn("__get__", () => "from descriptor"),
    __set__: R.pyfn("__set__", () => null),
  });
  const C = counterClass();
  const o = C();
  o.x = "from instance";
  const get = R.siteGet("x");
  assert.equal(get(o), "from instance");
  assert.equal(get(o), "from instance");
  R.setClassAttr(C, "x", Prop());
  assert.equal(get(o), "from descriptor");
});

test("megamorphic call sites stay correct", () => {
  const classes = Array.from({ length: 7 }, (_, i) =>
    R.makeClass("K" + i, { who: R.pyfn("who", () => i) })
  );
  const call = R.siteCall("who", 0);
  for (let round = 0; round < 3; round++) {
    classes.forEach((K, i) => assert.equal(call(K()), i));
  }
});

test("dict keys: 1, 1.0 and True are the same key", () => {
  const d = new R.PyDict();
  R.setitem(d, 1, "int");
  R.setitem(d, new R.FloatBox(1), "float");
  R.setitem(d, true, "bool");
  assert.equal(B.len(d), 1);
  assert.equal(R.getitem(d, 1), "bool");
});

test("slices follow CPython's index adjustment", () => {
  const l = [0, 1, 2, 3, 4, 5];
  assert.equal(repr(R.getitem(l, new R.PySlice(null, null, -2))), "[5, 3, 1]");
  assert.equal(repr(R.getitem(l, new R.PySlice(-2, null, null))), "[4, 5]");
  assert.equal(repr(R.getitem(l, new R.PySlice(10, 2, -1))), "[5, 4, 3]");
});
