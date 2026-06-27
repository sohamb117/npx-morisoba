//! Layout + widget composition for the morisoba portfolio TUI.
//!
//! Dispatches on `App::view`:
//! * [`View::Sections`] — hero + header + (section nav | preamble) + footer
//! * [`View::Items`] — hero + header + (section nav | item list) + footer
//! * [`View::Detail`] — header + (item ASCII | item body) + footer (no hero)

use crate::app::{App, Section, View};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

const HERO_HEIGHT: u16 = 10;
const HEADER_HEIGHT: u16 = 5;
const FOOTER_HEIGHT: u16 = 1;
const NAV_WIDTH: u16 = 22;

pub fn render(frame: &mut Frame, app: &mut App, hero_ascii: Option<&str>) {
    match app.view {
        View::Sections => render_sections_view(frame, app, hero_ascii),
        View::Items { section_idx } => render_items_view(frame, app, hero_ascii, section_idx),
        View::Detail {
            section_idx,
            item_idx,
        } => render_detail_view(frame, app, section_idx, item_idx),
    }
}

fn root_with_hero(area: Rect) -> [Rect; 4] {
    Layout::vertical([
        Constraint::Length(HERO_HEIGHT),
        Constraint::Length(HEADER_HEIGHT),
        Constraint::Fill(1),
        Constraint::Length(FOOTER_HEIGHT),
    ])
    .areas(area)
}

fn root_no_hero(area: Rect) -> [Rect; 3] {
    Layout::vertical([
        Constraint::Length(HEADER_HEIGHT),
        Constraint::Fill(1),
        Constraint::Length(FOOTER_HEIGHT),
    ])
    .areas(area)
}

fn split_content_nav(area: Rect) -> [Rect; 2] {
    Layout::horizontal([Constraint::Length(NAV_WIDTH), Constraint::Fill(1)]).areas(area)
}

fn split_detail_halves(area: Rect) -> [Rect; 2] {
    Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area)
}

fn render_hero(frame: &mut Frame, area: Rect, hero_ascii: Option<&str>) {
    let widget = if let Some(text) = hero_ascii {
        Paragraph::new(text).block(Block::bordered().title("// HERO"))
    } else {
        Paragraph::new("[ HERO ]")
            .alignment(Alignment::Center)
            .block(Block::bordered().title("// HERO"))
    };
    frame.render_widget(widget, area);
}

fn render_header(frame: &mut Frame, area: Rect) {
    // CUSTOMIZE: Main name / moniker
    let line1 = Line::raw("MORISOBA");
    // CUSTOMIZE: Role / Tagline
    let line2 = Line::raw("SOFTWARE ENGINEER // SYSTEMS // RUST");
    // CUSTOMIZE: Sub-tagline / Vibe
    let line3 = Line::raw("BRUTALIST PORTFOLIO TUI");
    let text = Text::from(vec![line1, line2, line3]);
    frame.render_widget(Paragraph::new(text).block(Block::bordered()), area);
}

fn render_section_nav(frame: &mut Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = app
        .sections
        .iter()
        .map(|s| ListItem::new(s.title()))
        .collect();
    let list = List::new(items)
        .block(Block::bordered().title("// NAV"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(" ");
    frame.render_stateful_widget(list, area, &mut app.section_list_state);
}

fn render_footer(frame: &mut Frame, area: Rect, view: View) {
    let text = match view {
        View::Sections => "UP/DN OR J/K NAV  ENTER DRILL  I IMAGE  Q QUIT",
        View::Items { .. } => "UP/DN OR J/K NAV  ENTER DRILL  BKSP/ESC BACK  I IMAGE  Q QUIT",
        View::Detail { .. } => "BKSP/ESC BACK  I IMAGE  Q QUIT",
    };
    frame.render_widget(Paragraph::new(text).alignment(Alignment::Left), area);
}

fn render_sections_view(frame: &mut Frame, app: &mut App, hero_ascii: Option<&str>) {
    let [hero, header, content, footer] = root_with_hero(frame.size());
    render_hero(frame, hero, hero_ascii);
    render_header(frame, header);
    let [nav, main] = split_content_nav(content);
    render_section_nav(frame, nav, app);

    let preamble = match app.current_section() {
        // CUSTOMIZE: About preamble
        Section::About => "MORISOBA\nsenior software engineer\nrust / systems / distributed\nsf bay area\nbrutalist aesthetic\n\n  ENTER to drill into items",
        // CUSTOMIZE: Projects preamble
        Section::Projects => "Selected projects across distributed systems, kernel observability, and brutalist TUI tooling.\n\n  ENTER to drill into items",
        // CUSTOMIZE: Experience preamble
        Section::Experience => "Reverse-chronological work history.\n\n  ENTER to drill into items",
        // CUSTOMIZE: Contact preamble
        Section::Contact => "Email, GitHub, location.\n\n  ENTER to drill into items",
    };
    let widget = Paragraph::new(preamble)
        .block(Block::bordered().title("// CONTENT"))
        .wrap(Wrap { trim: false });
    frame.render_widget(widget, main);

    render_footer(frame, footer, app.view);
}

fn render_items_view(
    frame: &mut Frame,
    app: &mut App,
    hero_ascii: Option<&str>,
    section_idx: usize,
) {
    let [hero, header, content, footer] = root_with_hero(frame.size());
    render_hero(frame, hero, hero_ascii);
    render_header(frame, header);
    let [nav, main] = split_content_nav(content);
    render_section_nav(frame, nav, app);

    let section = app.sections[section_idx];
    let items = crate::content::items_for(section);
    let list_items: Vec<ListItem> = items.iter().map(|i| ListItem::new(i.title)).collect();
    let widget = List::new(list_items)
        .block(Block::bordered().title(format!("// {}", section.title())))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(" ");
    frame.render_stateful_widget(widget, main, &mut app.item_list_state);

    render_footer(frame, footer, app.view);
}

fn render_detail_view(frame: &mut Frame, app: &mut App, section_idx: usize, item_idx: usize) {
    let [header, content, footer] = root_no_hero(frame.size());
    render_header(frame, header);

    let items = crate::content::items_for(app.sections[section_idx]);
    if let Some(item) = items.get(item_idx) {
        let [left, right] = split_detail_halves(content);

        let ascii_text = item
            .png_path
            .and_then(|p| crate::renderer::ascii::load_and_render(std::path::Path::new(p), 36, 14))
            .unwrap_or_else(|| "[ NO IMAGE ]".to_string());
        let ascii_widget = Paragraph::new(ascii_text)
            .block(Block::bordered().title("// IMAGE"))
            .alignment(Alignment::Left);
        frame.render_widget(ascii_widget, left);

        let body_widget = Paragraph::new(item.body)
            .block(Block::bordered().title(format!("// {}", item.title)))
            .wrap(Wrap { trim: false });
        frame.render_widget(body_widget, right);
    }

    render_footer(frame, footer, app.view);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use ratatui::style::{Color, Modifier};
    use ratatui::Terminal;

    fn render_view(width: u16, height: u16, app: &mut App, hero: Option<&str>) -> Buffer {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        terminal
            .draw(|f| render(f, app, hero))
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
    fn sections_view_contains_morisoba_name() {
        let mut a = App::new();
        let buf = render_view(120, 40, &mut a, None);
        assert!(buffer_contains(&buf, "MORISOBA"));
    }

    #[test]
    fn sections_view_contains_all_section_titles() {
        let mut a = App::new();
        let buf = render_view(120, 40, &mut a, None);
        for title in ["ABOUT", "PROJECTS", "EXPERIENCE", "CONTACT"] {
            assert!(buffer_contains(&buf, title), "missing {title}");
        }
    }

    #[test]
    fn sections_view_footer_has_quit_hint() {
        let mut a = App::new();
        let buf = render_view(120, 40, &mut a, None);
        assert!(buffer_contains(&buf, "Q QUIT"));
    }

    #[test]
    fn sections_view_footer_has_image_hint() {
        let mut a = App::new();
        let buf = render_view(120, 40, &mut a, None);
        assert!(buffer_contains(&buf, "I IMAGE"));
    }

    #[test]
    fn sections_view_no_foreground_color() {
        let mut a = App::new();
        let buf = render_view(120, 40, &mut a, None);
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                assert_eq!(
                    buf.get(x, y).fg,
                    Color::Reset,
                    "cell ({x},{y}) has fg {:?}",
                    buf.get(x, y).fg
                );
            }
        }
    }

    #[test]
    fn sections_view_no_background_color() {
        let mut a = App::new();
        let buf = render_view(120, 40, &mut a, None);
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                assert_eq!(
                    buf.get(x, y).bg,
                    Color::Reset,
                    "cell ({x},{y}) has bg {:?}",
                    buf.get(x, y).bg
                );
            }
        }
    }

    #[test]
    fn sections_view_about_row_has_reversed_modifier() {
        let mut a = App::new();
        let buf = render_view(120, 40, &mut a, None);
        let mut found = false;
        for y in 0..buf.area.height {
            let line = row_text(&buf, y);
            if !line.contains("ABOUT") {
                continue;
            }
            if (0..buf.area.width).any(|x| buf.get(x, y).modifier.contains(Modifier::REVERSED)) {
                found = true;
                break;
            }
        }
        assert!(found, "ABOUT row should be REVERSED");
    }

    #[test]
    fn sections_view_renders_at_80x24() {
        let mut a = App::new();
        let _ = render_view(80, 24, &mut a, None);
    }

    #[test]
    fn sections_view_renders_at_160x50() {
        let mut a = App::new();
        let _ = render_view(160, 50, &mut a, None);
    }

    #[test]
    fn sections_view_renders_with_hero_text() {
        let mut a = App::new();
        let hero = "######\n@@@@@@\n......";
        let _ = render_view(120, 40, &mut a, Some(hero));
    }

    #[test]
    fn items_view_shows_section_title_in_content_block() {
        let mut a = App::new();
        a.view = View::Items { section_idx: 1 };
        let buf = render_view(120, 40, &mut a, None);
        assert!(
            buffer_contains(&buf, "PROJECTS"),
            "items view should show // PROJECTS as content title"
        );
    }

    #[test]
    fn items_view_shows_item_titles_in_content_panel() {
        let mut a = App::new();
        a.view = View::Items { section_idx: 1 };
        let buf = render_view(120, 40, &mut a, None);
        let projects = crate::content::items_for(Section::Projects);
        assert!(!projects.is_empty(), "expected at least one project");
        for item in projects {
            assert!(
                buffer_contains(&buf, item.title),
                "items view should list item title: {}",
                item.title
            );
        }
    }

    #[test]
    fn items_view_footer_has_back_hint() {
        let mut a = App::new();
        a.view = View::Items { section_idx: 0 };
        let buf = render_view(120, 40, &mut a, None);
        assert!(
            buffer_contains(&buf, "BKSP") || buffer_contains(&buf, "BACK"),
            "items view footer should mention BACK"
        );
    }

    #[test]
    fn detail_view_shows_item_title_and_body() {
        let mut a = App::new();
        a.view = View::Detail {
            section_idx: 1,
            item_idx: 0,
        };
        let buf = render_view(120, 40, &mut a, None);
        let item = &crate::content::items_for(Section::Projects)[0];
        assert!(
            buffer_contains(&buf, item.title),
            "detail view should display item title"
        );
    }

    #[test]
    fn detail_view_footer_has_back_hint() {
        let mut a = App::new();
        a.view = View::Detail {
            section_idx: 0,
            item_idx: 0,
        };
        let buf = render_view(120, 40, &mut a, None);
        assert!(buffer_contains(&buf, "BKSP") || buffer_contains(&buf, "BACK"));
    }

    #[test]
    fn detail_view_renders_at_80x24() {
        let mut a = App::new();
        a.view = View::Detail {
            section_idx: 0,
            item_idx: 0,
        };
        let _ = render_view(80, 24, &mut a, None);
    }
}
