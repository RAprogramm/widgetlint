// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! `agent` renderer: compact single-line JSON for LLM fix loops.
//!
//! One input produces one object, several inputs produce an array. The shape
//! is fixed by the published schema (`widgetlint schema`), contains no ANSI
//! escapes and stays small by grouping repeats and truncating.

use std::collections::BTreeSet;

use masterror::AppResult;
use wl_check::{AgentOptions, Checked, Config, agent};
use wl_core::Severity;
use wl_profile::Profile;

use crate::{cli::FailOn, command::check::Input};

/// Everything the agent renderer needs besides the checked inputs.
pub struct RenderOptions<'a> {
    /// Profile providing the hard compressed limit.
    pub profile:      &'a Profile,
    /// Effective configuration providing the budget.
    pub config:       &'a Config,
    /// Severity that flips the verdict to `fail`.
    pub fail_on:      FailOn,
    /// Grouped findings kept per report.
    pub max_findings: usize,
    /// Fix-loop iteration echoed back.
    pub iteration:    u32,
    /// Fingerprints from the previous run, when given.
    pub since:        Option<&'a BTreeSet<String>>
}

/// Renders agent reports for every checked input.
///
/// * `inputs` — checked inputs in order.
/// * `results` — pipeline results, index-aligned with `inputs`.
/// * `options` — thresholds, limits and previous-run fingerprints.
pub fn print_check(
    inputs: &[Input],
    results: &[Checked],
    options: &RenderOptions<'_>
) -> AppResult<()> {
    let agent_options = AgentOptions {
        fail_on:      severity_of(options.fail_on),
        max_findings: options.max_findings,
        iteration:    options.iteration,
        since:        options.since,
        limit:        options.profile.limits.html_zlib9_bytes,
        budget:       options.config.size.budget,
        tool_version: env!("CARGO_PKG_VERSION")
    };
    let reports: Vec<_> = inputs
        .iter()
        .zip(results)
        .map(|(input, result)| agent::report(&input.path, &input.source, result, &agent_options))
        .collect();
    let json = if let [only] = reports.as_slice() {
        serde_json::to_string(only)
    } else {
        serde_json::to_string(&reports)
    }
    .map_err(|error| masterror::AppError::internal(format!("agent report: {error}")))?;
    println!("{json}");
    Ok(())
}

/// Maps the `--fail-on` argument onto a severity threshold.
#[must_use]
pub const fn severity_of(fail_on: FailOn) -> Severity {
    match fail_on {
        FailOn::Info => Severity::Info,
        FailOn::Warn => Severity::Warn,
        FailOn::Error => Severity::Error
    }
}
