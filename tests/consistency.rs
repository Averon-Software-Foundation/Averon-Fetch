// SPDX-License-Identifier: GPL-3.0-or-later

//! Guards against drift between the module registry, the configuration model,
//! and the generated default configuration.

use averon_fetch::config::{Config, DEFAULT_CONFIG_TEMPLATE};
use averon_fetch::modules::{canonical_id, list_modules, MODULES};

/// The config key for a module. `wm` is the only module whose id differs from
/// its `[modules]` field name.
fn config_key(module: &str) -> &str {
    match module {
        "wm" => "window_manager",
        other => other,
    }
}

#[test]
fn every_registry_id_resolves_to_itself() {
    for module in MODULES {
        assert_eq!(
            canonical_id(module.id()),
            Some(module.id()),
            "{} does not resolve to itself",
            module.id()
        );
    }
}

#[test]
fn every_registry_id_has_a_config_key() {
    for module in MODULES {
        let key = config_key(module.id());
        assert!(
            DEFAULT_CONFIG_TEMPLATE.contains(&format!("\n{key} = ")),
            "{} ({key}) is missing from DEFAULT_CONFIG_TEMPLATE",
            module.id()
        );
    }
}

#[test]
fn every_module_can_be_disabled_from_toml() {
    for module in MODULES {
        // Prefer the module id so `[modules] wm = false` keeps working as an
        // alias for `window_manager`.
        let key = module.id();
        let toml = format!("[modules]\n{key} = false\n");
        let config: Config =
            toml::from_str(&toml).unwrap_or_else(|e| panic!("{key} config failed to parse: {e}"));
        assert!(
            !config.modules.is_enabled(module.id()),
            "{key} = false did not disable {}",
            module.id()
        );
    }
}

#[test]
fn every_registry_id_is_listed() {
    let listing = list_modules(&Config::default());
    for module in MODULES {
        assert!(
            listing.contains(module.id()),
            "{} is missing from --list-modules output",
            module.id()
        );
    }
}
