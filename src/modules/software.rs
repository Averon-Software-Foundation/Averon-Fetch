// SPDX-License-Identifier: GPL-3.0-or-later

use crate::config::Config;
use crate::modules::{Line, Module};
use crate::system::SystemInfo;

pub struct PackagesModule;

impl Module for PackagesModule {
    fn id(&self) -> &'static str {
        "packages"
    }

    fn label(&self) -> &'static str {
        "Packages"
    }

    fn description(&self) -> &'static str {
        "Installed package counts"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        if info.packages.counts.is_empty() {
            return Vec::new();
        }
        let value = info
            .packages
            .counts
            .iter()
            .map(|package| format!("{} ({})", package.count, package.manager))
            .collect::<Vec<_>>()
            .join(", ");
        vec![Line::new(self.id(), self.label(), value)]
    }
}

pub struct ShellModule;

impl Module for ShellModule {
    fn id(&self) -> &'static str {
        "shell"
    }

    fn label(&self) -> &'static str {
        "Shell"
    }

    fn description(&self) -> &'static str {
        "Shell name and version"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        let Some(name) = &info.shell.name else {
            return Vec::new();
        };
        let value = match &info.shell.version {
            Some(version) => format!("{name} {version}"),
            None => name.clone(),
        };
        vec![Line::new(self.id(), self.label(), value)]
    }
}

pub struct TerminalModule;

impl Module for TerminalModule {
    fn id(&self) -> &'static str {
        "terminal"
    }

    fn label(&self) -> &'static str {
        "Terminal"
    }

    fn description(&self) -> &'static str {
        "Terminal emulator"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        let Some(terminal) = &info.shell.terminal else {
            return Vec::new();
        };
        let value = match &info.shell.terminal_version {
            Some(version) => format!("{terminal} {version}"),
            None => terminal.clone(),
        };
        vec![Line::new(self.id(), self.label(), value)]
    }
}

pub struct DisplayModule;

impl Module for DisplayModule {
    fn id(&self) -> &'static str {
        "display"
    }

    fn label(&self) -> &'static str {
        "Display"
    }

    fn description(&self) -> &'static str {
        "Connected displays"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        let mut lines = Vec::new();

        for display in &info.display {
            let mut parts = Vec::new();
            if let Some(name) = &display.name {
                parts.push(name.clone());
            } else if let Some(connector) = &display.connector {
                parts.push(connector.clone());
            }

            if let (Some(width), Some(height)) =
                (display.resolution_width, display.resolution_height)
            {
                let resolution = match display.refresh_rate {
                    Some(rate) => format!("{width}x{height} @ {:.0}Hz", rate.round()),
                    None => format!("{width}x{height}"),
                };
                parts.push(resolution);
            }

            if !parts.is_empty() {
                lines.push(Line::new(self.id(), self.label(), parts.join(" ")));
            }
        }

        lines
    }
}

pub struct DesktopModule;

impl Module for DesktopModule {
    fn id(&self) -> &'static str {
        "desktop"
    }

    fn label(&self) -> &'static str {
        "Desktop"
    }

    fn description(&self) -> &'static str {
        "Desktop environment"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        match &info.desktop.desktop_env {
            Some(desktop) => vec![Line::new(self.id(), self.label(), desktop)],
            None => Vec::new(),
        }
    }
}

pub struct WindowManagerModule;

impl Module for WindowManagerModule {
    fn id(&self) -> &'static str {
        "wm"
    }

    fn label(&self) -> &'static str {
        "WM"
    }

    fn description(&self) -> &'static str {
        "Window manager"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        match &info.desktop.window_manager {
            Some(wm) => vec![Line::new(self.id(), self.label(), wm)],
            None => Vec::new(),
        }
    }
}

pub struct DisplayServerModule;

impl Module for DisplayServerModule {
    fn id(&self) -> &'static str {
        "display_server"
    }

    fn label(&self) -> &'static str {
        "Display Server"
    }

    fn description(&self) -> &'static str {
        "Wayland or X11"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        match &info.desktop.display_server {
            Some(server) => vec![Line::new(self.id(), self.label(), server)],
            None => Vec::new(),
        }
    }
}

pub struct ThemeModule;

impl Module for ThemeModule {
    fn id(&self) -> &'static str {
        "theme"
    }

    fn label(&self) -> &'static str {
        "Theme"
    }

    fn description(&self) -> &'static str {
        "GTK, icon, cursor, and font settings"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        let mut lines = Vec::new();

        if let Some(theme) = &info.desktop.gtk_theme {
            lines.push(Line::new(self.id(), self.label(), theme));
        }
        if let Some(icons) = &info.desktop.icon_theme {
            lines.push(Line::new(self.id(), "Icons", icons));
        }
        if let Some(cursor) = &info.desktop.cursor_theme {
            lines.push(Line::new(self.id(), "Cursor", cursor));
        }
        if let Some(font) = &info.desktop.font {
            lines.push(Line::new(self.id(), "Font", font));
        }

        lines
    }
}

pub struct LocaleModule;

impl Module for LocaleModule {
    fn id(&self) -> &'static str {
        "locale"
    }

    fn label(&self) -> &'static str {
        "Locale"
    }

    fn description(&self) -> &'static str {
        "Locale and keyboard layout"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        let mut lines = Vec::new();

        if let Some(locale) = &info.locale.locale {
            lines.push(Line::new(self.id(), self.label(), locale));
        }
        if let Some(layout) = &info.locale.keyboard_layout {
            lines.push(Line::new(self.id(), "Keyboard", layout));
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::display::DisplayInfo;

    fn config() -> Config {
        Config::default()
    }

    #[test]
    fn test_display_module_reports_each_connected_monitor() {
        let info = SystemInfo {
            display: vec![
                DisplayInfo {
                    name: Some(String::from("Primary Monitor")),
                    resolution_width: Some(2560),
                    resolution_height: Some(1440),
                    refresh_rate: Some(165.0),
                    is_primary: true,
                    ..Default::default()
                },
                DisplayInfo {
                    name: Some(String::from("Secondary Monitor")),
                    resolution_width: Some(1920),
                    resolution_height: Some(1080),
                    refresh_rate: Some(60.0),
                    is_primary: false,
                    ..Default::default()
                },
            ],
            ..SystemInfo::default()
        };

        let lines = DisplayModule.render(&info, &config());
        assert_eq!(lines.len(), 2);
        assert!(lines[0].value.contains("2560x1440 @ 165Hz"));
        assert!(lines[1].value.contains("1920x1080 @ 60Hz"));
    }

    #[test]
    fn test_theme_module_emits_only_available_settings() {
        let mut info = SystemInfo::default();
        info.desktop.gtk_theme = Some(String::from("Averon-Dark"));
        info.desktop.icon_theme = Some(String::from("Averon-Icons"));
        let lines = ThemeModule.render(&info, &config());
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].label, "Theme");
        assert_eq!(lines[1].label, "Icons");
    }

    #[test]
    fn test_empty_software_info_emits_nothing() {
        let info = SystemInfo::default();
        assert!(PackagesModule.render(&info, &config()).is_empty());
        assert!(ShellModule.render(&info, &config()).is_empty());
        assert!(DisplayModule.render(&info, &config()).is_empty());
        assert!(LocaleModule.render(&info, &config()).is_empty());
    }
}
