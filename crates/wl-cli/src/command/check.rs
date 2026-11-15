// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! `check` subcommand: run the full pipeline over files, directories or
//! stdin and render the report.

use std::{
    collections::BTreeSet,
    path::{Path, PathBuf}
};

use globset::{Glob, GlobSet, GlobSetBuilder};
use masterror::{AppError, AppResult};
use wl_check::{Checked, Config, check};
use wl_core::Severity;
use wl_profile::Profile;

use crate::{
    cli::{CheckArgs, ConfidenceArg, FailOn, Format, SeverityArg, TargetArg},
    output
};

/// Widget file extensions collected from directories.
const WIDGET_EXTENSIONS: [&str; 2] = ["html", "htm"];

/// One checked input: display name plus its source text.
pub struct Input {
    /// Path as given on the command line, `-` for stdin.
    pub path:   String,
    /// Document text.
    pub source: String
}

/// Checks every input and prints the report.
///
/// * `args` — parsed `check` arguments.
///
/// Returns `true` when no finding reaches the `--fail-on` threshold.
pub fn run(args: &CheckArgs) -> AppResult<bool> {
    let config = load_config(args)?;
    let profile = profile_for(config.profile)?;
    let inputs = collect_inputs(&args.paths, &config)?;
    let mut checked: Vec<Input> = Vec::new();
    let mut results: Vec<Checked> = Vec::new();
    let mut failures = 0_usize;
    for input in inputs {
        match check(&input.source, &profile, &config) {
            Ok(result) => {
                checked.push(input);
                results.push(result);
            }
            Err(error) => {
                let text = error
                    .message
                    .as_deref()
                    .unwrap_or_else(|| error.kind.label());
                eprintln!("widgetlint: {}: {text}", input.path);
                failures += 1;
            }
        }
    }
    if failures > 0 {
        return Err(AppError::bad_request(format!(
            "{failures} input(s) could not be checked"
        )));
    }
    let format = args.format.unwrap_or(Format::Human);
    match format {
        Format::Human => output::human::print_check(&checked, &results),
        Format::Json => output::json::print_check(&checked, &results, &profile, &config)?,
        Format::Sarif => output::sarif::print_check(&checked, &results)?,
        Format::Agent => {
            let since = load_since(args)?;
            let options = output::agent::RenderOptions {
                profile:      &profile,
                config:       &config,
                fail_on:      args.fail_on,
                max_findings: args
                    .max_findings
                    .unwrap_or(wl_check::agent::DEFAULT_MAX_FINDINGS),
                iteration:    args.iteration,
                since:        since.as_ref()
            };
            output::agent::print_check(&checked, &results, &options)?;
        }
    }
    Ok(!breach(&results, args.fail_on))
}

/// Reads `widgetlint.toml` when present: explicit path first, then the
/// working directory.
pub fn load_config_file(path: Option<&str>) -> AppResult<Config> {
    let path = match path {
        Some(path) => Some(PathBuf::from(path)),
        None if Path::new("widgetlint.toml").is_file() => Some(PathBuf::from("widgetlint.toml")),
        None => None
    };
    match &path {
        Some(path) => {
            let text = std::fs::read_to_string(path).map_err(|error| {
                AppError::not_found(format!("cannot read '{}': {error}", path.display()))
            })?;
            toml::from_str::<Config>(&text).map_err(|error| {
                AppError::bad_request(format!("invalid '{}': {error}", path.display()))
            })
        }
        None => Ok(Config::default())
    }
}

/// Loads the `--since` fingerprint file: one fingerprint per line.
fn load_since(args: &CheckArgs) -> AppResult<Option<BTreeSet<String>>> {
    let Some(path) = &args.since else {
        return Ok(None);
    };
    let text = std::fs::read_to_string(path)
        .map_err(|error| AppError::not_found(format!("cannot read '{path}': {error}")))?;
    Ok(Some(
        text.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect()
    ))
}

/// Builds the effective configuration: file first, command line on top.
fn load_config(args: &CheckArgs) -> AppResult<Config> {
    let mut config = load_config_file(args.config.as_deref())?;
    config.profile = args.profile;
    config.target = match args.target {
        TargetArg::Production => wl_check::config::Target::Production,
        TargetArg::Test => wl_check::config::Target::Test
    };
    if let Some(budget) = args.budget {
        config.size.budget = budget;
    }
    if let Some(margin) = args.margin {
        config.size.margin = margin;
    }
    if let Some(severity) = args.min_severity {
        config.output.min_severity = match severity {
            SeverityArg::Info => wl_check::config::SeverityFilter::Info,
            SeverityArg::Warn => wl_check::config::SeverityFilter::Warn,
            SeverityArg::Error => wl_check::config::SeverityFilter::Error
        };
    }
    if let Some(confidence) = args.min_confidence {
        config.output.min_confidence = match confidence {
            ConfidenceArg::Heuristic => wl_check::config::ConfidenceFilter::Heuristic,
            ConfidenceArg::Certain => wl_check::config::ConfidenceFilter::Certain
        };
    }
    Ok(config)
}

pub fn profile_for(number: u32) -> AppResult<Profile> {
    if number == 1 {
        wl_profile::profile_1()
    } else {
        Err(AppError::bad_request(format!(
            "profile {number} does not exist; only profile 1 is defined"
        )))
    }
}

/// Expands files, directories and `-` into readable inputs.
pub fn collect_inputs(paths: &[String], config: &Config) -> AppResult<Vec<Input>> {
    if paths.is_empty() {
        return Err(AppError::bad_request(
            "no input given; pass a file, a directory or -"
        ));
    }
    let include = matcher(&config.paths.include)?;
    let exclude = matcher(&config.paths.exclude)?;
    let mut out = Vec::new();
    for path in paths {
        if path == "-" {
            out.push(Input {
                path:   "-".to_owned(),
                source: read_stdin()?
            });
            continue;
        }
        let path_path = Path::new(path);
        if path_path.is_dir() {
            let mut files = Vec::new();
            collect_dir(path_path, &mut files)?;
            files.sort();
            for file in files {
                let name = file.display().to_string();
                if !selected(&name, include.as_ref(), exclude.as_ref()) {
                    continue;
                }
                out.push(read_file(&name)?);
            }
            continue;
        }
        out.push(read_file(path)?);
    }
    Ok(out)
}

fn collect_dir(dir: &Path, files: &mut Vec<PathBuf>) -> AppResult<()> {
    let entries = std::fs::read_dir(dir).map_err(|error| {
        AppError::not_found(format!("cannot read '{}': {error}", dir.display()))
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            AppError::not_found(format!("cannot read '{}': {error}", dir.display()))
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_dir(&path, files)?;
            continue;
        }
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| WIDGET_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        {
            files.push(path);
        }
    }
    Ok(())
}

fn selected(name: &str, include: Option<&GlobSet>, exclude: Option<&GlobSet>) -> bool {
    if exclude.is_some_and(|set| set.is_match(name)) {
        return false;
    }
    include.is_none_or(|set| set.is_match(name))
}

fn matcher(patterns: &[String]) -> AppResult<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern).map_err(|error| {
            AppError::bad_request(format!("invalid glob '{pattern}': {error}"))
        })?;
        builder.add(glob);
    }
    builder
        .build()
        .map(Some)
        .map_err(|error| AppError::bad_request(format!("invalid path filters: {error}")))
}

fn read_file(path: &str) -> AppResult<Input> {
    let data = std::fs::read(path)
        .map_err(|error| AppError::not_found(format!("cannot read '{path}': {error}")))?;
    let source = String::from_utf8(data)
        .map_err(|_| AppError::bad_request(format!("'{path}' is not valid UTF-8")))?;
    Ok(Input {
        path: path.to_owned(),
        source
    })
}

fn read_stdin() -> AppResult<String> {
    use std::io::Read;
    let mut data = Vec::new();
    std::io::stdin()
        .lock()
        .take(1024 * 1024 + 1)
        .read_to_end(&mut data)
        .map_err(|error| AppError::bad_request(format!("cannot read stdin: {error}")))?;
    String::from_utf8(data).map_err(|_| AppError::bad_request("stdin is not valid UTF-8"))
}

/// Whether any finding reaches the `--fail-on` threshold.
fn breach(results: &[Checked], fail_on: FailOn) -> bool {
    let floor = match fail_on {
        FailOn::Info => 0,
        FailOn::Warn => 1,
        FailOn::Error => 2
    };
    let rank = |severity: Severity| match severity {
        Severity::Info => 0,
        Severity::Warn => 1,
        Severity::Error => 2
    };
    results
        .iter()
        .flat_map(|result| &result.findings)
        .any(|finding| rank(finding.severity) >= floor)
}
