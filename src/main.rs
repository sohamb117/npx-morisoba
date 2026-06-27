// MORISOBA :: SSH PROFILE TUI
// Brutalist B/W monochrome portfolio rendered as a full-screen ratatui application.
// See README.md for SSH integration and customization.

mod app;
mod renderer;
mod ui;

use crate::app::{Action, App};
use crate::renderer::RenderMode;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout, Write};
use std::path::Path;
use std::time::Duration;

type Term = Terminal<CrosstermBackend<Stdout>>;

fn main() -> io::Result<()> {
    install_panic_hook();

    let mode = renderer::detect_capability();
    let hero_ascii: Option<String> = if mode == RenderMode::Ascii {
        renderer::ascii::load_and_render(Path::new("assets/hero.png"), 60, 8)
    } else {
        None
    };

    let mut terminal = init_terminal()?;
    let mut app = App::new(mode);
    let result = run_loop(&mut terminal, &mut app, hero_ascii.as_deref(), mode);
    let _ = restore_terminal();
    result
}

fn init_terminal() -> io::Result<Term> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(io::stdout()))
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
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
    mode: RenderMode,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app, hero_ascii))?;

        if mode == RenderMode::Graphics {
            let area = terminal.size()?;
            let hero_rect = ui::compute_hero_rect(area);
            crossterm::queue!(
                io::stdout(),
                crossterm::cursor::MoveTo(hero_rect.x, hero_rect.y)
            )?;
            io::stdout().flush()?;
            let _ = renderer::graphics::draw_image(Path::new("assets/hero.png"), hero_rect);
        }

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press
                    && matches!(app.handle_key(key.code), Action::Quit)
                {
                    break;
                }
            }
        }
    }
    Ok(())
}
