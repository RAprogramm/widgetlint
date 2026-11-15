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
    /// Report raw and zlib level 9 compressed size of a widget file
    Size(SizeArgs)
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

/// Output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// Human-readable report
    Human,
    /// Single-line JSON document
    Json
}
