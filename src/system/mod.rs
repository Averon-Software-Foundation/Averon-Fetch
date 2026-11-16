pub mod audio;
pub mod battery;
pub mod cpu;
pub mod desktop;
pub mod display;
pub mod general;
pub mod gpu;
pub mod locale;
pub mod memory;
pub mod motherboard;
pub mod network;
pub mod os;
pub mod packages;
pub mod shell;
pub mod storage;

use crate::modules::Selection;

#[derive(Debug, Clone, Default)]
pub struct SystemInfo {
    pub general: general::GeneralInfo,
    pub os: os::OsInfo,
    pub cpu: cpu::CpuInfo,
    pub gpu: Vec<gpu::GpuInfo>,
    pub memory: memory::MemoryInfo,
    pub storage: Vec<storage::DiskInfo>,
    pub network: Vec<network::NetworkInterface>,
    pub display: Vec<display::DisplayInfo>,
    pub desktop: desktop::DesktopInfo,
    pub packages: packages::PackageInfo,
    pub shell: shell::ShellInfo,
    pub locale: locale::LocaleInfo,
    pub motherboard: motherboard::MotherboardInfo,
    pub battery: Vec<battery::BatteryInfo>,
    pub audio: audio::AudioInfo,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CollectPlan {
    pub general: bool,
    pub os: bool,
    pub cpu: bool,
    pub cpu_usage: bool,
    pub cpu_temperature: bool,
    pub gpu: bool,
    pub gpu_details: bool,
    pub memory: bool,
    pub storage: bool,
    pub network: bool,
    pub display: bool,
    pub desktop: bool,
    pub packages: bool,
    pub shell: bool,
    pub locale: bool,
    pub motherboard: bool,
    pub battery: bool,
    pub audio: bool,
}

impl CollectPlan {
    pub fn from_selection(selection: &Selection) -> Self {
        let gpu = selection.needs_any(&["gpu", "gpu_usage", "gpu_temperature", "gpu_vram"]);

        Self {
            general: selection.needs_any(&["username", "hostname", "host"]),
            os: selection.needs_any(&["os", "kernel", "uptime"]),
            cpu: selection.needs_any(&["cpu", "cpu_usage", "cpu_temperature"]),
            cpu_usage: selection.includes("cpu_usage"),
            cpu_temperature: selection.includes("cpu_temperature"),
            gpu,
            gpu_details: selection.needs_any(&["gpu_usage", "gpu_temperature", "gpu_vram"]),
            memory: selection.needs_any(&["memory", "swap"]),
            storage: selection.includes("disk"),
            network: selection.needs_any(&["network", "network_speed"]),
            display: selection.includes("display"),
            desktop: selection.needs_any(&["desktop", "wm", "display_server", "theme"]),
            packages: selection.includes("packages"),
            shell: selection.needs_any(&["shell", "terminal"]),
            locale: selection.includes("locale"),
            motherboard: selection.includes("bios"),
            battery: selection.includes("battery"),
            audio: selection.includes("audio"),
        }
    }

    pub fn all() -> Self {
        Self {
            general: true,
            os: true,
            cpu: true,
            cpu_usage: true,
            cpu_temperature: true,
            gpu: true,
            gpu_details: true,
            memory: true,
            storage: true,
            network: true,
            display: true,
            desktop: true,
            packages: true,
            shell: true,
            locale: true,
            motherboard: true,
            battery: true,
            audio: true,
        }
    }
}

pub fn collect(plan: &CollectPlan) -> SystemInfo {
    let mut sys = sysinfo::System::new();
    if plan.cpu {
        sys.refresh_cpu_all();
    }
    if plan.memory {
        sys.refresh_memory();
    }

    SystemInfo {
        general: if plan.general {
            general::collect(&sys)
        } else {
            general::GeneralInfo::default()
        },
        os: if plan.os {
            os::collect()
        } else {
            os::OsInfo::default()
        },
        cpu: if plan.cpu {
            cpu::collect(&sys, plan.cpu_usage, plan.cpu_temperature)
        } else {
            cpu::CpuInfo::default()
        },
        gpu: if plan.gpu {
            gpu::collect(plan.gpu_details)
        } else {
            Vec::new()
        },
        memory: if plan.memory {
            memory::collect(&sys)
        } else {
            memory::MemoryInfo::default()
        },
        storage: if plan.storage {
            storage::collect()
        } else {
            Vec::new()
        },
        network: if plan.network {
            network::collect()
        } else {
            Vec::new()
        },
        display: if plan.display {
            display::collect()
        } else {
            Vec::new()
        },
        desktop: if plan.desktop {
            desktop::collect()
        } else {
            desktop::DesktopInfo::default()
        },
        packages: if plan.packages {
            packages::collect()
        } else {
            packages::PackageInfo::default()
        },
        shell: if plan.shell {
            shell::collect()
        } else {
            shell::ShellInfo::default()
        },
        locale: if plan.locale {
            locale::collect()
        } else {
            locale::LocaleInfo::default()
        },
        motherboard: if plan.motherboard {
            motherboard::collect()
        } else {
            motherboard::MotherboardInfo::default()
        },
        battery: if plan.battery {
            battery::collect()
        } else {
            Vec::new()
        },
        audio: if plan.audio {
            audio::collect()
        } else {
            audio::AudioInfo::default()
        },
    }
}

pub fn collect_all() -> SystemInfo {
    collect(&CollectPlan::all())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_default_system_info_is_empty() {
        let info = SystemInfo::default();
        assert!(info.general.hostname.is_none());
        assert!(info.gpu.is_empty());
        assert!(info.network.is_empty());
        assert!(info.battery.is_empty());
        assert!(info.audio.devices.is_empty());
        assert_eq!(info.cpu.logical_cores, 0);
    }

    #[test]
    fn test_collect_plan_all() {
        let plan = CollectPlan::all();
        assert!(plan.general);
        assert!(plan.os);
        assert!(plan.gpu_details);
        assert!(plan.packages);
        assert!(plan.audio);
    }

    #[test]
    fn test_collect_plan_skips_unneeded_general_and_os_work() {
        let selection = Selection::default();
        let plan = CollectPlan::from_selection(&selection);
        assert!(!plan.general);
        assert!(!plan.os);
        assert!(!plan.cpu);
        assert!(!plan.memory);
    }

    #[test]
    fn test_collect_plan_enables_header_and_os_collectors() {
        let selection = selection_with(&["hostname", "kernel"]);
        let plan = CollectPlan::from_selection(&selection);
        assert!(plan.general);
        assert!(plan.os);
        assert!(!plan.cpu);
    }

    fn selection_with(ids: &[&'static str]) -> Selection {
        // Selection is normally built from config and CLI arguments; tests use
        // the public resolver with a minimal explicit module list.
        let args = crate::cli::Args::parse_from(
            std::iter::once("averon-fetch").chain(ids.iter().flat_map(|id| ["--module", *id])),
        );
        let config = crate::config::Config::default();
        Selection::resolve(&config, &args)
    }
}
