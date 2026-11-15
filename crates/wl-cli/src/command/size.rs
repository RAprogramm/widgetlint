// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! `size` subcommand: measure a widget and report against the profile limits.

use std::io::Read;

use masterror::{AppError, AppResult};
use wl_size::{SizeInfo, evaluate};

use crate::{
    cli::{Format, SizeArgs},
    output
};

/// Upper bound on accepted input: a widget is at most 32768 B, anything
/// beyond 1 MiB is a wrong file, not a widget.
const MAX_INPUT_BYTES: u64 = 1024 * 1024;

/// Measures the input and prints the report.
///
/// * `args` — parsed `size` arguments.
///
/// Returns `true` when no findings were produced.
pub fn run(args: &SizeArgs) -> AppResult<bool> {
    let data = read_input(&args.file)?;
    let info = SizeInfo::measure(&data)?;
    let limits = wl_profile::profile_1()?.limits;
    let findings = evaluate(&info, &limits, args.budget, args.margin);
    match args.format {
        Format::Human => {
            output::human::print_size(&args.file, &info, &limits, args.budget, &findings);
        }
        Format::Json => {
            output::json::print_size(&args.file, &info, &limits, args.budget, &findings)?;
        }
    }
    Ok(findings.is_empty())
}

fn read_input(path: &str) -> AppResult<Vec<u8>> {
    let data = if path == "-" {
        let mut data = Vec::new();
        std::io::stdin()
            .lock()
            .take(MAX_INPUT_BYTES + 1)
            .read_to_end(&mut data)
            .map_err(|error| AppError::bad_request(format!("cannot read stdin: {error}")))?;
        data
    } else {
        let meta = std::fs::metadata(path)
            .map_err(|error| AppError::not_found(format!("cannot read '{path}': {error}")))?;
        if meta.len() > MAX_INPUT_BYTES {
            return Err(AppError::bad_request(format!(
                "'{path}' is {} B, the input limit is {MAX_INPUT_BYTES} B",
                meta.len()
            )));
        }
        std::fs::read(path)
            .map_err(|error| AppError::not_found(format!("cannot read '{path}': {error}")))?
    };
    if data.len() as u64 > MAX_INPUT_BYTES {
        return Err(AppError::bad_request(format!(
            "input exceeds the {MAX_INPUT_BYTES} B limit"
        )));
    }
    Ok(data)
}
