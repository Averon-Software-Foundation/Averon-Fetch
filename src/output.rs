use crate::colour::{paint, paint_bold, ColourDepth, Palette};
use crate::config::Config;
use crate::logo;
use crate::modules::{Document, Header, Line};
use crate::text::{char_len, pad_visible, truncate_chars};
use serde::Serialize;

const LOGO_GAP: usize = 3;
const MIN_INFO_WIDTH: usize = 24;
const MAX_LABEL_WIDTH: usize = 24;

#[derive(Debug, Clone, Copy)]
pub struct TextOptions {
    pub depth: ColourDepth,
    pub palette: Palette,
    pub term_width: usize,
    pub is_tty: bool,
    pub show_logo: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry {
    label: String,
    value: String,
}

struct LogoLayout {
    lines: Vec<String>,
    width: usize,
    stacked: bool,
}

impl LogoLayout {
    fn column_width(&self) -> usize {
        self.width + LOGO_GAP
    }
}

pub fn render_text(doc: &Document, config: &Config, options: &TextOptions) -> String {
    let entries: Vec<Entry> = doc
        .lines
        .iter()
        .filter(|line| !line.value.is_empty())
        .map(|line| Entry {
            label: config.resolve_label(line.module, &line.label, line.label_suffix.as_deref()),
            value: line.value.clone(),
        })
        .collect();

    if doc.header.is_none() && entries.is_empty() {
        return String::new();
    }

    let separator = &config.general.separator;
    let separator_len = char_len(separator);
    let raw_label_width = entries
        .iter()
        .map(|entry| char_len(&entry.label))
        .max()
        .unwrap_or(0);

    let is_tty = options.is_tty;
    let term_width = options.term_width;

    let label_cap = if is_tty {
        term_width
            .saturating_sub(separator_len + 12)
            .clamp(1, MAX_LABEL_WIDTH)
    } else {
        usize::MAX
    };
    let mut label_width = raw_label_width.min(label_cap);

    let (logo_layout, info_width) = layout_logo(config, options, label_width, separator_len);

    if is_tty && info_width <= label_width + separator_len + 4 {
        label_width = info_width.saturating_sub(separator_len + 4).max(1);
    }

    let value_width = if is_tty {
        info_width
            .saturating_sub(label_width + separator_len)
            .max(1)
    } else {
        usize::MAX
    };

    let mut info_lines = Vec::new();

    if let Some(header) = &doc.header {
        info_lines.extend(render_header(header, options, info_width));
    }

    for entry in &entries {
        let label = truncate_chars(&entry.label, label_width);
        let label = pad_visible(&label, label_width);
        let chunks = if is_tty {
            wrap_value(&entry.value, value_width)
        } else {
            vec![entry.value.clone()]
        };

        for (index, chunk) in chunks.iter().enumerate() {
            if index == 0 {
                info_lines.push(format!(
                    "{}{}{}",
                    paint(&label, options.palette.label, options.depth),
                    paint(separator, options.palette.separator, options.depth),
                    paint(chunk, options.palette.value, options.depth)
                ));
            } else {
                let indent = " ".repeat(label_width + separator_len);
                info_lines.push(format!(
                    "{indent}{}",
                    paint(chunk, options.palette.value, options.depth)
                ));
            }
        }
    }

    compose(&logo_layout, &info_lines, is_tty)
}

fn layout_logo(
    config: &Config,
    options: &TextOptions,
    label_width: usize,
    separator_len: usize,
) -> (Option<LogoLayout>, usize) {
    if !options.show_logo {
        return (
            None,
            if options.is_tty {
                options.term_width
            } else {
                usize::MAX
            },
        );
    }

    let configured_width = logo::configured_width(config);
    let min_info_width = (label_width + separator_len + 16).max(MIN_INFO_WIDTH);

    if !options.is_tty {
        return (
            Some(LogoLayout {
                lines: logo::lines(config, options.depth, &options.palette, configured_width),
                width: configured_width,
                stacked: false,
            }),
            usize::MAX,
        );
    }

    let term_width = options.term_width;
    let side_by_side = configured_width + LOGO_GAP + min_info_width <= term_width;

    if side_by_side {
        let info_width = term_width - configured_width - LOGO_GAP;
        return (
            Some(LogoLayout {
                lines: logo::lines(config, options.depth, &options.palette, configured_width),
                width: configured_width,
                stacked: false,
            }),
            info_width,
        );
    }

    let available_logo = term_width.saturating_sub(LOGO_GAP + min_info_width);
    if available_logo >= logo::MIN_LOGO_WIDTH {
        let width = configured_width.min(available_logo);
        let info_width = term_width - width - LOGO_GAP;
        return (
            Some(LogoLayout {
                lines: logo::lines(config, options.depth, &options.palette, width),
                width,
                stacked: false,
            }),
            info_width,
        );
    }

    if term_width >= logo::MIN_LOGO_WIDTH {
        let width = configured_width.min(term_width);
        return (
            Some(LogoLayout {
                lines: logo::lines(config, options.depth, &options.palette, width),
                width,
                stacked: true,
            }),
            term_width,
        );
    }

    (None, term_width)
}

fn render_header(header: &Header, options: &TextOptions, width: usize) -> Vec<String> {
    let username = header.username.as_deref();
    let hostname = header.hostname.as_deref();

    let raw = match (username, hostname) {
        (Some(user), Some(host)) => format!("{user}@{host}"),
        (Some(user), None) => user.to_string(),
        (None, Some(host)) => host.to_string(),
        (None, None) => return Vec::new(),
    };

    let max_width = if options.is_tty {
        width.max(1)
    } else {
        usize::MAX
    };

    let header_line = if options.is_tty && char_len(&raw) > max_width {
        let truncated = truncate_chars(&raw, max_width);
        paint_bold(&truncated, options.palette.header_user, options.depth)
    } else {
        match (username, hostname) {
            (Some(user), Some(host)) => format!(
                "{}{}{}",
                paint_bold(user, options.palette.header_user, options.depth),
                paint("@", options.palette.accent, options.depth),
                paint_bold(host, options.palette.header_host, options.depth)
            ),
            (Some(user), None) => paint_bold(user, options.palette.header_user, options.depth),
            (None, Some(host)) => paint_bold(host, options.palette.header_host, options.depth),
            (None, None) => return Vec::new(),
        }
    };

    let divider_len = char_len(&raw).max(12);
    let divider_len = if options.is_tty {
        divider_len.min(max_width)
    } else {
        divider_len
    };
    let divider = paint(
        &"─".repeat(divider_len),
        options.palette.accent,
        options.depth,
    );

    vec![header_line, divider]
}

fn compose(logo: &Option<LogoLayout>, info_lines: &[String], is_tty: bool) -> String {
    let mut output = String::new();

    match logo {
        Some(layout) if layout.stacked => {
            for line in &layout.lines {
                output.push_str(line);
                output.push('\n');
            }
            if !layout.lines.is_empty() {
                output.push('\n');
            }
            for line in info_lines {
                output.push_str(line);
                output.push('\n');
            }
        }
        Some(layout) => {
            let column_width = layout.column_width();
            let rows = layout.lines.len().max(info_lines.len());
            for index in 0..rows {
                let left = match layout.lines.get(index) {
                    Some(line) => pad_visible(line, column_width),
                    None => " ".repeat(column_width),
                };
                let right = info_lines.get(index).map(String::as_str).unwrap_or("");

                if right.is_empty() {
                    output.push_str(left.trim_end());
                } else {
                    output.push_str(&left);
                    output.push_str(right);
                }
                output.push('\n');
            }
        }
        None => {
            for line in info_lines {
                output.push_str(line);
                output.push('\n');
            }
        }
    }

    if !is_tty {
        output = output
            .lines()
            .map(str::trim_end)
            .collect::<Vec<_>>()
            .join("\n");
        if !output.is_empty() {
            output.push('\n');
        }
    }

    output
}

fn wrap_value(value: &str, width: usize) -> Vec<String> {
    if width == 0 || char_len(value) <= width {
        return vec![value.to_string()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_len = 0usize;

    for word in value.split_whitespace() {
        let word_len = char_len(word);

        if word_len > width {
            if !current.is_empty() {
                lines.push(std::mem::take(&mut current));
                current_len = 0;
            }

            for ch in word.chars() {
                if current_len == width {
                    lines.push(std::mem::take(&mut current));
                    current_len = 0;
                }
                current.push(ch);
                current_len += 1;
            }
            continue;
        }

        if current_len == 0 {
            current.push_str(word);
            current_len = word_len;
        } else if current_len + 1 + word_len <= width {
            current.push(' ');
            current.push_str(word);
            current_len += 1 + word_len;
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
            current_len = word_len;
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

#[derive(Serialize)]
struct JsonDocument {
    header: Option<JsonHeader>,
    modules: Vec<JsonModule>,
}

#[derive(Serialize)]
struct JsonHeader {
    username: Option<String>,
    hostname: Option<String>,
}

#[derive(Serialize)]
struct JsonModule {
    module: &'static str,
    label: String,
    value: String,
}

pub fn render_json(doc: &Document, config: &Config) -> Result<String, serde_json::Error> {
    let document = JsonDocument {
        header: doc.header.as_ref().map(|header| JsonHeader {
            username: header.username.clone(),
            hostname: header.hostname.clone(),
        }),
        modules: doc
            .lines
            .iter()
            // Match text output: rows without a value are not rendered.
            .filter(|line| !line.value.is_empty())
            .map(|line: &Line| JsonModule {
                module: line.module,
                label: config.resolve_label(line.module, &line.label, line.label_suffix.as_deref()),
                value: line.value.clone(),
            })
            .collect(),
    };

    let mut output = serde_json::to_string_pretty(&document)?;
    output.push('\n');
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::Line;
    use crate::text::visible_len;

    fn options(
        depth: ColourDepth,
        term_width: usize,
        is_tty: bool,
        show_logo: bool,
    ) -> TextOptions {
        TextOptions {
            depth,
            palette: Palette::default(),
            term_width,
            is_tty,
            show_logo,
        }
    }

    fn sample() -> Document {
        Document {
            header: Some(Header {
                username: Some(String::from("josh")),
                hostname: Some(String::from("host")),
            }),
            lines: vec![
                Line::new("os", "OS", "Averon Linux x86_64"),
                Line::new("kernel", "Kernel", "Linux 6.11"),
                Line::new("memory", "Memory", "8.00 GiB / 16.00 GiB (50%)"),
            ],
        }
    }

    #[test]
    fn test_header_and_alignment() {
        let config = Config::default();
        let output = render_text(
            &sample(),
            &config,
            &options(ColourDepth::None, 100, true, false),
        );
        assert!(output.contains("josh@host"));
        assert!(output.contains("OS    : Averon Linux x86_64"));
        assert!(output.contains("Kernel: Linux 6.11"));
        assert!(output.contains("Memory: 8.00 GiB / 16.00 GiB (50%)"));
    }

    #[test]
    fn test_no_ansi_in_plain_output() {
        let config = Config::default();
        let output = render_text(
            &sample(),
            &config,
            &options(ColourDepth::None, 100, true, false),
        );
        assert!(!output.contains('\x1b'));
    }

    #[test]
    fn test_truecolour_output() {
        let config = Config::default();
        let output = render_text(
            &sample(),
            &config,
            &options(ColourDepth::TrueColour, 100, true, false),
        );
        assert!(output.contains("\x1b[38;2;"));
    }

    #[test]
    fn test_custom_separator() {
        let mut config = Config::default();
        config.general.separator = String::from(" -> ");
        let output = render_text(
            &sample(),
            &config,
            &options(ColourDepth::None, 100, true, false),
        );
        assert!(output.contains("Kernel -> Linux 6.11"));
    }

    #[test]
    fn test_custom_label_is_applied() {
        let mut config = Config::default();
        config
            .labels
            .insert(String::from("os"), String::from("System"));
        let output = render_text(
            &sample(),
            &config,
            &options(ColourDepth::None, 100, true, false),
        );
        assert!(output.contains("System: Averon Linux x86_64"));
    }

    #[test]
    fn test_long_value_wraps_to_terminal_width() {
        let config = Config::default();
        let mut doc = sample();
        doc.lines.push(Line::new(
            "packages",
            "Packages",
            "1234 (pacman), 20 (flatpak), 6 (snap), 42 (rpm), 100 (dpkg)",
        ));
        let width = 48;
        let output = render_text(
            &doc,
            &config,
            &options(ColourDepth::None, width, true, false),
        );
        for line in output.lines() {
            assert!(
                char_len(line) <= width,
                "line exceeds terminal width: {line:?} ({})",
                char_len(line)
            );
        }
        assert!(output.contains("Packages:"));
    }

    #[test]
    fn test_narrow_terminal_stacks_logo() {
        let config = Config::default();
        let output = render_text(
            &sample(),
            &config,
            &options(ColourDepth::None, 30, true, true),
        );
        for line in output.lines() {
            assert!(
                char_len(line) <= 30,
                "line exceeds narrow width: {line:?} ({})",
                char_len(line)
            );
        }

        assert!(output.contains("\n\n"));
    }

    #[test]
    fn test_very_narrow_terminal_still_fits() {
        let config = Config::default();
        let output = render_text(
            &sample(),
            &config,
            &options(ColourDepth::None, 20, true, true),
        );
        for line in output.lines() {
            assert!(
                char_len(line) <= 20,
                "line exceeds narrow width: {line:?} ({})",
                char_len(line)
            );
        }
    }

    #[test]
    fn test_logo_shrinks_before_stacking() {
        let config = Config::default();
        let output = render_text(
            &sample(),
            &config,
            &options(ColourDepth::None, 50, true, true),
        );

        for line in output.lines() {
            assert!(char_len(line) <= 50);
        }
    }

    #[test]
    fn test_piped_output_has_no_ansi_and_no_trailing_padding() {
        let config = Config::default();
        let output = render_text(
            &sample(),
            &config,
            &options(ColourDepth::None, 80, false, true),
        );
        assert!(!output.contains('\x1b'));
        for line in output.lines() {
            assert_eq!(line, line.trim_end());
        }
    }

    #[test]
    fn test_json_output() {
        let mut config = Config::default();
        config
            .labels
            .insert(String::from("os"), String::from("System"));
        let json = render_json(&sample(), &config).unwrap();
        assert!(json.contains("\"module\": \"os\""));
        assert!(json.contains("\"label\": \"System\""));
        assert!(json.contains("\"username\": \"josh\""));
        assert!(!json.contains('\x1b'));
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["modules"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_json_skips_empty_values_like_text_output() {
        let config = Config::default();
        let doc = Document {
            header: None,
            lines: vec![
                Line::new("os", "OS", "Averon Linux"),
                Line::new("bios", "BIOS", ""),
            ],
        };
        let json = render_json(&doc, &config).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let modules = value["modules"].as_array().unwrap();
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0]["module"], "os");
    }

    #[test]
    fn test_empty_document_renders_nothing() {
        let config = Config::default();
        let output = render_text(
            &Document::default(),
            &config,
            &options(ColourDepth::None, 80, true, true),
        );
        assert!(output.is_empty());
    }

    #[test]
    fn test_wrap_value_hard_splits_long_words() {
        let lines = wrap_value("abcdefghij", 4);
        assert_eq!(lines, vec!["abcd", "efgh", "ij"]);
    }

    #[test]
    fn test_wrap_value_prefers_word_boundaries() {
        let lines = wrap_value("alpha beta gamma", 10);
        assert_eq!(lines, vec!["alpha beta", "gamma"]);
    }

    #[test]
    fn test_divider_is_at_least_header_length() {
        let config = Config::default();
        let output = render_text(
            &sample(),
            &config,
            &options(ColourDepth::None, 100, true, false),
        );
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(visible_len(lines[0]), char_len("josh@host"));
        assert!(visible_len(lines[1]) >= visible_len(lines[0]));
    }
}
