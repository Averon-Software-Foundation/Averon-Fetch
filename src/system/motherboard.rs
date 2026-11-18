// SPDX-License-Identifier: GPL-3.0-or-later

use super::dmi;

#[derive(Debug, Clone, Default)]
pub struct MotherboardInfo {
    pub board_vendor: Option<String>,
    pub board_name: Option<String>,
    pub bios_vendor: Option<String>,
    pub bios_version: Option<String>,
}

pub fn collect() -> MotherboardInfo {
    MotherboardInfo {
        board_vendor: dmi::read("board_vendor"),
        board_name: dmi::read("board_name"),
        bios_vendor: dmi::read("bios_vendor"),
        bios_version: dmi::read("bios_version"),
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
