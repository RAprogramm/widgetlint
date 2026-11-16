// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! End-to-end CLI tests: every command, format, exit code and the config file.

use std::{
    path::PathBuf,
    process::{Command, Output}
};

use masterror::{AppError, AppResult};

fn widgetlint(args: &[&str]) -> AppResult<Output> {
    Command::new(env!("CARGO_BIN_EXE_widgetlint"))
        .args(args)
        .output()
        .map_err(|error| AppError::internal(format!("spawn: {error}")))
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn temp_widget(name: &str, source: &str) -> AppResult<PathBuf> {
    let dir = std::env::temp_dir().join(format!("wl-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir)
        .map_err(|error| AppError::internal(format!("temp dir: {error}")))?;
    let path = dir.join(name);
    std::fs::write(&path, source)
        .map_err(|error| AppError::internal(format!("write: {error}")))?;
    Ok(path)
}

fn check_file(name: &str, source: &str, extra: &[&str]) -> AppResult<Output> {
    let file = temp_widget(name, source)?;
    let path = file.display().to_string();
    let mut args = vec!["check", path.as_str()];
    args.extend_from_slice(extra);
    widgetlint(&args)
}

#[test]
fn check_clean_widget_exits_zero() -> AppResult<()> {
    let output = check_file("clean.html", "<div>hello</div>", &[])?;
    assert!(output.status.success(), "{}", stdout(&output));
    assert!(stdout(&output).contains("no findings"));
    Ok(())
}

#[test]
fn check_reports_findings_and_exits_one() -> AppResult<()> {
    let output = check_file("dirty.html", "<input>", &["--fail-on", "warn"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).contains("TGW100"));
    Ok(())
}

#[test]
fn check_json_is_a_valid_array() -> AppResult<()> {
    let output = check_file("json.html", "<input>", &["--format", "json"])?;
    let text = stdout(&output);
    let document: serde_json::Value = serde_json::from_str(text.trim()).expect("valid json");
    let reports = document.as_array().expect("array of reports");
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0]["summary"]["warn"].as_u64(), Some(1));
    Ok(())
}

#[test]
fn check_sarif_has_the_expected_shape() -> AppResult<()> {
    let output = check_file("sarif.html", "<input>", &["--format", "sarif"])?;
    let text = stdout(&output);
    let document: serde_json::Value = serde_json::from_str(text.trim()).expect("valid sarif");
    assert_eq!(document["version"].as_str(), Some("2.1.0"));
    let runs = document["runs"].as_array().expect("runs");
    assert_eq!(
        runs[0]["tool"]["driver"]["name"].as_str(),
        Some("widgetlint")
    );
    assert_ne!(runs[0]["results"].as_array().expect("results").len(), 0);
    Ok(())
}

#[test]
fn check_reads_stdin() -> AppResult<()> {
    use std::io::Write;
    let mut child = Command::new(env!("CARGO_BIN_EXE_widgetlint"))
        .args(["check", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| AppError::internal(format!("spawn: {error}")))?;
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| AppError::internal("no stdin"))?;
        stdin
            .write_all(b"<marquee>x</marquee>")
            .map_err(|error| AppError::internal(format!("write stdin: {error}")))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|error| AppError::internal(format!("wait: {error}")))?;
    assert!(stdout(&output).contains("TGW110"));
    Ok(())
}

#[test]
fn check_missing_file_exits_two() -> AppResult<()> {
    let output = widgetlint(&["check", "/nonexistent/widget.html"])?;
    assert_eq!(output.status.code(), Some(2));
    Ok(())
}

#[test]
fn check_config_turns_rules_off_and_overrides_severity() -> AppResult<()> {
    let config = temp_widget(
        "widgetlint.toml",
        "[rules]\nTGW100 = \"off\"\nTGW111 = \"error\"\n"
    )?;
    let output = check_file(
        "cfg.html",
        "<input>",
        &["--config", &config.display().to_string()]
    )?;
    assert!(output.status.success(), "{}", stdout(&output));
    assert!(!stdout(&output).contains("TGW100"));
    Ok(())
}

#[test]
fn check_fail_on_warn_promotes_warnings() -> AppResult<()> {
    let clean = check_file("warn.html", "<marquee>x</marquee>", &[])?;
    assert!(clean.status.success(), "info alone must pass");
    let strict = check_file("warn.html", "<marquee>x</marquee>", &["--fail-on", "info"])?;
    assert_eq!(strict.status.code(), Some(1));
    Ok(())
}

#[test]
fn explain_prints_rule_documentation() -> AppResult<()> {
    let known = widgetlint(&["explain", "TGW100"])?;
    assert!(known.status.success());
    let text = stdout(&known);
    assert!(text.contains("TGW100"));
    assert!(text.contains("spec:"));
    let unknown = widgetlint(&["explain", "TGW999"])?;
    assert_eq!(unknown.status.code(), Some(2));
    Ok(())
}

#[test]
fn rules_lists_every_rule() -> AppResult<()> {
    let human = widgetlint(&["rules"])?;
    assert!(human.status.success());
    assert!(stdout(&human).contains("TGW100"));
    let json = widgetlint(&["rules", "--format", "json"])?;
    let text = stdout(&json);
    let document: serde_json::Value = serde_json::from_str(text.trim()).expect("valid json");
    assert!(document.as_array().expect("array").len() > 30);
    Ok(())
}

#[test]
fn size_still_works() -> AppResult<()> {
    let file = temp_widget("size.html", "<div>hello</div>")?;
    let output = widgetlint(&["size", &file.display().to_string()])?;
    assert!(output.status.success(), "{}", stdout(&output));
    assert!(stdout(&output).contains("zlib-9"));
    Ok(())
}

#[test]
fn agent_format_is_a_single_compact_object() -> AppResult<()> {
    let output = check_file(
        "agent.html",
        "<input>",
        &["--format", "agent", "--fail-on", "warn"]
    )?;
    assert_eq!(output.status.code(), Some(1));
    let text = stdout(&output);
    assert!(
        !text.contains('\u{1b}'),
        "agent output must be free of ANSI escapes"
    );
    let report: serde_json::Value = serde_json::from_str(text.trim())
        .map_err(|error| AppError::internal(format!("agent json: {error}")))?;
    assert_eq!(report["verdict"], "fail");
    assert_eq!(report["findings"][0]["id"], "TGW100");
    assert_eq!(report["findings"][0]["fp"].as_str().unwrap_or("").len(), 8);
    assert!(report["caveats"].is_array());
    assert!(
        report["tool"]["ruleset"]
            .as_str()
            .is_some_and(|s| s.starts_with("fnv1a64:"))
    );
    Ok(())
}

#[test]
fn agent_format_is_an_array_for_several_inputs() -> AppResult<()> {
    let first = temp_widget("agent-a.html", "<input>")?;
    let second = temp_widget("agent-b.html", "<b>ok</b>")?;
    let output = widgetlint(&[
        "check",
        &first.display().to_string(),
        &second.display().to_string(),
        "--format",
        "agent"
    ])?;
    let reports: Vec<serde_json::Value> = serde_json::from_str(stdout(&output).trim())
        .map_err(|error| AppError::internal(format!("agent json: {error}")))?;
    assert_eq!(reports.len(), 2);
    assert_eq!(reports[1]["verdict"], "pass");
    Ok(())
}

#[test]
fn agent_since_reports_resolved_fingerprints() -> AppResult<()> {
    let since = temp_widget("fps.txt", "deadbeef\n")?;
    let output = check_file(
        "agent-since.html",
        "<input>",
        &[
            "--format",
            "agent",
            "--since",
            &since.display().to_string(),
            "--fail-on",
            "warn"
        ]
    )?;
    let report: serde_json::Value = serde_json::from_str(stdout(&output).trim())
        .map_err(|error| AppError::internal(format!("agent json: {error}")))?;
    assert_eq!(report["resolved"], serde_json::json!(["deadbeef"]));
    Ok(())
}

#[test]
fn agent_max_findings_truncates() -> AppResult<()> {
    let output = check_file(
        "agent-cut.html",
        "<input>\n<style>color: red; transform: preserve-3d; position: sticky</style>",
        &[
            "--format",
            "agent",
            "--max-findings",
            "1",
            "--fail-on",
            "warn"
        ]
    )?;
    let report: serde_json::Value = serde_json::from_str(stdout(&output).trim())
        .map_err(|error| AppError::internal(format!("agent json: {error}")))?;
    assert_eq!(report["findings"].as_array().map(Vec::len), Some(1));
    assert!(report["truncated"].as_u64().unwrap_or(0) > 0);
    Ok(())
}

#[test]
fn verify_prints_only_the_verdict() -> AppResult<()> {
    let file = temp_widget("verify.html", "<input>")?;
    let output = widgetlint(&["verify", &file.display().to_string(), "--fail-on", "warn"])?;
    assert_eq!(output.status.code(), Some(1));
    let verdict: serde_json::Value = serde_json::from_str(stdout(&output).trim())
        .map_err(|error| AppError::internal(format!("verify json: {error}")))?;
    assert_eq!(verdict["verdict"], "fail");
    assert!(
        verdict.get("findings").is_none(),
        "verify must stay minimal"
    );
    assert!(verdict["next_action"].is_string());
    let clean = temp_widget("verify-clean.html", "<b>ok</b>")?;
    let output = widgetlint(&["verify", &clean.display().to_string()])?;
    assert!(output.status.success());
    Ok(())
}

#[test]
fn version_reports_identity_in_both_formats() -> AppResult<()> {
    let version = env!("CARGO_PKG_VERSION");
    let output = widgetlint(&["version"])?;
    assert!(output.status.success());
    assert!(stdout(&output).contains(&format!("widgetlint {version}")));
    let output = widgetlint(&["version", "--json"])?;
    let value: serde_json::Value = serde_json::from_str(stdout(&output).trim())
        .map_err(|error| AppError::internal(format!("version json: {error}")))?;
    assert_eq!(value["name"], "widgetlint");
    assert_eq!(value["version"], version);
    assert_eq!(value["schema_version"], 1);
    assert!(value["rules"].as_u64().unwrap_or(0) > 0);
    Ok(())
}

#[test]
fn schema_publishes_the_agent_contract() -> AppResult<()> {
    let output = widgetlint(&["schema"])?;
    assert!(output.status.success());
    let schema: serde_json::Value = serde_json::from_str(&stdout(&output))
        .map_err(|error| AppError::internal(format!("schema json: {error}")))?;
    assert!(schema.pointer("/properties/next_action").is_some());
    Ok(())
}
