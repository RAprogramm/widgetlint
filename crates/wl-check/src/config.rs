// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! `widgetlint.toml` configuration model with defaults.
//!
//! The model is pure data: reading files is the caller's job so the same
//! structure serves the CLI, WASM and LSP front ends.
//!
//! # Examples
//!
//! ```
//! let config = wl_check::Config::default();
//! assert_eq!(config.profile, 1);
//! assert_eq!(config.size.budget, wl_size::DEFAULT_BUDGET);
//! ```

use std::collections::BTreeMap;

use serde::Deserialize;
use wl_core::{Confidence, Severity};

/// Where the widget is meant to run; feature-gated rules use it later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    /// Production clients: private chats only in v1.
    #[default]
    Production,
    /// Test server: groups, inline mode and forwards.
    Test
}

/// What to do with a rule: disable it or force a severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    /// Drop every finding of the rule.
    Off,
    /// Report as error.
    Error,
    /// Report as warning.
    Warn,
    /// Report as info.
    Info
}

impl RuleAction {
    /// The severity to force, when the action keeps the rule on.
    #[must_use]
    pub const fn severity(self) -> Option<Severity> {
        match self {
            Self::Off => None,
            Self::Error => Some(Severity::Error),
            Self::Warn => Some(Severity::Warn),
            Self::Info => Some(Severity::Info)
        }
    }
}

/// Size thresholds of the `size` section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct SizeConfig {
    /// Soft compressed budget, the TGW003 threshold.
    pub budget: u32,
    /// Half-width of the near-limit window around the hard limit (TGW005).
    pub margin: u32
}

impl Default for SizeConfig {
    fn default() -> Self {
        Self {
            budget: wl_size::DEFAULT_BUDGET,
            margin: wl_size::DEFAULT_MARGIN
        }
    }
}

/// Reporting thresholds of the `output` section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct OutputConfig {
    /// Drop findings below this severity.
    pub min_severity:   SeverityFilter,
    /// Drop rules below this confidence.
    pub min_confidence: ConfidenceFilter
}

/// Severity threshold usable in the config.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeverityFilter {
    /// Keep everything.
    #[default]
    Info,
    /// Keep warnings and errors.
    Warn,
    /// Keep errors only.
    Error
}

impl SeverityFilter {
    /// Whether a severity passes the threshold.
    #[must_use]
    pub const fn allows(self, severity: Severity) -> bool {
        let rank = match severity {
            Severity::Info => 0,
            Severity::Warn => 1,
            Severity::Error => 2
        };
        rank >= self.rank()
    }

    const fn rank(self) -> u8 {
        match self {
            Self::Info => 0,
            Self::Warn => 1,
            Self::Error => 2
        }
    }
}

/// Confidence threshold usable in the config.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceFilter {
    /// Keep every rule.
    #[default]
    Heuristic,
    /// Keep only spec-guaranteed rules.
    Certain
}

impl ConfidenceFilter {
    /// Whether a confidence passes the threshold.
    #[must_use]
    pub const fn allows(self, confidence: Confidence) -> bool {
        match self {
            Self::Heuristic => true,
            Self::Certain => matches!(confidence, Confidence::Certain)
        }
    }
}

/// Path filters of the `paths` section; patterns are globs evaluated by the
/// front end against directory inputs.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct PathsConfig {
    /// Files to check; empty means every widget file found.
    pub include: Vec<String>,
    /// Files to skip even when included.
    pub exclude: Vec<String>
}

/// Full configuration: `widgetlint.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    /// Profile number to check against; only `1` exists in v1.
    pub profile: u32,
    /// Deployment target.
    pub target:  Target,
    /// Size thresholds.
    pub size:    SizeConfig,
    /// Reporting thresholds.
    pub output:  OutputConfig,
    /// Per-rule overrides keyed by rule id.
    pub rules:   BTreeMap<String, RuleAction>,
    /// Path filters for directory inputs.
    pub paths:   PathsConfig
}

impl Default for Config {
    fn default() -> Self {
        Self {
            profile: 1,
            target:  Target::default(),
            size:    SizeConfig::default(),
            output:  OutputConfig::default(),
            rules:   BTreeMap::new(),
            paths:   PathsConfig::default()
        }
    }
}
