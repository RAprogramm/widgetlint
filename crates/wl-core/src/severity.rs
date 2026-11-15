// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Severity, confidence and source region of a finding.

/// How bad a finding is.
///
/// Ordered: `Info < Warn < Error`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Recommendation or heuristic.
    Info,
    /// Guaranteed wrong behavior below the hard error.
    Warn,
    /// The server refuses the widget.
    Error
}

impl Severity {
    /// Stable lowercase name for serialization.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error"
        }
    }
}

/// How sure the rule is about the finding.
///
/// Heuristic findings are hidden by default (spec risk R2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    /// The outcome is guaranteed by the specification.
    Certain,
    /// A strong guess; may produce false positives.
    Heuristic
}

impl Confidence {
    /// Stable lowercase name for serialization.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Certain => "certain",
            Self::Heuristic => "heuristic"
        }
    }
}

/// Which part of the widget a span points into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    /// The HTML document itself.
    Html,
    /// A CSS block (inside `<style>` or a `style=""` attribute).
    Css,
    /// A JavaScript block (inside `<script>` or an `on*` attribute).
    Js,
    /// Whole-document size, no textual span.
    Size
}

impl Region {
    /// Stable lowercase name for serialization.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Html => "html",
            Self::Css => "css",
            Self::Js => "js",
            Self::Size => "size"
        }
    }
}
