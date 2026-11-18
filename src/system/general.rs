// SPDX-License-Identifier: GPL-3.0-or-later

use super::dmi;
use std::fs;
use sysinfo::System;

#[derive(Debug, Clone, Default)]
pub struct GeneralInfo {
    pub username: Option<String>,
    pub hostname: Option<String>,
    pub host: Option<String>,
}

fn detect_host() -> Option<String> {
    let product_name = dmi::read("product_name");
    let product_version = dmi::read("product_version");

    if let Some(name) = product_name {
        if let Some(ver) = product_version {
            if !name.contains(&ver) {
                return Some(format!("{name} {ver}"));
            }
        }
        return Some(name);
    }

    let board_name = dmi::read("board_name");

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

pub fn collect() -> GeneralInfo {
    GeneralInfo {
        username: std::env::var("USER")
            .ok()
            .or_else(|| std::env::var("LOGNAME").ok()),
        hostname: System::host_name().ok(),
        host: detect_host(),
    }
}
