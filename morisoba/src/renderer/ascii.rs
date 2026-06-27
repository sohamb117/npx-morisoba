//! ASCII art renderer — thin wrapper around the [`rascii_art`] crate.
//!
//! All asset bytes are embedded at compile time via [`crate::content`]; this
//! module never touches the filesystem at runtime. Image decode + ramp mapping
//! is delegated to [`rascii_art`]; we layer a 24-level grayscale FG on top via
//! the 256-color palette (codes 232..=255) so the brutalist 10-char ramp gets
//! smooth luminance gradients.

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};

/// Brutalist 10-step luminance ramp: space = blackest, '@' = whitest.
pub const RAMP: [char; 10] = [' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

const RAMP_STR: [&str; 10] = [" ", ".", ":", "-", "=", "+", "*", "#", "%", "@"];

/// Decode PNG/JPEG bytes via [`rascii_art`] and wrap each output character in
/// a ratatui [`Span`] tinted with grayscale FG (256-color palette codes
/// 232..=255). Aspect-ratio + resize handled inside `rascii_art`. Returns
/// `None` on any decode failure — never panics.
pub fn load_and_render_bytes(bytes: &[u8], max_w: u32, max_h: u32) -> Option<Text<'static>> {
    const MAX_BYTES: usize = 10 * 1024 * 1024;
    const MAX_DIM: u32 = 2048;

    if bytes.len() > MAX_BYTES {
        return None;
    }
    let mut reader = image::io::Reader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let mut limits = image::io::Limits::default();
    limits.max_image_width = Some(MAX_DIM);
    limits.max_image_height = Some(MAX_DIM);
    reader.limits(limits);
    let img = reader.decode().ok()?;

    let (target_w, target_h) = fit_within_cells(img.width(), img.height(), max_w, max_h);

    let mut buf = String::new();
    let opts = rascii_art::RenderOptions::new()
        .width(target_w)
        .height(target_h)
        .colored(false)
        .charset(&RAMP_STR);
    rascii_art::render_image_to(&img, &mut buf, &opts).ok()?;

    Some(grayscale_text(&buf))
}

/// Compute aspect-preserving cell dimensions that fit within `max_w × max_h`,
/// accounting for the ~2:1 terminal cell ratio (1 cell ≈ 2 vertical pixels).
/// rascii_art stretches when both `.width` and `.height` are set, so we pass
/// it dimensions that already preserve the source aspect.
fn fit_within_cells(src_w: u32, src_h: u32, max_w: u32, max_h: u32) -> (u32, u32) {
    let src_w = (src_w.max(1)) as u64;
    let src_h = (src_h.max(1)) as u64;
    let max_w_u64 = max_w as u64;
    let max_h_u64 = max_h as u64;
    let width_limited = max_w_u64 * src_h <= max_h_u64 * 2 * src_w;
    if width_limited {
        let th = ((max_w_u64 * src_h) / (src_w * 2)).max(1);
        (max_w, th.min(max_h_u64) as u32)
    } else {
        let tw = ((max_h_u64 * 2 * src_w) / src_h).max(1);
        (tw.min(max_w_u64) as u32, max_h)
    }
}

fn grayscale_text(s: &str) -> Text<'static> {
    let lines: Vec<Line<'static>> = s
        .lines()
        .map(|line| {
            let spans: Vec<Span<'static>> = line
                .chars()
                .map(|c| {
                    let idx = RAMP.iter().position(|&rc| rc == c).unwrap_or(0);
                    let grey = ((idx * 23) / (RAMP.len() - 1).max(1)) as u8;
                    let style = Style::default().fg(Color::Indexed(232 + grey));
                    Span::styled(RAMP_STR[idx], style)
                })
                .collect();
            Line::from(spans)
        })
        .collect();
    Text::from(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ramp_ordering_is_dark_to_light() {
        assert_eq!(RAMP, [' ', '.', ':', '-', '=', '+', '*', '#', '%', '@']);
    }

    #[test]
    fn ramp_has_ten_entries() {
        assert_eq!(RAMP.len(), 10);
    }

    #[test]
    fn load_garbage_bytes_returns_none() {
        let bytes = b"not a valid png or jpeg payload";
        assert!(load_and_render_bytes(bytes, 10, 10).is_none());
    }

    #[test]
    fn load_empty_bytes_returns_none() {
        assert!(load_and_render_bytes(&[], 10, 10).is_none());
    }

    #[test]
    fn load_oversize_bytes_returns_none() {
        let huge = vec![0u8; 11 * 1024 * 1024];
        assert!(load_and_render_bytes(&huge, 10, 10).is_none());
    }

    #[test]
    fn real_hero_png_renders_with_grayscale_via_rascii_art() {
        let hero: &[u8] = include_bytes!("../../../assets/hero.png");
        let txt = load_and_render_bytes(hero, 36, 14).expect("hero should render");
        assert!(!txt.lines.is_empty(), "expected non-empty Text");
        let any_grayscale = txt.lines.iter().any(|l| {
            l.spans.iter().any(|s| {
                matches!(s.style.fg, Some(Color::Indexed(n)) if (232..=255).contains(&n))
            })
        });
        assert!(any_grayscale, "expected at least one grayscale Indexed span");
    }

    #[test]
    fn grayscale_text_space_char_maps_to_darkest_grey_232() {
        let txt = grayscale_text(" ");
        let span = &txt.lines[0].spans[0];
        assert_eq!(span.style.fg, Some(Color::Indexed(232)));
    }

    #[test]
    fn grayscale_text_at_char_maps_to_brightest_grey_255() {
        let txt = grayscale_text("@");
        let span = &txt.lines[0].spans[0];
        assert_eq!(span.style.fg, Some(Color::Indexed(255)));
    }
}
