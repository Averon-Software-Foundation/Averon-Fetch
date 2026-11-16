use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct ShellInfo {
    pub name: Option<String>,
    pub version: Option<String>,
    pub terminal: Option<String>,
    pub terminal_version: Option<String>,
}

const KNOWN_TERMINALS: &[&str] = &[
    "alacritty",
    "kitty",
    "wezterm",
    "wezterm-gui",
    "gnome-terminal",
    "gnome-terminal-",
    "konsole",
    "xfce4-terminal",
    "xterm",
    "urxvt",
    "rxvt",
    "foot",
    "footclient",
    "hyprterm",
    "tilix",
    "terminator",
    "mate-terminal",
    "lxterminal",
    "st",
    "cool-retro-term",
    "terminology",
    "contour",
    "rio",
];

fn detect_shell() -> (Option<String>, Option<String>) {
    let shell_path = match env::var("SHELL") {
        Ok(s) => s,
        Err(_) => return (None, None),
    };

    let name = Path::new(&shell_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string());

    let version = name.as_deref().and_then(parse_shell_version);

    (name, version)
}

fn parse_shell_version(shell: &str) -> Option<String> {
    let out = Command::new(shell).arg("--version").output().ok()?;

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let first_line = combined.lines().next()?.trim().to_string();
    if first_line.is_empty() {
        return None;
    }

    let lower = shell.to_lowercase();
    if lower == "bash" {
        for part in first_line.split_whitespace() {
            if part
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                let version = part.split('(').next().unwrap_or(part);
                return Some(version.to_string());
            }
        }
    } else if lower == "zsh" {
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        return parts.get(1).map(|s| s.to_string());
    } else if lower == "fish" {
        for part in first_line.split_whitespace() {
            if part
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                return Some(part.to_string());
            }
        }
    } else {
        let mut parts = first_line.split_whitespace().skip(1);
        if let Some(candidate) = parts.next() {
            if candidate
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                return Some(candidate.to_string());
            }
        }
    }

    Some(first_line)
}

fn detect_terminal() -> (Option<String>, Option<String>) {
    if let Ok(term) = env::var("TERM_PROGRAM") {
        if !term.is_empty() {
            let version = env::var("TERM_PROGRAM_VERSION").ok();
            return (Some(term), version);
        }
    }

    if env::var("VTE_VERSION").is_ok() {
        let name = walk_proc_for_terminal().unwrap_or_else(|| "vte-terminal".to_string());
        return (Some(name), None);
    }

    if let Ok(term) = env::var("TERMINAL") {
        if !term.is_empty() {
            return (Some(term), None);
        }
    }

    if let Ok(term) = env::var("COLORTERM") {
        if !term.is_empty() && term != "truecolor" && term != "24bit" {
            return (Some(term), None);
        }
    }

    if let Some(name) = walk_proc_for_terminal() {
        return (Some(name), None);
    }

    (None, None)
}

fn read_proc_status_field(pid: u32, field: &str) -> Option<u32> {
    let path = format!("/proc/{}/status", pid);
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix(field) {
            let rest = rest.trim_start_matches(':').trim();
            return rest.split_whitespace().next()?.parse::<u32>().ok();
        }
    }
    None
}

fn read_proc_comm(pid: u32) -> Option<String> {
    fs::read_to_string(format!("/proc/{}/comm", pid))
        .ok()
        .map(|s| s.trim().to_string())
}

fn walk_proc_for_terminal() -> Option<String> {
    let mut pid = std::process::id();

    for _ in 0..32 {
        let ppid = match read_proc_status_field(pid, "PPid") {
            Some(p) => p,
            None => break,
        };

        if ppid == 0 || ppid == 1 || ppid == pid {
            break;
        }

        let comm = read_proc_comm(ppid)?;
        let comm_lower = comm.to_lowercase();

        if KNOWN_TERMINALS.iter().any(|&t| comm_lower.starts_with(t)) {
            return Some(comm);
        }

        pid = ppid;
    }

    None
}

pub fn collect() -> ShellInfo {
    let (name, version) = detect_shell();
    let (terminal, terminal_version) = detect_terminal();

    ShellInfo {
        name,
        version,
        terminal,
        terminal_version,
    }
}
