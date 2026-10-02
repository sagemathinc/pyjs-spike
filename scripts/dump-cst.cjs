const {Parser, Language} = require("web-tree-sitter");
(async()=>{ await Parser.init(); const L = await Language.load("/home/user/pyjs-spike/node_modules/tree-sitter-python/tree-sitter-python.wasm"); const p=new Parser(); p.setLanguage(L);
const src = require("fs").readFileSync(0, "utf8");
function show(n, ind) { const f = []; for (let i=0;i<n.childCount;i++){ const c=n.child(i); const fn=n.fieldNameForChild(i); f.push([fn,c]); }
  console.log(" ".repeat(ind) + (n.isNamed? n.type : JSON.stringify(n.type)) + (n.childCount===0 && n.isNamed ? " " + JSON.stringify(n.text) : ""));
  for (const [fn,c] of f) { if (fn) console.log(" ".repeat(ind+2)+"@"+fn+":"); show(c, ind + (fn?4:2)); } }
show(p.parse(src).rootNode, 0); })();
