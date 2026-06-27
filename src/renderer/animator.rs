//! Procedural moving ASCII art for pages without authored static assets.
//!
//! Each item whose `.ascii` and `.png_bytes` are both `None` gets one of three
//! generators dispatched by a stable hash of its slug — so each animated page
//! looks distinctive but reproducible across launches.
//!
//! All generators write into a `(w, h)` cell grid and emit a ratatui [`Text`]
//! with per-cell grayscale FG from the 256-color palette (codes 232..=255).
//! `tick_ms` is wall-clock milliseconds since app start (caller supplies it),
//! so animations advance at the loop poll cadence (currently 10 fps).

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};

const RAMP: [char; 10] = [' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];
const RAMP_STR: [&str; 10] = [" ", ".", ":", "-", "=", "+", "*", "#", "%", "@"];

fn slug_hash(slug: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    slug.hash(&mut h);
    h.finish()
}

/// XOR-fold the high and low halves of the slug hash and add the slug length
/// before the modulo. Similarly-prefixed slugs (`01-foo`, `01-bar`, …) all hit
/// the same SipHash-low-bit cluster and would otherwise collapse onto one
/// animator under a plain `hash % n`. Mixing keeps the dispatch balanced.
fn dispatch_index(slug: &str, n: usize) -> usize {
    let h = slug_hash(slug);
    let mixed = (h ^ (h >> 32)).wrapping_add(slug.len() as u64);
    (mixed as usize) % n.max(1)
}

fn cell_from_luma(luma: u8) -> Span<'static> {
    let idx = (luma as usize * RAMP.len() / 256).min(RAMP.len() - 1);
    let grey = ((idx * 23) / (RAMP.len() - 1).max(1)) as u8;
    let style = Style::default().fg(Color::Indexed(232 + grey));
    Span::styled(RAMP_STR[idx], style)
}

fn rows_to_text(rows: Vec<Vec<u8>>) -> Text<'static> {
    let lines: Vec<Line<'static>> = rows
        .into_iter()
        .map(|row| {
            let spans: Vec<Span<'static>> = row.into_iter().map(cell_from_luma).collect();
            Line::from(spans)
        })
        .collect();
    Text::from(lines)
}

pub fn animated_frame(slug: &str, w: u16, h: u16, tick_ms: u64) -> Text<'static> {
    let t = tick_ms as f64 / 1000.0;
    match dispatch_index(slug, 5) {
        0 => plasma(slug, w, h, t),
        1 => rain(slug, w, h, t),
        2 => sine(slug, w, h, t),
        3 => worms(slug, w, h, t),
        _ => cellular(slug, w, h, t),
    }
}

fn plasma(slug: &str, w: u16, h: u16, t: f64) -> Text<'static> {
    let phase = (slug_hash(slug) % 1000) as f64 / 100.0;
    let w_f = (w as f64).max(1.0);
    let h_f = (h as f64).max(1.0);
    let mut rows = Vec::with_capacity(h as usize);
    for y in 0..h {
        let yf = y as f64 / h_f * 6.0;
        let mut row = Vec::with_capacity(w as usize);
        for x in 0..w {
            let xf = x as f64 / w_f * 6.0;
            let v = ((xf + t + phase).sin()
                + (yf + t * 0.7).sin()
                + ((xf * xf + yf * yf).sqrt() + t).sin())
                / 3.0;
            let luma = ((v + 1.0) * 127.5).clamp(0.0, 255.0) as u8;
            row.push(luma);
        }
        rows.push(row);
    }
    rows_to_text(rows)
}

fn rain(slug: &str, w: u16, h: u16, t: f64) -> Text<'static> {
    let seed = slug_hash(slug);
    let mut rows = vec![vec![0u8; w as usize]; h as usize];
    let h_i = h as i32;
    for x in 0..w {
        let col_seed = seed.wrapping_add(x as u64 * 2_654_435_769);
        let speed = 2.0 + (col_seed % 5) as f64;
        let offset = (col_seed % 100) as f64 / 100.0 * h_i as f64;
        let head_y = ((t * speed + offset) % (h_i as f64 + 8.0)) as i32 - 4;
        let length = 4 + (col_seed % 6) as i32;
        for trail in 0..length {
            let py = head_y - trail;
            if py < 0 || py >= h_i {
                continue;
            }
            let brightness = 255u32.saturating_sub(trail as u32 * 255 / length.max(1) as u32);
            rows[py as usize][x as usize] = brightness as u8;
        }
    }
    rows_to_text(rows)
}

fn sine(slug: &str, w: u16, h: u16, t: f64) -> Text<'static> {
    let phase = (slug_hash(slug) % 1000) as f64 / 100.0;
    let w_f = (w as f64).max(1.0);
    let h_f = (h as f64).max(1.0);
    let mut rows = vec![vec![0u8; w as usize]; h as usize];
    for y in 0..h {
        let row_phase = y as f64 * 0.3 + phase;
        for x in 0..w {
            let xf = x as f64 / w_f * 4.0 * std::f64::consts::PI;
            let v = (xf + t * 2.0 + row_phase).sin() * 0.5 + 0.5;
            let center_dist = (y as f64 - h_f / 2.0).abs() / (h_f / 2.0).max(1.0);
            let envelope = (1.0 - center_dist).max(0.0);
            let luma = (v * envelope * 255.0).clamp(0.0, 255.0) as u8;
            rows[y as usize][x as usize] = luma;
        }
    }
    rows_to_text(rows)
}

/// Multiple Lissajous-style worms crawling the pane, each leaving a fading
/// grayscale trail. Per-worm freq/speed/phase are seeded from `slug + worm_idx`
/// so every animated page looks distinct but reproducible.
fn worms(slug: &str, w: u16, h: u16, t: f64) -> Text<'static> {
    let seed = slug_hash(slug);
    let n_worms = 3 + (seed % 4) as usize;
    let mut grid = vec![vec![0u8; w as usize]; h as usize];
    let w_i = w as i32;
    let h_i = h as i32;
    for i in 0..n_worms {
        let s = seed.wrapping_add(i as u64 * 0x9E37_79B9_7F4A_7C15);
        let trail_len = 12 + (s % 10) as usize;
        let speed_x = 0.3 + (s % 100) as f64 / 200.0;
        let speed_y = 0.4 + ((s / 100) % 100) as f64 / 200.0;
        let freq_x = 1.0 + (s % 4) as f64;
        let freq_y = 1.0 + ((s / 4) % 4) as f64;
        let phase = (s % 1000) as f64 / 100.0;
        for step in 0..trail_len {
            let local_t = t - step as f64 * 0.10;
            let nx = (local_t * speed_x * freq_x + phase).sin();
            let ny = (local_t * speed_y * freq_y).cos();
            let px = (w.saturating_sub(1) as f64) * 0.5 * (1.0 + nx * 0.95);
            let py = (h.saturating_sub(1) as f64) * 0.5 * (1.0 + ny * 0.95);
            let x = px.round() as i32;
            let y = py.round() as i32;
            if x < 0 || x >= w_i || y < 0 || y >= h_i {
                continue;
            }
            let brightness =
                255u32.saturating_sub(step as u32 * 255 / trail_len.max(1) as u32);
            let current = &mut grid[y as usize][x as usize];
            *current = (*current).max(brightness as u8);
        }
    }
    rows_to_text(grid)
}

/// Voronoi-style cell membranes that morph over time. Drifting centers give
/// an "organoid" look — bright pixels mark the boundary between two cells
/// (small `second_d - min_d`), dark interiors lie within a single cell.
/// The y distance is doubled to compensate for the ~2:1 terminal cell ratio
/// so the cells look round, not vertically squashed.
fn cellular(slug: &str, w: u16, h: u16, t: f64) -> Text<'static> {
    let seed = slug_hash(slug);
    let n_centers = 5 + (seed % 5) as usize;
    let w_f = (w as f64).max(1.0);
    let h_f = (h as f64).max(1.0);
    let centers: Vec<(f64, f64)> = (0..n_centers)
        .map(|i| {
            let s = seed.wrapping_add(i as u64 * 0xBF58_476D_1CE4_E5B9);
            let cx = (s % 1000) as f64 / 1000.0 * w_f;
            let cy = ((s / 1000) % 1000) as f64 / 1000.0 * h_f;
            let phase_x = ((s / 100) % 1000) as f64 / 100.0;
            let phase_y = ((s / 10_000) % 1000) as f64 / 100.0;
            let drift_x = (t * 0.3 + phase_x).sin() * (w_f / 12.0);
            let drift_y = (t * 0.4 + phase_y).cos() * (h_f / 8.0);
            (cx + drift_x, cy + drift_y)
        })
        .collect();
    let mut grid = vec![vec![0u8; w as usize]; h as usize];
    for y in 0..h {
        for x in 0..w {
            let py = y as f64 + 0.5;
            let px = x as f64 + 0.5;
            let mut min_d = f64::INFINITY;
            let mut second_d = f64::INFINITY;
            for &(cx, cy) in &centers {
                let dx = px - cx;
                let dy = (py - cy) * 2.0;
                let d = (dx * dx + dy * dy).sqrt();
                if d < min_d {
                    second_d = min_d;
                    min_d = d;
                } else if d < second_d {
                    second_d = d;
                }
            }
            let boundary_proximity =
                (1.0 - ((second_d - min_d).min(4.0) / 4.0)).clamp(0.0, 1.0);
            grid[y as usize][x as usize] = (boundary_proximity * 255.0) as u8;
        }
    }
    rows_to_text(grid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animated_frame_dims_match_request() {
        let t = animated_frame("test-slug", 20, 10, 0);
        assert_eq!(t.lines.len(), 10);
        assert!(t.lines.iter().all(|l| l.spans.len() == 20));
    }

    #[test]
    fn animated_frame_is_stable_for_same_inputs() {
        let a = animated_frame("foo", 10, 5, 500);
        let b = animated_frame("foo", 10, 5, 500);
        assert_eq!(a.lines.len(), b.lines.len());
        for (la, lb) in a.lines.iter().zip(b.lines.iter()) {
            for (sa, sb) in la.spans.iter().zip(lb.spans.iter()) {
                assert_eq!(sa.content, sb.content);
                assert_eq!(sa.style, sb.style);
            }
        }
    }

    #[test]
    fn animated_frame_differs_across_slugs() {
        let a = animated_frame("alpha", 20, 10, 100);
        let b = animated_frame("beta", 20, 10, 100);
        let identical = a.lines.iter().zip(b.lines.iter()).all(|(la, lb)| {
            la.spans
                .iter()
                .zip(lb.spans.iter())
                .all(|(sa, sb)| sa.content == sb.content)
        });
        assert!(!identical, "different slugs must produce different patterns");
    }

    #[test]
    fn animated_frame_advances_over_time() {
        let a = animated_frame("delta", 20, 10, 0);
        let b = animated_frame("delta", 20, 10, 2000);
        let identical = a.lines.iter().zip(b.lines.iter()).all(|(la, lb)| {
            la.spans
                .iter()
                .zip(lb.spans.iter())
                .all(|(sa, sb)| sa.content == sb.content)
        });
        assert!(!identical, "same slug at different ticks must produce different frames");
    }

    #[test]
    fn animated_frame_all_spans_use_grayscale_palette() {
        let t = animated_frame("epsilon", 30, 15, 1234);
        for line in &t.lines {
            for span in &line.spans {
                let Some(Color::Indexed(n)) = span.style.fg else {
                    panic!("expected Indexed grayscale, got {:?}", span.style.fg);
                };
                assert!((232..=255).contains(&n), "out-of-range palette code {n}");
            }
        }
    }

    #[test]
    fn animated_frame_handles_tiny_dimensions_without_panic() {
        let _ = animated_frame("zero", 1, 1, 100);
        let _ = animated_frame("zero", 2, 1, 200);
        let _ = animated_frame("zero", 1, 2, 300);
    }

    #[test]
    fn dispatch_covers_all_animators_across_actual_content_slugs() {
        let slugs = [
            "02-philosophy",
            "03-off-hours",
            "01-distributed-log-engine",
            "02-tui-framework-mk1",
            "03-kernel-trace-toolkit",
            "01-systems-corp",
            "02-rust-startup",
            "03-web-agency",
            "01-github",
            "02-email",
            "03-location",
        ];
        let mut counts = [0u32; 5];
        for slug in slugs {
            counts[dispatch_index(slug, 5)] += 1;
        }
        eprintln!("animator distribution across actual slugs: {:?}", counts);
        assert!(
            counts.iter().all(|&c| c >= 1),
            "every animator type must appear at least once across the actual slug set: got {:?}",
            counts
        );
    }
}
