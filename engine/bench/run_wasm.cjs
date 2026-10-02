// modsym-engine from Node via WebAssembly: node run_wasm.cjs N q
const { hecke_charpoly } = require("../wasm/pkg/modsym_wasm.js");
const [n, q] = process.argv.slice(2).map(Number);
const r = JSON.parse(hecke_charpoly(n, q, 67108859));
console.log(`N=${n} q=${q} dim=${r.dim} eisenstein_root=${r.eisenstein_root ? "True" : "False"} charpoly_hash=${r.hash}`);
console.log(`times: total ${r.ms.reduce((a, b) => a + b, 0).toFixed(0)} ms`);
