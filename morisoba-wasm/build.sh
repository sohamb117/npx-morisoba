#!/usr/bin/env bash
# morisoba-wasm/build.sh — manual WASM bundle build, bypasses trunk's mandatory wasm-opt.
#
# Why this exists: trunk 0.21.14 forces wasm-opt in release mode with no skip option.
# Current binaryen (up to version_124) fails to parse wasm-bindgen 0.2.126's output.
# Until trunk gains a --no-wasm-opt flag (or binaryen catches up), we bypass trunk's
# build step entirely and assemble dist/ ourselves: cargo + wasm-bindgen + cp.
#
# Output: morisoba-wasm/dist/{index.html, style.css, desktop.js, morisoba-wasm.js, morisoba-wasm_bg.wasm}
# Deploy: drop dist/ on GitHub Pages, Netlify, S3, etc.

set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$HERE/.." && pwd)"

export PATH="${HOME}/.cargo/bin:$PATH"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${WORKSPACE_ROOT}/target}"
export CARGO_TARGET_DIR

WASM_BINDGEN="${WASM_BINDGEN:-${HOME}/.cache/trunk/wasm-bindgen-0.2.126/wasm-bindgen}"
if [ ! -x "$WASM_BINDGEN" ]; then
    echo "wasm-bindgen 0.2.126 not found at $WASM_BINDGEN" >&2
    echo "Install via: cargo install wasm-bindgen-cli --version 0.2.126 --locked" >&2
    echo "Or run trunk build once to let it auto-fetch the cached binary." >&2
    exit 1
fi

DIST="${HERE}/dist"
# Dev servers (python -m http.server) hold the dir handle on Windows mounts,
# making `rm -rf <dir>` fail. Removing contents in-place works regardless.
if ! rm -rf "$DIST" 2>/dev/null; then
    rm -rf "$DIST"/* "$DIST"/.[!.]* 2>/dev/null || true
fi
mkdir -p "$DIST"

echo "[1/4] cargo build --target wasm32-unknown-unknown -p morisoba-wasm --release"
(cd "$WORKSPACE_ROOT" && cargo build --target wasm32-unknown-unknown -p morisoba-wasm --release)

WASM_SRC="${CARGO_TARGET_DIR}/wasm32-unknown-unknown/release/morisoba_wasm.wasm"
if [ ! -f "$WASM_SRC" ]; then
    echo "expected wasm at $WASM_SRC, not found" >&2
    exit 1
fi

echo "[2/4] wasm-bindgen --target web"
"$WASM_BINDGEN" \
    --target web \
    --out-dir "$DIST" \
    --out-name morisoba-wasm \
    --no-typescript \
    "$WASM_SRC"

echo "[3/4] copy static assets"
cp "${HERE}/index.html" "$DIST/"
cp "${HERE}/style.css" "$DIST/"
cp "${HERE}/desktop.js" "$DIST/"

echo "[4/4] verify bundle"
ls -lh "$DIST/"
WASM_BG="${DIST}/morisoba-wasm_bg.wasm"
if [ -f "$WASM_BG" ]; then
    RAW=$(stat -c%s "$WASM_BG")
    GZ=$(gzip --stdout "$WASM_BG" | wc -c)
    printf "wasm raw:      %12d bytes  (%6.1f KB)\n" "$RAW" "$(echo "scale=1; $RAW/1024" | bc)"
    printf "wasm gzipped:  %12d bytes  (%6.1f KB)\n" "$GZ"  "$(echo "scale=1; $GZ/1024"  | bc)"
fi

echo ""
echo "OK. Deploy ${DIST}/ to any static host (GitHub Pages, Netlify, S3, file://)."
