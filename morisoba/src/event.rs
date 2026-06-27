#[cfg(not(target_arch = "wasm32"))]
pub use crossterm::event::KeyCode as KeyCode;

#[cfg(target_arch = "wasm32")]
pub use ratzilla::event::KeyCode as KeyCode;
