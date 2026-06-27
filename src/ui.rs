//! Layout + widget composition for the morisoba portfolio TUI.
//!
//! Dispatches on [`App::view`]:
//! * [`View::Menu`] — hero + header + (section nav | items-of-selected-section) + footer.
//!   Both panes are ALWAYS visible; [`Pane`] focus controls which list gets the
//!   `REVERSED` highlight and which list the arrow keys affect.
//! * [`View::Detail`] — header + (item ASCII | item body) + footer (no hero).

use crate::app::{App, Pane, View};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

const HERO_HEIGHT: u16 = 10;
const HEADER_HEIGHT: u16 = 5;
const FOOTER_HEIGHT: u16 = 1;
const NAV_WIDTH: u16 = 22;

pub fn render(frame: &mut Frame, app: &mut App, hero_ascii: Option<&Text<'_>>) {
    match app.view {
        View::Menu { pane } => render_menu_view(frame, app, hero_ascii, pane),
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

fn render_hero(frame: &mut Frame, area: Rect, hero_ascii: Option<&Text<'_>>) {
    let widget = if let Some(text) = hero_ascii {
        Paragraph::new(text.clone()).block(Block::bordered().title("// HERO"))
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

fn focus_style(focused: bool) -> Style {
    if focused {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default()
    }
}

fn render_section_pane(frame: &mut Frame, area: Rect, app: &mut App, focused: bool) {
    let items: Vec<ListItem> = app
        .sections
        .iter()
        .map(|s| ListItem::new(s.title()))
        .collect();
    let list = List::new(items)
        .block(Block::bordered().title("// NAV"))
        .highlight_style(focus_style(focused))
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, area, &mut app.section_list_state);
}

fn render_item_pane(frame: &mut Frame, area: Rect, app: &mut App, focused: bool) {
    let section = app.current_section();
    let items = crate::content::items_for(section);
    let list_items: Vec<ListItem> = items.iter().map(|i| ListItem::new(i.title)).collect();
    let widget = List::new(list_items)
        .block(Block::bordered().title(format!("// {}", section.title())))
        .highlight_style(focus_style(focused))
        .highlight_symbol("> ");
    frame.render_stateful_widget(widget, area, &mut app.item_list_state);
}

fn render_footer(frame: &mut Frame, area: Rect, view: View) {
    let text = match view {
        View::Menu {
            pane: Pane::Section,
        } => "UP/DN NAV  ENTER/RIGHT ITEMS  I IMAGE  Q QUIT",
        View::Menu { pane: Pane::Item } => {
            "UP/DN NAV  ENTER OPEN  BKSP/LEFT BACK  I IMAGE  Q QUIT"
        }
        View::Detail { .. } => "BKSP/ESC BACK  I IMAGE  Q QUIT",
    };
    frame.render_widget(Paragraph::new(text).alignment(Alignment::Left), area);
}

fn render_menu_view(frame: &mut Frame, app: &mut App, hero_ascii: Option<&Text<'_>>, pane: Pane) {
    let [hero, header, content, footer] = root_with_hero(frame.size());
    render_hero(frame, hero, hero_ascii);
    render_header(frame, header);
    let [nav, items] = split_content_nav(content);
    render_section_pane(frame, nav, app, pane == Pane::Section);
    render_item_pane(frame, items, app, pane == Pane::Item);
    render_footer(frame, footer, app.view);
}

fn render_detail_view(frame: &mut Frame, app: &mut App, section_idx: usize, item_idx: usize) {
    let [header, content, footer] = root_no_hero(frame.size());
    render_header(frame, header);

    let items = crate::content::items_for(app.sections[section_idx]);
    if let Some(item) = items.get(item_idx) {
        let [left, right] = split_detail_halves(content);

        let ascii_text = pick_detail_ascii(item, crate::content::HERO_ASCII, crate::content::HERO_PNG)
            .unwrap_or_else(|| Text::from("[ NO IMAGE ]"));
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

pub(crate) fn pick_detail_ascii(
    item: &crate::content::Item,
    hero_ascii: Option<&'static str>,
    hero_png: Option<&[u8]>,
) -> Option<Text<'static>> {
    if let Some(art) = item.ascii {
        return Some(Text::raw(art));
    }
    if let Some(b) = item.png_bytes {
        if let Some(t) = crate::renderer::ascii::load_and_render_bytes(b, 36, 14) {
            return Some(t);
        }
    }
    if let Some(art) = hero_ascii {
        return Some(Text::raw(art));
    }
    if let Some(b) = hero_png {
        if let Some(t) = crate::renderer::ascii::load_and_render_bytes(b, 36, 14) {
            return Some(t);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Section;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use ratatui::style::{Color, Modifier};
    use ratatui::Terminal;

    fn render_view(width: u16, height: u16, app: &mut App, hero: Option<&Text<'_>>) -> Buffer {
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
    fn menu_view_contains_morisoba_name() {
        let mut a = App::new();
        let buf = render_view(120, 40, &mut a, None);
        assert!(buffer_contains(&buf, "MORISOBA"));
    }

    #[test]
    fn menu_view_contains_all_section_titles() {
        let mut a = App::new();
        let buf = render_view(120, 40, &mut a, None);
        for title in ["ABOUT", "PROJECTS", "EXPERIENCE", "CONTACT"] {
            assert!(buffer_contains(&buf, title), "missing {title}");
        }
    }

    #[test]
    fn menu_view_footer_has_quit_hint() {
        let mut a = App::new();
        let buf = render_view(120, 40, &mut a, None);
        assert!(buffer_contains(&buf, "Q QUIT"));
    }

    #[test]
    fn menu_view_footer_has_image_hint() {
        let mut a = App::new();
        let buf = render_view(120, 40, &mut a, None);
        assert!(buffer_contains(&buf, "I IMAGE"));
    }

    #[test]
    fn menu_view_no_foreground_color() {
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
    fn menu_view_no_background_color() {
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
    fn menu_view_section_pane_focused_about_row_has_reversed_modifier() {
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
        assert!(found, "focused section row (ABOUT) should be REVERSED");
    }

    #[test]
    fn menu_view_shows_items_of_selected_section_immediately_without_drilling() {
        let mut a = App::new();
        let buf = render_view(120, 40, &mut a, None);
        let about_items = crate::content::items_for(Section::About);
        for item in about_items {
            assert!(
                buffer_contains(&buf, item.title),
                "items pane must show About item titles on first paint (no drill required): missing {}",
                item.title,
            );
        }
    }

    #[test]
    fn menu_view_right_pane_title_reflects_current_section() {
        let mut a = App::new();
        let buf = render_view(120, 40, &mut a, None);
        assert!(
            buffer_contains(&buf, "// ABOUT"),
            "right pane block title should say // ABOUT when ABOUT is selected"
        );
    }

    #[test]
    fn menu_view_section_navigation_updates_right_pane_live() {
        let mut a = App::new();
        a.handle_key(crossterm::event::KeyCode::Down);
        let buf = render_view(120, 40, &mut a, None);
        assert!(
            buffer_contains(&buf, "// PROJECTS"),
            "right pane title should switch to // PROJECTS after Down arrow"
        );
        let projects = crate::content::items_for(Section::Projects);
        for item in projects {
            assert!(
                buffer_contains(&buf, item.title),
                "right pane should show Projects items live: missing {}",
                item.title,
            );
        }
    }

    #[test]
    fn menu_view_item_pane_focused_first_item_has_reversed_modifier() {
        let mut a = App::new();
        a.handle_key(crossterm::event::KeyCode::Enter);
        let buf = render_view(120, 40, &mut a, None);
        let about_items = crate::content::items_for(Section::About);
        let needle = about_items[0].title;
        let mut found = false;
        for y in 0..buf.area.height {
            let line = row_text(&buf, y);
            if !line.contains(needle) {
                continue;
            }
            if (0..buf.area.width).any(|x| buf.get(x, y).modifier.contains(Modifier::REVERSED)) {
                found = true;
                break;
            }
        }
        assert!(
            found,
            "after focus shift to item pane, first item row should be REVERSED"
        );
    }

    #[test]
    fn menu_view_item_pane_focus_footer_has_back_hint() {
        let mut a = App::new();
        a.handle_key(crossterm::event::KeyCode::Enter);
        let buf = render_view(120, 40, &mut a, None);
        assert!(buffer_contains(&buf, "BACK"));
    }

    #[test]
    fn menu_view_section_pane_focus_footer_has_no_back_hint() {
        let mut a = App::new();
        let buf = render_view(120, 40, &mut a, None);
        assert!(
            !buffer_contains(&buf, "BACK"),
            "section pane footer should not advertise BACK (top of nav stack)"
        );
    }

    #[test]
    fn menu_view_renders_at_80x24() {
        let mut a = App::new();
        let _ = render_view(80, 24, &mut a, None);
    }

    #[test]
    fn menu_view_renders_at_160x50() {
        let mut a = App::new();
        let _ = render_view(160, 50, &mut a, None);
    }

    #[test]
    fn menu_view_renders_with_hero_text() {
        let mut a = App::new();
        let hero = Text::raw("######\n@@@@@@\n......");
        let _ = render_view(120, 40, &mut a, Some(&hero));
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
        assert!(buffer_contains(&buf, "BACK"));
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

    fn mk_item(
        png_bytes: Option<&'static [u8]>,
        ascii: Option<&'static str>,
    ) -> crate::content::Item {
        crate::content::Item {
            slug: "test",
            title: "TEST",
            body: "test body",
            png_bytes,
            ascii,
        }
    }

    fn text_plain(t: &Text) -> String {
        let mut s = String::new();
        for (i, line) in t.lines.iter().enumerate() {
            if i > 0 {
                s.push('\n');
            }
            for span in &line.spans {
                s.push_str(&span.content);
            }
        }
        s
    }

    #[test]
    fn pick_detail_ascii_prefers_item_ascii_over_item_png() {
        let item = mk_item(Some(b"NOT_A_REAL_PNG"), Some("PRE_RENDERED"));
        let result = pick_detail_ascii(&item, None, None);
        assert_eq!(text_plain(&result.unwrap()), "PRE_RENDERED");
    }

    #[test]
    fn pick_detail_ascii_falls_back_to_decoded_item_png_when_no_ascii() {
        let real_png: &[u8] = include_bytes!("../assets/hero.png");
        let item = mk_item(Some(real_png), None);
        let result = pick_detail_ascii(&item, None, None);
        assert!(result.is_some(), "decoded item PNG should produce ASCII");
    }

    #[test]
    fn pick_detail_ascii_falls_back_to_hero_ascii_when_item_empty() {
        let item = mk_item(None, None);
        let result = pick_detail_ascii(&item, Some("HERO_FALLBACK"), None);
        assert_eq!(text_plain(&result.unwrap()), "HERO_FALLBACK");
    }

    #[test]
    fn pick_detail_ascii_falls_back_to_decoded_hero_png_when_no_ascii_at_all() {
        let real_png: &[u8] = include_bytes!("../assets/hero.png");
        let item = mk_item(None, None);
        let result = pick_detail_ascii(&item, None, Some(real_png));
        assert!(result.is_some(), "decoded hero PNG should produce ASCII");
    }

    #[test]
    fn pick_detail_ascii_returns_none_when_nothing_available() {
        let item = mk_item(None, None);
        let result = pick_detail_ascii(&item, None, None);
        assert!(result.is_none());
    }
}
