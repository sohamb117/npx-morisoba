# MORISOBA

A black-and-white brutalist portfolio TUI written in Rust. Three-level navigation: Sections → Items → Detail. Each item has a high-density ASCII hero on the left and a markdown body on the right. Press `i` to open the underlying PNG in your OS-default image viewer. No server, no SSH, no graphics protocol — runs as a single static binary anywhere a TTY exists.

Built with [ratatui 0.26](https://github.com/ratatui-org/ratatui), [crossterm 0.27](https://github.com/crossterm-rs/crossterm), [image 0.25](https://github.com/image-rs/image), and [opener 0.7](https://github.com/Seeker14491/opener).

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

## CONTROLS

The TUI has three views: **SECTIONS** (the landing list), **ITEMS** (the list inside a section), and **DETAIL** (split pane — ASCII art left, markdown body right).

| Key                     | SECTIONS view            | ITEMS view             | DETAIL view                    |
|-------------------------|--------------------------|------------------------|--------------------------------|
| ↑ / ↓ / j / k           | Move section selection   | Move item selection    | (no-op)                        |
| Enter / l / →           | Drill into section       | Open item detail       | (no-op)                        |
| Backspace / h / ← / Esc | (no-op)                  | Back to SECTIONS       | Back to ITEMS                  |
| **i**                   | Open `assets/hero.png`   | Open `assets/hero.png` | Open this item's PNG (or hero) |
| q                       | Quit                     | Quit                   | Quit                           |

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
├── hero.png                                    # landing-page hero (above section list)
└── sections/
    ├── about/
    │   ├── 01-bio.md
    │   ├── 02-philosophy.md
    │   └── 03-off-hours.md
    ├── projects/
    │   ├── 01-distributed-log-engine.md
    │   ├── 01-distributed-log-engine.png       # (optional) per-item ASCII art
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
4. Optionally drop a sibling `.png` with the same stem (e.g. `01-distributed-log-engine.png`) to use as the item's ASCII art in the DETAIL view's left pane. If absent, `assets/hero.png` is used.
5. Run `cargo build --release` — the build script picks it up automatically.

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

Place a PNG or JPEG at `assets/hero.png` for the landing-page hero. Per-item PNGs (optional) go alongside the matching markdown file as `assets/sections/<section>/<slug>.png`.

- **Pixel dimensions**: up to **2048×2048**. Larger images are rejected at load time to bound decode CPU.
- **File size**: up to **10 MiB**.
- **In-TUI rendering**: pixels are mapped to the ramp `[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@']` via Luma8 conversion and Lanczos3 downscale.
- **Full image**: opened on demand via the `i` key in the OS-default viewer.
- **Missing / bad**: a `[ HERO ]` placeholder is rendered inside the TUI; the app never panics on asset failures.

## SPEC EXTENSIONS

This implementation extends the original brief in four intentional, additive ways:

- **`j` / `k` / `l` / `h` navigation aliases** — vim-style alongside arrow keys. Both pairs map to the same logic.
- **`i` opens the original image** in the OS-default viewer. Lets users see the full-resolution asset outside the terminal.
- **Footer key-hint row** — the bottom row shows context-sensitive hints per view. The brief specified a 3-region root layout; the footer is added as a 4th fixed-height region for first-launch discoverability.
- **Three-view drill-down** — Sections → Items → Detail. The brief specified static section content; this adds item-granularity with markdown bodies and per-item ASCII art.

`Esc` is context-sensitive: in SECTIONS view it is a no-op (`q` remains the only documented quit key, per the brief); in ITEMS and DETAIL views it acts as `Back`, alongside Backspace / h / ←.

## TESTING

```bash
cargo test         # 40 unit + integration tests
cargo clippy --all-targets -- -D warnings
```

Manual verification scenarios (S1–S8) can be re-run via the bundled harness:

```bash
cargo build --release
./tools/wave9_qa.sh
```

The harness asserts: first-paint content, full drill-down navigation (Sections → Items → Detail → back → back), clean quit + restored terminal, TMUX-still-shows-ASCII, missing-hero placeholder, resize coherence at 80×24 and 160×50, zero color SGR codes in output, and `i`-key does not crash the TUI. Requires `tmux` on `PATH`.

## LICENSE

MIT OR Apache-2.0

## DEPENDENCIES

- **ratatui**: 0.26 — terminal UI framework
- **crossterm**: 0.27 — terminal backend
- **image**: 0.25 — PNG/JPEG decoding for the ASCII hero
- **opener**: 0.7 — cross-platform OS-default file opener (handles WSL routing too)
