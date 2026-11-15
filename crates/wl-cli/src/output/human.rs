// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Human-readable report.

use wl_core::Finding;
use wl_profile::limits::Limits;
use wl_size::SizeInfo;

use crate::output::verdict;

/// Prints the one-line measurement plus one line per finding.
pub fn print_size(
    file: &str,
    info: &SizeInfo,
    limits: &Limits,
    budget: u32,
    findings: &[Finding]
) {
    let raw = info.raw;
    let zlib9 = i64::try_from(info.zlib9).unwrap_or(i64::MAX);
    let limit = i64::from(limits.html_zlib9_bytes);
    let delta = zlib9 - limit;
    let sign = if delta >= 0 { '+' } else { '-' };
    let verdict = verdict(findings);
    println!(
        "{file}: {raw} B -> {zlib9} B (zlib-9)    limit {limit}    budget {budget}    {verdict} \
         ({sign}{})",
        delta.abs()
    );
    for finding in findings {
        println!(
            "  {} [{}] {}",
            finding.rule.id,
            finding.severity.as_str(),
            finding.message
        );
    }
}
