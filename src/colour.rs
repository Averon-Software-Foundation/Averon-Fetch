use crate::cli::Args;
use crate::config::Config;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn parse(input: &str) -> Option<Self> {
        let value = input.trim().trim_start_matches('#');
        let (r, g, b) = match value.len() {
            3 => {
                let mut chars = value.chars();
                let r = chars.next()?.to_digit(16)? as u8;
                let g = chars.next()?.to_digit(16)? as u8;
                let b = chars.next()?.to_digit(16)? as u8;
                (r * 17, g * 17, b * 17)
            }
            6 => {
                // Slice bytes rather than the string so invalid UTF-8
                // boundaries cannot panic on non-ASCII input.
                let bytes = value.as_bytes();
                let component = |range: std::ops::Range<usize>| {
                    std::str::from_utf8(&bytes[range])
                        .ok()
                        .and_then(|part| u8::from_str_radix(part, 16).ok())
                };
                (component(0..2)?, component(2..4)?, component(4..6)?)
            }
            _ => return None,
        };
        Some(Self::new(r, g, b))
    }

    pub fn ansi256(self) -> u8 {
        if self.r == self.g && self.g == self.b {
            if self.r < 8 {
                return 16;
            }
            // 248..=255 all round to the white end of the grey ramp. Keeping
            // the cutoff at 247 avoids 232 + 24 overflowing u8 for r = 248.
            if self.r > 247 {
                return 231;
            }
            return 232 + ((self.r as u16 - 8) / 10) as u8;
        }

        let r = (self.r as u16 * 5 / 255) as u8;
        let g = (self.g as u16 * 5 / 255) as u8;
        let b = (self.b as u16 * 5 / 255) as u8;
        16 + 36 * r + 6 * g + b
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColourDepth {
    None,
    Ansi256,
    TrueColour,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum, Default)]
#[serde(rename_all = "lowercase")]
pub enum ColorChoice {
    #[default]
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColourEnvironment {
    pub no_color: bool,
    pub term_dumb: bool,
    pub truecolour: bool,
}

impl ColourEnvironment {
    pub fn from_env() -> Self {
        let term = std::env::var("TERM").unwrap_or_default();
        let colorterm = std::env::var("COLORTERM").unwrap_or_default();
        let term_lower = term.to_lowercase();
        let colorterm_lower = colorterm.to_lowercase();

        Self {
            no_color: std::env::var_os("NO_COLOR").is_some(),
            term_dumb: term.eq_ignore_ascii_case("dumb"),
            truecolour: colorterm_lower.contains("truecolor")
                || colorterm_lower.contains("24bit")
                || term_lower.contains("truecolor")
                || term_lower.contains("direct"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub label: Rgb,
    pub value: Rgb,
    pub separator: Rgb,
    pub accent: Rgb,
    pub header_user: Rgb,
    pub header_host: Rgb,
    pub logo_primary: Rgb,
    pub logo_accent: Rgb,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            label: Rgb::new(91, 83, 235),
            value: Rgb::new(220, 220, 220),
            separator: Rgb::new(100, 216, 245),
            accent: Rgb::new(91, 83, 235),
            header_user: Rgb::new(220, 220, 220),
            header_host: Rgb::new(91, 83, 235),
            logo_primary: Rgb::new(220, 220, 220),
            logo_accent: Rgb::new(91, 83, 235),
        }
    }
}

impl Palette {
    pub fn from_config(config: &Config) -> Self {
        let defaults = Self::default();
        Self {
            label: parse_or_default(&config.colours.label, defaults.label),
            value: parse_or_default(&config.colours.value, defaults.value),
            separator: parse_or_default(&config.colours.separator, defaults.separator),
            accent: parse_or_default(&config.colours.accent, defaults.accent),
            header_user: parse_or_default(&config.colours.header_user, defaults.header_user),
            header_host: parse_or_default(&config.colours.header_host, defaults.header_host),
            logo_primary: parse_or_default(&config.colours.logo_primary, defaults.logo_primary),
            logo_accent: parse_or_default(&config.colours.logo_accent, defaults.logo_accent),
        }
    }
}

fn parse_or_default(value: &str, fallback: Rgb) -> Rgb {
    match Rgb::parse(value) {
        Some(rgb) => rgb,
        None => {
            crate::diag::warn(format!(
                "invalid colour '{value}', using default #{:02X}{:02X}{:02X}",
                fallback.r, fallback.g, fallback.b
            ));
            fallback
        }
    }
}

pub fn paint(text: &str, colour: Rgb, depth: ColourDepth) -> String {
    match depth {
        ColourDepth::None => text.to_string(),
        ColourDepth::Ansi256 => {
            format!("\x1b[38;5;{}m{text}\x1b[0m", colour.ansi256())
        }
        ColourDepth::TrueColour => {
            format!(
                "\x1b[38;2;{};{};{}m{text}\x1b[0m",
                colour.r, colour.g, colour.b
            )
        }
    }
}

pub fn paint_bold(text: &str, colour: Rgb, depth: ColourDepth) -> String {
    match depth {
        ColourDepth::None => text.to_string(),
        ColourDepth::Ansi256 => {
            format!("\x1b[1;38;5;{}m{text}\x1b[0m", colour.ansi256())
        }
        ColourDepth::TrueColour => {
            format!(
                "\x1b[1;38;2;{};{};{}m{text}\x1b[0m",
                colour.r, colour.g, colour.b
            )
        }
    }
}

pub fn resolve_depth(
    args: &Args,
    config: &Config,
    env: &ColourEnvironment,
    is_tty: bool,
) -> ColourDepth {
    if args.pipe || args.no_color {
        return ColourDepth::None;
    }

    match args.color {
        ColorChoice::Never => ColourDepth::None,
        ColorChoice::Always => explicit_depth(config),
        ColorChoice::Auto => {
            if env.no_color || env.term_dumb || !is_tty || !config.general.colour {
                return ColourDepth::None;
            }
            match config.terminal.colour {
                ColorChoice::Never => ColourDepth::None,
                ColorChoice::Always => explicit_depth(config),
                ColorChoice::Auto => auto_depth(config, env),
            }
        }
    }
}

fn explicit_depth(config: &Config) -> ColourDepth {
    if config.terminal.truecolour {
        ColourDepth::TrueColour
    } else {
        ColourDepth::Ansi256
    }
}

fn auto_depth(config: &Config, env: &ColourEnvironment) -> ColourDepth {
    if !config.terminal.truecolour {
        return ColourDepth::Ansi256;
    }
    if env.truecolour {
        ColourDepth::TrueColour
    } else {
        ColourDepth::Ansi256
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_parse_hex_colours() {
        assert_eq!(Rgb::parse("#5B53EB"), Some(Rgb::new(0x5B, 0x53, 0xEB)));
        assert_eq!(Rgb::parse("5b53eb"), Some(Rgb::new(0x5B, 0x53, 0xEB)));
        assert_eq!(Rgb::parse("#fff"), Some(Rgb::new(255, 255, 255)));
        assert_eq!(Rgb::parse("#12345"), None);
        assert_eq!(Rgb::parse("nope"), None);
    }

    #[test]
    fn test_parse_ignores_non_ascii_without_panicking() {
        // Six bytes, but not valid ASCII and the byte boundaries fall inside
        // multi-byte characters. This used to panic on string slicing.
        assert_eq!(Rgb::parse("€€"), None);
        assert_eq!(Rgb::parse("#ééé"), None);
    }

    #[test]
    fn test_ansi256_conversion() {
        assert_eq!(Rgb::new(0, 0, 0).ansi256(), 16);
        assert_eq!(Rgb::new(255, 255, 255).ansi256(), 231);
        assert_eq!(Rgb::new(248, 248, 248).ansi256(), 231);
        assert_eq!(Rgb::new(247, 247, 247).ansi256(), 255);

        assert!((232..=255).contains(&Rgb::new(128, 128, 128).ansi256()));
    }

    #[test]
    fn test_paint_none_has_no_escape() {
        assert_eq!(
            paint("hello", Palette::default().label, ColourDepth::None),
            "hello"
        );
        assert!(!paint("hello", Palette::default().label, ColourDepth::None).contains('\x1b'));
    }

    #[test]
    fn test_paint_truecolour_and_256() {
        let rgb = Rgb::new(1, 2, 3);
        assert!(paint("x", rgb, ColourDepth::TrueColour).contains("\x1b[38;2;1;2;3m"));
        assert!(paint("x", rgb, ColourDepth::Ansi256).contains("\x1b[38;5;"));
    }

    fn env(no_color: bool, dumb: bool, truecolour: bool) -> ColourEnvironment {
        ColourEnvironment {
            no_color,
            term_dumb: dumb,
            truecolour,
        }
    }

    #[test]
    fn test_no_color_environment_disables_auto() {
        let args = Args::parse_from(["averon-fetch"]);
        let cfg = Config::default();
        let depth = resolve_depth(&args, &cfg, &env(true, false, true), true);
        assert_eq!(depth, ColourDepth::None);
    }

    #[test]
    fn test_piped_output_is_plain_by_default() {
        let args = Args::parse_from(["averon-fetch"]);
        let cfg = Config::default();
        let depth = resolve_depth(&args, &cfg, &env(false, false, true), false);
        assert_eq!(depth, ColourDepth::None);
    }

    #[test]
    fn test_explicit_always_allows_piped_colour() {
        let args = Args::parse_from(["averon-fetch", "--color", "always"]);
        let cfg = Config::default();
        let depth = resolve_depth(&args, &cfg, &env(true, true, false), false);
        assert_eq!(depth, ColourDepth::TrueColour);
    }

    #[test]
    fn test_no_color_flag_wins() {
        let args = Args::parse_from(["averon-fetch", "--color", "always", "--no-color"]);
        let cfg = Config::default();
        let depth = resolve_depth(&args, &cfg, &env(false, false, true), true);
        assert_eq!(depth, ColourDepth::None);
    }

    #[test]
    fn test_256_fallback_when_truecolour_disabled() {
        let args = Args::parse_from(["averon-fetch"]);
        let mut cfg = Config::default();
        cfg.terminal.truecolour = false;
        let depth = resolve_depth(&args, &cfg, &env(false, false, true), true);
        assert_eq!(depth, ColourDepth::Ansi256);
    }

    #[test]
    fn test_palette_from_config_uses_custom_colours() {
        let mut cfg = Config::default();
        cfg.colours.label = "#010203".to_string();
        let palette = Palette::from_config(&cfg);
        assert_eq!(palette.label, Rgb::new(1, 2, 3));
    }

    #[test]
    fn test_invalid_colour_falls_back() {
        let mut cfg = Config::default();
        cfg.colours.label = "not-a-colour".to_string();
        let palette = Palette::from_config(&cfg);
        assert_eq!(palette.label, Palette::default().label);
    }
}
