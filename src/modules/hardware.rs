use crate::config::Config;
use crate::modules::{Line, Module};
use crate::system::memory::{format_bytes_gib, usage_percent};
use crate::system::SystemInfo;

pub struct CpuModule;

impl Module for CpuModule {
    fn id(&self) -> &'static str {
        "cpu"
    }

    fn label(&self) -> &'static str {
        "CPU"
    }

    fn description(&self) -> &'static str {
        "CPU model, cores, and frequency"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        let Some(name) = &info.cpu.name else {
            return Vec::new();
        };

        let mut value = name.clone();
        if let Some(physical) = info.cpu.physical_cores {
            if physical > 0 && info.cpu.logical_cores > 0 {
                value = format!("{value} ({physical}C/{}T)", info.cpu.logical_cores);
            } else if info.cpu.logical_cores > 0 {
                value = format!("{value} ({})", info.cpu.logical_cores);
            }
        } else if info.cpu.logical_cores > 0 {
            value = format!("{value} ({})", info.cpu.logical_cores);
        }

        if let Some(frequency) = info.cpu.frequency_mhz {
            if frequency > 0 {
                if frequency >= 1000 {
                    value = format!("{value} @ {:.2} GHz", frequency as f64 / 1000.0);
                } else {
                    value = format!("{value} @ {frequency} MHz");
                }
            }
        }

        vec![Line::new(self.id(), self.label(), value)]
    }
}

pub struct CpuUsageModule;

impl Module for CpuUsageModule {
    fn id(&self) -> &'static str {
        "cpu_usage"
    }

    fn label(&self) -> &'static str {
        "CPU Usage"
    }

    fn description(&self) -> &'static str {
        "Current CPU utilisation"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        match info.cpu.usage_percent {
            Some(usage) => vec![Line::new(self.id(), self.label(), format!("{usage:.0}%"))],
            None => Vec::new(),
        }
    }
}

pub struct CpuTemperatureModule;

impl Module for CpuTemperatureModule {
    fn id(&self) -> &'static str {
        "cpu_temperature"
    }

    fn label(&self) -> &'static str {
        "CPU Temp"
    }

    fn description(&self) -> &'static str {
        "CPU temperature"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        match info.cpu.temperature_c {
            Some(temperature) => vec![Line::new(
                self.id(),
                self.label(),
                format!("{temperature:.1} °C"),
            )],
            None => Vec::new(),
        }
    }
}

pub struct GpuModule;

impl Module for GpuModule {
    fn id(&self) -> &'static str {
        "gpu"
    }

    fn label(&self) -> &'static str {
        "GPU"
    }

    fn description(&self) -> &'static str {
        "GPU model and driver"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        info.gpu
            .iter()
            .map(|gpu| {
                let value = match &gpu.driver {
                    Some(driver) => format!("{} [{driver}]", gpu.name),
                    None => gpu.name.clone(),
                };
                Line::new(self.id(), self.label(), value)
            })
            .collect()
    }
}

fn detail_line(
    module: &'static str,
    label: &'static str,
    index: usize,
    total: usize,
    value: impl Into<String>,
) -> Line {
    if total > 1 {
        Line::with_suffix(module, label, (index + 1).to_string(), value)
    } else {
        Line::new(module, label, value)
    }
}

pub struct GpuUsageModule;

impl Module for GpuUsageModule {
    fn id(&self) -> &'static str {
        "gpu_usage"
    }

    fn label(&self) -> &'static str {
        "GPU Usage"
    }

    fn description(&self) -> &'static str {
        "GPU utilisation"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        let total = info.gpu.len();
        info.gpu
            .iter()
            .enumerate()
            .filter_map(|(index, gpu)| {
                gpu.utilization_percent.map(|usage| {
                    detail_line(
                        self.id(),
                        self.label(),
                        index,
                        total,
                        format!("{usage:.0}%"),
                    )
                })
            })
            .collect()
    }
}

pub struct GpuTemperatureModule;

impl Module for GpuTemperatureModule {
    fn id(&self) -> &'static str {
        "gpu_temperature"
    }

    fn label(&self) -> &'static str {
        "GPU Temp"
    }

    fn description(&self) -> &'static str {
        "GPU temperature"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        let total = info.gpu.len();
        info.gpu
            .iter()
            .enumerate()
            .filter_map(|(index, gpu)| {
                gpu.temperature_c.map(|temperature| {
                    detail_line(
                        self.id(),
                        self.label(),
                        index,
                        total,
                        format!("{temperature:.1} °C"),
                    )
                })
            })
            .collect()
    }
}

pub struct GpuVramModule;

impl Module for GpuVramModule {
    fn id(&self) -> &'static str {
        "gpu_vram"
    }

    fn label(&self) -> &'static str {
        "GPU VRAM"
    }

    fn description(&self) -> &'static str {
        "GPU video memory usage"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        let total = info.gpu.len();
        info.gpu
            .iter()
            .enumerate()
            .filter_map(|(index, gpu)| {
                let total_bytes = gpu.vram_total_bytes?;
                let value = match gpu.vram_used_bytes {
                    Some(used) => format!(
                        "{} / {}",
                        format_bytes_gib(used),
                        format_bytes_gib(total_bytes)
                    ),
                    None => format_bytes_gib(total_bytes),
                };
                Some(detail_line(self.id(), self.label(), index, total, value))
            })
            .collect()
    }
}

pub struct MemoryModule;

impl Module for MemoryModule {
    fn id(&self) -> &'static str {
        "memory"
    }

    fn label(&self) -> &'static str {
        "Memory"
    }

    fn description(&self) -> &'static str {
        "RAM usage"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        if info.memory.total_bytes == 0 {
            return Vec::new();
        }
        let percent = usage_percent(info.memory.used_bytes, info.memory.total_bytes);
        vec![Line::new(
            self.id(),
            self.label(),
            format!(
                "{} / {} ({percent:.0}%)",
                format_bytes_gib(info.memory.used_bytes),
                format_bytes_gib(info.memory.total_bytes)
            ),
        )]
    }
}

pub struct SwapModule;

impl Module for SwapModule {
    fn id(&self) -> &'static str {
        "swap"
    }

    fn label(&self) -> &'static str {
        "Swap"
    }

    fn description(&self) -> &'static str {
        "Swap usage"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        if info.memory.total_swap_bytes == 0 {
            return Vec::new();
        }
        let percent = usage_percent(info.memory.used_swap_bytes, info.memory.total_swap_bytes);
        vec![Line::new(
            self.id(),
            self.label(),
            format!(
                "{} / {} ({percent:.0}%)",
                format_bytes_gib(info.memory.used_swap_bytes),
                format_bytes_gib(info.memory.total_swap_bytes)
            ),
        )]
    }
}

pub struct DiskModule;

impl Module for DiskModule {
    fn id(&self) -> &'static str {
        "disk"
    }

    fn label(&self) -> &'static str {
        "Disk"
    }

    fn description(&self) -> &'static str {
        "Mounted disk usage"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        info.storage
            .iter()
            .filter(|disk| disk.total_bytes > 0)
            .map(|disk| {
                let percent = usage_percent(disk.used_bytes, disk.total_bytes);
                Line::with_suffix(
                    self.id(),
                    self.label(),
                    disk.mount_point.clone(),
                    format!(
                        "{} / {} ({percent:.0}%)",
                        format_bytes_gib(disk.used_bytes),
                        format_bytes_gib(disk.total_bytes)
                    ),
                )
            })
            .collect()
    }
}

pub struct BiosModule;

impl Module for BiosModule {
    fn id(&self) -> &'static str {
        "bios"
    }

    fn label(&self) -> &'static str {
        "BIOS"
    }

    fn description(&self) -> &'static str {
        "Motherboard BIOS version"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        let Some(version) = &info.motherboard.bios_version else {
            return Vec::new();
        };
        let value = match &info.motherboard.board_name {
            Some(board) => format!("{version} ({board})"),
            None => version.clone(),
        };
        vec![Line::new(self.id(), self.label(), value)]
    }
}

pub struct BatteryModule;

impl Module for BatteryModule {
    fn id(&self) -> &'static str {
        "battery"
    }

    fn label(&self) -> &'static str {
        "Battery"
    }

    fn description(&self) -> &'static str {
        "Battery charge and status"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        let total = info.battery.len();
        info.battery
            .iter()
            .enumerate()
            .filter_map(|(index, battery)| {
                let mut parts = Vec::new();
                if let Some(capacity) = battery.capacity_percent {
                    parts.push(format!("{capacity}%"));
                }
                if let Some(status) = &battery.status {
                    parts.push(format!("({status})"));
                }
                if parts.is_empty() {
                    return None;
                }
                let value = parts.join(" ");
                Some(detail_line(self.id(), self.label(), index, total, value))
            })
            .collect()
    }
}

pub struct AudioModule;

impl Module for AudioModule {
    fn id(&self) -> &'static str {
        "audio"
    }

    fn label(&self) -> &'static str {
        "Audio"
    }

    fn description(&self) -> &'static str {
        "Audio devices and default output"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        let mut lines = Vec::new();

        for device in &info.audio.devices {
            lines.push(Line::new(self.id(), self.label(), device));
        }

        if let Some(sink) = &info.audio.default_sink {
            let label = if info.audio.devices.is_empty() {
                self.label()
            } else {
                "Audio Output"
            };
            lines.push(Line::new(self.id(), label, sink));
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::battery::BatteryInfo;
    use crate::system::gpu::{GpuInfo, GpuType};
    use crate::system::motherboard::MotherboardInfo;

    fn config() -> Config {
        Config::default()
    }

    #[test]
    fn test_gpu_detail_modules_omit_missing_values() {
        let mut info = SystemInfo::default();
        info.gpu.push(GpuInfo {
            name: String::from("Example GPU"),
            gpu_type: GpuType::Discrete,
            ..GpuInfo::default()
        });

        assert!(GpuUsageModule.render(&info, &config()).is_empty());
        assert!(GpuTemperatureModule.render(&info, &config()).is_empty());
        assert!(GpuVramModule.render(&info, &config()).is_empty());
    }

    #[test]
    fn test_gpu_detail_modules_emit_available_values() {
        let mut info = SystemInfo::default();
        info.gpu.push(GpuInfo {
            name: String::from("Example GPU"),
            gpu_type: GpuType::Discrete,
            utilization_percent: Some(42.0),
            temperature_c: Some(55.5),
            vram_total_bytes: Some(8 * 1024 * 1024 * 1024),
            vram_used_bytes: Some(2 * 1024 * 1024 * 1024),
            ..GpuInfo::default()
        });

        let usage = GpuUsageModule.render(&info, &config());
        assert_eq!(usage[0].value, "42%");
        let temp = GpuTemperatureModule.render(&info, &config());
        assert_eq!(temp[0].value, "55.5 °C");
        let vram = GpuVramModule.render(&info, &config());
        assert_eq!(vram[0].value, "2.00 GiB / 8.00 GiB");
    }

    #[test]
    fn test_multiple_gpu_suffixes() {
        let mut info = SystemInfo::default();
        for utilization in [10.0, 20.0] {
            info.gpu.push(GpuInfo {
                name: String::from("Example GPU"),
                gpu_type: GpuType::Discrete,
                utilization_percent: Some(utilization),
                ..GpuInfo::default()
            });
        }

        let lines = GpuUsageModule.render(&info, &config());
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].label_suffix.as_deref(), Some("1"));
        assert_eq!(lines[1].label_suffix.as_deref(), Some("2"));
    }

    #[test]
    fn test_bios_requires_version() {
        let mut info = SystemInfo {
            motherboard: MotherboardInfo {
                board_name: Some(String::from("B550")),
                ..MotherboardInfo::default()
            },
            ..SystemInfo::default()
        };
        assert!(BiosModule.render(&info, &config()).is_empty());

        info.motherboard.bios_version = Some(String::from("F14"));
        let lines = BiosModule.render(&info, &config());
        assert_eq!(lines[0].value, "F14 (B550)");
    }

    #[test]
    fn test_battery_requires_available_data() {
        let mut info = SystemInfo::default();
        info.battery.push(BatteryInfo::default());
        assert!(BatteryModule.render(&info, &config()).is_empty());

        info.battery[0].capacity_percent = Some(80);
        info.battery[0].status = Some(String::from("Discharging"));
        let lines = BatteryModule.render(&info, &config());
        assert_eq!(lines[0].value, "80% (Discharging)");
    }

    #[test]
    fn test_audio_uses_default_sink_when_no_devices() {
        let mut info = SystemInfo::default();
        info.audio.default_sink = Some(String::from("alsa_output.test"));
        let lines = AudioModule.render(&info, &config());
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].label, "Audio");
        assert_eq!(lines[0].value, "alsa_output.test");
    }

    #[test]
    fn test_disk_label_retains_mount_suffix() {
        let mut info = SystemInfo::default();
        info.storage.push(crate::system::storage::DiskInfo {
            device: String::from("/dev/nvme0n1p2"),
            mount_point: String::from("/"),
            total_bytes: 100,
            used_bytes: 50,
            ..Default::default()
        });
        let lines = DiskModule.render(&info, &config());
        assert_eq!(lines[0].label, "Disk");
        assert_eq!(lines[0].label_suffix.as_deref(), Some("/"));
    }
}
