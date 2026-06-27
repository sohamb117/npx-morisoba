# MORISOBA

A black-and-white brutalist portfolio TUI written in Rust. Three-level navigation: Sections → Items → Detail. Each item has a high-density ASCII hero on the left and a markdown body on the right. Press `i` to open the underlying PNG in your OS-default image viewer. No server, no SSH, no graphics protocol — runs as a single static binary anywhere a TTY exists.

Built with [ratatui 0.26](https://github.com/ratatui-org/ratatui), [crossterm 0.27](https://github.com/crossterm-rs/crossterm), [image 0.24](https://github.com/image-rs/image), [opener 0.7](https://github.com/Seeker14491/opener), and [rascii_art 0.4](https://github.com/UTFeight/RASCII) for the image-to-ASCII conversion.

## INSTALL

### Via npx (no install)

```bash
npx morisoba
```

The npm wrapper downloads the platform-specific prebuilt binary from GitHub Releases on first run, caches it, and execs it. Works on Linux, macOS, and Windows. Requires Node 18+.

### Via cargo

```bash
cargo install morisoba
```

Compiles from source. Needs the Rust toolchain.

### Via `cargo binstall`

```bash
cargo binstall morisoba
```

Downloads a prebuilt binary from GitHub Releases. Faster than `cargo install`, needs [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall) pre-installed.

### From source

```bash
git clone https://github.com/morisoba/morisoba
cd morisoba
cargo build --release
./target/release/morisoba
```

## RUN

```bash
morisoba
```

The binary needs a real TTY for raw mode + alternate screen. Plain shell prompts, tmux panes, Windows Terminal, WezTerm, iTerm2, kitty — all work.

## WEB

morisoba runs in the browser as a WASM bundle via [ratzilla](https://github.com/ratatui/ratzilla) — same Rust source, different terminal backend. Each item's images open as draggable sibling windows in a brutalist HTML "desktop." Static hosting, no backend.

**Live**: <https://morisoba.github.io/morisoba/> *(auto-deployed on push to master via [.github/workflows/pages.yml](.github/workflows/pages.yml))*

**Local dev**:

```bash
cd morisoba-wasm
./build.sh                                  # produces dist/ (~270 KB gzipped wasm)
python3 -m http.server --directory dist 8000
# open http://localhost:8000
```

**Architecture**:

- `morisoba-wasm/src/lib.rs` — `#[wasm_bindgen(start)]` entry; `DomBackend::new_by_id("tui-mount")`; translates `ratzilla::event::KeyCode` → `crossterm::event::KeyCode` so `morisoba::app::App::handle_key` is reused unchanged.
- `morisoba-wasm/desktop.js` — ~150-line vanilla JS window manager. Brutalist `.desktop-window` chrome (2px borders, drag-by-titlebar, `[X]` close, cascade-offset spawn). `window.morisobaDesktop.spawn_image(bytes, slug)` is the JS↔WASM interop point for the `i` key.
- `morisoba-wasm/build.sh` — bypasses trunk's mandatory `wasm-opt` step (binaryen ≤ 124 can't parse wasm-bindgen 0.2.126 output). When binaryen catches up, the [Trunk.toml](morisoba-wasm/Trunk.toml) config is ready for `trunk serve` hot-reload dev.

## CONTROLS

The TUI has two views: **MENU** (split pane — sections list left, items-of-selected-section right) and **DETAIL** (split pane — ASCII art left, markdown body right).

In MENU view, both panes are always visible and the right pane updates live as you arrow through sections. Pane focus controls which list the arrow keys move and which list shows the `REVERSED` highlight; the unfocused list still shows its selection via the `> ` prefix.

| Key                     | MENU (section pane focus)             | MENU (item pane focus)                       | DETAIL view                                  |
|-------------------------|---------------------------------------|----------------------------------------------|----------------------------------------------|
| ↑ / ↓ / j / k           | Move section sel; items refresh live  | Move item selection                          | (no-op)                                      |
| Enter / → / l           | Shift focus → items pane              | Open item DETAIL                             | (no-op)                                      |
| Backspace / h / ← / Esc | (no-op — top of nav stack)            | Shift focus → section pane                   | Back to MENU (item pane focus)               |
| **i**                   | Open `assets/hero.png` (landing hero) | Open highlighted item's PNG (if authored)    | Open this item's PNG (if authored)           |
| q                       | Quit                                  | Quit                                         | Quit                                         |

The `I IMAGE` footer hint is **hidden** when the current context has no PNG to open (e.g. an item with no `<slug>.png` authored, or a section list with no `hero.png`). This avoids advertising a key that would be a silent no-op.

The `i` dispatch uses the [`opener`](https://docs.rs/opener) crate, which handles:
- **Linux / WSL**: `wslview` (when present) → Windows Photos; else `xdg-open` → your desktop file handler; else a built-in WSL fallback uses `wslpath -w` + `cmd.exe /c start`
- **macOS**: `open` → Preview (or your default handler)
- **Windows**: shell association → Photos / your chosen handler

The spawn is fire-and-forget — if no GUI display is available (e.g. headless server), the spawn errors silently and the TUI keeps running.

## CONTENT

Every item in every section is a markdown file under [assets/sections/](assets/sections). The build script at [build.rs](build.rs) walks those directories at compile time and embeds the content directly into the binary, so the shipped artifact has zero runtime filesystem dependencies for portfolio data.

### Directory layout

```
assets/
├── hero.png                                    # landing-page hero (decoded to ASCII at runtime)
├── hero.txt                                    # (optional) pre-rendered ASCII — overrides hero.png in-TUI
└── sections/
    ├── about/
    │   ├── 01-bio.md
    │   ├── 01-bio.txt                          # (optional) pre-rendered ASCII for this item
    │   ├── 02-philosophy.md
    │   └── 03-off-hours.md
    ├── projects/
    │   ├── 01-distributed-log-engine.md
    │   ├── 01-distributed-log-engine.png       # (optional) per-item image — decoded to ASCII at runtime
    │   ├── 01-distributed-log-engine.txt       # (optional) pre-rendered — wins over .png in-TUI
    │   ├── 02-tui-framework-mk1.md
    │   └── 03-kernel-trace-toolkit.md
    ├── experience/
    │   └── *.md
    └── contact/
        └── *.md
```

### Adding an item

1. Drop a markdown file in `assets/sections/<section>/`. Filename becomes the slug (sortable — prefix with `01-`, `02-`, …).
2. First `# Heading` line becomes the item title shown in the list. If absent, the slug is upper-cased as the title.
3. Body content (everything after the title line) is rendered verbatim in the right pane of the DETAIL view.
4. **Optionally** drop a sibling `<slug>.txt` — pre-rendered ASCII art (e.g. from figlet, an image-to-ASCII tool, or hand-drawn) that appears **verbatim** in the DETAIL view's left pane. CRLF is normalized to LF at build time.
5. **Or** drop a sibling `<slug>.png` (or `.jpg`) — runtime-decoded to ASCII via the luminance ramp. The PNG is also what the `i` key opens in the OS image viewer.
6. If both `.txt` and `.png` exist for the same item, `.txt` wins for in-TUI rendering; `.png` is still used by `i`.
7. Fallback chain (left pane of DETAIL view): per-item `.txt` → decoded per-item `.png` → `hero.txt` → decoded `hero.png` → `[ NO IMAGE ]` placeholder.
8. Run `cargo build --release` — [build.rs](build.rs) picks everything up automatically.

### Customizing headers and preambles

These literals are still in source (no markdown indirection); edit them in [src/ui.rs](src/ui.rs):

- **src/ui.rs:70**: Main name / moniker (default: "MORISOBA")
- **src/ui.rs:72**: Role / Tagline (default: "SOFTWARE ENGINEER // SYSTEMS // RUST")
- **src/ui.rs:74**: Sub-tagline / Vibe (default: "BRUTALIST PORTFOLIO TUI :: SSH PROFILE")
- **src/ui.rs:110**: About-section preamble (shown above item list)
- **src/ui.rs:112**: Projects-section preamble
- **src/ui.rs:114**: Experience-section preamble
- **src/ui.rs:116**: Contact-section preamble

Navigation order is controlled by the `Section` enum in [src/app.rs](src/app.rs).

## HERO ASSET

Place a PNG or JPEG at `assets/hero.png` for the landing-page hero. **Or** place a pre-rendered ASCII text file at `assets/hero.txt` — when present, it takes precedence over the PNG (no decode step). Per-item siblings live alongside the matching markdown file as `assets/sections/<section>/<slug>.{png,txt}`.

PNG bytes and `.txt` content are **embedded at compile time** via `include_bytes!` and a build-script text reader (driven by [build.rs](build.rs)), so the binary works from any CWD — `cargo install morisoba`, `npx morisoba`, or running the release binary from a tmp dir all render identically.

- **PNG limits**: up to **2048×2048** pixels, **10 MiB** file size. Larger images are rejected at decode time to bound CPU.
- **PNG rendering**: image decoded via [`image` 0.24](https://github.com/image-rs/image) → handed to [`rascii_art`](https://github.com/UTFeight/RASCII) for the ramp mapping → each output character wrapped in a ratatui `Span` with **24-level grayscale FG** from the 256-color palette (codes 232..=255). **Subpage images are scaled to fill the bordered DETAIL pane** (typically ~78×32 cells at a 160×40 terminal) while preserving aspect ratio via `fit_within_cells`. Brutalist 10-char ramp `[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@']` is passed to rascii_art via `charset`.
- **Procedural animations**: items that have NEITHER an authored `.png` NOR a `.txt` get an algorithmically-generated moving ASCII pattern instead of a static placeholder. **Five generators** ship — `plasma` (drifting sin/cos field), `rain` (Matrix-style falling drops), `sine` (stacked horizontal waves with envelope), `worms` (Lissajous-style paths with fading grayscale trails), and `cellular` (drifting Voronoi cell membranes with the y-axis doubled to compensate for the 2:1 terminal cell ratio — "organoid" look). Dispatched per item via `dispatch_index(item.slug, 5)`, which XOR-folds the SipHash high/low halves and adds slug length to defeat low-bit clustering on similarly-prefixed slugs (`01-foo`, `01-bar`, …). A regression-locked test asserts every generator type appears across the actual content slug set. Animations run at **10 fps** (driven by the 100ms event-loop poll), use the same 256-color grayscale palette as decoded PNGs, and the `tick_ms` parameter is wall-clock-derived (`std::time::Instant::elapsed().as_millis()`) so motion is independent of frame rate jitter. See [src/renderer/animator.rs](src/renderer/animator.rs).
- **`.txt` handling**: embedded verbatim with CRLF → LF normalization at build time. Renders as plain Text without per-cell grayscale (you control the styling by choosing characters). No size cap.
- **Full image** (`i` key): opens an image in the OS-default viewer. **The image shown depends on what you have selected:** on section-pane focus the landing `assets/hero.png` is opened; on item-pane focus or DETAIL view the highlighted/viewed item's own `<slug>.png` is opened — and nothing happens if that item has no PNG authored. (No more hero fallback for item-specific contexts; that was a UX trap because every page looked like it shared the bio image.) The chosen PNG bytes are materialized to `$TMPDIR/morisoba-<hash>.png` on first press, then opened via `opener` (or the WSL `cmd.exe /c start` fallback).
- **Missing hero at build time**: a `[ HERO ]` placeholder is rendered inside the TUI; the app never panics on asset failures.

## SPEC EXTENSIONS

This implementation extends the original brief in five intentional, additive ways:

- **`j` / `k` / `l` / `h` navigation aliases** — vim-style alongside arrow keys. Both pairs map to the same logic.
- **`i` opens the original image** in the OS-default viewer. Lets users see the full-resolution asset outside the terminal.
- **Footer key-hint row** — the bottom row shows context-sensitive hints per pane focus. The brief specified a 3-region root layout; the footer is added as a 4th fixed-height region for first-launch discoverability.
- **Two-view menu UX** — single MENU view with focused-pane semantics (section nav left, items right, both always visible; right pane updates live on section navigation) plus DETAIL view. The brief specified static section content; this adds item-granularity with markdown bodies and per-item ASCII art while preserving a single-screen "menu with submenu" feel.
- **Grayscale ASCII rendering** — Per-pixel foreground from the 256-color grayscale palette (codes 232..=255 = 24 pure-achromatic luminance levels) is layered over the brutalist 10-char ramp for smooth gradients on decoded PNGs. All other UI chrome (borders, list highlights, footer, header, hand-authored `.txt` art) remains pure B/W via reverse-video. Wave 9 S7 enforces the carveout: chromatic SGR codes still cause the harness to fail.

`Esc` is context-sensitive: in section-pane focus it is a no-op (`q` remains the only documented quit key, per the brief); in item-pane focus it shifts focus back to the section pane; in DETAIL view it returns to MENU (item pane focus). Alongside Backspace / h / ←.

## TESTING

```bash
cargo test         # 62 unit + integration tests
cargo clippy --all-targets -- -D warnings
```

Manual verification scenarios (S1–S8) can be re-run via the bundled harness:

```bash
cargo build --release
./tools/wave9_qa.sh
```

The harness asserts: first-paint content (including items shown immediately in the right pane — no drill needed), menu navigation with live right-pane refresh + focus shift + drill into DETAIL + back-stack unwind, clean quit + restored terminal, TMUX-still-shows-ASCII, **CWD-independence** (hero renders even when run from a CWD with no `assets/` directory — locks in the compile-time embedding invariant), resize coherence at 80×24 and 160×50, zero color SGR codes in output, and `i`-key does not crash the TUI. Requires `tmux` on `PATH`.

## LICENSE

MIT OR Apache-2.0

## DEPENDENCIES

- **ratatui**: 0.26 — terminal UI framework
- **crossterm**: 0.27 — terminal backend
- **image**: 0.24 — PNG/JPEG decoding (downgraded from 0.25 to match rascii_art's expected DynamicImage type)
- **rascii_art**: 0.4 — image-to-ASCII ramp mapping (replaces the previous hand-rolled renderer)
- **opener**: 0.7 — cross-platform OS-default file opener (handles WSL routing too)
