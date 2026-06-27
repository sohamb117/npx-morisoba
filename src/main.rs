mod app;
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
use std::io::{self, Stdout};
use std::path::Path;
use std::time::Duration;

type Term = Terminal<CrosstermBackend<Stdout>>;

const HERO_PATH: &str = "assets/hero.png";

fn main() -> io::Result<()> {
    install_panic_hook();

    let hero_path = Path::new(HERO_PATH);
    let hero_ascii: Option<String> = renderer::ascii::load_and_render(hero_path, 60, 8);

    let mut terminal = init_terminal()?;
    let mut app = App::new();
    let result = run_loop(&mut terminal, &mut app, hero_ascii.as_deref(), hero_path);
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

fn run_loop(
    terminal: &mut Term,
    app: &mut App,
    hero_ascii: Option<&str>,
    hero_path: &Path,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app, hero_ascii))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.handle_key(key.code) {
                        Action::Quit => break,
                        Action::OpenImage => {
                            let _ = opener::open(hero_path);
                        }
                        Action::Select | Action::Noop => {}
                    }
                }
            }
        }
    }
    Ok(())
}
