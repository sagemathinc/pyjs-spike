// sagebrush.modsym from Node via WebAssembly: node run_wasm.cjs N q
const { hecke_charpoly } = require("../wasm/pkg/sagebrush_wasm.js");
const [n, q] = process.argv.slice(2).map(Number);
const t = performance.now();
const r = JSON.parse(hecke_charpoly(n, q, 67108859));
r.ms = [performance.now() - t];
console.log(`N=${n} q=${q} dim=${r.dim} eisenstein_root=${r.eisenstein_root ? "True" : "False"} charpoly_hash=${r.hash}`);
console.log(`times: total ${r.ms.reduce((a, b) => a + b, 0).toFixed(0)} ms`);
