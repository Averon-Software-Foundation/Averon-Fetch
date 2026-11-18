// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::{AtomicBool, Ordering};

static SHOW_ERRORS: AtomicBool = AtomicBool::new(false);

pub fn set_show_errors(show: bool) {
    SHOW_ERRORS.store(show, Ordering::Relaxed);
}

pub fn show_errors() -> bool {
    SHOW_ERRORS.load(Ordering::Relaxed)
}

pub fn warn(message: impl AsRef<str>) {
    if show_errors() {
        eprintln!("averon-fetch: {}", message.as_ref());
    }
}

pub fn error(message: impl AsRef<str>) {
    eprintln!("averon-fetch: {}", message.as_ref());
}
