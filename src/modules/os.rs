use crate::config::Config;
use crate::modules::{Line, Module};
use crate::system::os::format_uptime;
use crate::system::SystemInfo;

pub struct OsModule;

impl Module for OsModule {
    fn id(&self) -> &'static str {
        "os"
    }

    fn label(&self) -> &'static str {
        "OS"
    }

    fn description(&self) -> &'static str {
        "Operating system and architecture"
    }

    fn render(&self, info: &SystemInfo, config: &Config) -> Vec<Line> {
        let Some(name) = &info.os.name else {
            return Vec::new();
        };

        let value = match &info.os.arch {
            Some(arch) if config.modules.arch => format!("{name} {arch}"),
            _ => name.clone(),
        };
        vec![Line::new(self.id(), self.label(), value)]
    }
}

pub struct HostModule;

impl Module for HostModule {
    fn id(&self) -> &'static str {
        "host"
    }

    fn label(&self) -> &'static str {
        "Host"
    }

    fn description(&self) -> &'static str {
        "Machine model or motherboard"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        match &info.general.host {
            Some(host) => vec![Line::new(self.id(), self.label(), host)],
            None => Vec::new(),
        }
    }
}

pub struct KernelModule;

impl Module for KernelModule {
    fn id(&self) -> &'static str {
        "kernel"
    }

    fn label(&self) -> &'static str {
        "Kernel"
    }

    fn description(&self) -> &'static str {
        "Kernel version"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        match &info.os.kernel {
            Some(kernel) => vec![Line::new(self.id(), self.label(), kernel)],
            None => Vec::new(),
        }
    }
}

pub struct UptimeModule;

impl Module for UptimeModule {
    fn id(&self) -> &'static str {
        "uptime"
    }

    fn label(&self) -> &'static str {
        "Uptime"
    }

    fn description(&self) -> &'static str {
        "System uptime"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        if info.os.uptime_secs == 0 {
            return Vec::new();
        }
        vec![Line::new(
            self.id(),
            self.label(),
            format_uptime(info.os.uptime_secs),
        )]
    }
}
