// SPDX-License-Identifier: GPL-3.0-or-later

use crate::colour::ColorChoice;
use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "averon-fetch",
    // The project tags releases with four components (for example
    // `0.1.1.1`), but Cargo requires a semver-compatible package version.
    // Keep the user-visible version in sync with the release tag here.
    version = "0.1.1.1",
    author,
    about = "A modern, lightweight system information utility",
    disable_version_flag = false,
    disable_help_flag = false
)]
pub struct Args {
    #[arg(
        long,
        action = clap::ArgAction::SetTrue,
        help = "Disable logo display"
    )]
    pub no_logo: bool,

    #[arg(
        long,
        visible_alias = "logo-only",
        action = clap::ArgAction::SetTrue,
        help = "Only display the logo"
    )]
    pub logo: bool,

    #[arg(
        long,
        value_name = "FILE",
        num_args = 0..=1,
        default_missing_value = "",
        help = "Use a specific configuration file. Without a value, show the config path and contents."
    )]
    pub config: Option<String>,

    #[arg(
        long,
        action = clap::ArgAction::SetTrue,
        help = "Print the effective configuration path and exit"
    )]
    pub config_path: bool,

    #[arg(
        long,
        action = clap::ArgAction::SetTrue,
        help = "Write a documented default configuration and exit"
    )]
    pub generate_config: bool,

    #[arg(
        long,
        action = clap::ArgAction::SetTrue,
        help = "List available modules with their enabled state and exit"
    )]
    pub list_modules: bool,

    #[arg(
        long,
        action = clap::ArgAction::SetTrue,
        help = "Force plain text output (no colour)"
    )]
    pub pipe: bool,

    #[arg(
        long = "no-color",
        action = clap::ArgAction::SetTrue,
        help = "Disable colour output"
    )]
    pub no_color: bool,

    #[arg(
        long,
        value_enum,
        default_value_t = ColorChoice::Auto,
        value_name = "WHEN",
        help = "Colour mode: auto, always, or never"
    )]
    pub color: ColorChoice,

    #[arg(
        long = "module",
        short = 'm',
        value_name = "MODULE",
        value_delimiter = ',',
        action = clap::ArgAction::Append,
        help = "Show only specific module(s) (comma-separated or repeated)"
    )]
    pub module: Vec<String>,

    #[arg(
        long,
        action = clap::ArgAction::SetTrue,
        help = "Show all modules even if disabled in config"
    )]
    pub all: bool,

    #[arg(
        long = "noip",
        alias = "no-ip",
        action = clap::ArgAction::SetTrue,
        help = "Hide network IP addresses"
    )]
    pub noip: bool,

    #[arg(
        long,
        value_enum,
        default_value_t = OutputFormat::Text,
        value_name = "FORMAT",
        help = "Output format"
    )]
    pub format: OutputFormat,

    #[arg(
        long,
        action = clap::ArgAction::SetTrue,
        help = "Print collector diagnostics to stderr"
    )]
    pub show_errors: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_defaults() {
        let args = Args::parse_from(["averon-fetch"]);
        assert!(!args.no_logo);
        assert!(!args.logo);
        assert_eq!(args.config, None);
        assert!(!args.config_path);
        assert!(!args.generate_config);
        assert!(!args.list_modules);
        assert!(!args.pipe);
        assert!(!args.no_color);
        assert_eq!(args.color, ColorChoice::Auto);
        assert!(args.module.is_empty());
        assert!(!args.all);
        assert!(!args.noip);
        assert_eq!(args.format, OutputFormat::Text);
        assert!(!args.show_errors);
    }

    #[test]
    fn test_cli_flags() {
        let args = Args::parse_from([
            "averon-fetch",
            "--no-logo",
            "--pipe",
            "-m",
            "cpu",
            "--module",
            "memory",
            "--noip",
        ]);
        assert!(args.no_logo);
        assert!(args.pipe);
        assert_eq!(args.module, vec!["cpu", "memory"]);
        assert!(args.noip);
    }

    #[test]
    fn test_cli_noip_alias() {
        let args = Args::parse_from(["averon-fetch", "--no-ip"]);
        assert!(args.noip);
    }

    #[test]
    fn test_cli_logo_only_alias() {
        let args = Args::parse_from(["averon-fetch", "--logo-only"]);
        assert!(args.logo);
    }

    #[test]
    fn test_cli_new_flags() {
        let args = Args::parse_from([
            "averon-fetch",
            "--list-modules",
            "--no-color",
            "--format",
            "json",
            "--config",
            "/tmp/averon.toml",
            "--show-errors",
            "--generate-config",
            "--config-path",
            "--all",
        ]);
        assert!(args.list_modules);
        assert!(args.no_color);
        assert_eq!(args.format, OutputFormat::Json);
        assert_eq!(args.config.as_deref(), Some("/tmp/averon.toml"));
        assert!(args.show_errors);
        assert!(args.generate_config);
        assert!(args.config_path);
        assert!(args.all);
    }

    #[test]
    fn test_cli_config_without_value() {
        let args = Args::parse_from(["averon-fetch", "--config"]);
        assert_eq!(args.config.as_deref(), Some(""));
    }

    #[test]
    fn test_cli_invalid_format_rejected() {
        assert!(Args::try_parse_from(["averon-fetch", "--format", "xml"]).is_err());
    }

    #[test]
    fn test_cli_invalid_colour_mode_rejected() {
        assert!(Args::try_parse_from(["averon-fetch", "--color", "sometimes"]).is_err());
    }

    #[test]
    fn test_cli_module_list_accepts_commas_and_repeats() {
        let args = Args::parse_from(["averon-fetch", "-m", "cpu,memory", "-m", "gpu"]);
        assert_eq!(args.module, ["cpu", "memory", "gpu"]);
    }
}
