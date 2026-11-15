// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! `version` subcommand: tool and contract identity for agents and CI.

use wl_core::SCHEMA_VERSION;

use crate::cli::VersionArgs;

/// Prints the tool version, schema version and rule count.
///
/// * `args` — parsed `version` arguments.
pub fn run(args: &VersionArgs) -> bool {
    let rules = wl_core::rule::all().len();
    if args.json {
        let json = serde_json::json!({
            "name": "widgetlint",
            "version": env!("CARGO_PKG_VERSION"),
            "schema_version": SCHEMA_VERSION,
            "profile": 1,
            "rules": rules,
            "ruleset": format!("fnv1a64:{}", wl_core::rule::fingerprint())
        });
        println!("{json}");
    } else {
        println!(
            "widgetlint {} (schema {}, profile 1, {rules} rules)",
            env!("CARGO_PKG_VERSION"),
            SCHEMA_VERSION
        );
    }
    true
}
