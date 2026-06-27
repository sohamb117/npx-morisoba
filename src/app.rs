//! Application state: which section is selected, what render mode the terminal supports,
//! and the canonical list of portfolio sections.

use crate::renderer::RenderMode;
use crossterm::event::KeyCode;
use ratatui::widgets::ListState;

/// The four portfolio sections rendered in the nav.
///
/// Section copy lives in `ui.rs` so callers can change the order here without
/// touching content blobs; the `title()` strings are stable for tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    About,
    Projects,
    Experience,
    Contact,
}

impl Section {
    /// Display title (ALL CAPS, brutalist) — surfaced in the nav widget.
    pub fn title(&self) -> &'static str {
        match self {
            Section::About => "ABOUT",
            Section::Projects => "PROJECTS",
            Section::Experience => "EXPERIENCE",
            Section::Contact => "CONTACT",
        }
    }
}

/// Result of dispatching a key press through `App::handle_key`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Main loop should exit cleanly.
    Quit,
    /// User pressed Enter on a section — currently a no-op but reserved
    /// for future "open" gestures (e.g. detail view).
    Select,
    /// Nothing observable happened (or selection changed via internal state).
    Noop,
}

/// Top-level application state owned by `main()`.
#[allow(dead_code)]
pub struct App {
    pub selected_section: usize,
    pub sections: Vec<Section>,
    pub mode: RenderMode,
    pub list_state: ListState,
}

impl App {
    pub fn new(mode: RenderMode) -> Self {
        let sections = vec![
            Section::About,
            Section::Projects,
            Section::Experience,
            Section::Contact,
        ];
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            selected_section: 0,
            sections,
            mode,
            list_state,
        }
    }

    pub fn next(&mut self) {
        let len = self.sections.len();
        self.selected_section = (self.selected_section + 1) % len;
        self.list_state.select(Some(self.selected_section));
    }

    pub fn prev(&mut self) {
        let len = self.sections.len();
        self.selected_section = (self.selected_section + len - 1) % len;
        self.list_state.select(Some(self.selected_section));
    }

    pub fn handle_key(&mut self, code: KeyCode) -> Action {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            KeyCode::Down | KeyCode::Char('j') => {
                self.next();
                Action::Noop
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.prev();
                Action::Noop
            }
            KeyCode::Enter => Action::Select,
            _ => Action::Noop,
        }
    }

    pub fn current_section(&self) -> Section {
        self.sections[self.selected_section]
    }
}

// T04 (Wave 2) - RED tests against the unimplemented stub.
// These panic until T10 supplies real bodies; that panic IS the RED signal.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::RenderMode;
    use crossterm::event::KeyCode;

    fn fresh() -> App {
        App::new(RenderMode::Ascii)
    }

    #[test]
    fn sections_contains_all_four_variants() {
        let a = fresh();
        assert_eq!(a.sections.len(), 4);
        assert!(a.sections.contains(&Section::About));
        assert!(a.sections.contains(&Section::Projects));
        assert!(a.sections.contains(&Section::Experience));
        assert!(a.sections.contains(&Section::Contact));
    }

    #[test]
    fn new_starts_at_zero() {
        let a = fresh();
        assert_eq!(a.selected_section, 0);
        assert_eq!(a.list_state.selected(), Some(0));
    }

    #[test]
    fn next_advances_by_one() {
        let mut a = fresh();
        a.next();
        assert_eq!(a.selected_section, 1);
        assert_eq!(a.list_state.selected(), Some(1));
    }

    #[test]
    fn next_wraps_to_zero_past_end() {
        let mut a = fresh();
        let n = a.sections.len();
        for _ in 0..n {
            a.next();
        }
        assert_eq!(a.selected_section, 0);
    }

    #[test]
    fn prev_wraps_to_last_from_zero() {
        let mut a = fresh();
        a.prev();
        assert_eq!(a.selected_section, 3);
    }

    #[test]
    fn handle_key_q_returns_quit() {
        let mut a = fresh();
        assert_eq!(a.handle_key(KeyCode::Char('q')), Action::Quit);
    }

    #[test]
    fn handle_key_esc_returns_quit() {
        let mut a = fresh();
        assert_eq!(a.handle_key(KeyCode::Esc), Action::Quit);
    }

    #[test]
    fn handle_key_down_and_j_both_advance() {
        let mut a = fresh();
        assert_eq!(a.handle_key(KeyCode::Down), Action::Noop);
        assert_eq!(a.selected_section, 1);
        a.handle_key(KeyCode::Char('j'));
        assert_eq!(a.selected_section, 2);
    }

    #[test]
    fn handle_key_up_and_k_both_retreat() {
        let mut a = fresh();
        a.handle_key(KeyCode::Up);
        assert_eq!(a.selected_section, 3);
        a.handle_key(KeyCode::Char('k'));
        assert_eq!(a.selected_section, 2);
    }

    #[test]
    fn handle_key_enter_returns_select() {
        let mut a = fresh();
        assert_eq!(a.handle_key(KeyCode::Enter), Action::Select);
    }

    #[test]
    fn handle_key_unknown_returns_noop() {
        let mut a = fresh();
        assert_eq!(a.handle_key(KeyCode::Char('z')), Action::Noop);
        assert_eq!(a.selected_section, 0);
    }

    #[test]
    fn current_section_reflects_selected_index() {
        let mut a = fresh();
        assert_eq!(a.current_section(), Section::About);
        a.next();
        assert_eq!(a.current_section(), Section::Projects);
        a.next();
        assert_eq!(a.current_section(), Section::Experience);
        a.next();
        assert_eq!(a.current_section(), Section::Contact);
    }

    #[test]
    fn section_titles_are_all_caps_and_distinct() {
        let titles = [
            Section::About.title(),
            Section::Projects.title(),
            Section::Experience.title(),
            Section::Contact.title(),
        ];
        for t in titles {
            assert_eq!(t, t.to_uppercase(), "section title not all-caps: {t}");
        }
        let mut sorted = titles.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), titles.len(), "section titles not distinct");
    }
}
