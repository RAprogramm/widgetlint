// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! `verify` subcommand: the minimal verdict an agent needs between edits —
//! pass or fail, one reason, one next action, nothing else.

use masterror::AppResult;
use wl_check::agent::{self, AgentOptions};
use wl_profile::Profile;

use crate::{
    cli::{FailOn, VerifyArgs},
    command::check::{Input, collect_inputs, load_config_file, profile_for},
    output::agent::severity_of
};

/// Checks every input and prints only the verdict.
///
/// * `args` — parsed `verify` arguments.
///
/// Returns `true` when no finding reaches the `--fail-on` threshold.
pub fn run(args: &VerifyArgs) -> AppResult<bool> {
    let config = load_config_file(args.config.as_deref())?;
    let profile = profile_for(config.profile)?;
    let inputs = collect_inputs(&args.paths, &config)?;
    let mut verdicts: Vec<serde_json::Value> = Vec::new();
    for input in &inputs {
        let checked = wl_check::check(&input.source, &profile, &config)?;
        verdicts.push(build(input, &checked, &profile, &config, args.fail_on));
    }
    let failed = verdicts.iter().any(|verdict| verdict["verdict"] == "fail");
    let json = if let [only] = verdicts.as_slice() {
        serde_json::to_string(only)
    } else {
        serde_json::to_string(&verdicts)
    }
    .map_err(|error| masterror::AppError::internal(format!("verify json: {error}")))?;
    println!("{json}");
    Ok(!failed)
}

fn build(
    input: &Input,
    checked: &wl_check::Checked,
    profile: &Profile,
    config: &wl_check::Config,
    fail_on: FailOn
) -> serde_json::Value {
    let options = AgentOptions {
        fail_on: severity_of(fail_on),
        limit: profile.limits.html_zlib9_bytes,
        budget: config.size.budget,
        ..AgentOptions::default()
    };
    let report = agent::report(&input.path, &input.source, checked, &options);
    serde_json::json!({
        "schema_version": wl_core::SCHEMA_VERSION,
        "input": report.input,
        "verdict": report.verdict,
        "reason": report.reason,
        "next_action": report.next_action,
    })
}
