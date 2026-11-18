// SPDX-License-Identifier: GPL-3.0-or-later

use std::env;
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct LocaleInfo {
    pub locale: Option<String>,
    pub keyboard_layout: Option<String>,
}

fn detect_locale() -> Option<String> {
    for var in &["LC_ALL", "LANG", "LC_MESSAGES"] {
        if let Ok(val) = env::var(var) {
            if !val.is_empty() && val != "C" && val != "POSIX" {
                return Some(val);
            }
        }
    }

    if let Ok(content) = fs::read_to_string("/etc/locale.conf") {
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("LANG=") {
                let val = val.trim().trim_matches('"').to_string();
                if !val.is_empty() {
                    return Some(val);
                }
            }
        }
    }

    if let Ok(content) = fs::read_to_string("/etc/default/locale") {
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("LANG=") {
                let val = val.trim().trim_matches('"').to_string();
                if !val.is_empty() {
                    return Some(val);
                }
            }
        }
    }

    None
}

fn detect_keyboard_layout() -> Option<String> {
    if let Ok(content) = fs::read_to_string("/etc/vconsole.conf") {
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("KEYMAP=") {
                let val = val.trim().trim_matches('"').to_string();
                if !val.is_empty() {
                    return Some(val);
                }
            }
        }
    }

    if let Ok(entries) = fs::read_dir("/etc/X11/xorg.conf.d") {
        for entry in entries.flatten() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                for line in content.lines() {
                    let trimmed = line.trim();

                    if trimmed.to_lowercase().contains("xkblayout") {
                        let parts: Vec<&str> = trimmed.split('"').collect();

                        if let Some(layout) = parts.get(3) {
                            if !layout.is_empty() {
                                return Some(layout.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    if let Ok(out) = Command::new("localectl").arg("status").output() {
        let stdout = String::from_utf8_lossy(&out.stdout);

        let mut vc_keymap: Option<String> = None;
        for line in stdout.lines() {
            let trimmed = line.trim();
            if let Some(val) = trimmed
                .strip_prefix("X11 Layout:")
                .or_else(|| trimmed.strip_prefix("X11 layout:"))
            {
                let val = val.trim().to_string();
                if !val.is_empty() {
                    return Some(val);
                }
            }
            if let Some(val) = trimmed
                .strip_prefix("VC Keymap:")
                .or_else(|| trimmed.strip_prefix("VC keymap:"))
            {
                let val = val.trim().to_string();
                if !val.is_empty() {
                    vc_keymap = Some(val);
                }
            }
        }
        if let Some(k) = vc_keymap {
            return Some(k);
        }
    }

    None
}

pub fn collect() -> LocaleInfo {
    LocaleInfo {
        locale: detect_locale(),
        keyboard_layout: detect_keyboard_layout(),
    }
}
