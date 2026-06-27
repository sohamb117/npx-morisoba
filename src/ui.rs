//! Layout + widget composition for the full-screen portfolio TUI.
//!
//! Pure rendering: no I/O, no env access, no image decoding. Decisions about
//! capability mode live in [`crate::renderer`]; the ASCII hero string (when
//! Ascii mode) is precomputed by `main.rs` and passed in here.

use crate::app::App;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

const HERO_HEIGHT: u16 = 10;

/// Render one frame. Caller supplies the optional pre-rendered ASCII hero
/// (Some in Ascii mode, None in Graphics mode — Graphics mode draws the image
/// post-frame from `main.rs` over a placeholder block painted here).
pub fn render(frame: &mut Frame, app: &mut App, ascii_hero: Option<&str>) {
    const HEADER_HEIGHT: u16 = 5;
    const FOOTER_HEIGHT: u16 = 1;

    let [hero_area, header_area, content_area, footer_area]: [Rect; 4] =
        Layout::vertical([
            Constraint::Length(HERO_HEIGHT),
            Constraint::Length(HEADER_HEIGHT),
            Constraint::Fill(1),
            Constraint::Length(FOOTER_HEIGHT),
        ])
        .areas(frame.size());

    let [nav_area, main_area]: [Rect; 2] = Layout::horizontal([
        Constraint::Length(22),
        Constraint::Fill(1),
    ])
    .areas(content_area);

    let hero_widget = if let Some(text) = ascii_hero {
        Paragraph::new(text).block(Block::bordered().title("// HERO"))
    } else {
        Paragraph::new("[ HERO ]")
            .alignment(Alignment::Center)
            .block(Block::bordered().title("// HERO"))
    };
    frame.render_widget(hero_widget, hero_area);

    // CUSTOMIZE: Main name / moniker
    let header_line1 = Line::raw("MORISOBA");
    // CUSTOMIZE: Role / Tagline
    let header_line2 = Line::raw("SOFTWARE ENGINEER // SYSTEMS // RUST");
    // CUSTOMIZE: Sub-tagline / Vibe
    let header_line3 = Line::raw("BRUTALIST PORTFOLIO TUI :: SSH PROFILE");

    let header_text = Text::from(vec![header_line1, header_line2, header_line3]);
    let header_widget = Paragraph::new(header_text).block(Block::bordered());
    frame.render_widget(header_widget, header_area);

    let items: Vec<ListItem> = app
        .sections
        .iter()
        .map(|s| ListItem::new(s.title()))
        .collect();
    let nav_list = List::new(items)
        .block(Block::bordered().title("// NAV"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(" ");
    frame.render_stateful_widget(nav_list, nav_area, &mut app.list_state);

    let body = match app.current_section() {
        crate::app::Section::About => {
            // CUSTOMIZE: About section text
            "MORISOBA\nsenior software engineer\nrust / systems / distributed\nsf bay area\nbrutalist aesthetic / ssh-over-tui delivery"
        }
        crate::app::Section::Projects => {
            // CUSTOMIZE: Projects section text
            "DISTRIBUTED LOG ENGINE - append-only WAL over kQueue/epoll\n\nTUI FRAMEWORK MK1 - ratatui-based component lib\n\nKERNEL TRACE TOOLKIT - eBPF observability harness"
        }
        crate::app::Section::Experience => {
            // CUSTOMIZE: Experience section text
            "2023–PRESENT · SENIOR SWE · SYSTEMS CORP · building high throughput distributed log engine\n\n2020–2023 · SWE · RUST STARTUP · maintained backend services\n\n2018–2020 · JUNIOR SWE · WEB AGENCY · full stack web development"
        }
        crate::app::Section::Contact => {
            // CUSTOMIZE: Contact section text
            "GITHUB: @morisoba\nEMAIL: morisoba@example.com\nLOCATION: sf bay area"
        }
    };
    let content_widget = Paragraph::new(body)
        .block(Block::bordered().title("// CONTENT"))
        .wrap(Wrap { trim: false });
    frame.render_widget(content_widget, main_area);

    let footer_widget = Paragraph::new("UP/DN OR J/K NAVIGATE  ENTER SELECT  I IMAGE  Q QUIT")
        .alignment(Alignment::Left);
    frame.render_widget(footer_widget, footer_area);
}

/// Compute the hero Rect from a terminal area. Single source of truth shared
/// between `render` (which paints a placeholder) and `main.rs` (which moves the
/// cursor here and calls viuer post-frame). Keeps the two from drifting.
#[allow(dead_code)]
pub fn compute_hero_rect(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: HERO_HEIGHT.min(area.height),
    }
}

// T07 (Wave 2) - RED tests via ratatui TestBackend.
// We assert the rendered buffer contains required strings, has no foreground
// or background colors set on any cell (B/W invariant), and that the ABOUT
// row carries the REVERSED modifier (active-selection indicator).
#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::renderer::RenderMode;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use ratatui::style::{Color, Modifier};
    use ratatui::Terminal;

    fn render_to_buffer(width: u16, height: u16, hero: Option<&str>) -> Buffer {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let mut app = App::new(RenderMode::Ascii);
        terminal
            .draw(|f| render(f, &mut app, hero))
            .expect("frame draw");
        terminal.backend().buffer().clone()
    }

    fn row_text(buf: &Buffer, y: u16) -> String {
        let mut s = String::new();
        for x in 0..buf.area.width {
            s.push_str(buf.get(x, y).symbol());
        }
        s
    }

    fn buffer_contains(buf: &Buffer, needle: &str) -> bool {
        (0..buf.area.height).any(|y| row_text(buf, y).contains(needle))
    }

    #[test]
    fn buffer_contains_morisoba_name() {
        let buf = render_to_buffer(120, 40, None);
        assert!(
            buffer_contains(&buf, "MORISOBA"),
            "MORISOBA header missing from rendered frame"
        );
    }

    #[test]
    fn buffer_contains_all_section_titles() {
        let buf = render_to_buffer(120, 40, None);
        for title in ["ABOUT", "PROJECTS", "EXPERIENCE", "CONTACT"] {
            assert!(
                buffer_contains(&buf, title),
                "section title {title} missing from rendered frame"
            );
        }
    }

    #[test]
    fn buffer_contains_quit_hint_in_footer() {
        let buf = render_to_buffer(120, 40, None);
        assert!(
            buffer_contains(&buf, "Q QUIT") || buffer_contains(&buf, "QUIT"),
            "footer missing Q/QUIT hint"
        );
    }

    #[test]
    fn no_cell_uses_foreground_color() {
        let buf = render_to_buffer(120, 40, None);
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                let cell = buf.get(x, y);
                assert_eq!(
                    cell.fg,
                    Color::Reset,
                    "cell ({x},{y}) has fg color {:?} (should be Reset)",
                    cell.fg
                );
            }
        }
    }

    #[test]
    fn no_cell_uses_background_color() {
        let buf = render_to_buffer(120, 40, None);
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                let cell = buf.get(x, y);
                assert_eq!(
                    cell.bg,
                    Color::Reset,
                    "cell ({x},{y}) has bg color {:?} (should be Reset)",
                    cell.bg
                );
            }
        }
    }

    #[test]
    fn first_nav_row_has_reversed_modifier() {
        let buf = render_to_buffer(120, 40, None);
        let mut found = false;
        for y in 0..buf.area.height {
            let line = row_text(&buf, y);
            if !line.contains("ABOUT") {
                continue;
            }
            let any_reversed = (0..buf.area.width)
                .any(|x| buf.get(x, y).modifier.contains(Modifier::REVERSED));
            if any_reversed {
                found = true;
                break;
            }
        }
        assert!(
            found,
            "ABOUT row should carry Modifier::REVERSED (active selection indicator)"
        );
    }

    #[test]
    fn renders_at_80x24_without_panic() {
        let _ = render_to_buffer(80, 24, None);
    }

    #[test]
    fn renders_at_120x40_without_panic() {
        let _ = render_to_buffer(120, 40, None);
    }

    #[test]
    fn renders_at_160x50_without_panic() {
        let _ = render_to_buffer(160, 50, None);
    }

    #[test]
    fn renders_with_ascii_hero_text_without_panic() {
        let hero = "######\n@@@@@@\n......";
        let _ = render_to_buffer(120, 40, Some(hero));
    }

    #[test]
    fn compute_hero_rect_starts_at_origin_with_positive_height() {
        let r = compute_hero_rect(Rect::new(0, 0, 120, 40));
        assert_eq!(r.x, 0, "hero must start at column 0");
        assert_eq!(r.y, 0, "hero must start at row 0");
        assert!(r.width > 0, "hero width must be positive");
        assert!(r.height > 0, "hero height must be positive");
        assert!(
            r.height < 40,
            "hero must not consume entire vertical (got {})",
            r.height
        );
    }
}
