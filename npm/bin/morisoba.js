#!/usr/bin/env node
const fs = require('fs');
const os = require('os');
const path = require('path');
const https = require('https');
const { spawn } = require('child_process');

const pkg = require(path.join(__dirname, '..', 'package.json'));
const REPO = process.env.MORISOBA_REPO || 'morisoba/morisoba';

function platformKey() {
  const p = process.platform;
  const a = process.arch;
  if (p === 'linux' && a === 'x64') return { name: 'morisoba-linux-x86_64', exec: 'morisoba-linux-x86_64' };
  if (p === 'darwin' && a === 'x64') return { name: 'morisoba-macos-x86_64', exec: 'morisoba-macos-x86_64' };
  if (p === 'darwin' && a === 'arm64') return { name: 'morisoba-macos-aarch64', exec: 'morisoba-macos-aarch64' };
  if (p === 'win32' && a === 'x64') return { name: 'morisoba-windows-x86_64.exe', exec: 'morisoba-windows-x86_64.exe' };
  throw new Error(
    `morisoba: unsupported platform ${p}/${a}. Supported: linux x64, darwin x64, darwin arm64, win32 x64. ` +
    `Build from source: https://github.com/${REPO}`
  );
}

function cachePath() {
  const dir = path.join(os.homedir(), '.cache', 'morisoba', `v${pkg.version}`);
  fs.mkdirSync(dir, { recursive: true });
  return path.join(dir, platformKey().exec);
}

function download(url, dest, redirects = 0) {
  return new Promise((resolve, reject) => {
    if (redirects > 5) return reject(new Error('too many redirects'));
    const req = https.get(url, (res) => {
      if (res.statusCode === 301 || res.statusCode === 302 || res.statusCode === 307 || res.statusCode === 308) {
        res.resume();
        return resolve(download(res.headers.location, dest, redirects + 1));
      }
      if (res.statusCode !== 200) {
        res.resume();
        return reject(new Error(`download failed: HTTP ${res.statusCode} ${res.statusMessage} for ${url}`));
      }
      const out = fs.createWriteStream(dest);
      res.pipe(out);
      out.on('finish', () => out.close(() => resolve()));
      out.on('error', reject);
    });
    req.on('error', reject);
    req.setTimeout(60_000, () => req.destroy(new Error('download timeout after 60s')));
  });
}

async function ensureBinary() {
  const dest = cachePath();
  if (fs.existsSync(dest) && fs.statSync(dest).size > 0) {
    return dest;
  }
  const { name } = platformKey();
  const url = `https://github.com/${REPO}/releases/download/v${pkg.version}/${name}`;
  process.stderr.write(`morisoba: fetching ${name} v${pkg.version} from GitHub Releases...\n`);
  const tmp = dest + '.partial';
  try {
    await download(url, tmp);
    fs.renameSync(tmp, dest);
    if (process.platform !== 'win32') {
      fs.chmodSync(dest, 0o755);
    }
    process.stderr.write(`morisoba: cached at ${dest}\n`);
    return dest;
  } catch (err) {
    try { fs.unlinkSync(tmp); } catch (_) {}
    throw new Error(
      `failed to download morisoba v${pkg.version}: ${err.message}\n` +
      `If the GitHub Release for v${pkg.version} doesn't exist yet, try a different version: npm install -g morisoba@latest`
    );
  }
}

(async () => {
  try {
    const bin = await ensureBinary();
    const pkgDir = path.resolve(__dirname, '..');
    const child = spawn(bin, process.argv.slice(2), { cwd: pkgDir, stdio: 'inherit' });
    child.on('exit', (code, signal) => {
      if (signal) {
        process.kill(process.pid, signal);
      } else {
        process.exit(code ?? 0);
      }
    });
    child.on('error', (err) => {
      process.stderr.write(`morisoba: spawn failed: ${err.message}\n`);
      process.exit(1);
    });
  } catch (err) {
    process.stderr.write(`morisoba: ${err.message}\n`);
    process.exit(1);
  }
})();
