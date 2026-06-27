# SSH PROFILE TUI

A black-and-white brutalist SSH-accessed portfolio TUI. Engineered for direct delivery via `ssh user@host`. Built with Rust, leveraging [ratatui 0.26](https://github.com/ratatui-org/ratatui), [crossterm 0.27](https://github.com/crossterm-rs/crossterm), [viuer 0.7](https://github.com/atanunq/viuer), [image 0.25](https://github.com/image-rs/image), and [base64 0.22](https://github.com/marshallpierce/rust-base64).

## WHAT

This application serves as a terminal-based professional profile. It adheres to a strict monochrome aesthetic. It supports inline graphics on compatible terminal emulators and falls back to high-density ASCII ramp mapping otherwise.

## BUILD

Build requires a Rust toolchain. For WSL environments, use the following command to ensure the binary is built for the Linux target:

```bash
wsl -d Ubuntu --user root -- bash -c \
  'export CARGO_TARGET_DIR=$HOME/target_morisoba; \
   cd /mnt/c/Users/t-sohamb/Documents/code/ssh-tui-morisoba && \
   cargo build --release'
```

For native Linux environments:

```bash
cargo build --release
# binary at target/release/ssh-profile-tui
```

## RUN

```bash
./target/release/ssh-profile-tui
```

NOTE: TMUX forces Ascii fallback regardless of the `TERM` variable.

## CAPABILITY MATRIX

The renderer detects terminal capabilities at startup.

| Environment           | Mode       |
|-----------------------|------------|
| TMUX env set          | Ascii      |
| TERM=xterm-kitty      | Graphics   |
| TERM_PROGRAM=iTerm.app | Graphics  |
| TERM_PROGRAM=WezTerm  | Graphics   |
| LC_TERMINAL=iTerm2 / WezTerm (SSH compatibility) | Graphics |
| Everything else       | Ascii      |

Graphics mode draws via `viuer` post-frame. If `assets/hero.png` is missing, the application renders a `[ HERO ]` placeholder. Logic is defined in [src/renderer/mod.rs](src/renderer/mod.rs).

## SSH INTEGRATION

The TUI is designed to be the entry point for an SSH session.

### Method A: authorized_keys ForceCommand (Recommended)

Add the following to your `~/.ssh/authorized_keys` file. A PTY is required because `enable_raw_mode()` fails on a non-TTY stdin. Allocate one from EITHER side:

- **client side** — invoke as `ssh -t user@host`, or add `RequestTTY force` to your `~/.ssh/config`
- **server side** — keep `PermitTTY yes` in `/etc/ssh/sshd_config` (this is the OpenSSH default)

(`RequestTTY` is a *client* ssh_config option; the matching *server* setting is `PermitTTY`. Both layers must allow a TTY for the TUI to start.)

```
command="/usr/local/bin/ssh-profile-tui",no-port-forwarding,no-X11-forwarding,no-agent-forwarding ssh-ed25519 AAAA...
```

This method allows different SSH keys to trigger different commands or profile instances.

**Security hardening**: the binary reads `assets/hero.png` relative to the process's current working directory. For ForceCommand deployments, wrap with a tiny shim that switches to a root-owned asset directory first so an SSH-authenticated user cannot control which image is loaded:

```bash
#!/bin/sh
# /usr/local/bin/ssh-profile-tui-launch
cd /usr/local/share/ssh-profile-tui || exit 1
exec /usr/local/bin/ssh-profile-tui
```

Then point ForceCommand at `/usr/local/bin/ssh-profile-tui-launch` instead of the binary directly. The same hardening applies to the login-shell method (Method B): place the asset directory somewhere the user cannot write.

### Method B: As login shell

Assign the binary as the default shell for a specific user.

```bash
sudo cp target/release/ssh-profile-tui /usr/local/bin/
echo /usr/local/bin/ssh-profile-tui | sudo tee -a /etc/shells
sudo useradd -m -s /usr/local/bin/ssh-profile-tui visitor
# then: ssh visitor@host
```

This approach simplifies client-side connection (no `-t` required) but binds the entire session to the TUI.

## CUSTOMIZE

The following markers in [src/ui.rs](src/ui.rs) identify content for modification:

- **src/ui.rs:47**: Main name / moniker (default: "MORISOBA")
- **src/ui.rs:49**: Role / Tagline (default: "SOFTWARE ENGINEER // SYSTEMS // RUST")
- **src/ui.rs:51**: Sub-tagline / Vibe (default: "BRUTALIST PORTFOLIO TUI :: SSH PROFILE")
- **src/ui.rs:71**: About section body
- **src/ui.rs:75**: Projects section body
- **src/ui.rs:79**: Experience section body
- **src/ui.rs:83**: Contact section body

Navigation entries and order are controlled by the `Section` enum in [src/app.rs](src/app.rs).

## HERO ASSET

Place a PNG or JPEG at `assets/hero.png`.

- **Aspect**: Roughly square (approx. 60x16 cells; pixel dimensions anywhere from ~120x32 up to **2048x2048**, the pixel-bomb safety cap).
- **Size cap**: file must be **≤ 10 MiB**. Oversized files are rejected before decode.
- **Style**: High contrast, brutalist halftone or dithered.
- **Processing**: Graphics mode uses Lanczos3 downscaling. Ascii mode ramp-maps pixels via `[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@']`.
- **Failure**: If the file is missing, oversized, or has dimensions over 2048×2048, a `[ HERO ]` placeholder is rendered. The app does NOT panic on bad assets.

## SPEC EXTENSIONS

This implementation extends the brief in two intentional, additive ways. The original controls (↑/↓/Enter/q) and the original 3-region root layout still work; the extensions are pure additions:

- **`j` / `k` navigation aliases** — in addition to ↑/↓, the navigation accepts vim-style `j`/`k`. Both pairs map to the same advance/retreat logic; the footer documents both.
- **Footer key-hint row** — a 1-row footer below the content panel surfaces the keymap (`UP/DN OR J/K NAVIGATE  ENTER SELECT  Q QUIT`). The brief specifies Hero/Header/Content as the 3-region root layout; this footer is added as a 4th fixed-height region for first-launch discoverability.

`Esc` does **not** quit. Per the brief, the only documented quit key is `q`. Pressing Esc returns `Action::Noop`; the TUI stays running.

## TESTING

Execution of the test suite and linting:

```bash
cargo test         # 51 unit tests
cargo clippy --all-targets -- -D warnings
```

Manual verification scenarios (S1–S7) can be re-run via the bundled harness:

```bash
cargo build --release
./tools/wave9_qa.sh
```

The harness asserts: first-paint content, navigation cycles, clean quit + restored terminal, TMUX forces Ascii fallback, missing-hero placeholder, resize coherence at 80×24 and 160×50, and zero color SGR codes in output. Requires `tmux` on `PATH`.

## LICENSE

MIT OR Apache-2.0

## PROVENANCE

- **ratatui**: 0.26
- **crossterm**: 0.27
- **viuer**: 0.7
- **image**: 0.25
- **base64**: 0.22
