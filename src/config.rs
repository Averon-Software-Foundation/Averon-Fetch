// SPDX-License-Identifier: GPL-3.0-or-later

use crate::colour::ColorChoice;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub logo: LogoConfig,
    #[serde(alias = "colors")]
    pub colours: ColoursConfig,
    #[serde(alias = "color")]
    pub terminal: TerminalConfig,
    pub modules: ModulesConfig,

    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub labels: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(default)]
pub struct GeneralConfig {
    pub logo: bool,

    #[serde(alias = "color")]
    pub colour: bool,

    pub separator: String,

    pub show_errors: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(default)]
pub struct LogoConfig {
    pub width: u32,

    pub scale: f32,

    pub custom_ascii: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(default)]
pub struct ColoursConfig {
    pub label: String,
    pub value: String,
    pub separator: String,
    pub accent: String,
    pub header_user: String,
    pub header_host: String,
    pub logo_primary: String,
    pub logo_accent: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(default)]
pub struct TerminalConfig {
    #[serde(alias = "color")]
    pub colour: ColorChoice,

    pub truecolour: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(default)]
pub struct ModulesConfig {
    pub username: bool,
    pub hostname: bool,
    pub os: bool,
    pub arch: bool,
    pub host: bool,
    pub kernel: bool,
    pub uptime: bool,
    pub packages: bool,
    pub shell: bool,
    pub terminal: bool,
    pub display: bool,
    pub desktop: bool,
    // Accept the module id used by `--module` and `[modules] order`.
    #[serde(alias = "wm")]
    pub window_manager: bool,
    pub display_server: bool,
    pub theme: bool,
    pub cpu: bool,
    pub cpu_usage: bool,
    pub cpu_temperature: bool,
    pub gpu: bool,
    pub gpu_usage: bool,
    pub gpu_temperature: bool,
    pub gpu_vram: bool,
    pub memory: bool,
    pub swap: bool,
    pub disk: bool,
    pub network: bool,
    pub network_speed: bool,
    pub bios: bool,
    pub battery: bool,
    pub audio: bool,
    pub locale: bool,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub order: Vec<String>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            logo: true,
            colour: true,
            separator: String::from(": "),
            show_errors: false,
        }
    }
}

impl Default for LogoConfig {
    fn default() -> Self {
        Self {
            width: 28,
            scale: 1.0,
            custom_ascii: None,
        }
    }
}

impl Default for ColoursConfig {
    fn default() -> Self {
        Self {
            label: String::from("#5B53EB"),
            value: String::from("#DCDCDC"),
            separator: String::from("#64D8F5"),
            accent: String::from("#5B53EB"),
            header_user: String::from("#DCDCDC"),
            header_host: String::from("#5B53EB"),
            logo_primary: String::from("#DCDCDC"),
            logo_accent: String::from("#5B53EB"),
        }
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            colour: ColorChoice::Auto,
            truecolour: true,
        }
    }
}

impl Default for ModulesConfig {
    fn default() -> Self {
        Self {
            username: true,
            hostname: true,
            os: true,
            arch: true,
            host: true,
            kernel: true,
            uptime: true,
            packages: true,
            shell: true,
            terminal: true,
            display: true,
            desktop: true,
            window_manager: true,
            display_server: true,
            theme: true,
            cpu: true,
            cpu_usage: false,
            cpu_temperature: true,
            gpu: true,
            gpu_usage: false,
            gpu_temperature: false,
            gpu_vram: false,
            memory: true,
            swap: true,
            disk: true,
            network: true,
            network_speed: false,
            bios: true,
            battery: true,
            audio: false,
            locale: true,
            order: Vec::new(),
        }
    }
}

impl ModulesConfig {
    pub fn is_enabled(&self, id: &str) -> bool {
        match id {
            "username" => self.username,
            "hostname" => self.hostname,
            "os" => self.os,
            "arch" => self.arch,
            "host" => self.host,
            "kernel" => self.kernel,
            "uptime" => self.uptime,
            "packages" => self.packages,
            "shell" => self.shell,
            "terminal" => self.terminal,
            "display" => self.display,
            "desktop" => self.desktop,
            "wm" => self.window_manager,
            "display_server" => self.display_server,
            "theme" => self.theme,
            "cpu" => self.cpu,
            "cpu_usage" => self.cpu_usage,
            "cpu_temperature" => self.cpu_temperature,
            "gpu" => self.gpu,
            "gpu_usage" => self.gpu_usage,
            "gpu_temperature" => self.gpu_temperature,
            "gpu_vram" => self.gpu_vram,
            "memory" => self.memory,
            "swap" => self.swap,
            "disk" => self.disk,
            "network" => self.network,
            "network_speed" => self.network_speed,
            "bios" => self.bios,
            "battery" => self.battery,
            "audio" => self.audio,
            "locale" => self.locale,
            _ => false,
        }
    }
}

impl Config {
    pub fn load_from(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(contents) => match toml::from_str::<Config>(&contents) {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!(
                        "averon-fetch: failed to parse config {}: {e}",
                        path.display()
                    );
                    Config::default()
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Config::default(),
            Err(e) => {
                eprintln!(
                    "averon-fetch: failed to read config {}: {e}",
                    path.display()
                );
                Config::default()
            }
        }
    }

    pub fn path() -> PathBuf {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            if !xdg.is_empty() {
                return PathBuf::from(xdg).join("averon-fetch").join("config.toml");
            }
        }

        let base = std::env::var("HOME").unwrap_or_else(|_| String::from("."));
        PathBuf::from(base)
            .join(".config")
            .join("averon-fetch")
            .join("config.toml")
    }

    pub fn resolve_path(explicit: Option<&str>) -> PathBuf {
        match explicit {
            Some(p) if !p.is_empty() => expand_tilde(p),
            _ => Self::path(),
        }
    }

    pub fn custom_label(&self, module: &str, default_label: &str) -> Option<&str> {
        if let Some(value) = self.labels.get(module) {
            return Some(value.as_str());
        }

        self.labels
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(default_label))
            .map(|(_, value)| value.as_str())
    }

    pub fn resolve_label(&self, module: &str, label: &str, suffix: Option<&str>) -> String {
        let base = self
            .custom_label(module, label)
            .unwrap_or(label)
            .to_string();
        match suffix {
            Some(s) if !s.is_empty() => format!("{base} ({s})"),
            _ => base,
        }
    }

    pub fn write_default(path: &Path) -> std::io::Result<bool> {
        if path.exists() {
            return Ok(false);
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write beside the destination and rename, so an interrupted write
        // never leaves a truncated config behind.
        let temp = path.with_extension(format!("tmp.{}", std::process::id()));
        std::fs::write(&temp, DEFAULT_CONFIG_TEMPLATE)?;
        if let Err(error) = std::fs::rename(&temp, path) {
            let _ = std::fs::remove_file(&temp);
            return Err(error);
        }
        Ok(true)
    }
}

fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        return std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("~"));
    }
    if let Some(rest) = path.strip_prefix("~/") {
        let home = std::env::var("HOME").unwrap_or_else(|_| String::from("~"));
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(path)
}

pub const DEFAULT_CONFIG_TEMPLATE: &str = r##"# Averon Fetch configuration
# Generated by: averon-fetch --generate-config
# All fields are optional. Omitted values use the defaults shown here.

[general]
# Show the logo beside the information block.
logo = true
# Legacy colour toggle. Also respects NO_COLOR and non-tty output.
colour = true
# Text between a label and its value.
separator = ": "
# Print collector diagnostics to stderr.
show_errors = false

[logo]
# Target width in terminal columns.
width = 28
# Multiplicative scale. 1.0 uses width as-is.
scale = 1.0
# Optional replacement ASCII art.
# custom_ascii = """
#   /\
#  /  \
# /____\
# """

[colours]
label = "#5B53EB"
value = "#DCDCDC"
separator = "#64D8F5"
accent = "#5B53EB"
header_user = "#DCDCDC"
header_host = "#5B53EB"
logo_primary = "#DCDCDC"
logo_accent = "#5B53EB"

[terminal]
# auto, always, or never.
colour = "auto"
# Use 24-bit colour. When false, a 256-colour approximation is used.
truecolour = true

[modules]
username = true
hostname = true
os = true
# Include the architecture suffix on the OS row.
arch = true
host = true
kernel = true
uptime = true
packages = true
shell = true
terminal = true
display = true
desktop = true
window_manager = true
display_server = true
theme = true
cpu = true
# CPU usage requires a short sampling interval.
cpu_usage = false
cpu_temperature = true
gpu = true
gpu_usage = false
gpu_temperature = false
gpu_vram = false
memory = true
swap = true
disk = true
network = true
network_speed = false
bios = true
battery = true
audio = false
locale = true
# Optional ordering. Unlisted modules keep their built-in order.
# order = ["os", "host", "kernel", "uptime", "cpu", "memory", "disk"]

[labels]
# Custom labels may use module ids or default labels.
# os = "System"
# "Local IP" = "IP"
"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = Config::default();
        assert!(cfg.general.logo);
        assert!(cfg.general.colour);
        assert_eq!(cfg.general.separator, ": ");
        assert_eq!(cfg.logo.width, 28);
        assert_eq!(cfg.logo.scale, 1.0);
        assert!(cfg.modules.cpu);
        assert!(cfg.modules.memory);
        assert!(cfg.modules.os);
        assert!(!cfg.modules.cpu_usage);
        assert!(!cfg.modules.gpu_vram);
    }

    #[test]
    fn test_deserialize_partial_toml() {
        let toml_str = r#"
            [general]
            logo = false
            separator = " -> "

            [modules]
            cpu = true
            memory = false
        "#;
        let cfg: Config = toml::from_str(toml_str).expect("failed to deserialize");
        assert!(!cfg.general.logo);
        assert!(cfg.general.colour);
        assert_eq!(cfg.general.separator, " -> ");
        assert!(cfg.modules.cpu);
        assert!(!cfg.modules.memory);
        assert!(cfg.modules.os);
    }

    #[test]
    fn test_default_template_round_trip() {
        let cfg: Config =
            toml::from_str(DEFAULT_CONFIG_TEMPLATE).expect("default template must parse");
        assert_eq!(cfg, Config::default());
    }

    #[test]
    fn test_invalid_toml_is_rejected() {
        assert!(toml::from_str::<Config>("this is not toml = [").is_err());
    }

    #[test]
    fn test_custom_labels_and_suffixes() {
        let mut cfg = Config::default();
        cfg.labels
            .insert(String::from("disk"), String::from("Storage"));
        cfg.labels
            .insert(String::from("os"), String::from("System"));

        assert_eq!(cfg.resolve_label("disk", "Disk", Some("/")), "Storage (/)");
        assert_eq!(cfg.resolve_label("os", "OS", None), "System");
        assert_eq!(cfg.resolve_label("kernel", "Kernel", None), "Kernel");
    }

    #[test]
    fn test_custom_label_by_default_label_is_case_insensitive() {
        let mut cfg = Config::default();
        cfg.labels
            .insert(String::from("local ip"), String::from("Address"));
        assert_eq!(cfg.resolve_label("network", "Local IP", None), "Address");
    }

    #[test]
    fn test_explicit_config_path() {
        assert_eq!(
            Config::resolve_path(Some("/tmp/averon.toml")),
            PathBuf::from("/tmp/averon.toml")
        );
    }

    #[test]
    fn test_colour_aliases() {
        let cfg: Config = toml::from_str(
            r##"
            [colors]
            label = "#112233"

            [terminal]
            color = "never"
        "##,
        )
        .expect("colour aliases should parse");
        assert_eq!(cfg.colours.label, "#112233");
        assert_eq!(cfg.terminal.colour, ColorChoice::Never);
    }
}
