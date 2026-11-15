// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Size findings: profile limits (TGW001–TGW005) evaluated against
//! measurements.

use wl_profile::limits::Limits;

use crate::info::SizeInfo;

/// Default soft budget for the compressed size (spec: "aim for 3800").
pub const DEFAULT_BUDGET: u32 = 3800;

/// Default near-limit window: zlib builds differ by tens of bytes near 4096.
pub const DEFAULT_MARGIN: u32 = 64;

/// Severity of a finding (the canonical model arrives with `wl-core`).
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

/// Size rule identifiers (spec section 8.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeRule {
    /// Raw HTML exceeds 32768 bytes.
    Tgw001,
    /// Compressed size exceeds 4096 bytes.
    Tgw002,
    /// Compressed size exceeds the soft budget.
    Tgw003,
    /// Compressed size is within the margin of the hard limit.
    Tgw005
}

impl SizeRule {
    /// Stable rule code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Tgw001 => "TGW001",
            Self::Tgw002 => "TGW002",
            Self::Tgw003 => "TGW003",
            Self::Tgw005 => "TGW005"
        }
    }
}

/// One size finding with its rule, severity and human-readable message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SizeFinding {
    /// Rule that fired.
    pub rule:     SizeRule,
    /// Severity of the rule.
    pub severity: Severity,
    /// Message describing the violation.
    pub message:  String
}

/// Evaluates measured sizes against profile limits and the soft budget.
///
/// * `info` — measured raw and compressed sizes.
/// * `limits` — numeric limits from the profile.
/// * `budget` — soft compressed-size budget (TGW003 threshold).
/// * `margin` — half-width of the near-limit window around the hard limit.
///
/// Returns findings sorted by descending severity; at most one of TGW002 and
/// TGW003 fires for the same measurement.
///
/// # Examples
///
/// ```
/// use wl_size::{SizeInfo, status};
/// let profile = wl_profile::profile_1().expect("valid");
/// let info = SizeInfo {
///     raw:   100,
///     zlib9: 50
/// };
/// assert!(status::evaluate(&info, &profile.limits, 3800, 64).is_empty());
/// ```
#[must_use]
pub fn evaluate(info: &SizeInfo, limits: &Limits, budget: u32, margin: u32) -> Vec<SizeFinding> {
    let raw = u32::try_from(info.raw).unwrap_or(u32::MAX);
    let zlib9 = u32::try_from(info.zlib9).unwrap_or(u32::MAX);
    let mut findings = Vec::new();
    if raw > limits.html_bytes {
        findings.push(finding(
            SizeRule::Tgw001,
            Severity::Error,
            format!(
                "raw size {raw} B exceeds {} B (over by {})",
                limits.html_bytes,
                raw - limits.html_bytes
            )
        ));
    }
    if zlib9 > limits.html_zlib9_bytes {
        findings.push(finding(
            SizeRule::Tgw002,
            Severity::Error,
            format!(
                "compressed size {zlib9} B exceeds {} B (over by {})",
                limits.html_zlib9_bytes,
                zlib9 - limits.html_zlib9_bytes
            )
        ));
    } else if zlib9 > budget {
        findings.push(finding(
            SizeRule::Tgw003,
            Severity::Warn,
            format!("compressed size {zlib9} B exceeds budget {budget} B")
        ));
    }
    if zlib9.abs_diff(limits.html_zlib9_bytes) <= margin {
        findings.push(finding(
            SizeRule::Tgw005,
            Severity::Info,
            format!(
                "compressed size {zlib9} B is within {margin} B of the hard limit; \
                 zlib builds differ by tens of bytes, trust the Python reference"
            )
        ));
    }
    findings.sort_by_key(|f| std::cmp::Reverse(f.severity));
    findings
}

const fn finding(rule: SizeRule, severity: Severity, message: String) -> SizeFinding {
    SizeFinding {
        rule,
        severity,
        message
    }
}
