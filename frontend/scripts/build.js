const fs = require("fs");
const path = require("path");

const outDir = path.join(__dirname, "..", "build");
const publicDir = path.join(__dirname, "..", "public");

fs.rmSync(outDir, { recursive: true, force: true });
fs.mkdirSync(outDir, { recursive: true });

const indexSrc = path.join(publicDir, "index.html");
const indexDest = path.join(outDir, "index.html");
fs.copyFileSync(indexSrc, indexDest);

console.log(`build complete -> ${indexDest}`);
