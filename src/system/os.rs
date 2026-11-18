// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::fs;
use sysinfo::System;

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct OsInfo {
    pub name: Option<String>,
    pub version: Option<String>,
    pub id: Option<String>,
    pub arch: Option<String>,
    pub kernel: Option<String>,
    pub uptime_secs: u64,
}

fn parse_os_release() -> HashMap<String, String> {
    let content = fs::read_to_string("/etc/os-release")
        .or_else(|_| fs::read_to_string("/usr/lib/os-release"))
        .unwrap_or_default();

    let mut map = HashMap::new();
    for line in content.lines() {
        if let Some((key, val)) = line.split_once('=') {
            let val = val.trim().trim_matches('"').to_string();
            map.insert(key.trim().to_string(), val);
        }
    }
    map
}

pub fn collect() -> OsInfo {
    let os_release = parse_os_release();

    let name = os_release
        .get("PRETTY_NAME")
        .or_else(|| os_release.get("NAME"))
        .cloned();

    let version = os_release
        .get("VERSION_ID")
        .or_else(|| os_release.get("VERSION"))
        .cloned();

    let id = os_release.get("ID").cloned();

    let arch = Some(std::env::consts::ARCH.to_string());

    let kernel = System::kernel_version().ok();

    let uptime_secs = System::uptime().unwrap_or(0);

    OsInfo {
        name,
        version,
        id,
        arch,
        kernel,
        uptime_secs,
    }
}

pub fn format_uptime(secs: u64) -> String {
    let days = secs / 86_400;
    let hours = (secs % 86_400) / 3_600;
    let minutes = (secs % 3_600) / 60;

    let mut parts: Vec<String> = Vec::new();
    if days > 0 {
        parts.push(format!(
            "{} {}",
            days,
            if days == 1 { "day" } else { "days" }
        ));
    }
    if hours > 0 {
        parts.push(format!(
            "{} {}",
            hours,
            if hours == 1 { "hour" } else { "hours" }
        ));
    }
    if minutes > 0 || parts.is_empty() {
        parts.push(format!(
            "{} {}",
            minutes,
            if minutes == 1 { "minute" } else { "minutes" }
        ));
    }

    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_uptime_zero() {
        assert_eq!(format_uptime(0), "0 minutes");
    }

    #[test]
    fn test_format_uptime_minutes() {
        assert_eq!(format_uptime(120), "2 minutes");
        assert_eq!(format_uptime(60), "1 minute");
    }

    #[test]
    fn test_format_uptime_hours_and_minutes() {
        assert_eq!(format_uptime(3660), "1 hour, 1 minute");
        assert_eq!(format_uptime(7200), "2 hours");
    }

    #[test]
    fn test_format_uptime_days() {
        assert_eq!(format_uptime(86400 + 3600), "1 day, 1 hour");
        assert_eq!(format_uptime(172800), "2 days");
    }
}
