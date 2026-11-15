// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Exit code mapping: parse arguments, dispatch, translate failures.

use std::process::ExitCode;

use clap::Parser;
use masterror::AppError;

use crate::{
    cli::{Cli, Command},
    command
};

/// Runs the CLI and returns the process exit code.
///
/// `0` no findings, `1` findings present, `2` usage or input error,
/// `3` internal error.
pub fn run() -> ExitCode {
    let cli = Cli::parse();
    let outcome = match cli.command {
        Command::Check(args) => command::check::run(&args),
        Command::Size(args) => command::size::run(&args),
        Command::Explain(args) => command::explain::run(&args),
        Command::Rules(args) => command::rules::run(&args),
        Command::Verify(args) => command::verify::run(&args),
        Command::Schema(args) => command::schema::run(&args),
        Command::Version(args) => Ok(command::version::run(&args))
    };
    match outcome {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => report(&error)
    }
}

fn report(error: &AppError) -> ExitCode {
    let text = error
        .message
        .as_deref()
        .unwrap_or_else(|| error.kind.label());
    eprintln!("widgetlint: {text}");
    if error.kind.http_status() >= 500 {
        ExitCode::from(3)
    } else {
        ExitCode::from(2)
    }
}
