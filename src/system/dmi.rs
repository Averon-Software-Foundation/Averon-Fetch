// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;

/// Firmware often fills DMI fields with placeholder text instead of a real
/// value. Treat these strings as missing so they never reach the output.
pub fn is_placeholder(value: &str) -> bool {
    let lower = value.trim().to_lowercase();

    lower.is_empty()
        || matches!(
            lower.as_str(),
            "none"
                | "unknown"
                | "not specified"
                | "not available"
                | "not applicable"
                | "n/a"
                | "na"
                | "oem"
                | "o.e.m."
                | "oem default"
                | "default string"
                | "undefined"
                | "system manufacturer"
                | "system product name"
                | "system version"
                | "base board product name"
                | "type1productconfigid"
        )
        || lower.contains("to be filled")
        || lower.contains("filled by o.e.m.")
        || lower.contains("default string")
        || lower.contains("not specified")
        || lower.contains("not available")
        || lower.contains("not applicable")
}

/// Read a DMI field, preferring the canonical sysfs path but falling back to
/// the class alias used by some kernels. Placeholder values are rejected.
pub fn read(field: &str) -> Option<String> {
    fs::read_to_string(format!("/sys/devices/virtual/dmi/id/{field}"))
        .or_else(|_| fs::read_to_string(format!("/sys/class/dmi/id/{field}")))
        .ok()
        .map(|value| value.trim_matches('\0').trim().to_string())
        .filter(|value| !is_placeholder(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder_values_are_rejected() {
        for value in [
            "",
            "None",
            "To Be Filled By O.E.M.",
            "To be filled by O.E.M.",
            "Default string",
            "System Product Name",
            "System Manufacturer",
            "System Version",
            "Base Board Product Name",
            "Not Specified",
            "Not Available",
            "Not Applicable",
            "Type1ProductConfigId",
            "Undefined",
            "N/A",
        ] {
            assert!(is_placeholder(value), "expected placeholder: {value:?}");
        }
    }

    #[test]
    fn test_real_values_are_kept() {
        for value in [
            "ASUSTeK COMPUTER INC.",
            "B550 EAGLE",
            "F14",
            "1.0",
            "X570 AORUS ELITE",
            "American Megatrends International, LLC.",
        ] {
            assert!(!is_placeholder(value), "unexpected placeholder: {value:?}");
        }
    }
}
