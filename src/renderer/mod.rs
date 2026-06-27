//! Renderer module: terminal-capability detection and the two rendering paths.
//!
//! * [`ascii`] — text-only fallback. Always works.
//! * [`graphics`] — viuer-backed inline image rendering for kitty / iTerm /
//!   WezTerm-class terminals. Skipped automatically under tmux even when TERM
//!   would otherwise qualify, because tmux strips graphics protocols.

pub mod ascii;
pub mod graphics;

/// Which rendering path the application should use this run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Graphics,
    Ascii,
}

/// Inspect environment variables and decide a render mode for this session.
///
/// Order (first matching rule wins):
///   1. `TMUX` set → `Ascii` (tmux strips graphics escapes).
///   2. `TERM` contains `kitty` → `Graphics`.
///   3. `TERM_PROGRAM` is `iTerm.app` or `WezTerm` → `Graphics`.
///   4. Otherwise → `Ascii`.
pub fn detect_capability() -> RenderMode {
    if std::env::var("TMUX").is_ok() {
        return RenderMode::Ascii;
    }
    let term = std::env::var("TERM").unwrap_or_default();
    if term.contains("kitty") {
        return RenderMode::Graphics;
    }
    let prog = std::env::var("TERM_PROGRAM").unwrap_or_default();
    if prog.contains("iTerm") || prog.contains("WezTerm") {
        return RenderMode::Graphics;
    }
    RenderMode::Ascii
}

// T05 (Wave 2) - RED tests for capability detection.
// env::set_var is process-global; tests serialize via a mutex so they
// don't pollute each other when run in parallel.
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn scrub() {
        std::env::remove_var("TMUX");
        std::env::remove_var("TERM");
        std::env::remove_var("TERM_PROGRAM");
    }

    #[test]
    fn tmux_forces_ascii_even_with_kitty_term() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        scrub();
        std::env::set_var("TMUX", "/tmp/tmux-1000/default,12345,0");
        std::env::set_var("TERM", "xterm-kitty");
        let got = detect_capability();
        scrub();
        assert_eq!(got, RenderMode::Ascii);
    }

    #[test]
    fn kitty_term_yields_graphics_when_no_tmux() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        scrub();
        std::env::set_var("TERM", "xterm-kitty");
        let got = detect_capability();
        scrub();
        assert_eq!(got, RenderMode::Graphics);
    }

    #[test]
    fn iterm_program_yields_graphics() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        scrub();
        std::env::set_var("TERM_PROGRAM", "iTerm.app");
        let got = detect_capability();
        scrub();
        assert_eq!(got, RenderMode::Graphics);
    }

    #[test]
    fn wezterm_program_yields_graphics() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        scrub();
        std::env::set_var("TERM_PROGRAM", "WezTerm");
        let got = detect_capability();
        scrub();
        assert_eq!(got, RenderMode::Graphics);
    }

    #[test]
    fn default_xterm_yields_ascii() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        scrub();
        std::env::set_var("TERM", "xterm-256color");
        let got = detect_capability();
        scrub();
        assert_eq!(got, RenderMode::Ascii);
    }
}
