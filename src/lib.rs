pub mod cli;
pub mod colour;
pub mod config;
pub mod diag;
pub mod logo;
pub mod modules;
pub mod output;
pub mod system;
pub mod terminal;
pub mod text;

use clap::Parser;

pub fn run() {
    let args = cli::Args::parse();
    diag::set_show_errors(args.show_errors);

    let is_tty = terminal::is_stdout_tty();
    let colour_environment = colour::ColourEnvironment::from_env();
    let config_path = config::Config::resolve_path(args.config.as_deref());
    let config = config::Config::load_from(&config_path);
    let palette = colour::Palette::from_config(&config);

    if args.config_path {
        println!("{}", config_path.display());
        return;
    }

    if args.generate_config {
        match config::Config::write_default(&config_path) {
            Ok(true) => println!("Wrote default configuration to {}", config_path.display()),
            Ok(false) => println!("Configuration already exists at {}", config_path.display()),
            Err(error) => {
                diag::error(format!(
                    "failed to write config {}: {error}",
                    config_path.display()
                ));
                return;
            }
        }
        return;
    }

    if matches!(args.config.as_deref(), Some("")) {
        println!("Config file: {}", config_path.display());
        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(contents) => println!("{contents}"),
                Err(error) => diag::error(format!(
                    "failed to read config {}: {error}",
                    config_path.display()
                )),
            }
        } else {
            println!("(no config file found, using defaults)");
        }
        return;
    }

    let depth = colour::resolve_depth(&args, &config, &colour_environment, is_tty);

    if args.logo {
        let term_width = terminal::terminal_width();
        let width = if is_tty {
            logo::configured_width(&config).min(term_width)
        } else {
            logo::configured_width(&config)
        };
        print!(
            "{}",
            logo::render_to_string(&config, depth, &palette, width)
        );
        return;
    }

    if args.list_modules {
        print!("{}", modules::list_modules(&config));
        return;
    }

    let selection = modules::Selection::resolve(&config, &args);
    if selection.is_empty() {
        diag::warn("no modules selected");
        return;
    }

    let plan = system::CollectPlan::from_selection(&selection);
    let info = system::collect(&plan);
    let document = modules::build_document(&info, &config, &selection, args.noip);

    match args.format {
        cli::OutputFormat::Text => {
            let show_logo =
                config.general.logo && !args.no_logo && (args.module.is_empty() || args.all);
            let options = output::TextOptions {
                depth,
                palette,
                term_width: terminal::terminal_width(),
                is_tty,
                show_logo,
            };
            print!("{}", output::render_text(&document, &config, &options));
        }
        cli::OutputFormat::Json => match output::render_json(&document, &config) {
            Ok(json) => print!("{json}"),
            Err(error) => diag::error(format!("failed to render JSON: {error}")),
        },
    }
}
