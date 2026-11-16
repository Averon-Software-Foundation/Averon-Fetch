pub mod general;
pub mod hardware;
pub mod network;
pub mod os;
pub mod software;

use crate::cli::Args;
use crate::config::Config;
use crate::diag;
use crate::system::SystemInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub module: &'static str,
    pub label: String,
    pub label_suffix: Option<String>,
    pub value: String,
}

impl Line {
    pub fn new(module: &'static str, label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            module,
            label: label.into(),
            label_suffix: None,
            value: value.into(),
        }
    }

    pub fn with_suffix(
        module: &'static str,
        label: impl Into<String>,
        suffix: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            module,
            label: label.into(),
            label_suffix: Some(suffix.into()),
            value: value.into(),
        }
    }
}

pub trait Module: Sync {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn render(&self, info: &SystemInfo, config: &Config) -> Vec<Line>;
}

pub static MODULES: &[&dyn Module] = &[
    &general::UsernameModule,
    &general::HostnameModule,
    &os::OsModule,
    &os::HostModule,
    &os::KernelModule,
    &os::UptimeModule,
    &software::PackagesModule,
    &software::ShellModule,
    &software::TerminalModule,
    &software::DisplayModule,
    &software::DesktopModule,
    &software::WindowManagerModule,
    &software::DisplayServerModule,
    &software::ThemeModule,
    &hardware::CpuModule,
    &hardware::CpuUsageModule,
    &hardware::CpuTemperatureModule,
    &hardware::GpuModule,
    &hardware::GpuUsageModule,
    &hardware::GpuTemperatureModule,
    &hardware::GpuVramModule,
    &hardware::MemoryModule,
    &hardware::SwapModule,
    &hardware::DiskModule,
    &network::NetworkModule,
    &network::NetworkSpeedModule,
    &hardware::BiosModule,
    &hardware::BatteryModule,
    &hardware::AudioModule,
    &software::LocaleModule,
];

pub fn find_module(id: &str) -> Option<&'static dyn Module> {
    MODULES.iter().copied().find(|module| module.id() == id)
}

pub fn canonical_id(input: &str) -> Option<&'static str> {
    let normalised = input.trim().to_lowercase().replace('-', "_");
    let id = match normalised.as_str() {
        "wm" | "window_manager" => "wm",
        "local_ip" | "ip" | "network" => "network",
        "motherboard" | "bios" => "bios",
        "gpu_util" | "gpu_utilisation" | "gpu_utilization" | "gpu_usage" => "gpu_usage",
        "cpu_temp" | "cpu_temperature" => "cpu_temperature",
        "gpu_temp" | "gpu_temperature" => "gpu_temperature",
        "gpu_memory" | "vram" | "gpu_vram" => "gpu_vram",
        "ds" | "display_server" => "display_server",
        other => other,
    };

    MODULES
        .iter()
        .copied()
        .find(|module| module.id() == id)
        .map(|module| module.id())
}

fn all_ids_in_order(config: &Config) -> Vec<&'static str> {
    let mut ids: Vec<&'static str> = Vec::new();

    for requested in &config.modules.order {
        match canonical_id(requested) {
            Some(id) => {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
            None => diag::warn(format!("unknown module in config order: {requested}")),
        }
    }

    for module in MODULES {
        if !ids.contains(&module.id()) {
            ids.push(module.id());
        }
    }

    ids
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Selection {
    ids: Vec<&'static str>,
}

impl Selection {
    pub fn resolve(config: &Config, args: &Args) -> Self {
        let ordered = all_ids_in_order(config);

        let ids = if !args.module.is_empty() {
            let mut selected = Vec::new();
            for requested in &args.module {
                match canonical_id(requested) {
                    Some(id) => {
                        if !selected.contains(&id) {
                            selected.push(id);
                        }
                    }
                    None => diag::warn(format!("unknown module: {requested}")),
                }
            }
            selected
        } else if args.all {
            ordered
        } else {
            ordered
                .into_iter()
                .filter(|id| config.modules.is_enabled(id))
                .collect()
        };

        Self { ids }
    }

    pub fn ids(&self) -> &[&'static str] {
        &self.ids
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub fn includes(&self, id: &str) -> bool {
        self.ids.contains(&id)
    }

    pub fn needs_any(&self, ids: &[&str]) -> bool {
        ids.iter().any(|id| self.includes(id))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Header {
    pub username: Option<String>,
    pub hostname: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Document {
    pub header: Option<Header>,
    pub lines: Vec<Line>,
}

pub fn build_document(
    info: &SystemInfo,
    config: &Config,
    selection: &Selection,
    hide_ips: bool,
) -> Document {
    let username = if selection.includes("username") {
        info.general.username.clone()
    } else {
        None
    };
    let hostname = if selection.includes("hostname") {
        info.general.hostname.clone()
    } else {
        None
    };
    let header = if username.is_some() || hostname.is_some() {
        Some(Header { username, hostname })
    } else {
        None
    };

    let mut lines = Vec::new();
    for id in selection.ids() {
        if *id == "username" || *id == "hostname" {
            continue;
        }

        if hide_ips && *id == "network" {
            continue;
        }
        if let Some(module) = find_module(id) {
            lines.extend(module.render(info, config));
        }
    }

    Document { header, lines }
}

pub fn list_modules(config: &Config) -> String {
    let mut output = String::from("Available modules:\n");

    for module in MODULES {
        let state = if config.modules.is_enabled(module.id()) {
            "enabled"
        } else {
            "disabled"
        };
        output.push_str(&format!(
            "  {:<18} {:<38} [{}]\n",
            module.id(),
            module.description(),
            state
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Args;
    use clap::Parser;

    #[test]
    fn test_registry_ids_are_unique() {
        let mut ids = Vec::new();
        for module in MODULES {
            assert!(
                !ids.contains(&module.id()),
                "duplicate module id: {}",
                module.id()
            );
            ids.push(module.id());
        }
    }

    #[test]
    fn test_canonical_aliases() {
        assert_eq!(canonical_id("window_manager"), Some("wm"));
        assert_eq!(canonical_id("Local-IP"), Some("network"));
        assert_eq!(canonical_id("motherboard"), Some("bios"));
        assert_eq!(canonical_id("gpu_util"), Some("gpu_usage"));
        assert_eq!(canonical_id("nope"), None);
    }

    #[test]
    fn test_default_selection_respects_config() {
        let config = Config::default();
        let args = Args::parse_from(["averon-fetch"]);
        let selection = Selection::resolve(&config, &args);
        assert!(selection.includes("os"));
        assert!(selection.includes("cpu"));
        assert!(!selection.includes("cpu_usage"));
        assert!(!selection.includes("gpu_vram"));
    }

    #[test]
    fn test_disabled_module_is_skipped() {
        let mut config = Config::default();
        config.modules.memory = false;
        let args = Args::parse_from(["averon-fetch"]);
        let selection = Selection::resolve(&config, &args);
        assert!(!selection.includes("memory"));
    }

    #[test]
    fn test_module_flag_overrides_config() {
        let mut config = Config::default();
        config.modules.cpu = false;
        let args = Args::parse_from(["averon-fetch", "--module", "cpu"]);
        let selection = Selection::resolve(&config, &args);
        assert_eq!(selection.ids(), &["cpu"]);
    }

    #[test]
    fn test_all_flag_includes_disabled_modules() {
        let mut config = Config::default();
        config.modules.battery = false;
        let args = Args::parse_from(["averon-fetch", "--all"]);
        let selection = Selection::resolve(&config, &args);
        assert!(selection.includes("battery"));
    }

    #[test]
    fn test_config_order_is_applied() {
        let mut config = Config::default();
        config.modules.order = vec![String::from("memory"), String::from("os")];
        let args = Args::parse_from(["averon-fetch"]);
        let selection = Selection::resolve(&config, &args);
        assert_eq!(&selection.ids()[..2], &["memory", "os"]);
    }

    #[test]
    fn test_unknown_explicit_module_is_ignored() {
        let config = Config::default();
        let args = Args::parse_from(["averon-fetch", "--module", "does_not_exist"]);
        let selection = Selection::resolve(&config, &args);
        assert!(selection.is_empty());
    }

    #[test]
    fn test_build_document_skips_missing_information() {
        let config = Config::default();
        let args = Args::parse_from(["averon-fetch"]);
        let selection = Selection::resolve(&config, &args);
        let info = SystemInfo::default();
        let document = build_document(&info, &config, &selection, false);
        assert!(document.header.is_none());
        assert!(document.lines.is_empty());
    }

    #[test]
    fn test_noip_hides_network_module_only() {
        let config = Config::default();
        let args = Args::parse_from([
            "averon-fetch",
            "--module",
            "network",
            "--module",
            "network_speed",
        ]);
        let selection = Selection::resolve(&config, &args);
        let mut info = SystemInfo::default();
        info.network.push(crate::system::network::NetworkInterface {
            name: String::from("eth0"),
            ip_v4: vec![String::from("192.168.1.2")],
            ip_v6: Vec::new(),
            is_up: true,
            speed_mbps: Some(1000),
        });

        let document = build_document(&info, &config, &selection, true);
        assert_eq!(document.lines.len(), 1);
        assert_eq!(document.lines[0].module, "network_speed");
    }
}
