// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! `explain` subcommand: print the documentation of one rule.

use masterror::{AppError, AppResult};
use wl_core::rule::lookup;

use crate::cli::ExplainArgs;

/// Prints the rule documentation and returns `true`.
///
/// * `args` — parsed `explain` arguments.
pub fn run(args: &ExplainArgs) -> AppResult<bool> {
    let id = args.id.to_ascii_uppercase();
    let Some(rule) = lookup(&id) else {
        return Err(AppError::not_found(format!(
            "unknown rule '{}'; run `widgetlint rules` for the full list",
            args.id
        )));
    };
    println!(
        "{}  {}  {}",
        rule.id,
        rule.default_severity.as_str(),
        rule.confidence.as_str()
    );
    println!("{}", rule.title);
    if let Some(help) = rule.help {
        println!("help: {help}");
    }
    println!("spec: {}", rule.spec);
    Ok(true)
}
