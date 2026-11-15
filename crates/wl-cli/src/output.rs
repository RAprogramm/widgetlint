// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Report renderers.

pub mod human;
pub mod json;

use wl_size::{Severity, SizeFinding};

/// Overall verdict derived from finding severities.
pub fn verdict(findings: &[SizeFinding]) -> &'static str {
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
