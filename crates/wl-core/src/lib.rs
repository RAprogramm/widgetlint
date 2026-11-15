// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Canonical finding model, rule registry and report contract.
//!
//! This crate is the pure kernel of `widgetlint`: no I/O, no clocks, no
//! randomness. Rule metadata lives in TOML data (`ruleset/rules/`) and is
//! compiled into the binary; rules themselves are thin code over that data.
//!
//! # Examples
//!
//! ```
//! let meta = wl_core::rule::lookup("TGW100").expect("TGW100 is a known rule");
//! assert_eq!(meta.default_severity, wl_core::Severity::Warn);
//! ```

pub mod finding;
pub mod hash;
pub mod report;
pub mod rule;
pub mod severity;
pub mod span;
pub mod suppress;

pub use crate::{
    finding::Finding,
    report::{Diagnostic, Report},
    rule::RuleMeta,
    severity::{Confidence, Region, Severity},
    span::{LineIndex, Span},
    suppress::{Directive, DirectiveKind, Suppressions}
};

/// Schema version of the machine-readable report contract.
pub const SCHEMA_VERSION: u32 = 1;
