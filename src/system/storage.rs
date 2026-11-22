// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::CString;
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct DiskInfo {
    pub device: String,
    pub mount_point: String,
    pub filesystem: Option<String>,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
}

/// Pseudo-filesystems that never represent on-disk storage. Keeping this list
/// generous means `collect` does not need to special-case containers, where
/// almost every mount is `overlay`, `tmpfs`, or a kernel interface.
const SKIP_FS: &[&str] = &[
    "tmpfs",
    "devtmpfs",
    "devpts",
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
    "selinuxfs",
    "smackfs",
    "hugetlbfs",
    "mqueue",
    "fusectl",
    "configfs",
    "binfmt_misc",
    "rpc_pipefs",
    "ramfs",
    "overlay",
    "squashfs",
    "autofs",
    "nsfs",
];

struct MountEntry {
    device: String,
    mount_point: String,
    filesystem: String,
}

/// Expand the octal escapes used by `/proc/self/mounts`. Mount points may
/// contain spaces and tabs, so they are stored as `\040`, `\011`, `\012`, and
/// `\134` rather than as literal whitespace.
fn unescape_mount_field(field: &str) -> String {
    let bytes = field.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'\\' && index + 3 < bytes.len() {
            let digits = &bytes[index + 1..index + 4];
            if digits.iter().all(|b| (b'0'..=b'7').contains(b)) {
                let value = u32::from(digits[0] - b'0') * 64
                    + u32::from(digits[1] - b'0') * 8
                    + u32::from(digits[2] - b'0');
                out.push(value as u8);
                index += 4;
                continue;
            }
        }

        out.push(bytes[index]);
        index += 1;
    }

    String::from_utf8_lossy(&out).into_owned()
}

fn parse_mount_line(line: &str) -> Option<MountEntry> {
    let mut fields = line.split_whitespace();
    let device = unescape_mount_field(fields.next()?);
    let mount_point = unescape_mount_field(fields.next()?);
    let filesystem = unescape_mount_field(fields.next()?);

    if device.is_empty() || mount_point.is_empty() || filesystem.is_empty() {
        return None;
    }

    Some(MountEntry {
        device,
        mount_point,
        filesystem,
    })
}

fn is_virtual(entry: &MountEntry) -> bool {
    entry.filesystem.starts_with("fuse.")
        || SKIP_FS.contains(&entry.filesystem.as_str())
        || entry.device == "none"
        || entry.device.starts_with("/dev/loop")
}

// The cast is required on 32-bit targets but redundant on 64-bit ones.
#[allow(clippy::unnecessary_cast)]
#[cfg(unix)]
fn disk_usage(path: &str) -> Option<(u64, u64, u64)> {
    let path = CString::new(path).ok()?;
    let mut stats: libc::statvfs = unsafe { std::mem::zeroed() };
    let result = unsafe { libc::statvfs(path.as_ptr(), &mut stats) };

    if result != 0 {
        return None;
    }

    // f_frsize is the fundamental block size (the value `df` reports with).
    // A few filesystems leave it unset, in which case f_bsize is the best
    // available fallback.
    let block_size = if stats.f_frsize > 0 {
        stats.f_frsize
    } else {
        stats.f_bsize
    } as u64;
    let total = stats.f_blocks as u64 * block_size;
    let free = stats.f_bfree as u64 * block_size;
    let available = stats.f_bavail as u64 * block_size;
    let used = total.saturating_sub(free);

    Some((total, used, available))
}

#[cfg(not(unix))]
fn disk_usage(_path: &str) -> Option<(u64, u64, u64)> {
    None
}

fn collect_from_mounts() -> Option<Vec<DiskInfo>> {
    let content = fs::read("/proc/self/mounts").ok()?;
    let content = String::from_utf8_lossy(&content);
    let mut disks = Vec::new();
    let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();

    for line in content.lines() {
        let Some(entry) = parse_mount_line(line) else {
            continue;
        };

        if is_virtual(&entry) {
            continue;
        }

        if !seen.insert((entry.device.clone(), entry.mount_point.clone())) {
            continue;
        }

        let Some((total, used, free)) = disk_usage(&entry.mount_point) else {
            continue;
        };

        if total == 0 {
            continue;
        }

        disks.push(DiskInfo {
            device: entry.device,
            mount_point: entry.mount_point,
            filesystem: Some(entry.filesystem),
            total_bytes: total,
            used_bytes: used,
            free_bytes: free,
        });
    }

    Some(disks)
}

/// Fallback for systems without `/proc/self/mounts`. GNU `df --output` is not
/// available in BusyBox, so this is only used when the procfs path is missing.
fn collect_from_df() -> Vec<DiskInfo> {
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

pub fn collect() -> Vec<DiskInfo> {
    if let Some(disks) = collect_from_mounts() {
        return disks;
    }

    collect_from_df()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unescape_mount_field() {
        assert_eq!(unescape_mount_field("/mnt/my\\040disk"), "/mnt/my disk");
        assert_eq!(unescape_mount_field("/mnt/\\011tab"), "/mnt/\ttab");
        assert_eq!(unescape_mount_field("/mnt/plain"), "/mnt/plain");
        assert_eq!(unescape_mount_field("back\\134slash"), "back\\slash");
        // A backslash that does not introduce three octal digits is preserved.
        assert_eq!(unescape_mount_field("odd\\04"), "odd\\04");
    }

    #[test]
    fn test_parse_mount_line() {
        let line = "/dev/nvme0n1p2 / btrfs rw,relatime,compress=zstd 0 0";
        let entry = parse_mount_line(line).expect("valid mount line");
        assert_eq!(entry.device, "/dev/nvme0n1p2");
        assert_eq!(entry.mount_point, "/");
        assert_eq!(entry.filesystem, "btrfs");
    }

    #[test]
    fn test_parse_mount_line_with_escaped_space() {
        let line = "/dev/sda1 /mnt/my\\040disk ext4 rw 0 0";
        let entry = parse_mount_line(line).expect("valid mount line");
        assert_eq!(entry.mount_point, "/mnt/my disk");
        assert_eq!(entry.filesystem, "ext4");
    }

    #[test]
    fn test_parse_mount_line_rejects_short_lines() {
        assert!(parse_mount_line("/dev/sda1 /").is_none());
        assert!(parse_mount_line("").is_none());
    }

    #[test]
    fn test_is_virtual() {
        let fake = |device: &str, filesystem: &str| MountEntry {
            device: String::from(device),
            mount_point: String::from("/somewhere"),
            filesystem: String::from(filesystem),
        };

        assert!(is_virtual(&fake("proc", "proc")));
        assert!(is_virtual(&fake("tmpfs", "tmpfs")));
        assert!(is_virtual(&fake("overlay", "overlay")));
        assert!(is_virtual(&fake("/dev/loop0", "squashfs")));
        assert!(is_virtual(&fake("gvfsd", "fuse.gvfsd-fuse")));
        assert!(!is_virtual(&fake("/dev/sda1", "ext4")));
        assert!(!is_virtual(&fake("tank/root", "zfs")));
    }

    #[cfg(unix)]
    #[test]
    fn test_disk_usage_for_root() {
        let (total, used, available) = disk_usage("/").expect("root filesystem is readable");
        assert!(total > 0);
        assert!(used <= total);
        assert!(available <= total);
    }
}
