// SPDX-License-Identifier: GPL-3.0-or-later

//! Generate shell completions and man pages for release archives.
//!
//! This helper is only built with `--features docs`; the shipped binaries do
//! not depend on `clap_complete` or `clap_mangen`.
//!
//! Usage: `gen-docs [OUTPUT_DIR]` (defaults to `docs`). Files are written to
//! FHS-style paths under the output directory so the whole tree can be copied
//! into a release archive.

use clap::CommandFactory;
use clap_complete::Shell;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const BINARIES: [&str; 2] = ["averon-fetch", "av-fetch"];

fn completion_path(root: &Path, shell: Shell, bin: &str) -> PathBuf {
    match shell {
        Shell::Bash => root.join("share/bash-completion/completions").join(bin),
        Shell::Zsh => root
            .join("share/zsh/site-functions")
            .join(format!("_{bin}")),
        Shell::Fish => root
            .join("share/fish/vendor_completions.d")
            .join(format!("{bin}.fish")),
        Shell::PowerShell => root
            .join("share/powershell/completions")
            .join(format!("{bin}.ps1")),
        Shell::Elvish => root.join("share/elvish/lib").join(format!("{bin}.elv")),
        // `Shell` is non-exhaustive; this only triggers if clap_complete adds
        // a shell that the list above has not been extended for yet.
        _ => root.join("share").join(format!("{bin}.{shell}")),
    }
}

fn write_file(path: &Path, contents: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)
}

fn main() -> io::Result<()> {
    let root = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| String::from("docs")),
    );
    let shells = [
        Shell::Bash,
        Shell::Zsh,
        Shell::Fish,
        Shell::PowerShell,
        Shell::Elvish,
    ];

    for bin in BINARIES {
        let base = averon_fetch::cli::Args::command()
            .name(bin)
            .bin_name(bin)
            .display_name(bin);

        let mut man_page = Vec::new();
        clap_mangen::Man::new(base.clone()).render(&mut man_page)?;
        write_file(
            &root.join("share/man/man1").join(format!("{bin}.1")),
            &man_page,
        )?;

        for shell in shells {
            let mut completion = Vec::new();
            clap_complete::generate(shell, &mut base.clone(), bin, &mut completion);
            write_file(&completion_path(&root, shell, bin), &completion)?;
        }
    }

    println!("Wrote completions and man pages to {}", root.display());
    Ok(())
}
