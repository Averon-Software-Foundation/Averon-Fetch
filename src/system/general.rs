use std::fs;
use sysinfo::System;

#[derive(Debug, Clone, Default)]
pub struct GeneralInfo {
    pub username: Option<String>,
    pub hostname: Option<String>,
    pub host: Option<String>,
}

fn is_generic_dmi(s: &str) -> bool {
    let lower = s.to_lowercase();
    lower.is_empty()
        || lower.contains("to be filled")
        || lower.contains("system product name")
        || lower.contains("default string")
        || lower.contains("undefined")
        || lower == "none"
        || lower == "system manufacturer"
        || lower == "o.e.m."
}

fn detect_host() -> Option<String> {
    let product_name = fs::read_to_string("/sys/devices/virtual/dmi/id/product_name")
        .or_else(|_| fs::read_to_string("/sys/class/dmi/id/product_name"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !is_generic_dmi(s));

    let product_version = fs::read_to_string("/sys/devices/virtual/dmi/id/product_version")
        .or_else(|_| fs::read_to_string("/sys/class/dmi/id/product_version"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !is_generic_dmi(s));

    if let Some(name) = product_name {
        if let Some(ver) = product_version {
            if !name.contains(&ver) {
                return Some(format!("{name} {ver}"));
            }
        }
        return Some(name);
    }

    let board_name = fs::read_to_string("/sys/devices/virtual/dmi/id/board_name")
        .or_else(|_| fs::read_to_string("/sys/class/dmi/id/board_name"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !is_generic_dmi(s));

    if let Some(board) = board_name {
        return Some(board);
    }

    let dt_model = fs::read_to_string("/sys/firmware/devicetree/base/model")
        .ok()
        .map(|s| s.trim_matches('\0').trim().to_string())
        .filter(|s| !s.is_empty());

    if let Some(model) = dt_model {
        return Some(model);
    }

    None
}

pub fn collect(_sys: &System) -> GeneralInfo {
    GeneralInfo {
        username: std::env::var("USER")
            .ok()
            .or_else(|| std::env::var("LOGNAME").ok()),
        hostname: System::host_name(),
        host: detect_host(),
    }
}
