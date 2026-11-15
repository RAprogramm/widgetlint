// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! `rules` subcommand: list every known rule, human-readable or JSON.
//!
//! The JSON form is the machine contract for AI tooling: `widgetlint rules
//! --format json` gives an LLM the complete rule catalog with help texts and
//! specification links in one line.

use masterror::{AppError, AppResult};
use serde_json::json;
use wl_core::rule::all;

use crate::cli::{Format, RulesArgs};

/// Prints the rule catalog and returns `true`.
///
/// * `args` — parsed `rules` arguments.
pub fn run(args: &RulesArgs) -> AppResult<bool> {
    let mut rules = all();
    rules.sort_by_key(|rule| rule.id);
    match args.format {
        Format::Human => {
            for rule in rules {
                println!(
                    "{}  {:<5}  {:<9}  {}",
                    rule.id,
                    rule.default_severity.as_str(),
                    rule.confidence.as_str(),
                    rule.title
                );
            }
        }
        Format::Json | Format::Sarif => {
            let document: Vec<_> = rules
                .iter()
                .map(|rule| {
                    json!({
                        "id": rule.id,
                        "severity": rule.default_severity.as_str(),
                        "confidence": rule.confidence.as_str(),
                        "title": rule.title,
                        "help": rule.help,
                        "spec": rule.spec,
                    })
                })
                .collect();
            let text = serde_json::to_string(&document).map_err(|error| {
                AppError::internal(format!("json serialization failed: {error}"))
            })?;
            println!("{text}");
        }
    }
    Ok(true)
}
