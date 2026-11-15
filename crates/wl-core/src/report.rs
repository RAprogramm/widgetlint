// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The versioned machine-readable report (spec Appendix A).

use serde::Serialize;

use crate::{
    SCHEMA_VERSION,
    finding::Finding,
    severity::Severity,
    span::{LineIndex, Span}
};

/// Serialized report: `widgetlint --format json`.
#[derive(Debug, Clone, Serialize)]
pub struct Report<'a> {
    /// Report contract version; additive changes only within a major.
    pub schema_version: u32,
    /// What was checked.
    pub input:          Input<'a>,
    /// Size block, present when size was measured.
    pub size:           Option<SizeBlock>,
    /// Counts per severity.
    pub summary:        Summary,
    /// All findings in stable order.
    pub diagnostics:    Vec<Diagnostic>
}

/// Input descriptor of a report.
#[derive(Debug, Clone, Serialize)]
pub struct Input<'a> {
    /// Path as given on the command line, `None` for stdin.
    pub path:  Option<&'a str>,
    /// Input size in bytes.
    pub bytes: usize
}

/// Size block of a report (filled by `wl-size`).
#[derive(Debug, Clone, Serialize)]
pub struct SizeBlock {
    /// Raw size in bytes.
    pub raw:         usize,
    /// zlib-9 size in bytes.
    pub zlib9:       usize,
    /// Whether the zlib size is approximate (WASM without parity).
    pub approximate: bool,
    /// Hard compressed limit.
    pub limit:       u32,
    /// Soft budget.
    pub budget:      u32
}

/// Counts per severity.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct Summary {
    /// Number of error findings.
    pub error: usize,
    /// Number of warn findings.
    pub warn:  usize,
    /// Number of info findings.
    pub info:  usize
}

/// Serialized finding with resolved line/column.
#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    /// Rule id.
    pub id:         &'static str,
    /// Severity as lowercase string.
    pub severity:   &'static str,
    /// Confidence as lowercase string.
    pub confidence: &'static str,
    /// Concrete message.
    pub message:    String,
    /// Fix hint when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help:       Option<String>,
    /// Resolved span when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span:       Option<SpanJson>,
    /// Specification URL.
    pub spec:       &'static str
}

/// Serialized span with 1-based line/column.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct SpanJson {
    /// Byte offset of the first byte.
    pub start:  u32,
    /// Byte offset just past the last byte.
    pub end:    u32,
    /// 1-based line.
    pub line:   u32,
    /// 1-based column in code points.
    pub col:    u32,
    /// Region name.
    pub region: &'static str
}

impl<'a> Report<'a> {
    /// Builds a report from findings, resolving spans against `source`.
    ///
    /// * `path` — input path as given, `None` for stdin.
    /// * `source` — the checked source text.
    /// * `size` — optional size block.
    /// * `findings` — findings in emission order.
    #[must_use]
    pub fn build(
        path: Option<&'a str>,
        source: &str,
        size: Option<SizeBlock>,
        findings: &[Finding]
    ) -> Self {
        let index = LineIndex::new(source);
        let mut summary = Summary::default();
        let mut diagnostics = Vec::with_capacity(findings.len());
        for finding in findings {
            match finding.severity {
                Severity::Error => summary.error += 1,
                Severity::Warn => summary.warn += 1,
                Severity::Info => summary.info += 1
            }
            diagnostics.push(Diagnostic::from_finding(finding, &index, source));
        }
        Self {
            schema_version: SCHEMA_VERSION,
            input: Input {
                path,
                bytes: source.len()
            },
            size,
            summary,
            diagnostics
        }
    }

    /// Serializes to a single-line JSON document.
    ///
    /// # Errors
    ///
    /// [`masterror::AppError`] with `Internal` taxonomy if serialization fails.
    pub fn to_json(&self) -> masterror::AppResult<String> {
        serde_json::to_string(self)
            .map_err(|error| masterror::AppError::internal(format!("report json: {error}")))
    }
}

impl Diagnostic {
    /// Converts a finding into its serialized form.
    #[must_use]
    pub fn from_finding(finding: &Finding, index: &LineIndex, source: &str) -> Self {
        let span = finding
            .span
            .map(|span| span_json(span, finding.region, index, source));
        Self {
            id: finding.rule.id,
            severity: finding.severity.as_str(),
            confidence: finding.rule.confidence.as_str(),
            message: finding.message.clone(),
            help: finding.effective_help().map(str::to_owned),
            span,
            spec: finding.rule.spec
        }
    }
}

fn span_json(
    span: Span,
    region: crate::severity::Region,
    index: &LineIndex,
    source: &str
) -> SpanJson {
    let position = index.position(source, span.start);
    SpanJson {
        start:  span.start,
        end:    span.end,
        line:   position.line,
        col:    position.col,
        region: region.as_str()
    }
}
