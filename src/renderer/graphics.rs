//! Graphics renderer: thin wrapper over [`viuer`] that maps a ratatui [`Rect`]
//! to a [`viuer::Config`] and draws an image at that position.
//!
//! Draw happens AFTER `terminal.draw(...)` returns; see `main.rs`.

use ratatui::layout::Rect;
use std::path::Path;
use viuer::Config;

/// Build a viuer Config that targets `rect` in absolute terminal coordinates.
///
/// Width and height are clamped one cell smaller than the Rect to avoid the
/// scroll-tear viuer triggers when the image bottom edge collides with the
/// terminal scroll region.
pub fn config_for_rect(rect: Rect) -> Config {
    Config {
        absolute_offset: true,
        x: rect.x,
        y: rect.y as i16,
        width: Some(rect.width.saturating_sub(1) as u32),
        height: Some(rect.height.saturating_sub(1) as u32),
        restore_cursor: true,
        ..Default::default()
    }
}

pub fn draw_image(path: &Path, rect: Rect) -> std::io::Result<()> {
    let config = config_for_rect(rect);
    viuer::print_from_file(path, &config)
        .map(|_| ())
        .map_err(|e| std::io::Error::other(e.to_string()))
}

// T08 (Wave 2) - RED tests for the pure Config builder. We don't exercise
// `draw_image` here because it touches stdout and requires an image file.
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn config_uses_absolute_offset_true() {
        let c = config_for_rect(Rect::new(0, 0, 80, 24));
        assert!(c.absolute_offset, "absolute_offset must be true for ratatui Rect alignment");
    }

    #[test]
    fn config_x_equals_rect_x() {
        let c = config_for_rect(Rect::new(7, 3, 40, 10));
        assert_eq!(c.x, 7);
    }

    #[test]
    fn config_y_equals_rect_y() {
        let c = config_for_rect(Rect::new(7, 3, 40, 10));
        assert_eq!(c.y, 3);
    }

    #[test]
    fn config_width_is_rect_width_minus_one() {
        let c = config_for_rect(Rect::new(0, 0, 40, 10));
        assert_eq!(c.width, Some(39u32), "width must be clamped one below rect to avoid scroll");
    }

    #[test]
    fn config_height_is_rect_height_minus_one() {
        let c = config_for_rect(Rect::new(0, 0, 40, 10));
        assert_eq!(c.height, Some(9u32), "height must be clamped one below rect to avoid scroll");
    }

    #[test]
    fn config_restore_cursor_true() {
        let c = config_for_rect(Rect::new(0, 0, 80, 24));
        assert!(c.restore_cursor);
    }

    #[test]
    fn tiny_rect_yields_zero_dim_safely_via_saturating_sub() {
        let c = config_for_rect(Rect::new(0, 0, 0, 0));
        assert_eq!(c.width, Some(0u32));
        assert_eq!(c.height, Some(0u32));
    }
}
