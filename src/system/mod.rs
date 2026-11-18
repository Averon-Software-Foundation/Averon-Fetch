// SPDX-License-Identifier: GPL-3.0-or-later

pub mod audio;
pub mod battery;
pub mod cpu;
pub mod desktop;
pub mod display;
mod dmi;
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
    let plan = *plan;

    // Take the usage baseline before starting the other collectors: their
    // runtime then counts towards the sampling window instead of adding a
    // fixed delay on top of it.
    let usage = if plan.cpu_usage {
        cpu::UsageProbe::start()
    } else {
        None
    };

    // `System::new` is fallible; if it fails, the collectors that need it fall
    // back to their defaults instead of aborting the whole run.
    let mut sys = sysinfo::System::new().ok();
    if let Some(sys) = sys.as_mut() {
        if plan.cpu {
            sys.refresh_cpu_all();
        }
        if plan.memory {
            sys.refresh_memory();
        }
    }

    // Most collectors either spawn a helper process or walk sysfs, so they
    // spend their time waiting rather than computing. Run them concurrently to
    // make the total the slowest collector instead of the sum of all of them.
    // Longest-running collectors are started first so they finish earliest.
    let desktop = spawn_collector(plan.desktop, desktop::collect);
    let gpu = spawn_collector(plan.gpu, move || gpu::collect(plan.gpu_details));
    let packages = spawn_collector(plan.packages, packages::collect);
    let audio = spawn_collector(plan.audio, audio::collect);
    let display = spawn_collector(plan.display, display::collect);
    let storage = spawn_collector(plan.storage, storage::collect);
    let network = spawn_collector(plan.network, network::collect);
    let shell = spawn_collector(plan.shell, shell::collect);
    let battery = spawn_collector(plan.battery, battery::collect);
    let os = spawn_collector(plan.os, os::collect);
    let locale = spawn_collector(plan.locale, locale::collect);
    let motherboard = spawn_collector(plan.motherboard, motherboard::collect);

    // These read from the shared `System` handle and are already cached, so
    // collecting them here is cheaper than moving the handle into a thread.
    let general = if plan.general {
        general::collect()
    } else {
        general::GeneralInfo::default()
    };
    let memory = if plan.memory {
        sys.as_ref().map(memory::collect).unwrap_or_default()
    } else {
        memory::MemoryInfo::default()
    };

    SystemInfo {
        general,
        memory,
        os: join_collector(os).unwrap_or_default(),
        gpu: join_collector(gpu).unwrap_or_default(),
        storage: join_collector(storage).unwrap_or_default(),
        network: join_collector(network).unwrap_or_default(),
        display: join_collector(display).unwrap_or_default(),
        desktop: join_collector(desktop).unwrap_or_default(),
        packages: join_collector(packages).unwrap_or_default(),
        shell: join_collector(shell).unwrap_or_default(),
        locale: join_collector(locale).unwrap_or_default(),
        motherboard: join_collector(motherboard).unwrap_or_default(),
        battery: join_collector(battery).unwrap_or_default(),
        audio: join_collector(audio).unwrap_or_default(),
        // Last, so the usage window covers the collectors joined above.
        cpu: if plan.cpu {
            match sys.as_ref() {
                Some(sys) => cpu::collect(sys, usage, plan.cpu_temperature),
                None => cpu::CpuInfo::default(),
            }
        } else {
            cpu::CpuInfo::default()
        },
    }
}

fn spawn_collector<T, F>(enabled: bool, collect: F) -> Option<std::thread::JoinHandle<T>>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    if enabled {
        Some(std::thread::spawn(collect))
    } else {
        None
    }
}

fn join_collector<T>(handle: Option<std::thread::JoinHandle<T>>) -> Option<T> {
    let handle = handle?;
    match handle.join() {
        Ok(value) => Some(value),
        // Surface a panicking collector instead of silently dropping its row.
        Err(payload) => std::panic::resume_unwind(payload),
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

    #[test]
    fn test_collect_skips_unrequested_collectors() {
        let info = collect(&CollectPlan::default());
        assert!(info.gpu.is_empty());
        assert!(info.storage.is_empty());
        assert!(info.network.is_empty());
        assert!(info.packages.counts.is_empty());
        assert!(info.cpu.name.is_none());
        assert_eq!(info.cpu.logical_cores, 0);
    }

    #[test]
    fn test_collect_returns_requested_collector_data() {
        let plan = CollectPlan {
            os: true,
            ..CollectPlan::default()
        };
        let info = collect(&plan);
        assert!(info.os.kernel.is_some());
        assert!(info.gpu.is_empty());
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
