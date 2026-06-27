mod app;
mod content;
mod renderer;
mod ui;

use crate::app::{Action, App};
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{self, Stdout};
use std::path::{Path, PathBuf};
use std::time::Duration;

type Term = Terminal<CrosstermBackend<Stdout>>;

fn main() -> io::Result<()> {
    install_panic_hook();

    let hero_ascii: Option<String> = content::HERO_ASCII.map(str::to_string).or_else(|| {
        content::HERO_PNG.and_then(|bytes| renderer::ascii::load_and_render_bytes(bytes, 60, 8))
    });

    let mut terminal = init_terminal()?;
    let mut app = App::new();
    let result = run_loop(&mut terminal, &mut app, hero_ascii.as_deref());
    let _ = restore_terminal();
    result
}

fn init_terminal() -> io::Result<Term> {
    enable_raw_mode()?;
    if let Err(e) = execute!(io::stdout(), EnterAlternateScreen) {
        let _ = disable_raw_mode();
        return Err(e);
    }
    match Terminal::new(CrosstermBackend::new(io::stdout())) {
        Ok(t) => Ok(t),
        Err(e) => {
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
            let _ = disable_raw_mode();
            Err(e)
        }
    }
}

fn restore_terminal() -> io::Result<()> {
    let r1 = disable_raw_mode();
    let r2 = execute!(io::stdout(), LeaveAlternateScreen);
    r1.and(r2)
}

fn install_panic_hook() {
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        prev_hook(info);
    }));
}

fn run_loop(terminal: &mut Term, app: &mut App, hero_ascii: Option<&str>) -> io::Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app, hero_ascii))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.handle_key(key.code) {
                        Action::Quit => break,
                        Action::OpenImage => {
                            let bytes = app
                                .current_item()
                                .and_then(|i| i.png_bytes)
                                .or(content::HERO_PNG);
                            if let Some(bytes) = bytes {
                                let _ = open_image_bytes(bytes);
                            }
                        }
                        Action::Drill | Action::Back | Action::Noop => {}
                    }
                }
            }
        }
    }
    Ok(())
}

fn open_image_bytes(bytes: &[u8]) -> io::Result<()> {
    let path = materialize_temp_png(bytes)?;
    if opener::open(&path).is_ok() {
        return Ok(());
    }
    if is_wsl() {
        if let Ok(output) = std::process::Command::new("wslpath")
            .arg("-w")
            .arg(&path)
            .output()
        {
            if output.status.success() {
                let win_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return std::process::Command::new("cmd.exe")
                    .args(["/c", "start", "", &win_path])
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                    .map(|_| ());
            }
        }
    }
    Err(io::Error::other("no image opener available"))
}

/// Hash the bytes with DefaultHasher to derive a stable per-asset filename,
/// then write them to `$TMPDIR/morisoba-<hex>.png` if not already present.
/// The temp file persists across runs and is reused when the same asset
/// is opened repeatedly. Returns the temp file path.
fn materialize_temp_png(bytes: &[u8]) -> io::Result<PathBuf> {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    let h = hasher.finish();
    let path = std::env::temp_dir().join(format!("morisoba-{:016x}.png", h));
    if !Path::new(&path).exists() {
        std::fs::write(&path, bytes)?;
    }
    Ok(path)
}

fn is_wsl() -> bool {
    std::fs::read_to_string("/proc/version")
        .map(|s| {
            let lower = s.to_ascii_lowercase();
            lower.contains("microsoft") || lower.contains("wsl")
        })
        .unwrap_or(false)
}
