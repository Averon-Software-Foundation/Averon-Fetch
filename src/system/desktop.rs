use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct DesktopInfo {
    pub desktop_env: Option<String>,
    pub window_manager: Option<String>,
    pub display_server: Option<String>,
    pub gtk_theme: Option<String>,
    pub icon_theme: Option<String>,
    pub cursor_theme: Option<String>,
    pub font: Option<String>,
}

fn detect_display_server() -> Option<String> {
    if env::var("WAYLAND_DISPLAY").is_ok() {
        return Some("Wayland".to_string());
    }
    if env::var("DISPLAY").is_ok() {
        return Some("X11".to_string());
    }
    None
}

fn spawn_version(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .and_then(|o| {
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&o.stdout),
                String::from_utf8_lossy(&o.stderr)
            );
            combined.lines().next().map(|l| l.trim().to_string())
        })
        .filter(|s| !s.is_empty())
}

fn detect_desktop_env() -> Option<String> {
    let xdg = env::var("XDG_CURRENT_DESKTOP").ok();
    let session = env::var("DESKTOP_SESSION").ok();
    let kde_full = env::var("KDE_FULL_SESSION").ok();

    let raw = xdg
        .as_deref()
        .and_then(|s| s.split(':').rev().find(|p| !p.is_empty()))
        .map(|s| s.to_string())
        .or_else(|| session.clone());

    let lower = raw.as_deref().unwrap_or("").to_lowercase();

    if kde_full.is_some() || lower.contains("kde") || lower.contains("plasma") {
        if let Some(line) = spawn_version("plasmashell", &["--version"]) {
            let version = line.split_whitespace().last().unwrap_or("").to_string();
            if !version.is_empty() {
                return Some(format!("KDE Plasma {}", version));
            }
        }
        return Some("KDE Plasma".to_string());
    }

    if lower.contains("gnome") {
        if let Some(line) = spawn_version("gnome-shell", &["--version"]) {
            let version = line.split_whitespace().last().unwrap_or("").to_string();
            if !version.is_empty() {
                return Some(format!("GNOME {}", version));
            }
        }
        return Some("GNOME".to_string());
    }

    if lower.contains("xfce") {
        if let Some(line) = spawn_version("xfce4-session", &["--version"]) {
            let version = line.split_whitespace().last().unwrap_or("").to_string();
            if !version.is_empty() {
                return Some(format!("Xfce {}", version));
            }
        }
        return Some("Xfce".to_string());
    }

    if env::var("SWAYSOCK").is_ok() || lower.contains("sway") {
        if let Some(line) = spawn_version("sway", &["--version"]) {
            return Some(line);
        }
        return Some("Sway".to_string());
    }
    if lower.contains("hyprland") {
        if let Some(line) = spawn_version("hyprctl", &["version"]) {
            return Some(line);
        }
        return Some("Hyprland".to_string());
    }

    raw
}

fn detect_window_manager() -> Option<String> {
    let lower_de = env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_lowercase();
    let kde_full = env::var("KDE_FULL_SESSION").ok();

    if kde_full.is_some() || lower_de.contains("kde") || lower_de.contains("plasma") {
        return Some("KWin".to_string());
    }

    if lower_de.contains("gnome") {
        return Some("Mutter".to_string());
    }

    if env::var("SWAYSOCK").is_ok() || lower_de.contains("sway") {
        return Some("Sway".to_string());
    }

    if lower_de.contains("hyprland") || env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return Some("Hyprland".to_string());
    }

    if let Ok(wmde) = env::var("WMDE") {
        if !wmde.trim().is_empty() {
            return Some(wmde);
        }
    }

    if env::var("DISPLAY").is_ok() {
        if let Some(wm) = detect_window_manager_from_xprop() {
            return Some(wm);
        }
    }

    None
}

/// Extract the value from a single `xprop` output line.
fn xprop_value(output: &str) -> Option<String> {
    let (_, value) = output.lines().find_map(|line| line.split_once('='))?;
    let value = value.trim().trim_matches('"').trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

/// Ask the running X server for the window manager's advertised name.
fn detect_window_manager_from_xprop() -> Option<String> {
    let root = Command::new("xprop")
        .args(["-root", "_NET_SUPPORTING_WM_CHECK"])
        .output()
        .ok()?;
    if !root.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&root.stdout);
    let id = stdout
        .split_once("window id #")?
        .1
        .split_whitespace()
        .next()?;

    let name = Command::new("xprop")
        .args(["-id", id, "_NET_WM_NAME"])
        .output()
        .ok()?;
    if !name.status.success() {
        return None;
    }

    xprop_value(&String::from_utf8_lossy(&name.stdout))
}

struct GtkSettings {
    gtk_theme: Option<String>,
    icon_theme: Option<String>,
    cursor_theme: Option<String>,
    font: Option<String>,
}

fn config_home() -> PathBuf {
    if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg);
        if !path.as_os_str().is_empty() {
            return path;
        }
    }

    PathBuf::from(env::var("HOME").unwrap_or_default()).join(".config")
}

fn read_gtk_settings() -> GtkSettings {
    let base = config_home();
    let candidates = [
        base.join("gtk-4.0").join("settings.ini"),
        base.join("gtk-3.0").join("settings.ini"),
    ];

    for path in &candidates {
        if let Ok(content) = fs::read_to_string(path) {
            let settings = parse_ini_section(&content, "Settings");
            let gtk_theme = settings.get("gtk-theme-name").cloned();
            let icon_theme = settings.get("gtk-icon-theme-name").cloned();
            let cursor_theme = settings.get("gtk-cursor-theme-name").cloned();
            let font = settings.get("gtk-font-name").cloned();

            if gtk_theme.is_some()
                || icon_theme.is_some()
                || cursor_theme.is_some()
                || font.is_some()
            {
                return GtkSettings {
                    gtk_theme,
                    icon_theme,
                    cursor_theme,
                    font,
                };
            }
        }
    }

    let home = env::var("HOME").unwrap_or_default();
    let cursor_theme = read_default_cursor_theme(&home);

    GtkSettings {
        gtk_theme: None,
        icon_theme: None,
        cursor_theme,
        font: None,
    }
}

fn read_default_cursor_theme(home: &str) -> Option<String> {
    let path = format!("{}/.icons/default/index.theme", home);
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if let Some(val) = line.strip_prefix("Inherits=") {
            return Some(val.trim().to_string());
        }
    }
    None
}

fn parse_ini_section(content: &str, section: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let mut in_section = false;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_section = &line[1..line.len() - 1] == section;
            continue;
        }
        if !in_section {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    map
}

pub fn collect() -> DesktopInfo {
    let display_server = detect_display_server();
    let desktop_env = detect_desktop_env();
    let window_manager = detect_window_manager();
    let gtk = read_gtk_settings();

    DesktopInfo {
        desktop_env,
        window_manager,
        display_server,
        gtk_theme: gtk.gtk_theme,
        icon_theme: gtk.icon_theme,
        cursor_theme: gtk.cursor_theme,
        font: gtk.font,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ini_section() {
        let content = "[Settings]\ngtk-theme-name=Adwaita\n\n[Other]\ngtk-theme-name=Ignored\n";
        let settings = parse_ini_section(content, "Settings");
        assert_eq!(
            settings.get("gtk-theme-name").map(String::as_str),
            Some("Adwaita")
        );
    }

    #[test]
    fn test_parse_ini_section_skips_comments() {
        let content = "[Settings]\n# gtk-theme-name=Commented\n; gtk-icon-theme-name=AlsoCommented\ngtk-theme-name=Adwaita\n";
        let settings = parse_ini_section(content, "Settings");
        assert_eq!(
            settings.get("gtk-theme-name").map(String::as_str),
            Some("Adwaita")
        );
        assert!(!settings.contains_key("gtk-icon-theme-name"));
    }

    #[test]
    fn test_xprop_value() {
        let output = "_NET_WM_NAME(UTF8_STRING) = \"KWin\"\n";
        assert_eq!(xprop_value(output), Some(String::from("KWin")));
        assert_eq!(xprop_value("_NET_WM_NAME:  not found."), None);
        assert_eq!(xprop_value("_NET_WM_NAME(UTF8_STRING) = \"\""), None);
    }
}
