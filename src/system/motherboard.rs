use std::fs;

#[derive(Debug, Clone, Default)]
pub struct MotherboardInfo {
    pub board_vendor: Option<String>,
    pub board_name: Option<String>,
    pub bios_vendor: Option<String>,
    pub bios_version: Option<String>,
}

fn read_dmi(name: &str) -> Option<String> {
    fs::read_to_string(format!("/sys/devices/virtual/dmi/id/{name}"))
        .or_else(|_| fs::read_to_string(format!("/sys/class/dmi/id/{name}")))
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty() && value != "None")
}

pub fn collect() -> MotherboardInfo {
    MotherboardInfo {
        board_vendor: read_dmi("board_vendor"),
        board_name: read_dmi("board_name"),
        bios_vendor: read_dmi("bios_vendor"),
        bios_version: read_dmi("bios_version"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_has_no_fake_values() {
        let info = MotherboardInfo::default();
        assert!(info.board_name.is_none());
        assert!(info.bios_version.is_none());
    }
}
