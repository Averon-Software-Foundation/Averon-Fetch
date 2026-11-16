use crate::colour::{paint, ColourDepth, Palette};
use crate::config::Config;
use crate::text::{char_len, pad_visible};

pub const LOGO_WIDTH: usize = 28;

pub const LOGO_HEIGHT: usize = 15;

pub const MIN_LOGO_WIDTH: usize = 10;

pub const MAX_LOGO_WIDTH: usize = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tone {
    Transparent,
    Primary,
    Accent,
}

#[derive(Debug, Clone, Copy)]
struct Segment {
    text: &'static str,
    tone: Tone,
}

const fn seg(text: &'static str, tone: Tone) -> Segment {
    Segment { text, tone }
}

const ROW_0: &[Segment] = &[
    seg("         ", Tone::Transparent),
    seg("▄▄▄▄▄▄", Tone::Primary),
    seg("             ", Tone::Transparent),
];
const ROW_1: &[Segment] = &[
    seg("       ", Tone::Transparent),
    seg("▄█████████▄▄", Tone::Primary),
    seg("         ", Tone::Transparent),
];
const ROW_2: &[Segment] = &[
    seg("      ", Tone::Transparent),
    seg("██████", Tone::Primary),
    seg("  ", Tone::Transparent),
    seg("████████", Tone::Primary),
    seg("      ", Tone::Transparent),
];
const ROW_3: &[Segment] = &[
    seg("     ", Tone::Transparent),
    seg("████", Tone::Primary),
    seg("      ", Tone::Transparent),
    seg("████████", Tone::Primary),
    seg("     ", Tone::Transparent),
];
const ROW_4: &[Segment] = &[
    seg("    ", Tone::Transparent),
    seg("████", Tone::Primary),
    seg("        ", Tone::Transparent),
    seg("▐", Tone::Primary),
    seg("████████", Tone::Accent),
    seg("   ", Tone::Transparent),
];
const ROW_5: &[Segment] = &[
    seg("   ", Tone::Transparent),
    seg("████", Tone::Primary),
    seg("           ", Tone::Transparent),
    seg("████████", Tone::Accent),
    seg("  ", Tone::Transparent),
];
const ROW_6: &[Segment] = &[
    seg("  ", Tone::Transparent),
    seg("████", Tone::Primary),
    seg("             ", Tone::Transparent),
    seg("████████", Tone::Accent),
    seg(" ", Tone::Transparent),
];
const ROW_7: &[Segment] = &[
    seg(" ", Tone::Transparent),
    seg("████", Tone::Primary),
    seg("               ", Tone::Transparent),
    seg("████████", Tone::Accent),
];
const ROW_8: &[Segment] = &[
    seg(" ", Tone::Transparent),
    seg("████", Tone::Primary),
    seg("▄▄▄▄▄▄▄▄▄▄▄▄▄▄", Tone::Primary),
    seg("████████", Tone::Accent),
    seg(" ", Tone::Transparent),
];
const ROW_9: &[Segment] = &[
    seg("  ", Tone::Transparent),
    seg("██████████████████", Tone::Primary),
    seg("██████", Tone::Accent),
    seg("  ", Tone::Transparent),
];
const ROW_10: &[Segment] = &[
    seg("   ", Tone::Transparent),
    seg("██████", Tone::Primary),
    seg("         ", Tone::Transparent),
    seg("████████", Tone::Accent),
    seg("  ", Tone::Transparent),
];
const ROW_11: &[Segment] = &[
    seg("    ", Tone::Transparent),
    seg("████", Tone::Primary),
    seg("           ", Tone::Transparent),
    seg("███████", Tone::Accent),
    seg("  ", Tone::Transparent),
];
const ROW_12: &[Segment] = &[
    seg("     ", Tone::Transparent),
    seg("██", Tone::Primary),
    seg("             ", Tone::Transparent),
    seg("██████", Tone::Accent),
    seg("  ", Tone::Transparent),
];
const ROW_13: &[Segment] = &[
    seg("                   ", Tone::Transparent),
    seg("█████", Tone::Accent),
    seg("    ", Tone::Transparent),
];
const ROW_14: &[Segment] = &[
    seg("                    ", Tone::Transparent),
    seg("████", Tone::Accent),
    seg("    ", Tone::Transparent),
];

const ROWS: &[&[Segment]] = &[
    ROW_0, ROW_1, ROW_2, ROW_3, ROW_4, ROW_5, ROW_6, ROW_7, ROW_8, ROW_9, ROW_10, ROW_11, ROW_12,
    ROW_13, ROW_14,
];

pub fn configured_width(config: &Config) -> usize {
    if let Some(custom) = &config.logo.custom_ascii {
        let width = custom.lines().map(char_len).max().unwrap_or(0);
        if width > 0 {
            return width.clamp(1, MAX_LOGO_WIDTH);
        }
    }

    let width = if (config.logo.scale - 1.0).abs() > f32::EPSILON {
        (LOGO_WIDTH as f32 * config.logo.scale).round() as i64
    } else {
        i64::from(config.logo.width)
    };

    width.clamp(MIN_LOGO_WIDTH as i64, MAX_LOGO_WIDTH as i64) as usize
}

fn source_grid() -> Vec<Vec<(char, Tone)>> {
    ROWS.iter()
        .map(|row| {
            let mut cells: Vec<(char, Tone)> = Vec::with_capacity(LOGO_WIDTH);
            for segment in row.iter() {
                for ch in segment.text.chars() {
                    cells.push((ch, segment.tone));
                }
            }
            cells.resize(LOGO_WIDTH, (' ', Tone::Transparent));
            cells.truncate(LOGO_WIDTH);
            cells
        })
        .collect()
}

fn scaled_grid(target_width: usize) -> Vec<Vec<(char, Tone)>> {
    let source = source_grid();
    let target_width = target_width.max(1);
    let ratio = target_width as f64 / LOGO_WIDTH as f64;
    let target_height = ((LOGO_HEIGHT as f64 * ratio).round() as usize).max(1);

    let mut rows = Vec::with_capacity(target_height);
    for y in 0..target_height {
        let source_y = ((y as f64 / ratio).floor() as usize).min(LOGO_HEIGHT - 1);
        let mut row = Vec::with_capacity(target_width);
        for x in 0..target_width {
            let source_x = ((x as f64 / ratio).floor() as usize).min(LOGO_WIDTH - 1);
            row.push(source[source_y][source_x]);
        }
        rows.push(row);
    }
    rows
}

fn render_grid(grid: &[Vec<(char, Tone)>], depth: ColourDepth, palette: &Palette) -> Vec<String> {
    grid.iter()
        .map(|row| {
            let mut out = String::new();
            let mut run = String::new();
            let mut run_tone = Tone::Transparent;

            for &(ch, tone) in row {
                if tone != run_tone {
                    flush_run(&mut out, &mut run, run_tone, depth, palette);
                    run_tone = tone;
                }
                run.push(ch);
            }
            flush_run(&mut out, &mut run, run_tone, depth, palette);

            pad_visible(&out, row.len())
        })
        .collect()
}

fn flush_run(
    out: &mut String,
    run: &mut String,
    tone: Tone,
    depth: ColourDepth,
    palette: &Palette,
) {
    if run.is_empty() {
        return;
    }
    match tone {
        Tone::Transparent => out.push_str(run),
        Tone::Primary => out.push_str(&paint(run, palette.logo_primary, depth)),
        Tone::Accent => out.push_str(&paint(run, palette.logo_accent, depth)),
    }
    run.clear();
}

fn custom_ascii_lines(custom: &str) -> Vec<String> {
    let mut lines: Vec<String> = custom
        .lines()
        .map(|line| line.trim_end_matches('\r').to_string())
        .collect();

    while lines.first().is_some_and(|line| line.trim().is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }
    lines
}

pub fn lines(config: &Config, depth: ColourDepth, palette: &Palette, width: usize) -> Vec<String> {
    let width = width.max(1);

    if let Some(custom) = &config.logo.custom_ascii {
        let lines = custom_ascii_lines(custom);
        if !lines.is_empty() {
            return lines
                .into_iter()
                .map(|line| {
                    let truncated = crate::text::truncate_chars(&line, width);
                    let painted = paint(&truncated, palette.logo_primary, depth);
                    pad_visible(&painted, width)
                })
                .collect();
        }
    }

    render_grid(&scaled_grid(width), depth, palette)
}

pub fn lines_configured(config: &Config, depth: ColourDepth, palette: &Palette) -> Vec<String> {
    lines(config, depth, palette, configured_width(config))
}

pub fn render_to_string(
    config: &Config,
    depth: ColourDepth,
    palette: &Palette,
    width: usize,
) -> String {
    let mut output = lines(config, depth, palette, width)
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n");
    if !output.is_empty() {
        output.push('\n');
    }
    output
}

pub fn print_logo(config: &Config, depth: ColourDepth, palette: &Palette) {
    print!(
        "{}",
        render_to_string(config, depth, palette, configured_width(config))
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::visible_len;

    #[test]
    fn test_logo_default_size() {
        let config = Config::default();
        let palette = Palette::default();
        let lines = lines_configured(&config, ColourDepth::None, &palette);
        assert_eq!(lines.len(), LOGO_HEIGHT);
        for line in &lines {
            assert_eq!(visible_len(line), LOGO_WIDTH);
        }
    }

    #[test]
    fn test_logo_scales_width_and_height() {
        let config = Config::default();
        let palette = Palette::default();
        let lines = lines(&config, ColourDepth::None, &palette, 14);
        for line in &lines {
            assert_eq!(visible_len(line), 14);
        }
        assert!(lines.len() < LOGO_HEIGHT);
        assert!(lines.len() >= 7);
    }

    #[test]
    fn test_configured_width_scale() {
        let mut config = Config::default();
        config.logo.scale = 2.0;
        assert_eq!(configured_width(&config), LOGO_WIDTH * 2);
    }

    #[test]
    fn test_configured_width_clamped() {
        let mut config = Config::default();
        config.logo.scale = 100.0;
        assert_eq!(configured_width(&config), MAX_LOGO_WIDTH);
    }

    #[test]
    fn test_custom_ascii_is_used() {
        let mut config = Config::default();
        config.logo.custom_ascii = Some(String::from("A\nAA"));
        let palette = Palette::default();
        let lines = lines_configured(&config, ColourDepth::None, &palette);
        assert_eq!(lines, vec!["A ", "AA"]);
        assert_eq!(configured_width(&config), 2);
    }

    #[test]
    fn test_logo_no_ansi_without_colour() {
        let config = Config::default();
        let palette = Palette::default();
        let lines = lines_configured(&config, ColourDepth::None, &palette);
        assert!(lines.iter().all(|line| !line.contains('\x1b')));
    }

    #[test]
    fn test_logo_truecolour() {
        let config = Config::default();
        let palette = Palette::default();
        let lines = lines_configured(&config, ColourDepth::TrueColour, &palette);
        assert!(lines.iter().any(|line| line.contains("\x1b[38;2;")));
    }

    #[test]
    fn test_logo_only_string_has_no_trailing_padding() {
        let config = Config::default();
        let palette = Palette::default();
        let output = render_to_string(&config, ColourDepth::None, &palette, LOGO_WIDTH);
        for line in output.lines() {
            assert_eq!(line, line.trim_end());
        }
    }
}
