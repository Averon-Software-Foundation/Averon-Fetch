use std::collections::HashMap;
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct NetworkInterface {
    pub name: String,
    pub ip_v4: Vec<String>,
    pub ip_v6: Vec<String>,
    pub is_up: bool,
    pub speed_mbps: Option<u64>,
}

fn collect_addresses() -> HashMap<String, (Vec<String>, Vec<String>)> {
    let mut map: HashMap<String, (Vec<String>, Vec<String>)> = HashMap::new();

    let Ok(output) = Command::new("ip").args(["-o", "addr", "show"]).output() else {
        return map;
    };

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 4 {
            continue;
        }

        let iface = cols[1].trim_end_matches(':').to_string();
        let entry = map.entry(iface).or_default();

        if let Some(pos) = cols.iter().position(|&col| col == "inet") {
            if let Some(addr) = cols.get(pos + 1) {
                let ip = addr.split('/').next().unwrap_or(addr).to_string();
                if !entry.0.contains(&ip) {
                    entry.0.push(ip);
                }
            }
        } else if let Some(pos) = cols.iter().position(|&col| col == "inet6") {
            if let Some(addr) = cols.get(pos + 1) {
                let ip = addr.split('/').next().unwrap_or(addr).to_string();
                if !entry.1.contains(&ip) {
                    entry.1.push(ip);
                }
            }
        }
    }

    map
}

fn interface_names() -> Vec<String> {
    let mut names = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/net") else {
        return names;
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name != "lo" {
            names.push(name);
        }
    }
    names.sort();
    names
}

fn is_up(name: &str) -> bool {
    fs::read_to_string(format!("/sys/class/net/{name}/operstate"))
        .map(|state| state.trim() == "up")
        .unwrap_or(false)
}

fn link_speed(name: &str) -> Option<u64> {
    let raw = fs::read_to_string(format!("/sys/class/net/{name}/speed")).ok()?;
    let value = raw.trim().parse::<i64>().ok()?;
    if value > 0 {
        Some(value as u64)
    } else {
        None
    }
}

pub fn collect() -> Vec<NetworkInterface> {
    let addresses = collect_addresses();
    let names = interface_names();

    let mut interfaces: Vec<NetworkInterface> = names
        .into_iter()
        .map(|name| {
            let (ip_v4, ip_v6) = addresses.get(&name).cloned().unwrap_or_default();
            NetworkInterface {
                is_up: is_up(&name),
                speed_mbps: link_speed(&name),
                name,
                ip_v4,
                ip_v6,
            }
        })
        .collect();

    interfaces.sort_by(|a, b| a.name.cmp(&b.name));
    interfaces
}

pub fn format_link_speed(mbps: u64) -> String {
    if mbps >= 1000 {
        let whole_gb = mbps / 1000;
        if mbps == whole_gb * 1000 {
            format!("{whole_gb} Gb/s")
        } else {
            format!("{:.1} Gb/s", mbps as f64 / 1000.0)
        }
    } else {
        format!("{mbps} Mb/s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_link_speed() {
        assert_eq!(format_link_speed(100), "100 Mb/s");
        assert_eq!(format_link_speed(1000), "1 Gb/s");
        assert_eq!(format_link_speed(2500), "2.5 Gb/s");
        assert_eq!(format_link_speed(10000), "10 Gb/s");
    }

    #[test]
    fn test_default_interface_is_empty() {
        let iface = NetworkInterface::default();
        assert!(iface.ip_v4.is_empty());
        assert!(iface.ip_v6.is_empty());
        assert!(iface.speed_mbps.is_none());
    }
}
