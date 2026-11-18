use std::fs;
use std::io::BufRead;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct PackageCount {
    pub manager: String,
    pub count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct PackageInfo {
    pub counts: Vec<PackageCount>,
}

fn count_pacman() -> Option<PackageCount> {
    let dir = Path::new("/var/lib/pacman/local");
    if !dir.exists() {
        return None;
    }
    let count = fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .filter(|e| e.file_name() != "ALPM_DB_VERSION")
        .count();
    Some(PackageCount {
        manager: "pacman".to_string(),
        count,
    })
}

fn count_dpkg() -> Option<PackageCount> {
    let path = Path::new("/var/lib/dpkg/status");
    if !path.exists() {
        return None;
    }
    // The status database is tens of megabytes on a typical Debian install, so
    // count matching lines as they stream in rather than loading it all. The
    // bytes are compared directly: package descriptions are not always valid
    // UTF-8 and a decoding error must not stop the count.
    let file = fs::File::open(path).ok()?;
    let mut reader = std::io::BufReader::new(file);
    let mut line = Vec::new();
    let mut count = 0;
    while reader.read_until(b'\n', &mut line).unwrap_or(0) > 0 {
        if line.trim_ascii() == b"Status: install ok installed" {
            count += 1;
        }
        line.clear();
    }
    Some(PackageCount {
        manager: "dpkg".to_string(),
        count,
    })
}

fn count_rpm() -> Option<PackageCount> {
    let out = Command::new("rpm")
        .args(["-qa", "--qf", "\n"])
        .output()
        .ok()?;
    let count = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count();
    Some(PackageCount {
        manager: "rpm".to_string(),
        count,
    })
}

fn count_flatpak() -> Option<PackageCount> {
    let out = Command::new("flatpak")
        .args(["list", "--app", "--columns=application"])
        .output()
        .ok()?;
    let count = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count();
    Some(PackageCount {
        manager: "flatpak".to_string(),
        count,
    })
}

fn count_snap() -> Option<PackageCount> {
    let snap_dir = Path::new("/snap");
    if !snap_dir.exists() {
        return None;
    }
    let count = fs::read_dir(snap_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name != "bin" && !name.starts_with("core")
        })
        .count();

    if count == 0 {
        return None;
    }
    Some(PackageCount {
        manager: "snap".to_string(),
        count,
    })
}

pub fn collect() -> PackageInfo {
    let counters: Vec<Option<PackageCount>> = vec![
        count_pacman(),
        count_dpkg(),
        count_rpm(),
        count_flatpak(),
        count_snap(),
    ];

    let counts: Vec<PackageCount> = counters.into_iter().flatten().collect();
    PackageInfo { counts }
}
