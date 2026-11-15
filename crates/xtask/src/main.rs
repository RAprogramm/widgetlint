// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Repository build tasks.
//!
//! Run `cargo run -p xtask -- gen-docs` from anywhere inside the workspace to
//! regenerate the rule documentation, `llms.txt` and `llms-full.txt` from the
//! rule registry. Generated files are never edited by hand.

// The task runner is the only place allowed to write to the terminal.
#![allow(clippy::print_stdout, clippy::print_stderr)]

mod gen_docs;

use std::process::ExitCode;

use masterror::AppResult;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let text = error
                .message
                .as_deref()
                .unwrap_or_else(|| error.kind.label());
            eprintln!("xtask: {text}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> AppResult<()> {
    let task = std::env::args().nth(1).unwrap_or_default();
    match task.as_str() {
        "gen-docs" => gen_docs::run(false),
        "check-docs" => gen_docs::run(true),
        other => Err(masterror::AppError::bad_request(format!(
            "unknown task '{other}'; available: gen-docs, check-docs"
        )))
    }
}
