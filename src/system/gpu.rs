// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GpuType {
    Integrated,
    Discrete,
    #[default]
    Unknown,
}

#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub gpu_type: GpuType,
    pub driver: Option<String>,
    pub pci_slot: Option<String>,
    pub utilization_percent: Option<f32>,
    pub temperature_c: Option<f32>,
    pub vram_total_bytes: Option<u64>,
    pub vram_used_bytes: Option<u64>,
}

impl Default for GpuInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            gpu_type: GpuType::Unknown,
            driver: None,
            pci_slot: None,
            utilization_percent: None,
            temperature_c: None,
            vram_total_bytes: None,
            vram_used_bytes: None,
        }
    }
}

const VENDOR_INTEL: u32 = 0x8086;
const VENDOR_NVIDIA: u32 = 0x10de;
const VENDOR_AMD: u32 = 0x1002;

fn parse_vendor_id(s: &str) -> Option<u32> {
    let s = s.trim().trim_start_matches("0x").trim_start_matches("0X");
    u32::from_str_radix(s, 16).ok()
}

fn gpu_type_from_vendor(vendor_id: u32) -> GpuType {
    match vendor_id {
        VENDOR_INTEL => GpuType::Integrated,
        VENDOR_NVIDIA | VENDOR_AMD => GpuType::Discrete,
        _ => GpuType::Unknown,
    }
}

fn vendor_name(vendor_id: u32) -> &'static str {
    match vendor_id {
        VENDOR_INTEL => "Intel",
        VENDOR_NVIDIA => "NVIDIA",
        VENDOR_AMD => "AMD",
        _ => "Unknown",
    }
}

fn sysfs_read(path: &str) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn sysfs_u64(path: &str) -> Option<u64> {
    sysfs_read(path)?.parse::<u64>().ok()
}

fn driver_from_uevent(uevent_path: &str) -> Option<String> {
    uevent_value(uevent_path, "DRIVER")
}

fn uevent_value(uevent_path: &str, key: &str) -> Option<String> {
    let content = fs::read_to_string(uevent_path).ok()?;
    let prefix = format!("{key}=");
    content
        .lines()
        .find_map(|line| line.strip_prefix(&prefix).map(|v| v.trim().to_string()))
}

fn nvidia_driver_version() -> Option<String> {
    let content = fs::read_to_string("/proc/driver/nvidia/version").ok()?;
    for line in content.lines() {
        if line.starts_with("NVRM") {
            for token in line.split_whitespace() {
                if token.chars().next().is_some_and(|c| c.is_ascii_digit()) && token.contains('.') {
                    return Some(token.to_string());
                }
            }
        }
    }
    None
}

fn gpu_name_from_sysfs(card_path: &str, vendor_id: u32) -> String {
    if let Some(product) = sysfs_read(&format!("{card_path}/device/product")) {
        return product;
    }

    let sub_dev = sysfs_read(&format!("{card_path}/device/subsystem_device")).unwrap_or_default();
    let vname = vendor_name(vendor_id);
    if !sub_dev.is_empty() && sub_dev != "0x0000" {
        format!("{vname} GPU ({sub_dev})")
    } else {
        format!("{vname} GPU")
    }
}

fn collect_from_sysfs(cards: &[DrmCard]) -> Option<Vec<GpuInfo>> {
    let mut gpus: Vec<GpuInfo> = Vec::new();

    for card in cards {
        let Some(vendor_id) = card.vendor_id else {
            continue;
        };

        let drm_driver = driver_from_uevent(&format!("{}/device/uevent", card.path));
        let driver = match vendor_id {
            VENDOR_NVIDIA => nvidia_driver_version()
                .map(|version| format!("nvidia {version}"))
                .or(drm_driver),
            _ => drm_driver,
        };

        gpus.push(GpuInfo {
            name: gpu_name_from_sysfs(&card.path, vendor_id),
            gpu_type: gpu_type_from_vendor(vendor_id),
            driver,
            pci_slot: card.slot.clone(),
            ..GpuInfo::default()
        });
    }

    if gpus.is_empty() {
        None
    } else {
        Some(gpus)
    }
}

fn collect_from_lspci() -> Vec<GpuInfo> {
    let output = Command::new("lspci")
        .arg("-mm")
        .output()
        .ok()
        .filter(|o| o.status.success());

    let output = match output {
        Some(o) => o,
        None => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut gpus: Vec<GpuInfo> = Vec::new();

    for line in stdout.lines() {
        let lower = line.to_lowercase();
        if !lower.contains("vga")
            && !lower.contains("3d controller")
            && !lower.contains("display controller")
        {
            continue;
        }

        let fields = parse_lspci_mm_fields(line);
        if fields.len() < 4 {
            continue;
        }

        let vendor_str = fields[2].trim().to_string();
        let device_str = fields[3].trim().to_string();

        let vendor_clean = if vendor_str.starts_with("NVIDIA") {
            "NVIDIA"
        } else if vendor_str.starts_with("Advanced Micro Devices") || vendor_str.starts_with("AMD")
        {
            "AMD"
        } else if vendor_str.starts_with("Intel") {
            "Intel"
        } else {
            &vendor_str
        };

        let device_clean = if let Some(start) = device_str.find('[') {
            if let Some(end) = device_str[start..].find(']') {
                &device_str[start + 1..start + end]
            } else {
                &device_str
            }
        } else {
            &device_str
        };

        let gpu_name = if device_clean.is_empty() {
            vendor_clean.to_string()
        } else if device_clean.starts_with(vendor_clean) {
            device_clean.to_string()
        } else {
            format!("{vendor_clean} {device_clean}")
        };

        let gpu_type = {
            let vendor = vendor_str.to_lowercase();
            if vendor.contains("intel") {
                GpuType::Integrated
            } else if vendor.contains("nvidia")
                || vendor.contains("amd")
                || vendor.contains("advanced micro")
            {
                GpuType::Discrete
            } else {
                GpuType::Unknown
            }
        };

        let driver = if vendor_str.to_lowercase().contains("nvidia") {
            nvidia_driver_version().map(|version| format!("nvidia {version}"))
        } else {
            None
        };

        gpus.push(GpuInfo {
            name: gpu_name,
            gpu_type,
            driver,
            pci_slot: fields.first().cloned(),
            ..GpuInfo::default()
        });
    }

    gpus
}

fn parse_lspci_mm_fields(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    fields.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        fields.push(current);
    }

    fields
}

struct DrmCard {
    slot: Option<String>,
    path: String,
    vendor_id: Option<u32>,
}

fn drm_cards() -> Vec<DrmCard> {
    let mut cards = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/drm") else {
        return cards;
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("card") {
            continue;
        }
        let after = &name["card".len()..];
        if !after.chars().next().is_some_and(|c| c.is_ascii_digit()) || after.contains('-') {
            continue;
        }

        let path = format!("/sys/class/drm/{name}");
        let uevent = format!("{path}/device/uevent");
        let slot = uevent_value(&uevent, "PCI_SLOT_NAME").or_else(|| {
            fs::read_link(format!("{path}/device"))
                .ok()
                .and_then(|target| {
                    target
                        .file_name()
                        .map(|name| name.to_string_lossy().to_string())
                })
        });
        let vendor_id =
            sysfs_read(&format!("{path}/device/vendor")).and_then(|value| parse_vendor_id(&value));

        cards.push(DrmCard {
            slot,
            path,
            vendor_id,
        });
    }

    cards.sort_by(|a, b| a.path.cmp(&b.path));
    cards
}

fn card_for_gpu<'a>(gpu: &GpuInfo, cards: &'a [DrmCard], index: usize) -> Option<&'a DrmCard> {
    if let Some(slot) = &gpu.pci_slot {
        if let Some(card) = cards.iter().find(|card| {
            card.slot
                .as_deref()
                .is_some_and(|card_slot| card_slot.ends_with(slot) || slot.ends_with(card_slot))
        }) {
            return Some(card);
        }
    }
    cards.get(index)
}

fn hwmon_temperature(card_path: &str) -> Option<f32> {
    let hwmon_root = PathBuf::from(format!("{card_path}/device/hwmon"));
    let entries = fs::read_dir(hwmon_root).ok()?;
    let mut max_temp: Option<f32> = None;

    for entry in entries.flatten() {
        let dir = entry.path();
        let Ok(files) = fs::read_dir(&dir) else {
            continue;
        };
        for file in files.flatten() {
            let name = file.file_name().to_string_lossy().to_string();
            if !name.starts_with("temp") || !name.ends_with("_input") {
                continue;
            }
            let Ok(raw) = fs::read_to_string(file.path()) else {
                continue;
            };
            let Ok(value) = raw.trim().parse::<f32>() else {
                continue;
            };
            let celsius = value / 1000.0;
            if (0.0..150.0).contains(&celsius) {
                max_temp = Some(max_temp.map_or(celsius, |current| current.max(celsius)));
            }
        }
    }

    max_temp
}

fn enrich_from_sysfs(gpus: &mut [GpuInfo], cards: &[DrmCard]) {
    if cards.is_empty() {
        return;
    }

    for (index, gpu) in gpus.iter_mut().enumerate() {
        let Some(card) = card_for_gpu(gpu, cards, index) else {
            continue;
        };

        if gpu.utilization_percent.is_none() {
            gpu.utilization_percent = sysfs_u64(&format!("{}/device/gpu_busy_percent", card.path))
                .map(|value| value.min(100) as f32);
        }
        if gpu.temperature_c.is_none() {
            gpu.temperature_c = hwmon_temperature(&card.path);
        }
        if gpu.vram_total_bytes.is_none() {
            gpu.vram_total_bytes = sysfs_u64(&format!("{}/device/mem_info_vram_total", card.path));
        }
        if gpu.vram_used_bytes.is_none() {
            gpu.vram_used_bytes = sysfs_u64(&format!("{}/device/mem_info_vram_used", card.path));
        }
    }
}

fn nvidia_smi_details() -> Vec<(String, f32, f32, u64, u64)> {
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,utilization.gpu,temperature.gpu,memory.total,memory.used",
            "--format=csv,noheader,nounits",
        ])
        .output();

    let output = match output {
        Ok(output) if output.status.success() => output,
        Ok(_) => {
            crate::diag::warn("nvidia-smi exited unsuccessfully; GPU details unavailable");
            return Vec::new();
        }
        Err(error) => {
            crate::diag::warn(format!(
                "nvidia-smi unavailable; GPU details may be incomplete: {error}"
            ));
            return Vec::new();
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split(',').map(str::trim).collect();
            if fields.len() < 5 {
                return None;
            }
            let name = fields[0].to_string();
            let utilization = fields[1].parse::<f32>().ok()?;
            let temperature = fields[2].parse::<f32>().ok()?;
            let total_mib = fields[3].parse::<u64>().ok()?;
            let used_mib = fields[4].parse::<u64>().ok()?;
            Some((
                name,
                utilization,
                temperature,
                total_mib * 1024 * 1024,
                used_mib * 1024 * 1024,
            ))
        })
        .collect()
}

fn gpu_is_nvidia(gpu: &GpuInfo) -> bool {
    gpu.name.to_lowercase().contains("nvidia")
        || gpu
            .driver
            .as_deref()
            .is_some_and(|driver| driver.to_lowercase().starts_with("nvidia"))
}

fn nvidia_indices(gpus: &[GpuInfo]) -> Vec<usize> {
    gpus.iter()
        .enumerate()
        .filter(|(_, gpu)| gpu_is_nvidia(gpu))
        .map(|(index, _)| index)
        .collect()
}

fn needs_nvidia_details(gpus: &[GpuInfo], indices: &[usize]) -> bool {
    indices.iter().any(|&index| {
        let gpu = &gpus[index];
        gpu.utilization_percent.is_none()
            || gpu.temperature_c.is_none()
            || gpu.vram_total_bytes.is_none()
            || gpu.vram_used_bytes.is_none()
    })
}

fn apply_nvidia_details(
    gpus: &mut [GpuInfo],
    nvidia_indices: &[usize],
    details: &[(String, f32, f32, u64, u64)],
) {
    if details.is_empty() {
        return;
    }

    // Match by name where possible. The positional fallback only indexes the
    // NVIDIA GPUs, so a single NVIDIA entry can never be applied to an
    // integrated GPU that happens to appear earlier in the list.
    for (nvidia_index, &index) in nvidia_indices.iter().enumerate() {
        let gpu = &mut gpus[index];
        let matching = details
            .iter()
            .find(|(name, ..)| name.eq_ignore_ascii_case(&gpu.name))
            .or_else(|| details.get(nvidia_index));

        let Some((_, utilization, temperature, total, used)) = matching else {
            continue;
        };

        if gpu.utilization_percent.is_none() {
            gpu.utilization_percent = Some(*utilization);
        }
        if gpu.temperature_c.is_none() {
            gpu.temperature_c = Some(*temperature);
        }
        if gpu.vram_total_bytes.is_none() {
            gpu.vram_total_bytes = Some(*total);
        }
        if gpu.vram_used_bytes.is_none() {
            gpu.vram_used_bytes = Some(*used);
        }
    }
}

fn enrich_nvidia(gpus: &mut [GpuInfo]) {
    let indices = nvidia_indices(gpus);
    if !needs_nvidia_details(gpus, &indices) {
        return;
    }

    let details = nvidia_smi_details();
    apply_nvidia_details(gpus, &indices, &details);
}

/// `nvidia-smi` costs tens of milliseconds to start, so only run it when the
/// NVIDIA driver is actually present.
fn nvidia_driver_loaded() -> bool {
    Path::new("/proc/driver/nvidia/version").exists() || Path::new("/sys/module/nvidia").exists()
}

pub fn collect(details: bool) -> Vec<GpuInfo> {
    // `lspci` and `nvidia-smi` each pay a process start-up cost; start the
    // slower one first so the two overlap instead of running back to back.
    let nvidia_probe = if details && nvidia_driver_loaded() {
        Some(std::thread::spawn(nvidia_smi_details))
    } else {
        None
    };

    let mut gpus = collect_from_lspci();
    // Both the sysfs fallback and the detail enrichment need the DRM card
    // list, so build it once and share it.
    let cards = if details || gpus.is_empty() {
        drm_cards()
    } else {
        Vec::new()
    };
    if gpus.is_empty() {
        gpus = collect_from_sysfs(&cards).unwrap_or_default();
    }

    if details && !gpus.is_empty() {
        enrich_from_sysfs(&mut gpus, &cards);

        match nvidia_probe {
            Some(handle) => {
                let indices = nvidia_indices(&gpus);
                if needs_nvidia_details(&gpus, &indices) {
                    let details = handle.join().unwrap_or_default();
                    apply_nvidia_details(&mut gpus, &indices, &details);
                }
            }
            None => enrich_nvidia(&mut gpus),
        }
    }

    gpus
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lspci_fields() {
        let line = r#"01:00.0 "VGA compatible controller" "NVIDIA Corporation" "GB206 [GeForce RTX 5060 Ti]" "ASUSTeK Computer Inc." "Device 1234""#;
        let fields = parse_lspci_mm_fields(line);
        assert_eq!(fields[0], "01:00.0");
        assert_eq!(fields[2], "NVIDIA Corporation");
        assert_eq!(fields[3], "GB206 [GeForce RTX 5060 Ti]");
    }

    #[test]
    fn test_vendor_types() {
        assert_eq!(gpu_type_from_vendor(VENDOR_INTEL), GpuType::Integrated);
        assert_eq!(gpu_type_from_vendor(VENDOR_NVIDIA), GpuType::Discrete);
        assert_eq!(gpu_type_from_vendor(0xFFFF), GpuType::Unknown);
    }

    #[test]
    fn test_parse_vendor_id() {
        assert_eq!(parse_vendor_id("0x10de"), Some(VENDOR_NVIDIA));
        assert_eq!(parse_vendor_id("1002"), Some(VENDOR_AMD));
        assert_eq!(parse_vendor_id("nope"), None);
    }

    #[test]
    fn test_default_gpu_has_no_fake_details() {
        let gpu = GpuInfo::default();
        assert!(gpu.utilization_percent.is_none());
        assert!(gpu.temperature_c.is_none());
        assert!(gpu.vram_total_bytes.is_none());
        assert!(gpu.vram_used_bytes.is_none());
    }

    #[test]
    fn test_nvidia_detection_uses_name_or_driver() {
        let by_name = GpuInfo {
            name: String::from("NVIDIA GeForce RTX 4090"),
            ..GpuInfo::default()
        };
        let by_driver = GpuInfo {
            name: String::from("Discrete GPU"),
            driver: Some(String::from("nvidia 550.54.14")),
            ..GpuInfo::default()
        };
        let intel = GpuInfo {
            name: String::from("Intel Graphics"),
            driver: Some(String::from("i915")),
            ..GpuInfo::default()
        };

        assert!(gpu_is_nvidia(&by_name));
        assert!(gpu_is_nvidia(&by_driver));
        assert!(!gpu_is_nvidia(&intel));
    }
}
