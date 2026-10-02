// repr(), str(), format() and `str % args`.

import { T, FloatBox, PyDict, typeOf, lookupType, raise, isType, dictKeyOf, dictGet, hooks, Ellipsis, NotImplemented, PyBytes } from "./object";
import { PySet, setItems, isPyInt, fv, strFormatOpHook, id, index, normBig, PySlice } from "./ops";

const isInt = Number.isInteger;

// ------------------------------------------------------------------ floats

export function floatRepr(x: number): string {
  if (x !== x) return "nan";
  if (x === Infinity) return "inf";
  if (x === -Infinity) return "-inf";
  if (x === 0) return Object.is(x, -0) ? "-0.0" : "0.0";
  const [mant, exp] = x.toExponential().split("e");
  const e = +exp;
  const neg = x < 0;
  const digits = mant.replace("-", "").replace(".", "");
  let out: string;
  if (e < -4 || e >= 16) {
    out = digits[0] + (digits.length > 1 ? "." + digits.slice(1) : "") + "e" + (e < 0 ? "-" : "+") + String(Math.abs(e)).padStart(2, "0");
  } else if (e >= 0) {
    out = digits.length <= e + 1 ? digits + "0".repeat(e + 1 - digits.length) + ".0" : digits.slice(0, e + 1) + "." + digits.slice(e + 1);
  } else {
    out = "0." + "0".repeat(-e - 1) + digits;
  }
  return (neg ? "-" : "") + out;
}

// Exact binary value of a finite double as mant * 2^exp.
function decompose(x: number): [bigint, number] {
  const buf = new DataView(new ArrayBuffer(8));
  buf.setFloat64(0, Math.abs(x));
  const hi = buf.getUint32(0), lo = buf.getUint32(4);
  const e = (hi >>> 20) & 0x7ff;
  const m = (BigInt(hi & 0xfffff) << 32n) | BigInt(lo);
  if (e === 0) return [m, -1074];
  return [m | (1n << 52n), e - 1075];
}

// |x| rounded half-even to `p` digits after the point: [integer digits, fraction digits].
function fixedDigits(x: number, p: number): [string, string] {
  const [m, e] = decompose(x);
  let q: bigint;
  if (e >= 0) q = (m << BigInt(e)) * 10n ** BigInt(p);
  else {
    const N = m * 10n ** BigInt(p);
    const D = 1n << BigInt(-e);
    q = N / D;
    const r2 = (N % D) * 2n;
    if (r2 > D || (r2 === D && q % 2n === 1n)) q += 1n;
  }
  const s = q.toString().padStart(p + 1, "0");
  return [s.slice(0, s.length - p), s.slice(s.length - p)];
}

export function formatFixed(x: number, p: number, alt = false): string {
  const [i, f] = fixedDigits(x, p);
  return (x < 0 || Object.is(x, -0) ? "-" : "") + i + (p > 0 || alt ? "." + f : "");
}

// [digits (p+1 significant), decimal exponent]
function sciDigits(x: number, p: number): [string, number] {
  if (x === 0) return ["0".repeat(p + 1), 0];
  let k = Math.floor(Math.log10(Math.abs(x)));
  for (;;) {
    let digits: string;
    if (p - k >= 0) {
      const [i, f] = fixedDigits(x, p - k);
      digits = (i + f).replace(/^0+/, "");
    } else {
      // Round to a multiple of 10^(k-p).
      const [m, e] = decompose(x);
      const D = 10n ** BigInt(k - p);
      const N = e >= 0 ? m << BigInt(e) : m;
      const den = e >= 0 ? D : D << BigInt(-e);
      let q = N / den;
      const r2 = (N % den) * 2n;
      if (r2 > den || (r2 === den && q % 2n === 1n)) q += 1n;
      digits = q.toString();
    }
    if (digits.length === p + 1) return [digits, k];
    k += digits.length > p + 1 ? 1 : -1;
  }
}

export function formatExp(x: number, p: number, upper = false, alt = false): string {
  const [d, k] = sciDigits(x, p);
  const sign = x < 0 || Object.is(x, -0) ? "-" : "";
  const m = d[0] + (p > 0 || alt ? "." + d.slice(1) : "");
  const e = (k < 0 ? "-" : "+") + String(Math.abs(k)).padStart(2, "0");
  return sign + m + (upper ? "E" : "e") + e;
}

function formatGeneral(x: number, p: number, upper: boolean, alt: boolean, noneType: boolean): string {
  if (p === 0) p = 1;
  const [, k] = sciDigits(x, p - 1);
  let s: string;
  if (-4 <= k && k < p) {
    s = formatFixed(x, p - 1 - k, alt);
    if (!alt && s.includes(".")) s = s.replace(/0+$/, "").replace(/\.$/, noneType ? ".0" : "");
  } else {
    s = formatExp(x, p - 1, upper, alt);
    if (!alt) s = s.replace(/\.?0+(?=[eE])/, "");
  }
  return s;
}

// ------------------------------------------------------------------ repr / str

const NONPRINT = /[\p{Cc}\p{Cf}\p{Cs}\p{Co}\p{Cn}\p{Zl}\p{Zp}\p{Zs}]/u;

export function strRepr(s: string): string {
  const q = s.includes("'") && !s.includes('"') ? '"' : "'";
  let out = q;
  for (const ch of s) {
    const c = ch.codePointAt(0)!;
    if (ch === q || ch === "\\") out += "\\" + ch;
    else if (ch === "\n") out += "\\n";
    else if (ch === "\r") out += "\\r";
    else if (ch === "\t") out += "\\t";
    else if (c < 0x20 || c === 0x7f) out += "\\x" + c.toString(16).padStart(2, "0");
    else if (c < 0x7f || ch === " ") out += ch;
    else if (NONPRINT.test(ch)) {
      if (c <= 0xff) out += "\\x" + c.toString(16).padStart(2, "0");
      else if (c <= 0xffff) out += "\\u" + c.toString(16).padStart(4, "0");
      else out += "\\U" + c.toString(16).padStart(8, "0");
    } else out += ch;
  }
  return out + q;
}

export function bytesRepr(a: ArrayLike<number>): string {
  let hasSingle = false, hasDouble = false;
  for (let i = 0; i < a.length; i++) {
    if (a[i] === 39) hasSingle = true;
    if (a[i] === 34) hasDouble = true;
  }
  const q = hasSingle && !hasDouble ? '"' : "'";
  let out = "b" + q;
  for (let i = 0; i < a.length; i++) {
    const c = a[i];
    if (c === q.charCodeAt(0) || c === 92) out += "\\" + String.fromCharCode(c);
    else if (c === 10) out += "\\n";
    else if (c === 13) out += "\\r";
    else if (c === 9) out += "\\t";
    else if (c < 0x20 || c >= 0x7f) out += "\\x" + c.toString(16).padStart(2, "0");
    else out += String.fromCharCode(c);
  }
  return out + q;
}

export const hex = (n: number) => "0x" + n.toString(16);

const reprActive = new Set<any>();

export function repr(x: any): string {
  switch (typeof x) {
    case "string":
      return strRepr(x);
    case "number":
      return isInt(x) ? String(x) : floatRepr(x);
    case "bigint":
      return x.toString();
    case "boolean":
      return x ? "True" : "False";
    case "function":
      if (isType(x)) return `<class '${x.$module === "builtins" ? "" : x.$module + "."}${x.$qualname}'>`;
      if (x.$self !== undefined) return `<bound method ${x.$func.__qualname__ ?? x.$func.__name__} of ${repr(x.$self)}>`;
      if (x.$pyfn === true && x.$builtinMethod !== true) return `<function ${x.__qualname__} at ${hex(id(x))}>`;
      return `<built-in function ${x.__name__ ?? x.name}>`;
  }
  if (x === null) return "None";
  if (x instanceof FloatBox) return floatRepr(x.v);
  if (x instanceof PyBytes) {
    const r = bytesRepr(x.a.subarray(0, x.n));
    return typeOf(x) === T.bytearray ? `bytearray(${r})` : r;
  }
  if (x === Ellipsis) return "Ellipsis";
  if (x === NotImplemented) return "NotImplemented";
  if (Array.isArray(x) && (x as any).$cls === undefined) return seqRepr(x);
  const t = typeOf(x);
  const f = lookupType(t, "__repr__");
  if (f !== undefined) {
    const r = f(x);
    if (typeof r !== "string") raise(T.TypeError, `__repr__ returned non-string (type ${typeOf(r).$name})`);
    return r;
  }
  return defaultRepr(x);
}

export function seqRepr(x: any[]): string {
  {
    if (reprActive.has(x)) return (x as any).$t ? "(...)" : "[...]";
    reprActive.add(x);
    try {
      const body = x.map(repr).join(", ");
      return (x as any).$t ? (x.length === 1 ? `(${body},)` : `(${body})`) : `[${body}]`;
    } finally {
      reprActive.delete(x);
    }
  }
}

export function defaultRepr(x: any): string {
  const t = typeOf(x);
  return `<${t.$module === "builtins" ? "" : t.$module + "."}${t.$qualname} object at ${hex(id(x))}>`;
}

export function dictRepr(d: PyDict): string {
  if (reprActive.has(d)) return "{...}";
  reprActive.add(d);
  try {
    const parts: string[] = [];
    for (const [k, v] of d.$m) parts.push(repr(dictKeyOf(d, k)) + ": " + repr(v));
    return "{" + parts.join(", ") + "}";
  } finally {
    reprActive.delete(d);
  }
}

export function setRepr(s: PySet): string {
  const items = setItems(s);
  const name = s.$frozen ? "frozenset" : "set";
  if (items.length === 0) return name + "()";
  const body = "{" + items.map(repr).join(", ") + "}";
  return s.$frozen ? `frozenset(${body})` : body;
}

export function str(x: any): string {
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
  const t = typeOf(x);
  const f = lookupType(t, "__str__");
  if (f !== undefined) {
    const r = f(x);
    if (typeof r !== "string") raise(T.TypeError, `__str__ returned non-string (type ${typeOf(r).$name})`);
    return r;
  }
  return repr(x);
}

hooks.repr = repr;

// ------------------------------------------------------------------ format()

interface Spec {
  fill: string;
  align: string;
  sign: string;
  z: boolean;
  alt: boolean;
  zero: boolean;
  width: number;
  grouping: string;
  precision: number;
  type: string;
}

function parseSpec(spec: string): Spec {
  const m = /^(?:(.)?([<>=^]))?([-+ ])?(z)?(#)?(0)?(\d+)?([,_])?(?:\.(\d+))?([bcdeEfFgGnosxX%])?$/su.exec(spec);
  if (m === null) raise(T.ValueError, "Invalid format specifier '" + spec + "'");
  return {
    fill: m[1] ?? (m[6] && !m[2] ? "0" : " "),
    align: m[2] ?? (m[6] ? "=" : ""),
    sign: m[3] ?? "-",
    z: !!m[4],
    alt: !!m[5],
    zero: !!m[6],
    width: m[7] ? parseInt(m[7]) : 0,
    grouping: m[8] ?? "",
    precision: m[9] !== undefined ? parseInt(m[9]) : -1,
    type: m[10] ?? "",
  };
}

function group(digits: string, sep: string, n: number): string {
  if (!sep) return digits;
  let out = "";
  for (let i = digits.length; i > 0; i -= n) out = digits.slice(Math.max(0, i - n), i) + (out ? sep + out : "");
  return out;
}

function pad(body: string, sign: string, s: Spec, defaultAlign: string): string {
  const align = s.align || defaultAlign;
  const len = [...body].length + sign.length;
  if (len >= s.width) return sign + body;
  const fill = s.fill.repeat(s.width - len);
  switch (align) {
    case "<":
      return sign + body + fill;
    case "^": {
      const half = Math.floor((s.width - len) / 2);
      return s.fill.repeat(half) + sign + body + s.fill.repeat(s.width - len - half);
    }
    case "=":
      return sign + fill + body;
    default:
      return fill + sign + body;
  }
}

function signOf(neg: boolean, s: Spec): string {
  return neg ? "-" : s.sign === "+" ? "+" : s.sign === " " ? " " : "";
}

function formatInt(v: bigint, s: Spec): string {
  const neg = v < 0n;
  const a = neg ? -v : v;
  let digits: string, prefix = "";
  switch (s.type) {
    case "b": digits = a.toString(2); prefix = "0b"; break;
    case "o": digits = a.toString(8); prefix = "0o"; break;
    case "x": digits = a.toString(16); prefix = "0x"; break;
    case "X": digits = a.toString(16).toUpperCase(); prefix = "0X"; break;
    case "c":
      return pad(String.fromCodePoint(Number(v)), "", s, "<");
    case "":
    case "d":
    case "n":
      digits = a.toString();
      break;
    default:
      return formatFloat(Number(v), s);
  }
  if (s.precision >= 0) raise(T.ValueError, "Precision not allowed in integer format specifier");
  digits = group(digits, s.grouping, s.type === "" || s.type === "d" || s.type === "n" ? 3 : 4);
  const sign = signOf(neg, s) + (s.alt ? prefix : "");
  return pad(digits, sign, s, ">");
}

function formatFloat(x: number, s: Spec): string {
  const neg = x < 0 || Object.is(x, -0);
  let a = Math.abs(x);
  let body: string;
  const p = s.precision;
  if (!Number.isFinite(a)) body = a !== a ? "nan" : "inf";
  else {
    switch (s.type) {
      case "f":
      case "F":
        body = formatFixed(a, p < 0 ? 6 : p, s.alt);
        break;
      case "e":
      case "E":
        body = formatExp(a, p < 0 ? 6 : p, s.type === "E", s.alt);
        break;
      case "g":
      case "G":
      case "n":
        body = formatGeneral(a, p < 0 ? 6 : p, s.type === "G", s.alt, false);
        break;
      case "%":
        body = formatFixed(a * 100, p < 0 ? 6 : p, s.alt) + "%";
        break;
      case "":
        body = p < 0 ? floatRepr(a) : formatGeneral(a, p, false, s.alt, true);
        break;
      default:
        raise(T.ValueError, `Unknown format code '${s.type}' for object of type 'float'`);
    }
    if (s.type === "F" || s.type === "E" || s.type === "G") body = body.toUpperCase();
  }
  if (s.grouping) {
    const [ip, fp] = body.split(/(?=[.eE%])/, 2);
    body = group(ip, s.grouping, 3) + body.slice(ip.length);
    void fp;
  }
  const negZero = s.z && neg && Number(body.replace(/[^0-9.e]/g, "")) === 0;
  return pad(body, signOf(neg && !negZero, s), s, ">");
}

export function format(v: any, spec: string): string {
  if (typeof v === "string") {
    if (spec === "") return v;
    const s = parseSpec(spec);
    if (s.type !== "" && s.type !== "s") raise(T.ValueError, `Unknown format code '${s.type}' for object of type 'str'`);
    if (s.sign !== "-" && spec.match(/[-+ ]/)) raise(T.ValueError, "Sign not allowed in string format specifier");
    return pad(s.precision >= 0 ? [...v].slice(0, s.precision).join("") : v, "", s, "<");
  }
  if (typeof v === "boolean" && spec === "") return v ? "True" : "False";
  if (isPyInt(v)) {
    if (spec === "") return String(+(v as any) === Number(v) && typeof v !== "bigint" ? Number(v) : v);
    return formatInt(typeof v === "bigint" ? v : BigInt(+v), parseSpec(spec));
  }
  if (typeof v === "number" || v instanceof FloatBox) {
    const x = fv(v)!;
    if (spec === "") return floatRepr(x);
    return formatFloat(x, parseSpec(spec));
  }
  const f = lookupType(typeOf(v), "__format__");
  if (f !== undefined) {
    const r = f(v, spec);
    if (typeof r !== "string") raise(T.TypeError, "__format__ must return a str");
    return r;
  }
  if (spec !== "") raise(T.TypeError, `unsupported format string passed to ${typeOf(v).$name}.__format__`);
  return str(v);
}

// f"{x!r:>10}"
export function fmt(v: any, conv: string | null, spec: string): string {
  if (conv === "r") v = repr(v);
  else if (conv === "s") v = str(v);
  else if (conv === "a") v = repr(v).replace(/[^\x00-\x7f]/gu, (c) => {
    const n = c.codePointAt(0)!;
    return n <= 0xff ? "\\x" + n.toString(16).padStart(2, "0") : n <= 0xffff ? "\\u" + n.toString(16).padStart(4, "0") : "\\U" + n.toString(16).padStart(8, "0");
  });
  else if (spec === "" && typeof v === "string") return v;
  return format(v, spec);
}

// ------------------------------------------------------------------ str % args

export function percentFormat(fmtStr: string, args: any): string {
  const isMap = args instanceof PyDict || (args !== null && typeof args === "object" && !Array.isArray(args) && lookupType(typeOf(args), "__getitem__") !== undefined && !(typeof args === "string"));
  const items: any[] = Array.isArray(args) && (args as any).$t === true ? args : [args];
  let ai = 0;
  const nextArg = () => {
    if (ai >= items.length) raise(T.TypeError, "not enough arguments for format string");
    return items[ai++];
  };
  let out = "";
  const re = /%(?:\(([^)]*)\))?([-+ #0]*)(\*|\d+)?(?:\.(\*|\d+))?[hlL]?([diouxXeEfFgGcrsa%])/gy;
  let last = 0;
  for (let i = fmtStr.indexOf("%"); i >= 0; i = fmtStr.indexOf("%", last)) {
    out += fmtStr.slice(last, i);
    re.lastIndex = i;
    const m = re.exec(fmtStr);
    if (m === null) raise(T.ValueError, `unsupported format character '${fmtStr[i + 1] ?? ""}' (0x${(fmtStr.charCodeAt(i + 1) || 0).toString(16)}) at index ${i + 1}`);
    last = re.lastIndex;
    const [, key, flags, w, prec, type] = m;
    if (type === "%") {
      out += "%";
      continue;
    }
    let v: any;
    let width = w === "*" ? Number(nextArg()) : w ? parseInt(w) : 0;
    const precision = prec === "*" ? Number(nextArg()) : prec !== undefined ? parseInt(prec) : -1;
    if (key !== undefined) {
      if (!isMap) raise(T.TypeError, "format requires a mapping");
      v = getitemMapping(args, key);
    } else v = nextArg();
    const left = flags.includes("-");
    const s: Spec = { fill: flags.includes("0") && !left && !"sraTc".includes(type) ? "0" : " ", align: left ? "<" : flags.includes("0") && !"srac".includes(type) ? "=" : ">", sign: flags.includes("+") ? "+" : flags.includes(" ") ? " " : "-", z: false, alt: flags.includes("#"), zero: false, width, grouping: "", precision, type: "" };
    let body: string;
    switch (type) {
      case "s": body = str(v); if (precision >= 0) body = body.slice(0, precision); body = pad(body, "", s, ">"); break;
      case "r": case "a": body = repr(v); if (precision >= 0) body = body.slice(0, precision); body = pad(body, "", s, ">"); break;
      case "c": body = pad(typeof v === "string" ? v : String.fromCodePoint(Number(v)), "", s, ">"); break;
      case "d": case "i": case "u": {
        if (!isPyInt(v) && fv(v) === undefined) raise(T.TypeError, `%${type} format: a real number is required, not ${typeOf(v).$name}`);
        const n = isPyInt(v) ? BigInt(typeof v === "boolean" ? +v : v) : BigInt(Math.trunc(fv(v)!));
        s.type = "d";
        body = formatInt(n, s);
        break;
      }
      case "o": case "x": case "X": {
        if (!isPyInt(v)) raise(T.TypeError, `%${type} format: an integer is required, not ${typeOf(v).$name}`);
        s.type = type;
        body = formatInt(BigInt(typeof v === "boolean" ? +v : v), s);
        break;
      }
      default: {
        const x = fv(v);
        if (x === undefined) raise(T.TypeError, `must be real number, not ${typeOf(v).$name}`);
        s.type = type;
        if (precision < 0) s.precision = 6;
        body = formatFloat(x, s);
      }
    }
    out += body;
  }
  out += fmtStr.slice(last);
  if (!isMap && ai < items.length) raise(T.TypeError, "not all arguments converted during string formatting");
  return out;
}

function getitemMapping(m: any, k: string): any {
  if (m instanceof PyDict) {
    const v = dictGet(m, k);
    if (v === undefined) throw T.KeyError(k);
    return v;
  }
  return lookupType(typeOf(m), "__getitem__")(m, k);
}

strFormatOpHook.f = percentFormat;

export { index, normBig, PySlice };
