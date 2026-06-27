//! Application state: which view is active, which section/item is selected.

use crossterm::event::KeyCode;
use ratatui::widgets::ListState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    About,
    Projects,
    Experience,
    Contact,
}

impl Section {
    pub fn title(&self) -> &'static str {
        match self {
            Section::About => "ABOUT",
            Section::Projects => "PROJECTS",
            Section::Experience => "EXPERIENCE",
            Section::Contact => "CONTACT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Sections,
    Items { section_idx: usize },
    Detail { section_idx: usize, item_idx: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    OpenImage,
    Drill,
    Back,
    Noop,
}

pub struct App {
    pub view: View,
    pub sections: Vec<Section>,
    pub section_list_state: ListState,
    pub item_list_state: ListState,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let sections = vec![
            Section::About,
            Section::Projects,
            Section::Experience,
            Section::Contact,
        ];
        let mut section_list_state = ListState::default();
        section_list_state.select(Some(0));
        let mut item_list_state = ListState::default();
        item_list_state.select(Some(0));
        Self {
            view: View::Sections,
            sections,
            section_list_state,
            item_list_state,
        }
    }

    pub fn selected_section_idx(&self) -> usize {
        self.section_list_state.selected().unwrap_or(0)
    }

    pub fn current_section(&self) -> Section {
        let idx = match self.view {
            View::Sections => self.selected_section_idx(),
            View::Items { section_idx } | View::Detail { section_idx, .. } => section_idx,
        };
        self.sections[idx]
    }

    pub fn current_item(&self) -> Option<&'static crate::content::Item> {
        match self.view {
            View::Items { section_idx } => {
                let items = crate::content::items_for(self.sections[section_idx]);
                let idx = self.item_list_state.selected().unwrap_or(0);
                items.get(idx)
            }
            View::Detail {
                section_idx,
                item_idx,
            } => {
                let items = crate::content::items_for(self.sections[section_idx]);
                items.get(item_idx)
            }
            View::Sections => None,
        }
    }

    pub fn handle_key(&mut self, code: KeyCode) -> Action {
        match code {
            KeyCode::Char('q') => return Action::Quit,
            KeyCode::Char('i') => return Action::OpenImage,
            _ => {}
        }
        match self.view {
            View::Sections => self.handle_key_sections(code),
            View::Items { section_idx } => self.handle_key_items(code, section_idx),
            View::Detail { section_idx, .. } => self.handle_key_detail(code, section_idx),
        }
    }

    fn handle_key_sections(&mut self, code: KeyCode) -> Action {
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                cycle_next(&mut self.section_list_state, self.sections.len());
                Action::Noop
            }
            KeyCode::Up | KeyCode::Char('k') => {
                cycle_prev(&mut self.section_list_state, self.sections.len());
                Action::Noop
            }
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                let idx = self.selected_section_idx();
                if crate::content::items_for(self.sections[idx]).is_empty() {
                    return Action::Noop;
                }
                self.item_list_state.select(Some(0));
                self.view = View::Items { section_idx: idx };
                Action::Drill
            }
            _ => Action::Noop,
        }
    }

    fn handle_key_items(&mut self, code: KeyCode, section_idx: usize) -> Action {
        let item_count = crate::content::items_for(self.sections[section_idx]).len();
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                cycle_next(&mut self.item_list_state, item_count);
                Action::Noop
            }
            KeyCode::Up | KeyCode::Char('k') => {
                cycle_prev(&mut self.item_list_state, item_count);
                Action::Noop
            }
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                if item_count == 0 {
                    return Action::Noop;
                }
                let item_idx = self.item_list_state.selected().unwrap_or(0);
                self.view = View::Detail {
                    section_idx,
                    item_idx,
                };
                Action::Drill
            }
            KeyCode::Backspace | KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => {
                self.view = View::Sections;
                Action::Back
            }
            _ => Action::Noop,
        }
    }

    fn handle_key_detail(&mut self, code: KeyCode, section_idx: usize) -> Action {
        match code {
            KeyCode::Backspace | KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => {
                self.view = View::Items { section_idx };
                Action::Back
            }
            _ => Action::Noop,
        }
    }
}

fn cycle_next(state: &mut ListState, len: usize) {
    if len == 0 {
        return;
    }
    let i = state.selected().unwrap_or(0);
    state.select(Some((i + 1) % len));
}

fn cycle_prev(state: &mut ListState, len: usize) {
    if len == 0 {
        return;
    }
    let i = state.selected().unwrap_or(0);
    state.select(Some((i + len - 1) % len));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;

    fn fresh() -> App {
        App::new()
    }

    #[test]
    fn starts_in_sections_view_with_about_selected() {
        let a = fresh();
        assert!(matches!(a.view, View::Sections));
        assert_eq!(a.selected_section_idx(), 0);
        assert_eq!(a.current_section(), Section::About);
    }

    #[test]
    fn sections_view_arrows_cycle_sections() {
        let mut a = fresh();
        a.handle_key(KeyCode::Down);
        assert_eq!(a.selected_section_idx(), 1);
        a.handle_key(KeyCode::Down);
        a.handle_key(KeyCode::Down);
        a.handle_key(KeyCode::Down);
        assert_eq!(a.selected_section_idx(), 0);
        a.handle_key(KeyCode::Up);
        assert_eq!(a.selected_section_idx(), 3);
    }

    #[test]
    fn sections_view_jk_cycle_sections() {
        let mut a = fresh();
        a.handle_key(KeyCode::Char('j'));
        assert_eq!(a.selected_section_idx(), 1);
        a.handle_key(KeyCode::Char('k'));
        assert_eq!(a.selected_section_idx(), 0);
    }

    #[test]
    fn enter_in_sections_drills_to_items() {
        let mut a = fresh();
        a.handle_key(KeyCode::Down);
        let act = a.handle_key(KeyCode::Enter);
        assert_eq!(act, Action::Drill);
        assert!(matches!(a.view, View::Items { section_idx: 1 }));
    }

    #[test]
    fn right_arrow_in_sections_also_drills() {
        let mut a = fresh();
        let act = a.handle_key(KeyCode::Right);
        assert_eq!(act, Action::Drill);
        assert!(matches!(a.view, View::Items { section_idx: 0 }));
    }

    #[test]
    fn items_view_arrows_cycle_items() {
        let mut a = fresh();
        a.handle_key(KeyCode::Enter);
        let count = crate::content::items_for(a.current_section()).len();
        if count >= 2 {
            a.handle_key(KeyCode::Down);
            assert_eq!(a.item_list_state.selected(), Some(1));
        }
    }

    #[test]
    fn enter_in_items_drills_to_detail() {
        let mut a = fresh();
        a.handle_key(KeyCode::Enter);
        let act = a.handle_key(KeyCode::Enter);
        assert_eq!(act, Action::Drill);
        assert!(matches!(
            a.view,
            View::Detail {
                section_idx: 0,
                item_idx: 0
            }
        ));
    }

    #[test]
    fn backspace_from_items_returns_to_sections() {
        let mut a = fresh();
        a.handle_key(KeyCode::Enter);
        let act = a.handle_key(KeyCode::Backspace);
        assert_eq!(act, Action::Back);
        assert!(matches!(a.view, View::Sections));
    }

    #[test]
    fn esc_from_items_returns_to_sections() {
        let mut a = fresh();
        a.handle_key(KeyCode::Enter);
        let act = a.handle_key(KeyCode::Esc);
        assert_eq!(act, Action::Back);
        assert!(matches!(a.view, View::Sections));
    }

    #[test]
    fn h_from_items_returns_to_sections() {
        let mut a = fresh();
        a.handle_key(KeyCode::Enter);
        let act = a.handle_key(KeyCode::Char('h'));
        assert_eq!(act, Action::Back);
        assert!(matches!(a.view, View::Sections));
    }

    #[test]
    fn backspace_from_detail_returns_to_items() {
        let mut a = fresh();
        a.handle_key(KeyCode::Enter);
        a.handle_key(KeyCode::Enter);
        let act = a.handle_key(KeyCode::Backspace);
        assert_eq!(act, Action::Back);
        assert!(matches!(a.view, View::Items { section_idx: 0 }));
    }

    #[test]
    fn q_quits_from_any_view() {
        let mut a = fresh();
        assert_eq!(a.handle_key(KeyCode::Char('q')), Action::Quit);
        a.view = View::Items { section_idx: 1 };
        assert_eq!(a.handle_key(KeyCode::Char('q')), Action::Quit);
        a.view = View::Detail {
            section_idx: 1,
            item_idx: 0,
        };
        assert_eq!(a.handle_key(KeyCode::Char('q')), Action::Quit);
    }

    #[test]
    fn i_returns_open_image_from_any_view() {
        let mut a = fresh();
        assert_eq!(a.handle_key(KeyCode::Char('i')), Action::OpenImage);
        a.view = View::Detail {
            section_idx: 0,
            item_idx: 0,
        };
        assert_eq!(a.handle_key(KeyCode::Char('i')), Action::OpenImage);
    }

    #[test]
    fn unknown_key_returns_noop() {
        let mut a = fresh();
        assert_eq!(a.handle_key(KeyCode::Char('z')), Action::Noop);
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

    #[test]
    fn current_section_reflects_view() {
        let mut a = fresh();
        assert_eq!(a.current_section(), Section::About);
        a.handle_key(KeyCode::Down);
        assert_eq!(a.current_section(), Section::Projects);
        a.handle_key(KeyCode::Enter);
        assert_eq!(a.current_section(), Section::Projects);
    }
}
