// SPDX-License-Identifier: GPL-3.0-or-later

pub fn visible_len(input: &str) -> usize {
    let mut length = 0usize;
    let mut in_escape = false;

    for ch in input.chars() {
        if ch == '\x1b' {
            in_escape = true;
        } else if in_escape && ch == 'm' {
            in_escape = false;
        } else if !in_escape {
            length += 1;
        }
    }

    length
}

pub fn pad_visible(input: &str, width: usize) -> String {
    let current = visible_len(input);
    if current < width {
        format!("{input}{}", " ".repeat(width - current))
    } else {
        input.to_string()
    }
}

pub fn truncate_chars(input: &str, width: usize) -> String {
    if input.chars().count() <= width {
        return input.to_string();
    }
    if width == 0 {
        return String::new();
    }
    let mut out: String = input.chars().take(width.saturating_sub(1)).collect();
    out.push('…');
    out
}

pub fn char_len(input: &str) -> usize {
    input.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visible_len_strips_ansi() {
        assert_eq!(visible_len("hello"), 5);
        assert_eq!(visible_len("\x1b[38;2;255;255;255mhello\x1b[0m"), 5);
        assert_eq!(visible_len("\x1b[1;38;5;12mhello\x1b[0m"), 5);
    }

    #[test]
    fn test_pad_visible() {
        assert_eq!(pad_visible("hi", 4), "hi  ");
        assert_eq!(pad_visible("hello", 4), "hello");
        let coloured = pad_visible("\x1b[38;2;1;2;3mhi\x1b[0m", 4);
        assert_eq!(visible_len(&coloured), 4);
    }

    #[test]
    fn test_truncate_chars() {
        assert_eq!(truncate_chars("hello", 10), "hello");
        assert_eq!(truncate_chars("hello", 4), "hel…");
        assert_eq!(truncate_chars("hello", 0), "");
    }
}
