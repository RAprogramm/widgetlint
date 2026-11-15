// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! A single rule hit: rule metadata, resolved severity, message and span.

use crate::{
    rule::RuleMeta,
    severity::{Region, Severity},
    span::Span
};

/// One finding produced by a rule.
///
/// The rule metadata is `'static` (embedded data), so a finding carries no
/// per-rule allocation beyond its message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Rule that produced the finding.
    pub rule:     &'static RuleMeta,
    /// Severity after configuration overrides (defaults to the rule default).
    pub severity: Severity,
    /// Concrete description for this occurrence.
    pub message:  String,
    /// Fix hint; falls back to the rule help when rendering.
    pub help:     Option<String>,
    /// Byte span in the source, when the finding points at text.
    pub span:     Option<Span>,
    /// Which region the span points into.
    pub region:   Region
}

impl Finding {
    /// Creates a finding for `rule` with the default severity and no span.
    #[must_use]
    pub fn new(rule: &'static RuleMeta, message: impl Into<String>) -> Self {
        Self {
            rule,
            severity: rule.default_severity,
            message: message.into(),
            help: None,
            span: None,
            region: Region::Size
        }
    }

    /// Attaches a span and its region.
    #[must_use]
    pub const fn with_span(mut self, span: Span, region: Region) -> Self {
        self.span = Some(span);
        self.region = region;
        self
    }

    /// Overrides the fix hint for this occurrence.
    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Effective help: occurrence hint first, then the rule default.
    #[must_use]
    pub fn effective_help(&self) -> Option<&str> {
        self.help.as_deref().or(self.rule.help)
    }
}
