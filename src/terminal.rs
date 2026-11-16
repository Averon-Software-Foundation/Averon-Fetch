#[cfg(unix)]
pub fn is_stdout_tty() -> bool {
    unsafe { libc::isatty(libc::STDOUT_FILENO) == 1 }
}

#[cfg(not(unix))]
pub fn is_stdout_tty() -> bool {
    false
}

pub fn supports_colour() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if std::env::var("TERM").as_deref() == Ok("dumb") {
        return false;
    }
    is_stdout_tty()
}

pub fn terminal_width() -> usize {
    if let Ok(cols) = std::env::var("COLUMNS") {
        if let Ok(n) = cols.parse::<usize>() {
            return n.clamp(20, 500);
        }
    }

    if let Some(width) = ioctl_width() {
        return width.clamp(20, 500);
    }

    80
}

#[cfg(unix)]
fn ioctl_width() -> Option<usize> {
    let mut size: libc::winsize = unsafe { std::mem::zeroed() };
    let result = unsafe {
        libc::ioctl(
            libc::STDOUT_FILENO,
            libc::TIOCGWINSZ,
            &mut size as *mut libc::winsize,
        )
    };
    if result == 0 && size.ws_col > 0 {
        Some(size.ws_col as usize)
    } else {
        None
    }
}

#[cfg(not(unix))]
fn ioctl_width() -> Option<usize> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_width_is_sane() {
        let width = terminal_width();
        assert!((20..=500).contains(&width));
    }
}
