// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! `widgetlint` command line entry point.
//!
//! Exit codes: `0` no findings, `1` findings at or above the threshold,
//! `2` usage or input error, `3` internal error.

// The CLI is the only place allowed to write to the terminal.
#![allow(clippy::print_stdout, clippy::print_stderr)]

mod cli;
mod command;
mod exit;
mod output;

use std::process::ExitCode;

fn main() -> ExitCode {
    exit::run()
}
