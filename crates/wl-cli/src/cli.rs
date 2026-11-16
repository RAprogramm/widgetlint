// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Command line surface: argument parsing only, no behavior.

use clap::{Args, Parser, Subcommand, ValueEnum};
use wl_size::{DEFAULT_BUDGET, DEFAULT_MARGIN};

/// Root parser: `widgetlint <command>`.
#[derive(Debug, Parser)]
#[command(
    name = "widgetlint",
    version,
    about = "Validator, linter and optimizer for Telegram Bot API 10.4 widgets",
    propagate_version = true
)]
pub struct Cli {
    /// Selected subcommand.
    #[command(subcommand)]
    pub command: Command
}

/// Available subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Check widget files against the profile
    Check(CheckArgs),
    /// Print only the pass/fail verdict and the next action
    Verify(VerifyArgs),
    /// Report raw and zlib level 9 compressed size of a widget file
    Size(SizeArgs),
    /// Print the documentation of one rule
    Explain(ExplainArgs),
    /// List every known rule
    Rules(RulesArgs),
    /// Print the JSON Schema of the agent report
    Schema(SchemaArgs),
    /// Serve the linter over the Model Context Protocol on stdio
    Mcp(McpArgs),
    /// Print tool, schema and ruleset version
    Version(VersionArgs)
}

/// Arguments of the `mcp` subcommand.
#[derive(Debug, Args)]
pub struct McpArgs {}

/// Arguments of the `check` subcommand.
#[derive(Debug, Args)]
pub struct CheckArgs {
    /// Widget files, directories, or `-` to read from stdin
    pub paths:          Vec<String>,
    /// Profile number to check against
    #[arg(long, default_value_t = 1)]
    pub profile:        u32,
    /// Deployment target of the widget
    #[arg(long, value_enum, default_value_t = TargetArg::Production)]
    pub target:         TargetArg,
    /// Output format, overrides the config file
    #[arg(long, value_enum)]
    pub format:         Option<Format>,
    /// Drop findings below this severity, overrides the config file
    #[arg(long, value_enum)]
    pub min_severity:   Option<SeverityArg>,
    /// Drop rules below this confidence, overrides the config file
    #[arg(long, value_enum)]
    pub min_confidence: Option<ConfidenceArg>,
    /// Exit with `1` when a finding at or above this severity remains
    #[arg(long, value_enum, default_value_t = FailOn::Error)]
    pub fail_on:        FailOn,
    /// Path to `widgetlint.toml`, discovered in the working directory by
    /// default
    #[arg(long)]
    pub config:         Option<String>,
    /// Soft compressed-size budget, the TGW003 threshold
    #[arg(long)]
    pub budget:         Option<u32>,
    /// Half-width of the near-limit window around the hard limit (TGW005)
    #[arg(long)]
    pub margin:         Option<u32>,
    /// Maximum grouped findings kept in `--format agent` reports
    #[arg(long)]
    pub max_findings:   Option<usize>,
    /// File of fingerprints from the previous run; report only new and
    /// resolved findings
    #[arg(long)]
    pub since:          Option<String>,
    /// Fix-loop iteration echoed back in agent reports
    #[arg(long, default_value_t = 1)]
    pub iteration:      u32
}

/// Arguments of the `size` subcommand.
#[derive(Debug, Args)]
pub struct SizeArgs {
    /// Path to the widget HTML file, or `-` to read from stdin
    pub file:   String,
    /// Soft compressed-size budget, the TGW003 threshold
    #[arg(long, default_value_t = DEFAULT_BUDGET)]
    pub budget: u32,
    /// Half-width of the near-limit window around the hard limit (TGW005)
    #[arg(long, default_value_t = DEFAULT_MARGIN)]
    pub margin: u32,
    /// Output format
    #[arg(long, value_enum, default_value_t = Format::Human)]
    pub format: Format
}

/// Arguments of the `explain` subcommand.
#[derive(Debug, Args)]
pub struct ExplainArgs {
    /// Rule id such as `TGW100`
    pub id: String
}

/// Arguments of the `rules` subcommand.
#[derive(Debug, Args)]
pub struct RulesArgs {
    /// Output format
    #[arg(long, value_enum, default_value_t = Format::Human)]
    pub format: Format
}

/// Arguments of the `verify` subcommand.
#[derive(Debug, Args)]
pub struct VerifyArgs {
    /// Widget files, directories, or `-` to read from stdin
    pub paths:   Vec<String>,
    /// Exit with `1` when a finding at or above this severity remains
    #[arg(long, value_enum, default_value_t = FailOn::Error)]
    pub fail_on: FailOn,
    /// Path to `widgetlint.toml`, discovered in the working directory by
    /// default
    #[arg(long)]
    pub config:  Option<String>
}

/// Arguments of the `schema` subcommand.
#[derive(Debug, Args)]
pub struct SchemaArgs {}

/// Arguments of the `version` subcommand.
#[derive(Debug, Args)]
pub struct VersionArgs {
    /// Emit a JSON object instead of one text line
    #[arg(long)]
    pub json: bool
}

/// Output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// Human-readable report
    Human,
    /// Single-line JSON document
    Json,
    /// SARIF 2.1.0 document for GitHub code scanning
    Sarif,
    /// Compact single-line JSON report for AI agents
    Agent
}

/// Deployment target accepted on the command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TargetArg {
    /// Production clients: private chats only in v1
    Production,
    /// Test server: groups, inline mode and forwards
    Test
}

/// Severity threshold accepted on the command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SeverityArg {
    /// Keep everything
    Info,
    /// Keep warnings and errors
    Warn,
    /// Keep errors only
    Error
}

/// Confidence threshold accepted on the command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ConfidenceArg {
    /// Keep every rule
    Heuristic,
    /// Keep only spec-guaranteed rules
    Certain
}

/// Severity that makes `check` exit with `1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum FailOn {
    /// Fail on any finding
    Info,
    /// Fail on warnings and errors
    Warn,
    /// Fail on errors only
    Error
}
