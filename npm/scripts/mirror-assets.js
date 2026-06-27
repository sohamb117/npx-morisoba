#!/usr/bin/env node
const fs = require('fs');
const path = require('path');

const repoRoot = path.resolve(__dirname, '..', '..');
const srcRoot = path.join(repoRoot, 'assets');
const destRoot = path.join(__dirname, '..', 'assets');

if (!fs.existsSync(srcRoot)) {
  console.error(`mirror-assets: ${srcRoot} not found`);
  process.exit(1);
}

if (fs.existsSync(destRoot)) {
  fs.rmSync(destRoot, { recursive: true, force: true });
}
fs.mkdirSync(destRoot, { recursive: true });

let count = 0;
let bytes = 0;
function copyRecursive(src, dst) {
  const stat = fs.statSync(src);
  if (stat.isDirectory()) {
    fs.mkdirSync(dst, { recursive: true });
    for (const name of fs.readdirSync(src)) {
      copyRecursive(path.join(src, name), path.join(dst, name));
    }
  } else if (stat.isFile()) {
    fs.copyFileSync(src, dst);
    count += 1;
    bytes += stat.size;
  }
}

copyRecursive(srcRoot, destRoot);
console.log(`mirror-assets: copied ${count} files (${bytes} bytes) -> ${destRoot}`);
