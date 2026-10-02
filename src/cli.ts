// pyjs [--emit] program.py [args...]

import { readFileSync } from "fs";
import { dirname, resolve } from "path";
import { initParser, compile, execModule, R } from "./compile";

async function main() {
  const argv = process.argv.slice(2);
  const emit = argv[0] === "--emit";
  if (emit) argv.shift();
  const file = argv[0];
  if (file === undefined) {
    process.stderr.write("usage: pyjs [--emit] program.py [args...]\n");
    process.exit(2);
  }
  await initParser();
  const source = readFileSync(file, "utf8");
  if (emit) {
    process.stdout.write(compile(source, file, "__main__").code + "\n");
    return;
  }
  const sys = R.importModule("sys");
  sys.argv.push(...argv);
  sys.path.push(dirname(resolve(file)));
  let code = 0;
  try {
    execModule(source, file, "__main__");
  } catch (e: any) {
    const exc = R.toPyExc(e);
    if (R.typeOf(exc).$mro.includes(R.T.SystemExit)) {
      const c = exc.code;
      if (c === null || c === undefined) code = 0;
      else if (typeof c === "number") code = c;
      else {
        R.stderr.write(R.str(c) + "\n");
        code = 1;
      }
    } else {
      R.stdout.flush();
      R.stderr.write(R.formatException(exc));
      code = 1;
    }
  }
  R.stdout.flush();
  R.stderr.flush();
  process.exitCode = code;
}

main();
