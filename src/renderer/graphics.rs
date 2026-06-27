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
#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn validate_hero_path(path: &Path) -> bool {
    const MAX_BYTES: u64 = 10 * 1024 * 1024;
    const MAX_DIM: u32 = 2048;

    let meta = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return false,
    };
    if !meta.is_file() || meta.len() > MAX_BYTES {
        return false;
    }
    let Ok(reader) = image::ImageReader::open(path) else {
        return false;
    };
    let Ok(mut reader) = reader.with_guessed_format() else {
        return false;
    };
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_DIM);
    limits.max_image_height = Some(MAX_DIM);
    reader.limits(limits);
    match reader.into_dimensions() {
        Ok((w, h)) => w <= MAX_DIM && h <= MAX_DIM,
        Err(_) => false,
    }
}

#[allow(dead_code)]
pub fn draw_image(path: &Path, rect: Rect) -> std::io::Result<()> {
    if !validate_hero_path(path) {
        return Err(std::io::Error::other(
            "hero image missing, too large, or not a regular file",
        ));
    }
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

    #[test]
    fn validate_hero_path_missing_returns_false() {
        let p = std::path::Path::new("/this/path/does/not/exist/__missing_hero_x.png");
        assert!(!validate_hero_path(p));
    }

    #[test]
    fn validate_hero_path_directory_returns_false() {
        let p = std::env::temp_dir();
        assert!(!validate_hero_path(&p));
    }

    #[test]
    fn validate_hero_path_valid_small_png_returns_true() {
        let tmp = std::env::temp_dir().join("ssh_tui_test_valid_small.png");
        let img = image::ImageBuffer::<image::Rgb<u8>, _>::from_pixel(
            16,
            16,
            image::Rgb([0u8, 0, 0]),
        );
        img.save(&tmp).expect("write valid test png");
        let ok = validate_hero_path(&tmp);
        let _ = std::fs::remove_file(&tmp);
        assert!(ok, "valid 16x16 PNG within all limits must validate true");
    }

    #[test]
    fn validate_hero_path_oversized_dimension_returns_false() {
        let tmp = std::env::temp_dir().join("ssh_tui_test_oversized_dim.png");
        let img = image::ImageBuffer::<image::Rgb<u8>, _>::from_pixel(
            2100,
            2100,
            image::Rgb([0u8, 0, 0]),
        );
        img.save(&tmp).expect("write oversized test png");
        let ok = validate_hero_path(&tmp);
        let _ = std::fs::remove_file(&tmp);
        assert!(
            !ok,
            "2100x2100 PNG must be rejected by 2048 dimension cap"
        );
    }
}
