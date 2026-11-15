// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Report renderers.

pub mod agent;
pub mod human;
pub mod json;
pub mod sarif;

use wl_core::{Finding, Severity};

/// Overall verdict derived from finding severities.
pub fn verdict(findings: &[Finding]) -> &'static str {
    if findings
        .iter()
        .any(|finding| finding.severity == Severity::Error)
    {
        "fail"
    } else if findings
        .iter()
        .any(|finding| finding.severity == Severity::Warn)
    {
        "warn"
    } else {
        "pass"
    }
}
