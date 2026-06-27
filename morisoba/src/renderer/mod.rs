//! Renderer module: high-density ASCII image rendering for the hero region.

pub mod animator;

#[cfg(not(target_arch = "wasm32"))]
pub mod ascii;

