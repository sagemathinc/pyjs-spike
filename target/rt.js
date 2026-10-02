"use strict";
// Spike runtime: just enough to run hand-written "ideal compiler output" for
// bench/bench.py with honest Python semantics on every fast path.
//
// Value representation
//   int   : primitive number that is an integer (always within +-(2^53-1)),
//           or BigInt outside that range (canonical: never a BigInt in range)
//   float : primitive number that is NOT an integer (incl. NaN, +-Infinity),
//           or FloatBox for integral values (incl. -0.0)
//   bool  : JS boolean      None : null      str : JS string
//   list  : JS Array        tuple: JS Array with $t === true
//   dict  : PyDict (Map)    instances of Python classes: objects whose
//           prototype chain holds only $cls (so missing names read undefined)

class PyError extends Error {
  constructor(type, msg) {
    super(`${type}: ${msg}`);
    this.pytype = type;
    this.pymsg = msg;
  }
}
function raise(type, msg) {
  throw new PyError(type, msg);
}

// ---------------------------------------------------------------- numbers
class FloatBox {
  constructor(v) {
    this.v = v;
  }
}
const isInt = Number.isInteger;
const isSafe = Number.isSafeInteger;
const MINB = -9007199254740991n;
const MAXB = 9007199254740991n;
const normBig = (r) => (r >= MINB && r <= MAXB ? Number(r) : r);
const mkfloat = (r) => (isInt(r) ? new FloatBox(r) : r);
const big = (x) => (typeof x === "bigint" ? x : BigInt(x));
function isPyInt(x) {
  const t = typeof x;
  return (t === "number" && isInt(x)) || t === "bigint" || t === "boolean";
}
function fv(x) {
  const t = typeof x;
  if (t === "number") return x;
  if (x instanceof FloatBox) return x.v;
  if (t === "bigint") return Number(x);
  if (t === "boolean") return +x;
  return undefined;
}

function typeName(x) {
  if (x === null) return "NoneType";
  switch (typeof x) {
    case "boolean":
      return "bool";
    case "number":
      return isInt(x) ? "int" : "float";
    case "bigint":
      return "int";
    case "string":
      return "str";
    case "function":
      return x.$name !== undefined ? "type" : "function";
  }
  if (x instanceof FloatBox) return "float";
  if (Array.isArray(x)) return x.$t ? "tuple" : "list";
  if (x instanceof PyDict) return "dict";
  if (x instanceof PyByteArray) return "bytearray";
  if (x instanceof PyRange) return "range";
  if (x instanceof PySlice) return "slice";
  if (x.$cls !== undefined) return x.$cls.$name;
  return "object";
}

const NotImplemented = Object.freeze({ notImplemented: true });

function typeMethod(o, name) {
  if (o === null || typeof o !== "object") return undefined;
  if (o.$cls !== undefined) return lookupType(o.$cls, name);
  const f = o[name];
  return typeof f === "function" ? (self, ...a) => f.apply(self, a) : undefined;
}
function binaryDunder(a, b, op, rop, sym) {
  let f = typeMethod(a, op);
  if (f !== undefined) {
    const r = f(a, b);
    if (r !== NotImplemented) return r;
  }
  f = typeMethod(b, rop);
  if (f !== undefined) {
    const r = f(b, a);
    if (r !== NotImplemented) return r;
  }
  raise("TypeError", `unsupported operand type(s) for ${sym}: '${typeName(a)}' and '${typeName(b)}'`);
}

function add(a, b) {
  if (typeof a === "number" && typeof b === "number") {
    const r = a + b;
    if (isInt(a) && isInt(b)) return isSafe(r) ? r : normBig(BigInt(a) + BigInt(b));
    return isInt(r) ? new FloatBox(r) : r;
  }
  return addSlow(a, b);
}
function addSlow(a, b) {
  if (isPyInt(a) && isPyInt(b)) return normBig(big(a) + big(b));
  const x = fv(a), y = fv(b);
  if (x !== undefined && y !== undefined) return mkfloat(x + y);
  if (typeof a === "string" && typeof b === "string") return a + b;
  if (Array.isArray(a) && Array.isArray(b) && !a.$t === !b.$t) {
    const r = a.concat(b);
    if (a.$t) r.$t = true;
    return r;
  }
  return binaryDunder(a, b, "__add__", "__radd__", "+");
}
function sub(a, b) {
  if (typeof a === "number" && typeof b === "number") {
    const r = a - b;
    if (isInt(a) && isInt(b)) return isSafe(r) ? r : normBig(BigInt(a) - BigInt(b));
    return isInt(r) ? new FloatBox(r) : r;
  }
  return subSlow(a, b);
}
function subSlow(a, b) {
  if (isPyInt(a) && isPyInt(b)) return normBig(big(a) - big(b));
  const x = fv(a), y = fv(b);
  if (x !== undefined && y !== undefined) return mkfloat(x - y);
  return binaryDunder(a, b, "__sub__", "__rsub__", "-");
}
function mul(a, b) {
  if (typeof a === "number" && typeof b === "number") {
    const r = a * b;
    if (isInt(a) && isInt(b)) return isSafe(r) ? r + 0 : normBig(BigInt(a) * BigInt(b));
    return isInt(r) ? new FloatBox(r) : r;
  }
  return mulSlow(a, b);
}
function repeatCount(n) {
  if (!isPyInt(n)) return undefined;
  const k = Number(n);
  return k < 0 ? 0 : k;
}
function mulSlow(a, b) {
  if (isPyInt(a) && isPyInt(b)) return normBig(big(a) * big(b));
  const x = fv(a), y = fv(b);
  if (x !== undefined && y !== undefined) return mkfloat(x * y);
  if (typeof a === "string" && repeatCount(b) !== undefined) return a.repeat(repeatCount(b));
  if (Array.isArray(a) && repeatCount(b) !== undefined) {
    const n = repeatCount(b), r = new Array(a.length * n);
    for (let i = 0; i < n; i++) for (let j = 0; j < a.length; j++) r[i * a.length + j] = a[j];
    if (a.$t) r.$t = true;
    return r;
  }
  if (typeof b === "string" && repeatCount(a) !== undefined) return b.repeat(repeatCount(a));
  return binaryDunder(a, b, "__mul__", "__rmul__", "*");
}
function truediv(a, b) {
  const x = fv(a), y = fv(b);
  if (x !== undefined && y !== undefined) {
    if (y === 0) raise("ZeroDivisionError", "division by zero");
    if (typeof a === "bigint" || typeof b === "bigint") return mkfloat(bigTrueDiv(big(a), big(b)));
    return mkfloat(x / y);
  }
  return binaryDunder(a, b, "__truediv__", "__rtruediv__", "/");
}
function bigTrueDiv(a, b) {
  // Spike: not correctly rounded for huge operands; CPython is.
  return Number(a) / Number(b);
}
function floordiv(a, b) {
  if (typeof a === "number" && typeof b === "number" && isInt(a) && isInt(b)) {
    if (b === 0) raise("ZeroDivisionError", "integer division or modulo by zero");
    const r = a % b; // exact for safe integers, so (a - r) / b is exact too
    const q = (a - r) / b;
    return (r !== 0 && r < 0 !== b < 0 ? q - 1 : q) + 0;
  }
  return floordivSlow(a, b);
}
function floordivSlow(a, b) {
  if (isPyInt(a) && isPyInt(b)) {
    const x = big(a), y = big(b);
    if (y === 0n) raise("ZeroDivisionError", "integer division or modulo by zero");
    let q = x / y;
    if (x % y !== 0n && x < 0n !== y < 0n) q -= 1n;
    return normBig(q);
  }
  const x = fv(a), y = fv(b);
  if (x !== undefined && y !== undefined) {
    if (y === 0) raise("ZeroDivisionError", "float floor division by zero");
    return mkfloat(Math.floor(x / y));
  }
  return binaryDunder(a, b, "__floordiv__", "__rfloordiv__", "//");
}
function mod(a, b) {
  if (typeof a === "number" && typeof b === "number" && isInt(a) && isInt(b)) {
    if (b === 0) raise("ZeroDivisionError", "integer modulo by zero");
    let r = a % b;
    if (r !== 0 && r < 0 !== b < 0) r += b;
    return r + 0;
  }
  return modSlow(a, b);
}
function modSlow(a, b) {
  if (isPyInt(a) && isPyInt(b)) {
    const x = big(a), y = big(b);
    if (y === 0n) raise("ZeroDivisionError", "integer modulo by zero");
    let r = x % y;
    if (r !== 0n && r < 0n !== y < 0n) r += y;
    return normBig(r);
  }
  const x = fv(a), y = fv(b);
  if (x !== undefined && y !== undefined) {
    if (y === 0) raise("ZeroDivisionError", "float modulo");
    let r = x % y;
    if (r !== 0 && r < 0 !== y < 0) r += y;
    else if (r === 0) r = y < 0 ? -0 : 0;
    return mkfloat(r);
  }
  return binaryDunder(a, b, "__mod__", "__rmod__", "%");
}
function pow(a, b) {
  if (isPyInt(a) && isPyInt(b) && fv(b) >= 0) {
    if (typeof a === "number" && typeof b === "number") {
      const r = Math.pow(a, b);
      if (isSafe(r)) return r + 0;
    }
    return normBig(big(a) ** big(b));
  }
  const x = fv(a), y = fv(b);
  if (x !== undefined && y !== undefined) {
    if (x === 0 && y < 0) raise("ZeroDivisionError", "zero to a negative power");
    if (x < 0 && !isInt(y)) raise("NotImplementedError", "spike: complex result of pow");
    return mkfloat(Math.pow(x, y));
  }
  return binaryDunder(a, b, "__pow__", "__rpow__", "** or pow()");
}
// In-place operators: numbers have no __iadd__, so they share the binary fast path.
function iadd(a, b) {
  if (typeof a === "number" && typeof b === "number") return add(a, b);
  if (Array.isArray(a) && !a.$t) {
    for (const v of pyiter(b)) a.push(v);
    return a;
  }
  const f = typeMethod(a, "__iadd__");
  if (f !== undefined) {
    const r = f(a, b);
    if (r !== NotImplemented) return r;
  }
  return add(a, b);
}
function isub(a, b) {
  if (typeof a === "number" && typeof b === "number") return sub(a, b);
  const f = typeMethod(a, "__isub__");
  if (f !== undefined) {
    const r = f(a, b);
    if (r !== NotImplemented) return r;
  }
  return sub(a, b);
}

function lt(a, b) {
  return typeof a === "number" && typeof b === "number" ? a < b : cmpSlow(a, b, "<");
}
function le(a, b) {
  return typeof a === "number" && typeof b === "number" ? a <= b : cmpSlow(a, b, "<=");
}
function cmpSlow(a, b, op) {
  let x = a, y = b;
  if (x instanceof FloatBox) x = x.v;
  if (y instanceof FloatBox) y = y.v;
  const tx = typeof x, ty = typeof y;
  const num = (t) => t === "number" || t === "bigint" || t === "boolean";
  if ((num(tx) && num(ty)) || (tx === "string" && ty === "string")) {
    // Spike: JS compares strings by UTF-16 code unit, Python by code point.
    return op === "<" ? x < y : x <= y;
  }
  const name = op === "<" ? "__lt__" : "__le__";
  const f = typeMethod(a, name);
  if (f !== undefined) {
    const r = f(a, b);
    if (r !== NotImplemented) return r;
  }
  raise("TypeError", `'${op}' not supported between instances of '${typeName(a)}' and '${typeName(b)}'`);
}

function truth(x) {
  if (x === true) return true;
  if (x === false) return false;
  if (typeof x === "number") return x !== 0;
  return truthSlow(x);
}
function truthSlow(x) {
  if (x === null) return false;
  if (typeof x === "string") return x.length !== 0;
  if (typeof x === "bigint") return x !== 0n;
  if (x instanceof FloatBox) return x.v !== 0;
  if (Array.isArray(x)) return x.length !== 0;
  if (x instanceof PyDict) return x.m.size !== 0;
  if (x instanceof PyByteArray) return x.n !== 0;
  let f = typeMethod(x, "__bool__");
  if (f !== undefined) return f(x);
  f = typeMethod(x, "__len__");
  if (f !== undefined) return f(x) !== 0;
  return true;
}

// ---------------------------------------------------------------- classes
let versionCounter = 1;
function pyfn(name, f) {
  f.$pyfn = true;
  f.__name__ = name;
  return f;
}
function makeClass(name, namespace) {
  // Spike: single base `object`, default metaclass.
  const proto = Object.create(null);
  const Ctor = function () {};
  Ctor.prototype = proto;
  const cls = function (...args) {
    return construct(cls, args);
  };
  cls.$name = name;
  cls.$dict = new Map(Object.entries(namespace));
  cls.$ver = versionCounter++;
  cls.$ctor = Ctor;
  cls.$mro = [cls];
  cls.$subclasses = [];
  proto.$cls = cls;
  return cls;
}
function lookupType(cls, name) {
  for (const c of cls.$mro) {
    const v = c.$dict.get(name);
    if (v !== undefined) return v;
  }
  return undefined;
}
function bumpVersion(cls) {
  cls.$ver = versionCounter++;
  for (const s of cls.$subclasses) bumpVersion(s);
}
function setClassAttr(cls, name, value) {
  cls.$dict.set(name, value);
  bumpVersion(cls);
}
function isDataDescriptor(v) {
  return v !== null && typeof v === "object" && typeMethod(v, "__set__") !== undefined;
}
function construct(cls, args) {
  const o = new cls.$ctor();
  const init = lookupType(cls, "__init__");
  if (init !== undefined) {
    const r = init(o, ...args);
    if (r !== null) raise("TypeError", `__init__() should return None, not '${typeName(r)}'`);
  } else if (args.length) {
    raise("TypeError", `${cls.$name}() takes no arguments`);
  }
  return o;
}
function argcError(name, expected, got) {
  raise("TypeError", `${name}() takes ${expected} positional argument${expected === 1 ? "" : "s"} but ${got} ${got === 1 ? "was" : "were"} given`);
}
function bindMethod(f, self) {
  const m = (...a) => f(self, ...a);
  m.$self = self;
  m.$func = f;
  return m;
}
function callSlow(f, args) {
  if (f !== null && typeof f === "object") {
    const c = typeMethod(f, "__call__");
    if (c !== undefined) return c(f, ...args);
  }
  raise("TypeError", `'${typeName(f)}' object is not callable`);
}

// Per-site inline caches.  A real compiler emits these inline; the spike
// generates one small function per site so every site owns its V8 feedback.
const hasOwn = Object.prototype.hasOwnProperty;
function getattrMiss(o, name, S) {
  if (o !== null && typeof o === "object" && o.$cls !== undefined) {
    const cls = o.$cls;
    const cv = lookupType(cls, name);
    if (cv !== undefined && isDataDescriptor(cv)) {
      S.c = null;
      return typeMethod(cv, "__get__")(cv, o, cls);
    }
    if (hasOwn.call(o, name)) {
      S.c = cls;
      S.v = cls.$ver;
      S.k = 0;
      return o[name];
    }
    if (cv !== undefined) {
      if (typeof cv === "function" && cv.$pyfn) return bindMethod(cv, o);
      S.c = cls;
      S.v = cls.$ver;
      S.k = 1;
      S.val = cv;
      return cv;
    }
    const ga = lookupType(cls, "__getattr__");
    if (ga !== undefined) return ga(o, name);
  }
  raise("AttributeError", `'${typeName(o)}' object has no attribute '${name}'`);
}
function setattrMiss(o, name, v, S) {
  if (o !== null && typeof o === "object" && o.$cls !== undefined) {
    const cls = o.$cls;
    const cv = lookupType(cls, name);
    if (cv !== undefined && isDataDescriptor(cv)) {
      typeMethod(cv, "__set__")(cv, o, v);
      return;
    }
    const sa = lookupType(cls, "__setattr__");
    if (sa !== undefined) {
      sa(o, name, v);
      return;
    }
    S.c = cls;
    S.v = cls.$ver;
    o[name] = v;
    return;
  }
  raise("AttributeError", `'${typeName(o)}' object has no attribute '${name}' and no __dict__ for setting new attributes`);
}
// Builtin receiver kinds for method-call caches.
const BK_STR = 1, BK_LIST = 2, BK_DICT = 3;
function builtinKind(o) {
  if (typeof o === "string") return BK_STR;
  if (Array.isArray(o)) return o.$t ? 0 : BK_LIST;
  if (o instanceof PyDict) return BK_DICT;
  return 0;
}
const BUILTIN_METHODS = {
  [BK_STR]: {
    join(self, iterable) {
      const parts = [];
      for (const s of pyiter(iterable)) {
        if (typeof s !== "string") raise("TypeError", `sequence item ${parts.length}: expected str instance, ${typeName(s)} found`);
        parts.push(s);
      }
      return parts.join(self);
    },
    split(self, sep) {
      if (sep === "") raise("ValueError", "empty separator");
      return self.split(sep);
    },
  },
  [BK_LIST]: {
    append(self, x) {
      self.push(x);
      return null;
    },
  },
  [BK_DICT]: {
    values(self) {
      return Array.from(self.m.values()); // Spike: a list, not a live view.
    },
  },
};
function callMethodMiss(o, name, args, S) {
  if (o !== null && typeof o === "object" && o.$cls !== undefined) {
    if (hasOwn.call(o, name)) return callAny(o[name], args);
    const cls = o.$cls;
    const cv = lookupType(cls, name);
    if (typeof cv === "function" && cv.$pyfn) {
      fillCallSite(S, cls, cv);
      return cv(o, ...args);
    }
    return callAny(getattrMiss(o, name, {}), args);
  }
  const k = builtinKind(o);
  const f = k !== 0 ? BUILTIN_METHODS[k][name] : undefined;
  if (f !== undefined) {
    S.bk = k;
    S.bfn = f;
    return f(o, ...args);
  }
  raise("AttributeError", `'${typeName(o)}' object has no attribute '${name}'`);
}
function callAny(f, args) {
  return typeof f === "function" ? f(...args) : callSlow(f, args);
}
function siteGet(name) {
  const S = { c: null, v: -1, k: 0, val: undefined };
  return new Function("S", "miss", `"use strict"; return function get_${name}(o) {
    if (o != null && o.$cls === S.c && S.c.$ver === S.v) {
      if (S.k === 0) { const t = o.${name}; if (t !== undefined) return t; }
      else if (o.${name} === undefined) return S.val;
    }
    return miss(o, ${JSON.stringify(name)}, S);
  };`)(S, getattrMiss);
}
function siteSet(name) {
  const S = { c: null, v: -1 };
  return new Function("S", "miss", `"use strict"; return function set_${name}(o, v) {
    if (o != null && o.$cls === S.c && S.c.$ver === S.v) { o.${name} = v; return; }
    miss(o, ${JSON.stringify(name)}, v, S);
  };`)(S, setattrMiss);
}
// Method-call sites cache up to four receiver classes (polymorphic inline
// cache), filled round-robin, plus one builtin receiver kind.
const CALL_WAYS = 4;
function siteCall(name, n) {
  const S = { next: 0, bk: 0, bfn: null };
  for (let i = 0; i < CALL_WAYS; i++) {
    S["c" + i] = null;
    S["v" + i] = -1;
    S["f" + i] = null;
  }
  const ps = Array.from({ length: n }, (_, i) => "a" + i).join(", ");
  const c = n ? ", " : "";
  const ways = Array.from({ length: CALL_WAYS }, (_, i) =>
    `      if (k === S.c${i} && k.$ver === S.v${i}) return S.f${i}(o${c}${ps});`).join("\n");
  return new Function("S", "miss", "kind", `"use strict"; return function call_${name}(o${c}${ps}) {
    if (o != null) {
      const k = o.$cls;
      if (k !== undefined && o.${name} === undefined) {
${ways}
      }
    }
    if (S.bk !== 0 && kind(o) === S.bk) return S.bfn(o${c}${ps});
    return miss(o, ${JSON.stringify(name)}, [${ps}], S);
  };`)(S, callMethodMiss, builtinKind);
}
function fillCallSite(S, cls, fn) {
  for (let i = 0; i < CALL_WAYS; i++) {
    if (S["c" + i] === cls) {
      S["v" + i] = cls.$ver;
      S["f" + i] = fn;
      return;
    }
  }
  const i = S.next;
  S.next = (i + 1) % CALL_WAYS;
  S["c" + i] = cls;
  S["v" + i] = cls.$ver;
  S["f" + i] = fn;
}

// ---------------------------------------------------------------- containers
class PyDict {
  constructor() {
    this.m = new Map();
  }
}
function dictKey(k) {
  const t = typeof k;
  if (t === "string" || t === "number") return k;
  if (t === "boolean") return +k;
  if (k instanceof FloatBox) return k.v + 0;
  raise("NotImplementedError", `spike: dict keys of type '${typeName(k)}'`);
}
function dictFrom(pairs) {
  const d = new PyDict();
  for (const [k, v] of pairs) d.m.set(dictKey(k), v);
  return d;
}
function tuple(arr) {
  arr.$t = true;
  return arr;
}
class PyByteArray {
  constructor(n) {
    this.a = new Uint8Array(n);
    this.n = n;
  }
  __mul__(k) {
    const n = repeatCount(k);
    if (n === undefined) return NotImplemented;
    const r = new PyByteArray(this.n * n);
    if (r.n === 0) return r;
    r.a.set(this.a.subarray(0, this.n), 0);
    for (let filled = this.n; filled < r.n; filled *= 2) r.a.copyWithin(filled, 0, Math.min(filled, r.n - filled));
    return r;
  }
}
class PyRange {
  constructor(start, stop, step) {
    if (step === 0) raise("ValueError", "range() arg 3 must not be zero");
    this.start = start;
    this.stop = stop;
    this.step = step;
  }
  get length() {
    const { start, stop, step } = this;
    if (step > 0) return start < stop ? Math.floor((stop - start - 1) / step) + 1 : 0;
    return start > stop ? Math.floor((start - stop - 1) / -step) + 1 : 0;
  }
  *[Symbol.iterator]() {
    const n = this.length;
    for (let i = 0, v = this.start; i < n; i++, v += this.step) yield v;
  }
}
class PySlice {
  constructor(start, stop, step) {
    this.start = start;
    this.stop = stop;
    this.step = step;
  }
  // CPython's PySlice_AdjustIndices.
  indices(len) {
    const step = this.step === null ? 1 : this.step;
    if (step === 0) raise("ValueError", "slice step cannot be zero");
    const lo = step < 0 ? -1 : 0, hi = step < 0 ? len - 1 : len;
    const fix = (v, dflt) => {
      if (v === null) return dflt;
      if (v < 0) return Math.max(v + len, lo);
      return Math.min(v, hi);
    };
    const start = fix(this.start, step < 0 ? hi : lo);
    const stop = fix(this.stop, step < 0 ? lo : hi);
    let n = 0;
    if (step > 0 ? start < stop : start > stop) n = Math.floor((stop - start - (step > 0 ? 1 : -1)) / step) + 1;
    return [start, step, n];
  }
}
function normIndex(i, len, what) {
  if (!(typeof i === "number" && isInt(i))) raise("TypeError", `${what} indices must be integers or slices, not ${typeName(i)}`);
  const j = i < 0 ? i + len : i;
  if (j < 0 || j >= len) raise("IndexError", `${what} index out of range`);
  return j;
}
function getitem(o, k) {
  if (Array.isArray(o)) {
    if (typeof k === "number" && k >= 0 && k < o.length && isInt(k)) return o[k];
    if (k instanceof PySlice) {
      const [start, step, n] = k.indices(o.length);
      const r = new Array(n);
      for (let i = 0, j = start; i < n; i++, j += step) r[i] = o[j];
      if (o.$t) r.$t = true;
      return r;
    }
    return o[normIndex(k, o.length, o.$t ? "tuple" : "list")];
  }
  if (o instanceof PyDict) {
    const v = o.m.get(dictKey(k));
    if (v === undefined) raise("KeyError", repr(k));
    return v;
  }
  if (o instanceof PyByteArray) return o.a[normIndex(k, o.n, "bytearray")];
  const f = typeMethod(o, "__getitem__");
  if (f !== undefined) return f(o, k);
  raise("TypeError", `'${typeName(o)}' object is not subscriptable`);
}
function setitem(o, k, v) {
  if (Array.isArray(o) && !o.$t) {
    if (typeof k === "number" && k >= 0 && k < o.length && isInt(k)) {
      o[k] = v;
      return;
    }
    o[normIndex(k, o.length, "list")] = v;
    return;
  }
  if (o instanceof PyDict) {
    o.m.set(dictKey(k), v);
    return;
  }
  if (o instanceof PyByteArray) {
    if (k instanceof PySlice) {
      const [start, step, n] = k.indices(o.n);
      const src = v instanceof PyByteArray ? v : bytearray(v);
      if (step === 1) raise("NotImplementedError", "spike: resizing slice assignment");
      if (src.n !== n) raise("ValueError", `attempt to assign bytes of size ${src.n} to extended slice of size ${n}`);
      for (let i = 0, j = start; i < n; i++, j += step) o.a[j] = src.a[i];
      return;
    }
    if (!(typeof v === "number" && isInt(v) && v >= 0 && v < 256)) raise("ValueError", "byte must be in range(0, 256)");
    o.a[normIndex(k, o.n, "bytearray")] = v;
    return;
  }
  const f = typeMethod(o, "__setitem__");
  if (f !== undefined) {
    f(o, k, v);
    return;
  }
  raise("TypeError", `'${typeName(o)}' object does not support item assignment`);
}
function pyiter(x) {
  if (Array.isArray(x)) return x;
  if (x instanceof PyDict) return x.m.keys();
  if (typeof x === "string") return x; // Spike: iterates code points, as Python does.
  if (x instanceof PyByteArray) return x.a.subarray(0, x.n);
  if (x instanceof PyRange) return x;
  if (x !== null && typeof x === "object" && typeof x[Symbol.iterator] === "function") return x;
  raise("TypeError", `'${typeName(x)}' object is not iterable`);
}
function unpack(x, n) {
  if (Array.isArray(x)) {
    if (x.length === n) return x;
    if (x.length < n) raise("ValueError", `not enough values to unpack (expected ${n}, got ${x.length})`);
    raise("ValueError", `too many values to unpack (expected ${n})`);
  }
  return unpack(Array.from(pyiter(x)), n);
}

// ---------------------------------------------------------------- builtins
function floatRepr(x) {
  if (x !== x) return "nan";
  if (x === Infinity) return "inf";
  if (x === -Infinity) return "-inf";
  if (x === 0) return Object.is(x, -0) ? "-0.0" : "0.0";
  const [mant, exp] = x.toExponential().split("e");
  const e = +exp, neg = x < 0;
  const digits = mant.replace("-", "").replace(".", "");
  let out;
  if (e < -4 || e >= 16) {
    out = digits[0] + (digits.length > 1 ? "." + digits.slice(1) : "") + "e" + (e < 0 ? "-" : "+") + String(Math.abs(e)).padStart(2, "0");
  } else if (e >= 0) {
    out = digits.length <= e + 1 ? digits + "0".repeat(e + 1 - digits.length) + ".0" : digits.slice(0, e + 1) + "." + digits.slice(e + 1);
  } else {
    out = "0." + "0".repeat(-e - 1) + digits;
  }
  return (neg ? "-" : "") + out;
}
function str(x) {
  switch (typeof x) {
    case "string":
      return x;
    case "number":
      return isInt(x) ? String(x) : floatRepr(x);
    case "bigint":
      return x.toString();
    case "boolean":
      return x ? "True" : "False";
  }
  if (x === null) return "None";
  if (x instanceof FloatBox) return floatRepr(x.v);
  return repr(x);
}
function repr(x) {
  if (typeof x === "string") return "'" + x.replace(/\\/g, "\\\\").replace(/'/g, "\\'") + "'";
  if (Array.isArray(x)) {
    const body = x.map(repr).join(", ");
    return x.$t ? (x.length === 1 ? `(${body},)` : `(${body})`) : `[${body}]`;
  }
  if (x !== null && typeof x === "object" && !(x instanceof FloatBox)) return `<${typeName(x)} object>`;
  return str(x);
}
function len(x) {
  if (typeof x === "string") return x.length; // Spike: UTF-16 units; Python counts code points.
  if (Array.isArray(x)) return x.length;
  if (x instanceof PyDict) return x.m.size;
  if (x instanceof PyByteArray) return x.n;
  if (x instanceof PyRange) return x.length;
  const f = typeMethod(x, "__len__");
  if (f !== undefined) return f(x);
  raise("TypeError", `object of type '${typeName(x)}' has no len()`);
}
function sum(iterable, start = 0) {
  let s = start;
  if (Array.isArray(iterable)) {
    for (let i = 0; i < iterable.length; i++) s = add(s, iterable[i]);
    return s;
  }
  if (iterable instanceof PyByteArray) {
    const a = iterable.a;
    if (typeof s === "number" && isInt(s) && isSafe(Math.abs(s) + 255 * iterable.n)) {
      // Bytes are ints < 256, so this plain sum cannot leave the safe range.
      for (let i = 0; i < iterable.n; i++) s += a[i];
      return s;
    }
    for (let i = 0; i < iterable.n; i++) s = add(s, a[i]);
    return s;
  }
  for (const v of pyiter(iterable)) s = add(s, v);
  return s;
}
function range(a, b, c) {
  const n = arguments.length;
  const args = n === 1 ? [0, a, 1] : n === 2 ? [a, b, 1] : [a, b, c];
  for (const v of args) if (!(typeof v === "number" && isInt(v))) raise("TypeError", `'${typeName(v)}' object cannot be interpreted as an integer`);
  return new PyRange(...args);
}
function round(x, nd) {
  const v = fv(x);
  if (arguments.length === 1) raise("NotImplementedError", "spike: round(x)");
  if (!isInt(v)) {
    // Spike: toFixed rounds the exact binary value half-up; CPython rounds it half-even.
    return mkfloat(Number(v.toFixed(nd)));
  }
  return x;
}
function bytearray(arg) {
  if (typeof arg === "number" && isInt(arg)) {
    if (arg < 0) raise("ValueError", "negative count");
    return new PyByteArray(arg);
  }
  const vals = Array.from(pyiter(arg));
  const r = new PyByteArray(vals.length);
  vals.forEach((v, i) => {
    if (!(typeof v === "number" && isInt(v) && v >= 0 && v < 256)) raise("ValueError", "byte must be in range(0, 256)");
    r.a[i] = v;
  });
  return r;
}
function list(x) {
  return Array.from(pyiter(x));
}
const B = { str, len, sum, range, round, bytearray, list, repr };
const RANGE = range;

module.exports = {
  PyError, raise, FloatBox, add, sub, mul, truediv, floordiv, mod, pow, iadd, isub, lt, le, truth,
  pyfn, makeClass, setClassAttr, argcError, callSlow, siteGet, siteSet, siteCall,
  PyDict, dictFrom, tuple, PySlice, getitem, setitem, pyiter, unpack, B, RANGE, repr, typeName,
};
