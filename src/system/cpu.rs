// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;
use std::time::{Duration, Instant};
use sysinfo::System;

/// Minimum window between two `/proc/stat` samples. Shorter intervals make the
/// rounding noise in the counters dominate the result.
const USAGE_SAMPLE_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CpuInfo {
    pub name: Option<String>,
    pub arch: Option<String>,
    pub physical_cores: Option<usize>,
    pub logical_cores: usize,
    pub frequency_mhz: Option<u64>,
    pub usage_percent: Option<f32>,
    pub temperature_c: Option<f32>,
}

impl Default for CpuInfo {
    fn default() -> Self {
        Self {
            name: None,
            arch: Some(std::env::consts::ARCH.to_string()),
            physical_cores: None,
            logical_cores: 0,
            frequency_mhz: None,
            usage_percent: None,
            temperature_c: None,
        }
    }
}

fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn cpu_name_from_proc() -> Option<String> {
    let content = fs::read_to_string("/proc/cpuinfo").ok()?;
    for line in content.lines() {
        if let Some(val) = line.strip_prefix("model name") {
            let val = val.trim_start_matches(':').trim();
            let name = normalize_whitespace(val);
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

#[derive(Debug, Clone, Copy)]
struct CpuTimes {
    idle: u64,
    total: u64,
}

fn read_cpu_times() -> Option<CpuTimes> {
    let content = fs::read_to_string("/proc/stat").ok()?;
    let line = content.lines().find(|line| line.starts_with("cpu "))?;
    let values: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|value| value.parse::<u64>().ok())
        .collect();

    if values.len() < 4 {
        return None;
    }

    let idle = values[3] + values.get(4).copied().unwrap_or(0);
    let total: u64 = values.iter().sum();
    Some(CpuTimes { idle, total })
}

/// A CPU usage measurement in two halves. Taking the baseline sample before
/// the other collectors run means their work counts towards the sampling
/// window, so the wait is usually free instead of a fixed 100 ms delay.
pub struct UsageProbe {
    first: CpuTimes,
    started: Instant,
}

impl UsageProbe {
    pub fn start() -> Option<Self> {
        Some(Self {
            first: read_cpu_times()?,
            started: Instant::now(),
        })
    }

    pub fn finish(self) -> Option<f32> {
        let elapsed = self.started.elapsed();
        if elapsed < USAGE_SAMPLE_INTERVAL {
            std::thread::sleep(USAGE_SAMPLE_INTERVAL - elapsed);
        }

        let second = read_cpu_times()?;
        let total_delta = second.total.saturating_sub(self.first.total);
        let idle_delta = second.idle.saturating_sub(self.first.idle);
        if total_delta == 0 {
            return None;
        }

        let busy = total_delta.saturating_sub(idle_delta) as f64;
        Some(((busy / total_delta as f64) * 100.0) as f32)
    }
}

fn read_temp_millidegrees(path: &str) -> Option<f32> {
    let raw = fs::read_to_string(path).ok()?;
    let value = raw.trim().parse::<f32>().ok()?;
    if value <= 0.0 {
        return None;
    }

    Some(value / 1000.0)
}

fn max_temp_from_hwmon_dir(dir: &str) -> Option<f32> {
    let entries = fs::read_dir(dir).ok()?;
    let mut max_temp: Option<f32> = None;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("temp") || !name.ends_with("_input") {
            continue;
        }
        let path = entry.path();
        if let Some(temp) = read_temp_millidegrees(&path.to_string_lossy()) {
            if temp > 0.0 && temp < 150.0 {
                max_temp = Some(max_temp.map_or(temp, |current| current.max(temp)));
            }
        }
    }

    max_temp
}

fn is_cpu_hwmon(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("coretemp")
        || lower.contains("k10temp")
        || lower.contains("zenpower")
        || lower.contains("cpu_thermal")
        || lower.contains("cpu-thermal")
        || lower.contains("soc_thermal")
        || lower.contains("soc-thermal")
}

fn cpu_temperature() -> Option<f32> {
    let mut fallback: Option<f32> = None;

    if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
        for entry in entries.flatten() {
            let dir = entry.path();
            let name = fs::read_to_string(dir.join("name"))
                .unwrap_or_default()
                .trim()
                .to_string();
            let temp = max_temp_from_hwmon_dir(&dir.to_string_lossy());
            if is_cpu_hwmon(&name) {
                if let Some(temp) = temp {
                    fallback = Some(fallback.map_or(temp, |current| current.max(temp)));
                }
            }
        }
    }

    if fallback.is_some() {
        return fallback;
    }

    if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
        for entry in entries.flatten() {
            let dir = entry.path();
            let zone_type = fs::read_to_string(dir.join("type"))
                .unwrap_or_default()
                .trim()
                .to_lowercase();
            if !zone_type.contains("cpu")
                && !zone_type.contains("soc")
                && !zone_type.contains("x86_pkg_temp")
            {
                continue;
            }
            if let Some(temp) = read_temp_millidegrees(&dir.join("temp").to_string_lossy()) {
                if temp > 0.0 && temp < 150.0 {
                    fallback = Some(fallback.map_or(temp, |current| current.max(temp)));
                }
            }
        }
    }

    fallback
}

pub fn collect(sys: &System, usage: Option<UsageProbe>, want_temperature: bool) -> CpuInfo {
    let cpus = sys.cpus();

    let name = {
        let brand = cpus.first().map(|c| c.brand().to_string());
        let brand = brand.filter(|b| !b.is_empty());
        let name = brand.or_else(cpu_name_from_proc);
        name.map(|n| normalize_whitespace(&n))
    };

    let logical_cores = cpus.len();
    let physical_cores = sys.physical_core_count();

    let frequency_mhz = if cpus.is_empty() {
        None
    } else {
        let total: u64 = cpus.iter().map(|c| c.frequency()).sum();
        let average = total / cpus.len() as u64;
        if average > 0 {
            Some(average)
        } else {
            None
        }
    };

    CpuInfo {
        name,
        arch: Some(std::env::consts::ARCH.to_string()),
        physical_cores,
        logical_cores,
        frequency_mhz,
        usage_percent: usage.and_then(UsageProbe::finish),
        temperature_c: if want_temperature {
            cpu_temperature()
        } else {
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(normalize_whitespace("  AMD   Ryzen  "), "AMD Ryzen");
    }

    #[test]
    fn test_temp_reading_ignores_zero_and_invalid() {
        // Unique per process so parallel test binaries cannot share a file.
        let dir =
            std::env::temp_dir().join(format!("averon-fetch-cpu-temp-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("temp1_input");

        fs::write(&path, "42000\n").unwrap();
        assert_eq!(read_temp_millidegrees(&path.to_string_lossy()), Some(42.0));
        fs::write(&path, "0\n").unwrap();
        assert_eq!(read_temp_millidegrees(&path.to_string_lossy()), None);
        fs::write(&path, "not-a-number\n").unwrap();
        assert_eq!(read_temp_millidegrees(&path.to_string_lossy()), None);

        fs::remove_dir_all(&dir).unwrap();
    }
}
