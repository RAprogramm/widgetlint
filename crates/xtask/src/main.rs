// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Repository build tasks.
//!
//! Run `cargo run -p xtask -- gen-docs` from anywhere inside the workspace to
//! regenerate the rule documentation, `llms.txt` and `llms-full.txt` from the
//! rule registry. Run `cargo run -p xtask -- gen-site` to build the browsable
//! rule catalog published next to the playground. Generated files are never
//! edited by hand.

// The task runner is the only place allowed to write to the terminal.
#![allow(clippy::print_stdout, clippy::print_stderr)]

mod gen_docs;
mod gen_site;

use std::process::ExitCode;

use masterror::AppResult;

fn main() -> ExitCode {
    let task = std::env::args().nth(1).unwrap_or_default();
    match dispatch(&task) {
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

/// Runs one task by name.
///
/// * `task` — `gen-docs`, `check-docs`, `gen-site` or an unknown name.
///
/// # Errors
///
/// [`masterror::AppError`] with `BadRequest` taxonomy for unknown tasks, or the
/// task's own error.
fn dispatch(task: &str) -> AppResult<()> {
    match task {
        "gen-docs" => gen_docs::run(false),
        "check-docs" => gen_docs::run(true),
        "gen-site" => gen_site::run(),
        other => Err(masterror::AppError::bad_request(format!(
            "unknown task '{other}'; available: gen-docs, check-docs, gen-site"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_task_is_rejected() {
        let error = dispatch("nope").expect_err("unknown task must be rejected");
        assert!(error.to_string().contains("nope"), "message: {error}");
    }

    #[test]
    fn empty_task_is_rejected() {
        assert!(dispatch("").is_err(), "missing task name must be rejected");
    }

    #[test]
    fn check_docs_succeeds_on_a_fresh_tree() {
        dispatch("check-docs").expect("committed docs must be fresh");
    }
}
