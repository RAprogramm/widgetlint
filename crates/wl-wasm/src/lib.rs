// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Browser entry points of `widgetlint`: the same pipeline as the CLI,
//! compiled to WebAssembly for the playground.
//!
//! The compressed size measured here uses the pure-Rust zlib backend and can
//! differ from the reference zlib by a byte or two; the playground labels it
//! as approximate. Everything else — rules, spans, verdicts — is identical to
//! the native binary.

use wasm_bindgen::prelude::*;
use wl_check::{AgentOptions, Config, check};
use wl_core::Severity;

/// Converts a pipeline error into a JavaScript exception.
fn to_js(error: masterror::AppError) -> JsError {
    let text = error
        .message
        .as_deref()
        .unwrap_or_else(|| error.kind.label());
    JsError::new(text)
}

fn severity(fail_on: &str) -> Severity {
    match fail_on {
        "info" => Severity::Info,
        "warn" => Severity::Warn,
        _ => Severity::Error,
    }
}

/// Checks one widget document and returns the agent report as JSON.
///
/// * `source` — the widget HTML.
/// * `fail_on` — `info`, `warn` or `error`: the severity that fails the check.
///
/// # Errors
///
/// [`JsError`] when the document cannot be checked (for example a script with
/// a syntax error) or the profile fails to load.
#[wasm_bindgen]
pub fn check_agent(source: &str, fail_on: &str) -> Result<String, JsError> {
    let profile = wl_profile::profile_1().map_err(to_js)?;
    let config = Config::default();
    let checked = check(source, &profile, &config).map_err(to_js)?;
    let options = AgentOptions {
        fail_on: severity(fail_on),
        limit: profile.limits.html_zlib9_bytes,
        budget: config.size.budget,
        tool_version: env!("CARGO_PKG_VERSION"),
        ..AgentOptions::default()
    };
    let report = wl_check::agent::report("widget.html", source, &checked, &options);
    serde_json::to_string(&report).map_err(|error| JsError::new(&error.to_string()))
}

/// Returns the tool identity as JSON: name, version, schema and ruleset.
///
/// # Errors
///
/// [`JsError`] if serialization fails.
#[wasm_bindgen]
pub fn version_json() -> Result<String, JsError> {
    let json = serde_json::json!({
        "name": "widgetlint",
        "version": env!("CARGO_PKG_VERSION"),
        "schema_version": wl_core::SCHEMA_VERSION,
        "profile": 1,
        "rules": wl_core::rule::all().len(),
        "ruleset": format!("fnv1a64:{}", wl_core::rule::fingerprint()),
    });
    serde_json::to_string(&json).map_err(|error| JsError::new(&error.to_string()))
}

/// Returns the rule catalog as JSON: id, severity, confidence, title, help and
/// the specification link of every known rule.
///
/// # Errors
///
/// [`JsError`] if serialization fails.
#[wasm_bindgen]
pub fn rules_json() -> Result<String, JsError> {
    let rules: Vec<_> = wl_core::rule::all()
        .into_iter()
        .map(|rule| {
            serde_json::json!({
                "id": rule.id,
                "severity": rule.default_severity.as_str(),
                "confidence": rule.confidence.as_str(),
                "title": rule.title,
                "help": rule.help,
                "spec": rule.spec,
            })
        })
        .collect();
    serde_json::to_string(&rules).map_err(|error| JsError::new(&error.to_string()))
}
