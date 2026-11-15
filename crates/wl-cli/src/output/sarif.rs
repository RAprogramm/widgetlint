// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! SARIF 2.1.0 report for GitHub code scanning.

use masterror::{AppError, AppResult};
use serde_json::{Value, json};
use wl_check::Checked;
use wl_core::{LineIndex, Severity, rule::lookup};

use crate::command::check::Input;

/// Prints one SARIF document covering every input.
///
/// # Errors
///
/// [`masterror::AppError`] with `Internal` taxonomy if serialization fails.
pub fn print_check(inputs: &[Input], results: &[Checked]) -> AppResult<()> {
    let mut rule_ids: Vec<&'static str> = Vec::new();
    for checked in results {
        for finding in &checked.findings {
            if !rule_ids.contains(&finding.rule.id) {
                rule_ids.push(finding.rule.id);
            }
        }
    }
    let rules: Vec<Value> = rule_ids
        .iter()
        .filter_map(|id| lookup(id).map(|rule| (rule, *id)))
        .map(|(rule, id)| {
            json!({
                "id": id,
                "shortDescription": { "text": rule.title },
                "fullDescription": { "text": rule.help.unwrap_or(rule.title) },
                "helpUri": rule.spec,
                "defaultConfiguration": { "level": level(rule.default_severity) },
            })
        })
        .collect();
    let mut results_json: Vec<Value> = Vec::new();
    for (input, checked) in inputs.iter().zip(results) {
        let index = LineIndex::new(&input.source);
        for finding in &checked.findings {
            let region = finding.span.map(|span| {
                let start = index.position(&input.source, span.start);
                let end = index.position(&input.source, span.end);
                json!({
                    "startLine": start.line,
                    "startColumn": start.col,
                    "endLine": end.line,
                    "endColumn": end.col,
                })
            });
            let location = json!({
                "physicalLocation": {
                    "artifactLocation": { "uri": artifact_uri(&input.path) },
                    "region": region,
                }
            });
            results_json.push(json!({
                "ruleId": finding.rule.id,
                "ruleIndex": rule_ids.iter().position(|id| *id == finding.rule.id),
                "level": level(finding.severity),
                "message": { "text": finding.message },
                "locations": [location],
            }));
        }
    }
    let document = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "widgetlint",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/RAprogramm/widgetlint",
                    "rules": rules,
                }
            },
            "results": results_json,
        }]
    });
    let text = serde_json::to_string(&document)
        .map_err(|error| AppError::internal(format!("sarif serialization failed: {error}")))?;
    println!("{text}");
    Ok(())
}

const fn level(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warn => "warning",
        Severity::Info => "note"
    }
}

fn artifact_uri(path: &str) -> String {
    if path == "-" {
        "stdin".to_owned()
    } else {
        path.to_owned()
    }
}
