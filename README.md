# MORISOBA

A black-and-white brutalist portfolio TUI written in Rust. Hero region renders as a high-density ASCII ramp; press `i` to open the original image in your OS-default image viewer. No server, no SSH, no graphics protocol — runs as a single static binary anywhere a TTY exists.

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

| Key             | Action                                                  |
|-----------------|---------------------------------------------------------|
| ↑ / ↓ / j / k   | Navigate sections                                       |
| Enter           | Select (reserved for future detail view)                |
| **i**           | Open `assets/hero.png` in the OS-default image viewer   |
| q               | Quit cleanly                                            |
| Esc             | No-op (intentionally not a quit key)                    |

The `i` dispatch uses the [`opener`](https://docs.rs/opener) crate, which handles:
- **Linux / WSL**: `wslview` (when present) → Windows Photos; else `xdg-open` → your desktop file handler
- **macOS**: `open` → Preview (or your default handler)
- **Windows**: shell association → Photos / your chosen handler

The spawn is fire-and-forget — if no GUI display is available (e.g. headless server), the spawn errors silently and the TUI keeps running.

## CUSTOMIZE

Modify the following markers in [src/ui.rs](src/ui.rs):

- **src/ui.rs:47**: Main name / moniker (default: "MORISOBA")
- **src/ui.rs:49**: Role / Tagline (default: "SOFTWARE ENGINEER // SYSTEMS // RUST")
- **src/ui.rs:51**: Sub-tagline / Vibe (default: "BRUTALIST PORTFOLIO TUI :: SSH PROFILE")
- **src/ui.rs:71**: About section body
- **src/ui.rs:75**: Projects section body
- **src/ui.rs:79**: Experience section body
- **src/ui.rs:83**: Contact section body

Navigation order is controlled by the `Section` enum in [src/app.rs](src/app.rs).

## HERO ASSET

Place a PNG or JPEG at `assets/hero.png`.

- **Pixel dimensions**: up to **2048×2048**. Larger images are rejected at load time to bound decode CPU.
- **File size**: up to **10 MiB**.
- **In-TUI rendering**: pixels are mapped to the ramp `[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@']` via Luma8 conversion and Lanczos3 downscale.
- **Full image**: opened on demand via the `i` key in the OS-default viewer.
- **Missing / bad**: a `[ HERO ]` placeholder is rendered inside the TUI; the app never panics on asset failures.

## SPEC EXTENSIONS

This implementation extends the original brief in three intentional, additive ways:

- **`j` / `k` navigation aliases** — vim-style alongside ↑/↓. Both pairs map to the same logic.
- **`i` opens the original image** in the OS-default viewer. Lets users see the full-resolution asset outside the terminal.
- **Footer key-hint row** — `UP/DN OR J/K NAVIGATE  ENTER SELECT  I IMAGE  Q QUIT`. The brief specified a 3-region root layout; this footer is added as a 4th fixed-height region for first-launch discoverability.

`Esc` does **not** quit. Per the brief, the only documented quit key is `q`. Pressing Esc returns `Action::Noop`; the TUI stays running.

## TESTING

```bash
cargo test         # 33 unit tests
cargo clippy --all-targets -- -D warnings
```

Manual verification scenarios (S1–S8) can be re-run via the bundled harness:

```bash
cargo build --release
./tools/wave9_qa.sh
```

The harness asserts: first-paint content, navigation cycles, clean quit + restored terminal, TMUX-still-shows-ASCII (since hero is always ASCII), missing-hero placeholder, resize coherence at 80×24 and 160×50, zero color SGR codes in output, and `i`-key does not crash the TUI. Requires `tmux` on `PATH`.

## LICENSE

MIT OR Apache-2.0

## DEPENDENCIES

- **ratatui**: 0.26 — terminal UI framework
- **crossterm**: 0.27 — terminal backend
- **image**: 0.25 — PNG/JPEG decoding for the ASCII hero
- **opener**: 0.7 — cross-platform OS-default file opener (handles WSL routing too)
