// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Tool parameters of the Model Context Protocol server.
//!
//! Every parameter type carries a JSON Schema so that MCP clients render a
//! correct input form, and every enumeration mirrors the vocabulary of the
//! command line interface.

use schemars::JsonSchema;
use serde::Deserialize;
use wl_check::config::{ConfidenceFilter, SeverityFilter, Target};
use wl_core::{Confidence, Severity};

/// Severity threshold accepted by the tools.
///
/// * `info` keeps every finding.
/// * `warn` keeps warnings and errors.
/// * `error` keeps errors only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SeverityArg {
    /// Keep every finding.
    #[default]
    Info,
    /// Keep warnings and errors.
    Warn,
    /// Keep errors only.
    Error
}

impl From<SeverityArg> for SeverityFilter {
    fn from(value: SeverityArg) -> Self {
        match value {
            SeverityArg::Info => Self::Info,
            SeverityArg::Warn => Self::Warn,
            SeverityArg::Error => Self::Error
        }
    }
}

impl From<SeverityArg> for Severity {
    fn from(value: SeverityArg) -> Self {
        match value {
            SeverityArg::Info => Self::Info,
            SeverityArg::Warn => Self::Warn,
            SeverityArg::Error => Self::Error
        }
    }
}

/// Confidence threshold accepted by the tools.
///
/// * `heuristic` keeps every rule.
/// * `certain` keeps only the rules the specification guarantees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceArg {
    /// Keep every rule.
    #[default]
    Heuristic,
    /// Keep only spec-guaranteed rules.
    Certain
}

impl From<ConfidenceArg> for ConfidenceFilter {
    fn from(value: ConfidenceArg) -> Self {
        match value {
            ConfidenceArg::Heuristic => Self::Heuristic,
            ConfidenceArg::Certain => Self::Certain
        }
    }
}

impl From<ConfidenceArg> for Confidence {
    fn from(value: ConfidenceArg) -> Self {
        match value {
            ConfidenceArg::Heuristic => Self::Heuristic,
            ConfidenceArg::Certain => Self::Certain
        }
    }
}

/// Deployment target of the widget.
///
/// * `production` covers private chats.
/// * `test` covers groups, inline mode and forwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TargetArg {
    /// Production clients.
    #[default]
    Production,
    /// Test server.
    Test
}

impl From<TargetArg> for Target {
    fn from(value: TargetArg) -> Self {
        match value {
            TargetArg::Production => Self::Production,
            TargetArg::Test => Self::Test
        }
    }
}

/// Parameters of the `check` tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CheckParams {
    /// Widget HTML source to check.
    pub source:         String,
    /// Label of the input, echoed in the report.
    #[serde(default)]
    pub path:           Option<String>,
    /// Soft compressed-size budget in bytes; the TGW003 threshold.
    #[serde(default)]
    pub budget:         Option<u32>,
    /// Deployment target of the widget.
    #[serde(default)]
    pub target:         Option<TargetArg>,
    /// Drop findings below this severity.
    #[serde(default)]
    pub min_severity:   Option<SeverityArg>,
    /// Drop rules below this confidence.
    #[serde(default)]
    pub min_confidence: Option<ConfidenceArg>,
    /// Severity at or above which the verdict becomes `fail`.
    #[serde(default)]
    pub fail_on:        Option<SeverityArg>
}

/// Parameters of the `verify` tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct VerifyParams {
    /// Widget HTML source to check.
    pub source:  String,
    /// Label of the input, echoed in the report.
    #[serde(default)]
    pub path:    Option<String>,
    /// Severity at or above which the verdict becomes `fail`.
    #[serde(default)]
    pub fail_on: Option<SeverityArg>
}

/// Parameters of the `explain` tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ExplainParams {
    /// Rule id such as `TGW100`.
    pub rule_id: String
}

/// Parameters of the `rules` tool.
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub struct RulesParams {
    /// Keep only the rules of this severity.
    #[serde(default)]
    pub severity: Option<SeverityArg>
}
