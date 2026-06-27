//! Application state: which view, which pane has focus, current selections.

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
pub enum Pane {
    Section,
    Item,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Menu { pane: Pane },
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
            view: View::Menu {
                pane: Pane::Section,
            },
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
            View::Menu { .. } => self.selected_section_idx(),
            View::Detail { section_idx, .. } => section_idx,
        };
        self.sections[idx]
    }

    pub fn current_item(&self) -> Option<&'static crate::content::Item> {
        let (section_idx, item_idx) = match self.view {
            View::Menu { .. } => (
                self.selected_section_idx(),
                self.item_list_state.selected().unwrap_or(0),
            ),
            View::Detail {
                section_idx,
                item_idx,
            } => (section_idx, item_idx),
        };
        let items = crate::content::items_for(self.sections[section_idx]);
        items.get(item_idx)
    }

    /// PNG bytes to feed the `i` key. On section-pane focus there is no
    /// specific item context, so we return the landing hero. On every other
    /// view the highlighted/viewed item must have its OWN PNG — no hero
    /// fallback. This avoids the trap where every page appears to show the
    /// same image just because the user has not authored per-item art yet.
    pub fn image_to_open(&self) -> Option<&'static [u8]> {
        if matches!(
            self.view,
            View::Menu {
                pane: Pane::Section
            }
        ) {
            crate::content::HERO_PNG
        } else {
            self.current_item().and_then(|i| i.png_bytes)
        }
    }

    pub fn handle_key(&mut self, code: KeyCode) -> Action {
        match code {
            KeyCode::Char('q') => return Action::Quit,
            KeyCode::Char('i') => return Action::OpenImage,
            _ => {}
        }
        match self.view {
            View::Menu {
                pane: Pane::Section,
            } => self.handle_key_section_pane(code),
            View::Menu { pane: Pane::Item } => self.handle_key_item_pane(code),
            View::Detail { .. } => self.handle_key_detail(code),
        }
    }

    fn handle_key_section_pane(&mut self, code: KeyCode) -> Action {
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                cycle_next(&mut self.section_list_state, self.sections.len());
                self.item_list_state.select(Some(0));
                Action::Noop
            }
            KeyCode::Up | KeyCode::Char('k') => {
                cycle_prev(&mut self.section_list_state, self.sections.len());
                self.item_list_state.select(Some(0));
                Action::Noop
            }
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                let idx = self.selected_section_idx();
                if crate::content::items_for(self.sections[idx]).is_empty() {
                    return Action::Noop;
                }
                self.view = View::Menu { pane: Pane::Item };
                Action::Noop
            }
            _ => Action::Noop,
        }
    }

    fn handle_key_item_pane(&mut self, code: KeyCode) -> Action {
        let section_idx = self.selected_section_idx();
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
                self.view = View::Menu {
                    pane: Pane::Section,
                };
                Action::Back
            }
            _ => Action::Noop,
        }
    }

    fn handle_key_detail(&mut self, code: KeyCode) -> Action {
        match code {
            KeyCode::Backspace | KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => {
                self.view = View::Menu { pane: Pane::Item };
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
    fn starts_in_menu_view_with_section_pane_focused_and_about_selected() {
        let a = fresh();
        assert_eq!(
            a.view,
            View::Menu {
                pane: Pane::Section
            }
        );
        assert_eq!(a.selected_section_idx(), 0);
        assert_eq!(a.current_section(), Section::About);
    }

    #[test]
    fn section_pane_arrows_cycle_sections() {
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
    fn section_pane_jk_cycle_sections() {
        let mut a = fresh();
        a.handle_key(KeyCode::Char('j'));
        assert_eq!(a.selected_section_idx(), 1);
        a.handle_key(KeyCode::Char('k'));
        assert_eq!(a.selected_section_idx(), 0);
    }

    #[test]
    fn section_navigation_resets_item_selection_to_zero() {
        let mut a = fresh();
        a.item_list_state.select(Some(2));
        a.handle_key(KeyCode::Down);
        assert_eq!(a.item_list_state.selected(), Some(0));
    }

    #[test]
    fn enter_on_section_pane_shifts_focus_to_item_pane_without_drilling_to_detail() {
        let mut a = fresh();
        let act = a.handle_key(KeyCode::Enter);
        assert_eq!(act, Action::Noop);
        assert_eq!(a.view, View::Menu { pane: Pane::Item });
    }

    #[test]
    fn right_arrow_on_section_pane_also_shifts_focus() {
        let mut a = fresh();
        let act = a.handle_key(KeyCode::Right);
        assert_eq!(act, Action::Noop);
        assert_eq!(a.view, View::Menu { pane: Pane::Item });
    }

    #[test]
    fn l_on_section_pane_also_shifts_focus() {
        let mut a = fresh();
        let act = a.handle_key(KeyCode::Char('l'));
        assert_eq!(act, Action::Noop);
        assert_eq!(a.view, View::Menu { pane: Pane::Item });
    }

    #[test]
    fn item_pane_arrows_cycle_items_not_sections() {
        let mut a = fresh();
        a.handle_key(KeyCode::Enter);
        let section_before = a.selected_section_idx();
        let count = crate::content::items_for(a.current_section()).len();
        if count >= 2 {
            a.handle_key(KeyCode::Down);
            assert_eq!(a.item_list_state.selected(), Some(1));
            assert_eq!(a.selected_section_idx(), section_before);
        }
    }

    #[test]
    fn enter_on_item_pane_opens_detail() {
        let mut a = fresh();
        a.handle_key(KeyCode::Enter);
        let act = a.handle_key(KeyCode::Enter);
        assert_eq!(act, Action::Drill);
        assert_eq!(
            a.view,
            View::Detail {
                section_idx: 0,
                item_idx: 0
            }
        );
    }

    #[test]
    fn right_on_item_pane_also_opens_detail() {
        let mut a = fresh();
        a.handle_key(KeyCode::Enter);
        let act = a.handle_key(KeyCode::Right);
        assert_eq!(act, Action::Drill);
        assert!(matches!(a.view, View::Detail { .. }));
    }

    #[test]
    fn backspace_from_item_pane_returns_focus_to_section_pane() {
        let mut a = fresh();
        a.handle_key(KeyCode::Enter);
        let act = a.handle_key(KeyCode::Backspace);
        assert_eq!(act, Action::Back);
        assert_eq!(
            a.view,
            View::Menu {
                pane: Pane::Section
            }
        );
    }

    #[test]
    fn esc_from_item_pane_returns_focus_to_section_pane() {
        let mut a = fresh();
        a.handle_key(KeyCode::Enter);
        let act = a.handle_key(KeyCode::Esc);
        assert_eq!(act, Action::Back);
        assert_eq!(
            a.view,
            View::Menu {
                pane: Pane::Section
            }
        );
    }

    #[test]
    fn h_from_item_pane_returns_focus_to_section_pane() {
        let mut a = fresh();
        a.handle_key(KeyCode::Enter);
        let act = a.handle_key(KeyCode::Char('h'));
        assert_eq!(act, Action::Back);
        assert_eq!(
            a.view,
            View::Menu {
                pane: Pane::Section
            }
        );
    }

    #[test]
    fn backspace_from_detail_returns_to_item_pane_preserving_selection() {
        let mut a = fresh();
        a.handle_key(KeyCode::Enter);
        a.handle_key(KeyCode::Down);
        let item_before = a.item_list_state.selected();
        a.handle_key(KeyCode::Enter);
        assert!(matches!(a.view, View::Detail { .. }));
        let act = a.handle_key(KeyCode::Backspace);
        assert_eq!(act, Action::Back);
        assert_eq!(a.view, View::Menu { pane: Pane::Item });
        assert_eq!(a.item_list_state.selected(), item_before);
    }

    #[test]
    fn q_quits_from_any_view() {
        let mut a = fresh();
        assert_eq!(a.handle_key(KeyCode::Char('q')), Action::Quit);
        a.view = View::Menu { pane: Pane::Item };
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
        a.view = View::Menu { pane: Pane::Item };
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

    #[test]
    fn current_item_returns_highlighted_item_on_section_pane() {
        let a = fresh();
        let item = a.current_item().expect("first about item exists");
        let about_items = crate::content::items_for(Section::About);
        assert_eq!(item.title, about_items[0].title);
    }

    #[test]
    fn image_to_open_in_section_pane_focus_returns_hero_png() {
        let a = fresh();
        assert_eq!(
            a.view,
            View::Menu {
                pane: Pane::Section
            }
        );
        assert_eq!(a.image_to_open(), crate::content::HERO_PNG);
    }

    #[test]
    fn image_to_open_in_item_pane_focus_returns_highlighted_items_png_bytes() {
        let mut a = fresh();
        a.handle_key(KeyCode::Enter);
        assert_eq!(a.view, View::Menu { pane: Pane::Item });
        let first_about = &crate::content::items_for(Section::About)[0];
        assert_eq!(a.image_to_open(), first_about.png_bytes);
    }

    #[test]
    fn image_to_open_in_detail_view_returns_viewed_items_png_bytes() {
        let mut a = fresh();
        a.handle_key(KeyCode::Enter);
        a.handle_key(KeyCode::Enter);
        assert!(matches!(a.view, View::Detail { .. }));
        let first_about = &crate::content::items_for(Section::About)[0];
        assert_eq!(a.image_to_open(), first_about.png_bytes);
    }

    #[test]
    fn image_to_open_does_not_fall_back_to_hero_for_item_without_png() {
        let mut a = fresh();
        for _ in 0..3 {
            a.handle_key(KeyCode::Down);
        }
        a.handle_key(KeyCode::Enter);
        assert_eq!(a.current_section(), Section::Contact);
        let first_contact = &crate::content::items_for(Section::Contact)[0];
        assert!(
            first_contact.png_bytes.is_none(),
            "test fixture assumption: contact items have no PNG sibling"
        );
        assert!(
            a.image_to_open().is_none(),
            "must NOT fall back to HERO_PNG on item-pane focus"
        );
    }
}
