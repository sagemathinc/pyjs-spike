// The runtime object handed to compiled modules, plus function creation and
// tracebacks.

import * as Obj from "./object";
import * as O from "./ops";
import * as F from "./format";
import * as Ty from "./types";
import * as B from "./builtins";
import * as M from "./modules";
import * as Arr from "./array";

O.arrayHooks.cls = Arr.PyArray;
O.arrayHooks.get = Arr.arrayGetitem;
O.arrayHooks.set = Arr.arraySetitem;

const { T, raise, tuple, bindArgs, typeName } = Obj;

Error.stackTraceLimit = 200;

// ------------------------------------------------------------------ functions

// Create a Python function from a compiled implementation.  Simple
// signatures (positional parameters with optional defaults) call the
// implementation directly; others go through a binding wrapper.
export function defn(impl: any, name: string, qualname: string, module: string, defaults: any[] | null, kwdefaults: Obj.PyDict | null, sig: Obj.Signature, doc: any): any {
  const simple = sig.vararg === null && sig.kwarg === null && sig.kwonly.length === 0;
  let f = impl;
  if (!simple) {
    f = function (...a: any[]) {
      return impl(...bindArgs(f.__name__, sig, a, [], []));
    };
    f.$kw = (pos: any[], names: string[], values: any[]) => impl(...bindArgs(f.__name__, sig, pos, names, values));
    impl.$public = f;
  }
  f.$pyfn = true;
  f.$sig = sig;
  f.__name__ = name;
  f.__qualname__ = qualname;
  f.__module__ = module;
  f.__doc__ = doc;
  f.__defaults__ = defaults === null ? null : tuple(defaults);
  f.__kwdefaults__ = kwdefaults;
  return f;
}

// Default for parameter `i` of `nargs` when the caller omitted it.
export function dflt(f: any, i: number, nargs: number, name: string): any {
  const d = (f.$public ?? f).__defaults__;
  const j = i - (nargs - (d === null ? 0 : d.length));
  if (d === null || j < 0) Obj.missingArg(f.$public ?? f, name);
  return d[j];
}
export function kwdflt(f: any, name: string): any {
  const d = (f.$public ?? f).__kwdefaults__;
  const v = d === null ? undefined : Obj.dictGet(d, name);
  if (v === undefined) Obj.missingArg(f.$public ?? f, name, true);
  return v;
}

// ------------------------------------------------------------------ calls with *args / **kwargs

// Merge a call's keyword pieces: names/values for k=v, and mappings for **m.
export function kwMerge(names: string[], values: any[], mapping: any): void {
  if (mapping instanceof Obj.PyDict) {
    for (const [k, v] of mapping.$m) {
      const key = Obj.dictKeyOf(mapping, k);
      if (typeof key !== "string") raise(T.TypeError, "keywords must be strings");
      if (names.includes(key)) raise(T.TypeError, `got multiple values for keyword argument '${key}'`);
      names.push(key);
      values.push(v);
    }
    return;
  }
  const keys = Ty.lookupDunder(mapping, "keys");
  if (keys === undefined) raise(T.TypeError, `argument after ** must be a mapping, not ${typeName(mapping)}`);
  O.forEach(keys(mapping), (k) => {
    names.push(k);
    values.push(O.getitem(mapping, k));
  });
}

export function callEx(f: any, pos: any[], names: string[], values: any[], maps: any[]): any {
  for (const m of maps) kwMerge(names, values, m);
  return Obj.callKw(f, pos, names, values);
}

export function dictOf(...kv: any[]): Obj.PyDict {
  const d = new Obj.PyDict();
  for (let i = 0; i < kv.length; i += 2) Obj.dictSet(d, kv[i], kv[i + 1]);
  return d;
}

// `yield from x`: a JS iterable delegating to x.
export function yieldFrom(x: any): any {
  if (x !== null && typeof x === "object" && x[Symbol.toStringTag] === "Generator") return x;
  if (Array.isArray(x) && (x as any).$cls === undefined) return x;
  const it = O.iter(x);
  // A Python generator behind __iter__: delegate to it natively.
  if (it instanceof O.GenIter) return it.g;
  return {
    [Symbol.iterator]() {
      return {
        next() {
          const v = it.$next();
          return v === Obj.DONE ? { done: true, value: null } : { done: false, value: v };
        },
      };
    },
  };
}

export function withEnter(m: any): any {
  const f = Ty.lookupDunder(m, "__enter__");
  if (f === undefined) raise(T.TypeError, `'${typeName(m)}' object does not support the context manager protocol`);
  return f(m);
}
export function withExit(m: any): (t: any, v: any, tb: any) => any {
  const f = Ty.lookupDunder(m, "__exit__");
  if (f === undefined) raise(T.TypeError, `'${typeName(m)}' object does not support the context manager protocol (missed __exit__ method)`);
  return (t, v, tb) => f(m, t, v, tb);
}

export function reraise(): any {
  return Obj.newException(T.RuntimeError, ["No active exception to reraise"]);
}

// ------------------------------------------------------------------ classes

export function classDef(name: string, qualname: string, module: string, bases: any[], ns: Map<string, any>, kwNames: string[], kwValues: any[]): any {
  return Ty.makeClass(name, bases, ns, module, qualname, kwNames, kwValues);
}

// ------------------------------------------------------------------ tracebacks

export interface ScriptInfo {
  filename: string;
  lines: string[]; // Python source lines
  lineMap: number[]; // JS line (1-based) -> Python line
}
export const scripts = new Map<string, ScriptInfo>();

interface Frame {
  file: string;
  line: number;
  name: string;
  text: string;
}

function framesOf(e: any): Frame[] {
  const holder = e.$tb;
  if (holder === undefined) return [];
  const prev = (Error as any).prepareStackTrace;
  (Error as any).prepareStackTrace = (_: any, cs: any[]) => cs;
  let sites: any;
  try {
    sites = holder.stack;
  } finally {
    (Error as any).prepareStackTrace = prev;
  }
  if (!Array.isArray(sites)) return [];
  const frames: Frame[] = [];
  for (const cs of sites) {
    const info = scripts.get(cs.getFileName?.() ?? "");
    if (info === undefined) continue;
    const pyLine = info.lineMap[cs.getLineNumber()] ?? 0;
    let name = String(cs.getFunctionName() ?? "");
    name = name.endsWith("$$") ? name.slice(0, -2) : name === "module$$" || name === "" ? "<module>" : name;
    if (name === "module") name = "<module>";
    frames.push({ file: info.filename, line: pyLine, name, text: (info.lines[pyLine - 1] ?? "").trim() });
  }
  return frames.reverse();
}

function formatOne(e: any): string {
  let out = "";
  const frames = framesOf(e);
  if (frames.length) {
    out += "Traceback (most recent call last):\n";
    for (const f of frames) {
      out += `  File "${f.file}", line ${f.line}, in ${f.name}\n`;
      if (f.text) out += `    ${f.text}\n`;
    }
  }
  const t = Obj.typeOf(e);
  let msg: string;
  try {
    msg = F.str(e);
  } catch {
    msg = "<exception str() failed>";
  }
  const mod = t.$module === "builtins" || t.$module === "__main__" ? "" : t.$module + ".";
  out += `${mod}${t.$qualname}${msg ? ": " + msg : ""}\n`;
  return out;
}

export function formatException(e: any): string {
  const parts: string[] = [];
  const seen = new Set<any>();
  let cur = e;
  while (cur !== null && cur !== undefined && !seen.has(cur)) {
    seen.add(cur);
    parts.unshift(formatOne(cur));
    if (cur.__cause__ !== null && cur.__cause__ !== undefined) {
      parts.unshift("\nThe above exception was the direct cause of the following exception:\n\n");
      cur = cur.__cause__;
    } else if (cur.__context__ !== null && cur.__context__ !== undefined && cur.__suppress_context__ !== true) {
      parts.unshift("\nDuring handling of the above exception, another exception occurred:\n\n");
      cur = cur.__context__;
    } else break;
  }
  return parts.join("");
}

// ------------------------------------------------------------------ the runtime object

export const R: any = {
  ...O,
  ...F,
  T,
  FloatBox: Obj.FloatBox,
  PyDict: Obj.PyDict,
  DONE: Obj.DONE,
  NotImplemented: Obj.NotImplemented,
  Ellipsis: Obj.Ellipsis,
  tuple,
  raise,
  raiseExc: Obj.raiseExc,
  toPyExc: Obj.toPyExc,
  excMatch: Obj.excMatch,
  getattr: Obj.getattr,
  setattr: Obj.setattr,
  delattr: Obj.delattr,
  siteGet: Obj.siteGet,
  siteSet: Obj.siteSet,
  siteCall: Obj.siteCall,
  callObj: Obj.callObj,
  callKw: Obj.callKw,
  tooManyArgs: Obj.tooManyArgs,
  missingArg: Obj.missingArg,
  dictSet: Obj.dictSet,
  newDict: Obj.newDict,
  typeOf: Obj.typeOf,
  superOf: Ty.superOf,
  makeRange: Ty.makeRange,
  PyRange: Ty.PyRange,
  defn,
  callEx,
  dictOf,
  yieldFrom,
  withEnter,
  withExit,
  reraise,
  PyBytes: Obj.PyBytes,
  dflt,
  kwdflt,
  kwMerge,
  classDef,
  builtins: B.builtins,
  gname: B.gname,
  unboundLocal: B.unboundLocal,
  unboundFree: B.unboundFree,
  stdout: B.stdout,
  stderr: B.stderr,
  importModule: M.importModule,
  importTop: M.importTop,
  importFrom: M.importFrom,
  importStar: M.importStar,
  resolveRelative: M.resolveRelative,
  sysModules: M.sysModules,
  loader: M.loader,
  newModule: Ty.newModule,
  formatException,
  scripts,
};
