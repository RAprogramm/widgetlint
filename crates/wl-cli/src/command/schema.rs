// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! `schema` subcommand: publishes the JSON Schema of the agent report so CI
//! and agent frameworks can validate reports without reading the source.

use masterror::AppResult;

use crate::cli::SchemaArgs;

/// Prints the agent-report JSON Schema.
///
/// * `args` — parsed `schema` arguments.
pub fn run(_args: &SchemaArgs) -> AppResult<bool> {
    println!("{}", wl_check::agent::schema_json()?);
    Ok(true)
}
