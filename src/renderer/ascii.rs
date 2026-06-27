//! ASCII art renderer. Pure pixel→character mapping plus an in-memory loader.
//!
//! All asset bytes are embedded at compile time via [`crate::content`]; this
//! module never touches the filesystem at runtime. Image decode preserves the
//! source aspect ratio (fits within the requested cell box) and emits a
//! grayscale-tinted [`Text`] so the brutalist 10-char ramp gets 24 luminance
//! buckets from the 256-color grayscale palette (codes 232..=255).

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};

/// Brutalist 10-step luminance ramp: space = blackest, '@' = whitest.
pub const RAMP: [char; 10] = [' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

const RAMP_STR: [&str; 10] = [" ", ".", ":", "-", "=", "+", "*", "#", "%", "@"];

/// Map a flat Luma8 buffer (`luma.len() == (w * h) as usize`) to an ASCII grid.
///
/// Steps the Y axis by 2 to compensate for the ~2:1 terminal cell aspect ratio.
/// Result has `(h + 1) / 2` lines and `w` columns per line, newline-terminated
/// each.
#[cfg(test)]
fn pixels_to_ascii(luma: &[u8], w: u32, h: u32) -> String {
    let w_usize = w as usize;
    let h_usize = h as usize;
    let line_count = h_usize.div_ceil(2);
    let mut out = String::with_capacity(line_count * (w_usize + 1));
    for y in (0..h_usize).step_by(2) {
        for x in 0..w_usize {
            let pixel = luma[y * w_usize + x] as usize;
            let idx = (pixel * RAMP.len() / 256).min(RAMP.len() - 1);
            out.push(RAMP[idx]);
        }
        out.push('\n');
    }
    out
}

/// Same logic as [`pixels_to_ascii`] but emits a [`Text`] with each character
/// wrapped in its own [`Span`] tinted via the 256-color grayscale palette
/// (codes 232..=255). The ramp character provides structural texture; the
/// 24-level grayscale FG carries the actual luminance for smooth gradients.
pub fn pixels_to_grayscale_text(luma: &[u8], w: u32, h: u32) -> Text<'static> {
    let w_usize = w as usize;
    let h_usize = h as usize;
    let line_count = h_usize.div_ceil(2);
    let mut lines: Vec<Line<'static>> = Vec::with_capacity(line_count);
    for y in (0..h_usize).step_by(2) {
        let mut spans: Vec<Span<'static>> = Vec::with_capacity(w_usize);
        for x in 0..w_usize {
            let pixel = luma[y * w_usize + x] as usize;
            let idx = (pixel * RAMP.len() / 256).min(RAMP.len() - 1);
            let grey = ((pixel * 24) / 256).min(23) as u8;
            let style = Style::default().fg(Color::Indexed(232 + grey));
            spans.push(Span::styled(RAMP_STR[idx], style));
        }
        lines.push(Line::from(spans));
    }
    Text::from(lines)
}

/// Decode embedded PNG/JPEG bytes and render to ASCII text that fits within
/// `max_w` columns × `max_h` rows **preserving the source aspect ratio**.
/// Returns `None` on any decode failure (corrupt bytes, exceeded limits) —
/// never panics. Output is a grayscale-tinted [`Text`].
pub fn load_and_render_bytes(bytes: &[u8], max_w: u32, max_h: u32) -> Option<Text<'static>> {
    const MAX_BYTES: usize = 10 * 1024 * 1024;
    const MAX_DIM: u32 = 2048;

    if bytes.len() > MAX_BYTES {
        return None;
    }
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_DIM);
    limits.max_image_height = Some(MAX_DIM);
    reader.limits(limits);
    let decoded = reader.decode().ok()?;
    let resized = decoded.resize(
        max_w,
        max_h.saturating_mul(2).max(1),
        image::imageops::FilterType::Lanczos3,
    );
    let gray = resized.to_luma8();
    let (w, h) = gray.dimensions();
    Some(pixels_to_grayscale_text(gray.as_raw(), w, h))
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
    fn all_zero_pixels_map_to_space() {
        let buf = [0u8; 16];
        let out = pixels_to_ascii(&buf, 4, 4);
        for line in out.lines() {
            assert!(
                line.chars().all(|c| c == ' '),
                "expected all spaces, got: {line:?}"
            );
        }
    }

    #[test]
    fn all_max_pixels_map_to_at_sign() {
        let buf = [255u8; 16];
        let out = pixels_to_ascii(&buf, 4, 4);
        for line in out.lines() {
            assert!(
                line.chars().all(|c| c == '@'),
                "expected all '@', got: {line:?}"
            );
        }
    }

    #[test]
    fn mid_grey_maps_to_middle_of_ramp() {
        let buf = [128u8; 4];
        let out = pixels_to_ascii(&buf, 2, 2);
        for line in out.lines() {
            assert!(
                line.chars().all(|c| c == '+'),
                "expected all '+', got: {line:?}"
            );
        }
    }

    #[test]
    fn output_height_is_input_height_div_two_due_to_step_by_2() {
        let buf = [0u8; 40];
        let out = pixels_to_ascii(&buf, 4, 10);
        assert_eq!(out.lines().count(), 5);
    }

    #[test]
    fn output_width_equals_input_width() {
        let buf = [0u8; 12];
        let out = pixels_to_ascii(&buf, 6, 2);
        let lines: Vec<&str> = out.lines().collect();
        assert!(
            lines.iter().all(|l| l.chars().count() == 6),
            "expected width 6, got: {lines:?}"
        );
    }

    #[test]
    fn load_garbage_bytes_returns_none() {
        let bytes = b"not a valid png or jpeg payload";
        assert!(
            load_and_render_bytes(bytes, 10, 10).is_none(),
            "garbage bytes must NOT panic and MUST return None"
        );
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
    fn grayscale_text_dims_match_div_two_height_and_full_width() {
        let buf = [0u8; 36 * 14];
        let txt = pixels_to_grayscale_text(&buf, 36, 14);
        assert_eq!(txt.lines.len(), 7);
        assert!(txt.lines.iter().all(|l| l.spans.len() == 36));
    }

    #[test]
    fn grayscale_text_black_pixel_uses_darkest_grey_232() {
        let buf = [0u8; 4];
        let txt = pixels_to_grayscale_text(&buf, 2, 2);
        let first_span = &txt.lines[0].spans[0];
        assert_eq!(first_span.style.fg, Some(Color::Indexed(232)));
    }

    #[test]
    fn grayscale_text_white_pixel_uses_brightest_grey_255() {
        let buf = [255u8; 4];
        let txt = pixels_to_grayscale_text(&buf, 2, 2);
        let first_span = &txt.lines[0].spans[0];
        assert_eq!(first_span.style.fg, Some(Color::Indexed(255)));
    }
}
