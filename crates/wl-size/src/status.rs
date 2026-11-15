// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Size findings: profile limits (TGW001–TGW005) evaluated against
//! measurements. Findings use the canonical [`Finding`] model from `wl-core`,
//! with severities taken from the embedded rule registry.

use wl_core::{Finding, rule::lookup};
use wl_profile::limits::Limits;

use crate::info::SizeInfo;

/// Default soft budget for the compressed size (spec: "aim for 3800").
pub const DEFAULT_BUDGET: u32 = 3800;

/// Default near-limit window: zlib builds differ by tens of bytes near 4096.
pub const DEFAULT_MARGIN: u32 = 64;

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
pub fn evaluate(info: &SizeInfo, limits: &Limits, budget: u32, margin: u32) -> Vec<Finding> {
    let raw = u32::try_from(info.raw).unwrap_or(u32::MAX);
    let zlib9 = u32::try_from(info.zlib9).unwrap_or(u32::MAX);
    let mut findings = Vec::new();
    if raw > limits.html_bytes {
        findings.push(finding(
            SizeRule::Tgw001,
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
            format!(
                "compressed size {zlib9} B exceeds {} B (over by {})",
                limits.html_zlib9_bytes,
                zlib9 - limits.html_zlib9_bytes
            )
        ));
    } else if zlib9 > budget {
        findings.push(finding(
            SizeRule::Tgw003,
            format!("compressed size {zlib9} B exceeds budget {budget} B")
        ));
    }
    if zlib9.abs_diff(limits.html_zlib9_bytes) <= margin {
        findings.push(finding(
            SizeRule::Tgw005,
            format!(
                "compressed size {zlib9} B is within {margin} B of the hard limit; \
                 zlib builds differ by tens of bytes, trust the Python reference"
            )
        ));
    }
    findings.sort_by_key(|f| std::cmp::Reverse(f.severity));
    findings
}

fn finding(rule: SizeRule, message: String) -> Finding {
    let meta = lookup(rule.code());
    Finding::new(
        meta.unwrap_or_else(|| unreachable!("size rules are embedded in the registry")),
        message
    )
}
