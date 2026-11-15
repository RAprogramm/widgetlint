// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! End-to-end widget checking: the pure core every front end reuses.
//!
//! [`check`] takes one widget document and produces every finding: size
//! (TGW001 to TGW005), HTML (TGW100 to TGW150, TGW302 to TGW304), CSS from
//! `<style>` and `style=""` (TGW200 to TGW216, TGW300/301) and JavaScript
//! from `<script>` and `on*=""` (TGW400 to TGW411). Inline suppression
//! directives and the [`Config`] thresholds are applied before the findings
//! are returned, sorted by position.
//!
//! The crate performs no input or output: the CLI, WASM and LSP layers feed
//! the source text and render the result.
//!
//! # Examples
//!
//! ```
//! use wl_check::Config;
//!
//! let profile = wl_profile::profile_1().expect("valid profile");
//! let checked = wl_check::check("<input>", &profile, &Config::default()).expect("valid widget");
//! assert_eq!(
//!     checked.findings.first().expect("one finding").rule.id,
//!     "TGW100"
//! );
//! ```
//!
//! ```
//! use wl_check::Config;
//!
//! let profile = wl_profile::profile_1().expect("valid profile");
//! let source = "<!-- tgw-disable-file TGW100 --><input>";
//! let checked = wl_check::check(source, &profile, &Config::default()).expect("valid widget");
//! assert!(
//!     checked
//!         .findings
//!         .iter()
//!         .all(|finding| finding.rule.id != "TGW100")
//! );
//! ```

pub mod agent;
pub mod config;
pub mod suppress;

use masterror::AppResult;
use wl_core::{Finding, Span};
use wl_profile::Profile;
use wl_size::SizeInfo;

pub use crate::{agent::AgentOptions, config::Config, suppress::suppressions};

/// The result of checking one document.
#[derive(Debug)]
pub struct Checked {
    /// Findings that passed suppression and thresholds, in document order.
    pub findings:          Vec<Finding>,
    /// Measured size of the document.
    pub size:              SizeInfo,
    /// Findings removed by inline suppression directives.
    pub suppressed:        usize,
    /// Directives that suppressed no finding.
    pub unused_directives: usize
}

/// Checks one widget document.
///
/// * `source` — the widget HTML document.
/// * `profile` — the widget profile with the out-lists and the limits.
/// * `config` — thresholds, rule overrides and size budgets.
///
/// # Errors
///
/// [`masterror::AppError`] with `BadRequest` taxonomy when an embedded script
/// does not parse; the message names the first syntax problem.
pub fn check(source: &str, profile: &Profile, config: &Config) -> AppResult<Checked> {
    let size = SizeInfo::measure(source.as_bytes())?;
    let doc = wl_html::parse(source);
    let mut findings = wl_html::rules::run(&doc, source, profile);
    findings.extend(wl_size::evaluate(
        &size,
        &profile.limits,
        config.size.budget,
        config.size.margin
    ));
    for block in wl_html::extract::style_blocks(&doc.dom) {
        let text = slice(source, block.content);
        findings.extend(wl_css::check(text, block.content.start, profile));
    }
    for attr in wl_html::extract::inline_styles(&doc.dom) {
        if let Some(span) = attr.value_span {
            findings.extend(wl_css::check(&attr.value, span.start, profile));
        }
    }
    for block in wl_html::extract::script_blocks(&doc.dom) {
        if let Some(span) = block.content {
            let text = slice(source, span);
            findings.extend(wl_js::check(text, span.start, profile)?);
        }
    }
    for attr in wl_html::extract::event_handlers(&doc.dom) {
        if let Some(span) = attr.value_span {
            findings.extend(wl_js::check(&attr.value, span.start, profile)?);
        }
    }
    let suppressed = suppressions(source, &doc);
    let before = findings.len();
    findings.retain(|finding| !suppressed.is_suppressed(finding.rule.id, finding.span));
    let suppressed_count = before - findings.len();
    apply_rules(&mut findings, config);
    apply_thresholds(&mut findings, config);
    findings.sort_by_key(|finding| {
        (
            finding.span.map_or(u32::MAX, |span| span.start),
            finding.rule.id
        )
    });
    Ok(Checked {
        findings,
        size,
        suppressed: suppressed_count,
        unused_directives: suppressed.unused_count()
    })
}

/// Applies the `[rules]` overrides: `off` drops findings, severities replace
/// the default.
fn apply_rules(findings: &mut Vec<Finding>, config: &Config) {
    findings.retain(|finding| {
        config
            .rules
            .get(finding.rule.id)
            .is_none_or(|action| action.severity().is_some())
    });
    for finding in findings {
        if let Some(Some(severity)) = config
            .rules
            .get(finding.rule.id)
            .map(|action| action.severity())
        {
            finding.severity = severity;
        }
    }
}

/// Drops findings below the severity and confidence thresholds.
fn apply_thresholds(findings: &mut Vec<Finding>, config: &Config) {
    let output = config.output;
    findings.retain(|finding| {
        output.min_severity.allows(finding.severity)
            && output.min_confidence.allows(finding.rule.confidence)
    });
}

/// Slices `source` by a byte span, clamped to the document bounds.
#[must_use]
pub fn slice(source: &str, span: Span) -> &str {
    let from = usize::try_from(span.start).unwrap_or(source.len());
    let to = usize::try_from(span.end).unwrap_or(source.len());
    source.get(from..to).unwrap_or_default()
}
