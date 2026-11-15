// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Human-readable report.

use wl_check::Checked;
use wl_core::{Finding, LineIndex, Severity};
use wl_profile::limits::Limits;
use wl_size::SizeInfo;

use crate::{command::check::Input, output::verdict};

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

/// Prints one section per input plus the totals line.
pub fn print_check(inputs: &[Input], results: &[Checked]) {
    let mut error = 0_usize;
    let mut warn = 0_usize;
    let mut info = 0_usize;
    for (input, checked) in inputs.iter().zip(results) {
        println!("{}", input.path);
        let index = LineIndex::new(&input.source);
        for finding in &checked.findings {
            match finding.severity {
                Severity::Error => error += 1,
                Severity::Warn => warn += 1,
                Severity::Info => info += 1
            }
            let location = finding.span.map_or_else(String::new, |span| {
                let position = index.position(&input.source, span.start);
                format!("    {}:{}", position.line, position.col)
            });
            println!(
                "  {:<5}  {}  {}{location}",
                finding.severity.as_str(),
                finding.rule.id,
                finding.message
            );
            if let Some(help) = finding.effective_help() {
                println!("         help: {help}");
            }
        }
    }
    let total = error + warn + info;
    if total == 0 {
        println!("no findings");
        return;
    }
    println!(
        "{}, {}, {info} info",
        plural(error, "error"),
        plural(warn, "warning")
    );
}

fn plural(count: usize, word: &str) -> String {
    if count == 1 {
        format!("{count} {word}")
    } else {
        format!("{count} {word}s")
    }
}
