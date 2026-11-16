use std::process::Command;

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct DiskInfo {
    pub device: String,
    pub mount_point: String,
    pub filesystem: Option<String>,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
}

const SKIP_FS: &[&str] = &[
    "tmpfs",
    "devtmpfs",
    "sysfs",
    "proc",
    "cgroup",
    "cgroup2",
    "pstore",
    "efivarfs",
    "bpf",
    "tracefs",
    "debugfs",
    "securityfs",
    "hugetlbfs",
    "mqueue",
    "fusectl",
    "configfs",
    "ramfs",
    "overlay",
    "squashfs",
    "autofs",
    "nsfs",
];

pub fn collect() -> Vec<DiskInfo> {
    let output = match Command::new("df")
        .args(["--output=source,target,fstype,size,used,avail", "-B1"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return vec![],
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut disks = Vec::new();

    for line in stdout.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 6 {
            continue;
        }

        let device = cols[0];
        let mount = cols[1];
        let fstype = cols[2];
        let total_str = cols[3];
        let used_str = cols[4];
        let avail_str = cols[5];

        if !device.starts_with("/dev/") {
            continue;
        }
        if device.starts_with("/dev/loop") {
            continue;
        }

        if SKIP_FS.contains(&fstype) {
            continue;
        }

        let total_bytes = total_str.parse::<u64>().unwrap_or(0);
        let used_bytes = used_str.parse::<u64>().unwrap_or(0);
        let free_bytes = avail_str.parse::<u64>().unwrap_or(0);

        disks.push(DiskInfo {
            device: device.to_string(),
            mount_point: mount.to_string(),
            filesystem: Some(fstype.to_string()),
            total_bytes,
            used_bytes,
            free_bytes,
        });
    }

    disks
}
