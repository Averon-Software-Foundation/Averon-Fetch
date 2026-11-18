// SPDX-License-Identifier: GPL-3.0-or-later

use sysinfo::System;

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub total_swap_bytes: u64,
    pub used_swap_bytes: u64,
}

pub fn collect(sys: &System) -> MemoryInfo {
    MemoryInfo {
        total_bytes: sys.total_memory(),
        used_bytes: sys.used_memory(),
        available_bytes: sys.available_memory(),
        total_swap_bytes: sys.total_swap(),
        used_swap_bytes: sys.used_swap(),
    }
}

pub fn format_bytes_gib(bytes: u64) -> String {
    format!("{:.2} GiB", bytes as f64 / 1_073_741_824.0)
}

pub fn usage_percent(used: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (used as f64 / total as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes_gib() {
        assert_eq!(format_bytes_gib(1_073_741_824), "1.00 GiB");
        assert_eq!(format_bytes_gib(2_147_483_648), "2.00 GiB");
        assert_eq!(format_bytes_gib(0), "0.00 GiB");
    }

    #[test]
    fn test_usage_percent() {
        assert_eq!(usage_percent(50, 100), 50.0);
        assert_eq!(usage_percent(0, 100), 0.0);
        assert_eq!(usage_percent(100, 100), 100.0);
        assert_eq!(usage_percent(50, 0), 0.0);
    }
}
