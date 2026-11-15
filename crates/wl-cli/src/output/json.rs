// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Machine-readable single-line JSON report.

use masterror::{AppError, AppResult};
use serde_json::json;
use wl_core::Finding;
use wl_profile::limits::Limits;
use wl_size::SizeInfo;

use crate::output::verdict;

/// Prints one JSON document describing the measurement.
///
/// # Errors
///
/// [`masterror::AppError`] with `Internal` taxonomy if serialization fails.
pub fn print_size(
    file: &str,
    info: &SizeInfo,
    limits: &Limits,
    budget: u32,
    findings: &[Finding]
) -> AppResult<()> {
    let over_by = info
        .zlib9
        .saturating_sub(usize::try_from(limits.html_zlib9_bytes).unwrap_or(usize::MAX));
    let document = json!({
        "schema_version": 1,
        "file": file,
        "size": {
            "raw": info.raw,
            "zlib9": info.zlib9,
            "limit": limits.html_zlib9_bytes,
            "budget": budget,
            "over_by": over_by,
        },
        "findings": findings
            .iter()
            .map(|finding| {
                json!({
                    "rule": finding.rule.id,
                    "severity": finding.severity.as_str(),
                    "message": finding.message,
                })
            })
            .collect::<Vec<_>>(),
        "verdict": verdict(findings),
    });
    let text = serde_json::to_string(&document)
        .map_err(|error| AppError::internal(format!("json serialization failed: {error}")))?;
    println!("{text}");
    Ok(())
}
