//! WASM entry point for morisoba. Wires the shared app/render/animator logic
//! from the `morisoba` lib into ratzilla's DomBackend so the brutalist TUI
//! runs inside a browser DOM element.

use std::cell::RefCell;
use std::rc::Rc;

use morisoba::app::{Action, App};
use morisoba::ui;
use ratatui::Terminal;
use ratzilla::event::{KeyCode as ZKeyCode, KeyEvent as ZKeyEvent};
use ratzilla::{DomBackend, WebRenderer};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "morisobaDesktop"])]
    pub fn spawn_image(bytes: &[u8], slug: &str);
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let backend = DomBackend::new_by_id("tui-mount")
        .map_err(|e| JsValue::from_str(&format!("DomBackend: {e:?}")))?;
    let mut terminal = Terminal::new(backend)
        .map_err(|e| JsValue::from_str(&format!("Terminal: {e}")))?;

    let app = Rc::new(RefCell::new(App::new()));
    let start_ms = web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0);

    let app_for_keys = app.clone();
    terminal
        .on_key_event(move |key_event: ZKeyEvent| {
            let mapped = translate_key(key_event.code);
            if let Some(code) = mapped {
                let action = app_for_keys.borrow_mut().handle_key(code);
                match action {
                    Action::OpenImage => {
                        let bytes_owned: Option<(Vec<u8>, String)> = {
                            let a = app_for_keys.borrow();
                            a.image_to_open().map(|b| {
                                let slug = a
                                    .current_item()
                                    .map(|i| i.slug.to_string())
                                    .unwrap_or_else(|| "image".to_string());
                                (b.to_vec(), slug)
                            })
                        };
                        if let Some((bytes, slug)) = bytes_owned {
                            spawn_image(&bytes, &slug);
                        }
                    }
                    Action::Quit | Action::Drill | Action::Back | Action::Noop => {}
                }
            }
        })
        .map_err(|e| JsValue::from_str(&format!("on_key_event: {e}")))?;

    let app_for_draw = app.clone();
    terminal.draw_web(move |f| {
        let now = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);
        let tick_ms = (now - start_ms).max(0.0) as u64;
        let mut a = app_for_draw.borrow_mut();
        ui::render(f, &mut a, None, tick_ms);
    });

    Ok(())
}

fn translate_key(code: ZKeyCode) -> Option<morisoba::event::KeyCode> {
    use morisoba::event::KeyCode as MKeyCode;
    Some(match code {
        ZKeyCode::Char(c) => MKeyCode::Char(c),
        ZKeyCode::Enter => MKeyCode::Enter,
        ZKeyCode::Backspace => MKeyCode::Backspace,
        ZKeyCode::Esc => MKeyCode::Esc,
        ZKeyCode::Left => MKeyCode::Left,
        ZKeyCode::Right => MKeyCode::Right,
        ZKeyCode::Up => MKeyCode::Up,
        ZKeyCode::Down => MKeyCode::Down,
        ZKeyCode::Tab => MKeyCode::Tab,
        ZKeyCode::Delete => MKeyCode::Delete,
        ZKeyCode::Home => MKeyCode::Home,
        ZKeyCode::End => MKeyCode::End,
        ZKeyCode::PageUp => MKeyCode::PageUp,
        ZKeyCode::PageDown => MKeyCode::PageDown,
        ZKeyCode::F(n) => MKeyCode::F(n),
        ZKeyCode::Unidentified => return None,
    })
}
