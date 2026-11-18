// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct DisplayInfo {
    pub name: Option<String>,
    pub connector: Option<String>,
    pub resolution_width: Option<u32>,
    pub resolution_height: Option<u32>,
    pub refresh_rate: Option<f64>,
    pub is_primary: bool,
}

fn edid_vendor_name(edid: &[u8]) -> Option<String> {
    if edid.len() < 10 {
        return None;
    }
    let b1 = edid[8];
    let b2 = edid[9];
    let c1 = ((b1 >> 2) & 0x1F) + b'A' - 1;
    let c2 = (((b1 & 0x3) << 3) | ((b2 >> 5) & 0x7)) + b'A' - 1;
    let c3 = (b2 & 0x1F) + b'A' - 1;

    if c1.is_ascii_uppercase() && c2.is_ascii_uppercase() && c3.is_ascii_uppercase() {
        let code = format!("{}{}{}", c1 as char, c2 as char, c3 as char);
        let name = match code.as_str() {
            "ACR" => "Acer",
            "DEL" => "Dell",
            "AOC" => "AOC",
            "ASU" => "ASUS",
            "BNQ" => "BenQ",
            "ENC" => "Eizo",
            "HPN" | "HPD" | "HPC" => "HP",
            "LGD" => "LG",
            "SAM" => "Samsung",
            "SNY" => "Sony",
            "VSC" => "ViewSonic",
            "MSI" => "MSI",
            "GGL" => "Google",
            "APP" => "Apple",
            _ => return Some(code),
        };
        Some(name.to_string())
    } else {
        None
    }
}

fn edid_monitor_name(edid: &[u8]) -> Option<String> {
    if edid.len() < 128 {
        return None;
    }
    let vendor = edid_vendor_name(edid);
    let mut model: Option<String> = None;

    for block in 0..4usize {
        let off = 54 + block * 18;
        if off + 18 > edid.len() {
            break;
        }

        if edid[off] == 0x00
            && edid[off + 1] == 0x00
            && edid[off + 2] == 0x00
            && edid[off + 3] == 0xFC
        {
            let name_bytes = &edid[off + 5..off + 18];
            let name = String::from_utf8_lossy(name_bytes)
                .trim_end_matches(['\n', ' ', '\0', '\r'])
                .trim()
                .to_string();
            if !name.is_empty() {
                model = Some(name);
                break;
            }
        }
    }

    match (vendor, model) {
        (Some(v), Some(m)) => {
            if m.to_lowercase().starts_with(&v.to_lowercase()) {
                Some(m)
            } else {
                Some(format!("{} {}", v, m))
            }
        }
        (Some(v), None) => Some(format!("{} Monitor", v)),
        (None, Some(m)) => Some(m),
        (None, None) => None,
    }
}

fn parse_mode_resolution(modes_content: &str) -> (Option<u32>, Option<u32>) {
    let first = modes_content.lines().next().unwrap_or("").trim();
    if let Some((w, h)) = first.split_once('x') {
        let width = w.trim().parse::<u32>().ok();
        let height = h.trim().parse::<u32>().ok();
        return (width, height);
    }
    (None, None)
}

fn connector_from_entry(entry_name: &str) -> String {
    if let Some(pos) = entry_name.find('-') {
        entry_name[pos + 1..].to_string()
    } else {
        entry_name.to_string()
    }
}

struct XrandrOutputInfo {
    refresh_rate: Option<f64>,
    is_primary: bool,
    width: Option<u32>,
    height: Option<u32>,
}

fn xrandr_extra_info() -> HashMap<String, XrandrOutputInfo> {
    let mut map = HashMap::new();
    let Ok(out) = Command::new("xrandr").arg("--query").output() else {
        return map;
    };
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut current_connector: Option<String> = None;
    let mut current_is_primary = false;
    let mut current_width: Option<u32> = None;
    let mut current_height: Option<u32> = None;

    for line in stdout.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            if line.contains('*') {
                if let Some(ref conn) = current_connector {
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    for tok in tokens {
                        if tok.ends_with('*') || tok.contains('*') {
                            let num_str =
                                tok.trim_matches(|c: char| !c.is_ascii_digit() && c != '.');
                            if let Ok(rate) = num_str.parse::<f64>() {
                                map.insert(
                                    conn.clone(),
                                    XrandrOutputInfo {
                                        refresh_rate: Some(rate),
                                        is_primary: current_is_primary,
                                        width: current_width,
                                        height: current_height,
                                    },
                                );
                            }
                        }
                    }
                }
            }
        } else {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == "connected" {
                let conn = parts[0].to_string();
                let is_primary = parts.get(2).map(|&s| s == "primary").unwrap_or(false);
                let mut w: Option<u32> = None;
                let mut h: Option<u32> = None;
                for token in &parts {
                    if let Some((res, _pos)) = token.split_once('+') {
                        if let Some((w_s, h_s)) = res.split_once('x') {
                            w = w_s.parse::<u32>().ok();
                            h = h_s.parse::<u32>().ok();
                            break;
                        }
                    }
                }
                current_connector = Some(conn);
                current_is_primary = is_primary;
                current_width = w;
                current_height = h;
            } else {
                current_connector = None;
            }
        }
    }
    map
}

fn collect_from_drm(xrandr_info: &HashMap<String, XrandrOutputInfo>) -> Vec<DisplayInfo> {
    let drm_dir = Path::new("/sys/class/drm");
    if !drm_dir.exists() {
        return vec![];
    }

    let Ok(entries) = fs::read_dir(drm_dir) else {
        return vec![];
    };

    // Directory order is not defined, so sort connectors to keep the
    // rendered display order stable between runs.
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|entry| entry.file_name());

    let mut displays = Vec::new();

    for entry in entries {
        let entry_path = entry.path();
        let entry_name = entry.file_name().to_string_lossy().to_string();

        if !entry_name.contains('-') {
            continue;
        }

        let status_path = entry_path.join("status");
        let status = fs::read_to_string(&status_path)
            .unwrap_or_default()
            .trim()
            .to_string();
        if status != "connected" {
            continue;
        }

        let modes_content = fs::read_to_string(entry_path.join("modes")).unwrap_or_default();
        let (mut width, mut height) = parse_mode_resolution(&modes_content);

        let edid_bytes = fs::read(entry_path.join("edid")).unwrap_or_default();
        let monitor_name = edid_monitor_name(&edid_bytes);

        let connector = connector_from_entry(&entry_name);

        let mut refresh_rate = None;
        let mut is_primary = false;

        if let Some(xi) = xrandr_info.get(&connector) {
            refresh_rate = xi.refresh_rate;
            is_primary = xi.is_primary;
            if xi.width.is_some() && xi.height.is_some() {
                width = xi.width;
                height = xi.height;
            }
        }

        displays.push(DisplayInfo {
            name: monitor_name,
            connector: Some(connector),
            resolution_width: width,
            resolution_height: height,
            refresh_rate,
            is_primary,
        });
    }

    displays
}

fn displays_from_xrandr(xrandr_info: &HashMap<String, XrandrOutputInfo>) -> Vec<DisplayInfo> {
    let mut entries: Vec<_> = xrandr_info.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));

    entries
        .into_iter()
        .map(|(conn, xi)| DisplayInfo {
            name: None,
            connector: Some(conn.clone()),
            resolution_width: xi.width,
            resolution_height: xi.height,
            refresh_rate: xi.refresh_rate,
            is_primary: xi.is_primary,
        })
        .collect()
}

pub fn collect() -> Vec<DisplayInfo> {
    let xrandr_info = xrandr_extra_info();
    let mut displays = collect_from_drm(&xrandr_info);

    if displays.is_empty() {
        displays = displays_from_xrandr(&xrandr_info);
    }

    if !displays.is_empty() && !displays.iter().any(|d| d.is_primary) {
        displays[0].is_primary = true;
    }

    displays
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xrandr_fallback_is_sorted_by_connector() {
        let mut info = HashMap::new();
        info.insert(
            String::from("HDMI-1"),
            XrandrOutputInfo {
                refresh_rate: Some(60.0),
                is_primary: false,
                width: Some(1920),
                height: Some(1080),
            },
        );
        info.insert(
            String::from("DP-1"),
            XrandrOutputInfo {
                refresh_rate: Some(144.0),
                is_primary: true,
                width: Some(2560),
                height: Some(1440),
            },
        );

        let displays = displays_from_xrandr(&info);
        assert_eq!(displays.len(), 2);
        assert_eq!(displays[0].connector.as_deref(), Some("DP-1"));
        assert_eq!(displays[1].connector.as_deref(), Some("HDMI-1"));
    }
}
