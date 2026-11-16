use std::fs;

#[derive(Debug, Clone, Default)]
pub struct BatteryInfo {
    pub name: String,
    pub capacity_percent: Option<u8>,
    pub status: Option<String>,
    pub power_now_uw: Option<u64>,
    pub energy_now_uwh: Option<u64>,
    pub energy_full_uwh: Option<u64>,
}

fn read_optional(path: &str) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn read_number<T: std::str::FromStr>(path: &str) -> Option<T> {
    read_optional(path)?.parse::<T>().ok()
}

fn collect_one(name: &str) -> BatteryInfo {
    let base = format!("/sys/class/power_supply/{name}");
    BatteryInfo {
        name: name.to_string(),
        capacity_percent: read_number::<u8>(&format!("{base}/capacity")),
        status: read_optional(&format!("{base}/status")),
        power_now_uw: read_number::<u64>(&format!("{base}/power_now")),
        energy_now_uwh: read_number::<u64>(&format!("{base}/energy_now")),
        energy_full_uwh: read_number::<u64>(&format!("{base}/energy_full")),
    }
}

pub fn collect() -> Vec<BatteryInfo> {
    let mut batteries = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/power_supply") else {
        return batteries;
    };

    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let kind = read_optional(&format!("/sys/class/power_supply/{name}/type"))
            .unwrap_or_default()
            .to_lowercase();
        if kind == "battery" {
            names.push(name);
        }
    }
    names.sort();

    for name in names {
        batteries.push(collect_one(&name));
    }

    batteries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_battery_is_empty() {
        let battery = BatteryInfo::default();
        assert!(battery.capacity_percent.is_none());
        assert!(battery.status.is_none());
    }
}
