#!/usr/bin/env node
// Mirror assets/ from the repo root into npm/assets/ so the published npm package
// ships the same hero.png. Run by hand before `npm publish`, or wire into CI.
const fs = require('fs');
const path = require('path');

const repoRoot = path.resolve(__dirname, '..', '..');
const src = path.join(repoRoot, 'assets', 'hero.png');
const destDir = path.join(__dirname, '..', 'assets');
const dest = path.join(destDir, 'hero.png');

if (!fs.existsSync(src)) {
  console.error(`mirror-assets: ${src} not found`);
  process.exit(1);
}
fs.mkdirSync(destDir, { recursive: true });
fs.copyFileSync(src, dest);
const bytes = fs.statSync(dest).size;
console.log(`mirror-assets: copied hero.png (${bytes} bytes) -> ${dest}`);
