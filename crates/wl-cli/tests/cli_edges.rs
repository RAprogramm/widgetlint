// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! End-to-end coverage of the CLI edges: flags, directory walking, glob
//! filters, unreadable inputs and the `size` command formats.

use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Output, Stdio}
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

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn temp_dir(name: &str) -> AppResult<PathBuf> {
    let dir = std::env::temp_dir().join(format!("wl-cli-{}", std::process::id()));
    let dir = dir.join(name);
    std::fs::create_dir_all(&dir)
        .map_err(|error| AppError::internal(format!("temp dir: {error}")))?;
    Ok(dir)
}

fn temp_widget(name: &str, source: &str) -> AppResult<PathBuf> {
    let path = temp_dir("files")?.join(name);
    std::fs::write(&path, source)
        .map_err(|error| AppError::internal(format!("write: {error}")))?;
    Ok(path)
}

fn temp_bytes(name: &str, data: &[u8]) -> AppResult<PathBuf> {
    let path = temp_dir("files")?.join(name);
    std::fs::write(&path, data).map_err(|error| AppError::internal(format!("write: {error}")))?;
    Ok(path)
}

fn path_of(file: &std::path::Path) -> String {
    file.display().to_string()
}

fn run_stdin(args: &[&str], input: &[u8]) -> AppResult<Output> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_widgetlint"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| AppError::internal(format!("spawn: {error}")))?;
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| AppError::internal("no stdin"))?;
        stdin
            .write_all(input)
            .map_err(|error| AppError::internal(format!("write stdin: {error}")))?;
    }
    child
        .wait_with_output()
        .map_err(|error| AppError::internal(format!("wait: {error}")))
}

#[test]
fn check_target_test_accepts_a_clean_widget() -> AppResult<()> {
    let file = temp_widget("edge-target.html", "<div>hello</div>")?;
    let output = widgetlint(&["check", &path_of(&file), "--target", "test"])?;
    assert!(output.status.success(), "{}", stdout(&output));
    Ok(())
}

#[test]
fn check_budget_flag_lowers_the_soft_limit() -> AppResult<()> {
    let file = temp_widget("edge-budget.html", "<div>hello</div>")?;
    let output = widgetlint(&[
        "check",
        &path_of(&file),
        "--budget",
        "1",
        "--margin",
        "0",
        "--fail-on",
        "info"
    ])?;
    assert_eq!(output.status.code(), Some(1), "budget 1 B must be exceeded");
    assert!(stdout(&output).contains("TGW003"), "{}", stdout(&output));
    Ok(())
}

#[test]
fn check_min_severity_filters_quieter_findings() -> AppResult<()> {
    let file = temp_widget("edge-sev.html", "<marquee>x</marquee>")?;
    let quiet = widgetlint(&[
        "check",
        &path_of(&file),
        "--min-severity",
        "warn",
        "--fail-on",
        "info"
    ])?;
    assert!(
        quiet.status.success(),
        "the info finding must be filtered: {}",
        stdout(&quiet)
    );
    let strict = widgetlint(&["check", &path_of(&file), "--min-severity", "error"])?;
    assert!(strict.status.success(), "nothing is an error here");
    Ok(())
}

#[test]
fn check_min_confidence_certain_keeps_certain_findings() -> AppResult<()> {
    let file = temp_widget("edge-conf.html", "<input>")?;
    let output = widgetlint(&[
        "check",
        &path_of(&file),
        "--min-confidence",
        "certain",
        "--fail-on",
        "warn"
    ])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).contains("TGW100"));
    let loose = widgetlint(&["check", &path_of(&file), "--min-confidence", "heuristic"])?;
    assert!(loose.status.success() || loose.status.code() == Some(1));
    Ok(())
}

#[test]
fn check_unknown_profile_is_rejected() -> AppResult<()> {
    let file = temp_widget("edge-profile.html", "<div>x</div>")?;
    let output = widgetlint(&["check", &path_of(&file), "--profile", "2"])?;
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("profile 2 does not exist"),
        "{}",
        stderr(&output)
    );
    Ok(())
}

#[test]
fn check_without_input_is_rejected() -> AppResult<()> {
    let output = widgetlint(&["check"])?;
    assert!(!output.status.success());
    let text = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        text.contains("no input given"),
        "expected the no-input hint, got: {text}"
    );
    Ok(())
}

#[test]
fn check_directory_walks_subdirectories() -> AppResult<()> {
    let root = temp_dir("walk")?;
    let nested = root.join("sub");
    std::fs::create_dir_all(&nested)
        .map_err(|error| AppError::internal(format!("mkdir: {error}")))?;
    std::fs::write(root.join("a.html"), "<input>")
        .map_err(|error| AppError::internal(format!("write: {error}")))?;
    std::fs::write(nested.join("b.htm"), "<b>ok</b>")
        .map_err(|error| AppError::internal(format!("write: {error}")))?;
    std::fs::write(root.join("notes.txt"), "ignored")
        .map_err(|error| AppError::internal(format!("write: {error}")))?;
    let output = widgetlint(&["check", &path_of(&root), "--format", "json"])?;
    let reports: Vec<serde_json::Value> = serde_json::from_str(stdout(&output).trim())
        .map_err(|error| AppError::internal(format!("json: {error}")))?;
    assert_eq!(reports.len(), 2, "both html files are collected");
    Ok(())
}

#[test]
fn check_directory_honours_include_and_exclude() -> AppResult<()> {
    let root = temp_dir("filter")?;
    let nested = root.join("skipme");
    std::fs::create_dir_all(&nested)
        .map_err(|error| AppError::internal(format!("mkdir: {error}")))?;
    std::fs::write(root.join("keep.html"), "<input>")
        .map_err(|error| AppError::internal(format!("write: {error}")))?;
    std::fs::write(nested.join("drop.html"), "<input>")
        .map_err(|error| AppError::internal(format!("write: {error}")))?;

    let exclude = temp_widget("exclude.toml", "[paths]\nexclude = [\"**/skipme/**\"]\n")?;
    let output = widgetlint(&[
        "check",
        &path_of(&root),
        "--config",
        &path_of(&exclude),
        "--format",
        "json"
    ])?;
    let reports: Vec<serde_json::Value> = serde_json::from_str(stdout(&output).trim())
        .map_err(|error| AppError::internal(format!("json: {error}")))?;
    assert_eq!(reports.len(), 1, "the excluded subtree is skipped");

    let include = temp_widget("include.toml", "[paths]\ninclude = [\"**/keep.html\"]\n")?;
    let output = widgetlint(&[
        "check",
        &path_of(&root),
        "--config",
        &path_of(&include),
        "--format",
        "json"
    ])?;
    let reports: Vec<serde_json::Value> = serde_json::from_str(stdout(&output).trim())
        .map_err(|error| AppError::internal(format!("json: {error}")))?;
    assert_eq!(reports.len(), 1, "only the included file is checked");
    Ok(())
}

#[test]
fn check_invalid_glob_in_config_is_reported() -> AppResult<()> {
    let config = temp_widget("badglob.toml", "[paths]\ninclude = [\"[oops\"]\n")?;
    let file = temp_widget("globfile.html", "<div>x</div>")?;
    let output = widgetlint(&["check", &path_of(&file), "--config", &path_of(&config)])?;
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("invalid glob"),
        "{}",
        stderr(&output)
    );
    Ok(())
}

#[test]
fn check_non_utf8_file_is_reported() -> AppResult<()> {
    let file = temp_bytes("binary.html", &[0xC3, 0x28, 0xA0])?;
    let output = widgetlint(&["check", &path_of(&file)])?;
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("not valid UTF-8"),
        "{}",
        stderr(&output)
    );
    Ok(())
}

#[test]
fn check_stdin_non_utf8_is_reported() -> AppResult<()> {
    let output = run_stdin(&["check", "-"], &[0xFF, 0xFE, 0x00])?;
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("stdin is not valid UTF-8"),
        "{}",
        stderr(&output)
    );
    Ok(())
}

#[test]
fn check_reports_per_input_failures() -> AppResult<()> {
    let good = temp_widget("edge-good.html", "<div>x</div>")?;
    let broken = temp_widget("edge-broken-script.html", "<script>let = = 5</script>")?;
    let output = widgetlint(&["check", &path_of(&good), &path_of(&broken)])?;
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("1 input(s) could not be checked"),
        "{}",
        stderr(&output)
    );
    Ok(())
}

#[test]
fn check_unreadable_config_is_reported() -> AppResult<()> {
    let file = temp_widget("edge-cfgfile.html", "<div>x</div>")?;
    let output = widgetlint(&[
        "check",
        &path_of(&file),
        "--config",
        "/nonexistent/widgetlint.toml"
    ])?;
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("cannot read"),
        "{}",
        stderr(&output)
    );
    Ok(())
}

#[test]
fn check_invalid_config_is_reported() -> AppResult<()> {
    let config = temp_widget("broken.toml", "this is not = = toml\n")?;
    let file = temp_widget("edge-broken.html", "<div>x</div>")?;
    let output = widgetlint(&["check", &path_of(&file), "--config", &path_of(&config)])?;
    assert!(!output.status.success());
    assert!(stderr(&output).contains("invalid"), "{}", stderr(&output));
    Ok(())
}

#[test]
fn check_unreadable_since_file_is_reported() -> AppResult<()> {
    let file = temp_widget("edge-since.html", "<div>x</div>")?;
    let output = widgetlint(&[
        "check",
        &path_of(&file),
        "--format",
        "agent",
        "--since",
        "/nonexistent/fps.txt"
    ])?;
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("cannot read"),
        "{}",
        stderr(&output)
    );
    Ok(())
}

#[test]
fn check_promoted_error_finding_fails_on_error() -> AppResult<()> {
    let config = temp_widget("promote.toml", "[rules]\nTGW100 = \"error\"\n")?;
    let file = temp_widget("edge-error.html", "<input>")?;
    let output = widgetlint(&[
        "check",
        &path_of(&file),
        "--config",
        &path_of(&config),
        "--fail-on",
        "error"
    ])?;
    assert_eq!(
        output.status.code(),
        Some(1),
        "the promoted error must fail"
    );
    Ok(())
}

#[test]
fn size_json_reports_the_measurement() -> AppResult<()> {
    let file = temp_widget("edge-size.html", "<div>hello</div>")?;
    let output = widgetlint(&["size", &path_of(&file), "--format", "json"])?;
    assert!(output.status.success(), "{}", stdout(&output));
    let document: serde_json::Value = serde_json::from_str(stdout(&output).trim())
        .map_err(|error| AppError::internal(format!("json: {error}")))?;
    assert_eq!(document["schema_version"].as_u64(), Some(1));
    assert!(document["size"]["zlib9"].as_u64().unwrap_or(0) > 0);
    assert_eq!(document["verdict"], "pass");
    Ok(())
}

#[test]
fn size_rejects_machine_formats() -> AppResult<()> {
    let file = temp_widget("edge-sizefmt.html", "<div>x</div>")?;
    for format in ["sarif", "agent"] {
        let output = widgetlint(&["size", &path_of(&file), "--format", format])?;
        assert!(!output.status.success());
        assert!(
            stderr(&output).contains("human|json"),
            "{}",
            stderr(&output)
        );
    }
    Ok(())
}

#[test]
fn size_reads_stdin() -> AppResult<()> {
    let output = run_stdin(&["size", "-"], b"<div>hello</div>")?;
    assert!(output.status.success(), "{}", stdout(&output));
    assert!(stdout(&output).contains("zlib-9"));
    Ok(())
}

#[test]
fn size_missing_file_is_reported() -> AppResult<()> {
    let output = widgetlint(&["size", "/nonexistent/widget.html"])?;
    assert_eq!(output.status.code(), Some(2));
    Ok(())
}

#[test]
fn size_rejects_oversized_input() -> AppResult<()> {
    let big = temp_bytes("huge.html", &vec![b' '; 1024 * 1024 + 1])?;
    let output = widgetlint(&["size", &path_of(&big)])?;
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("input limit"),
        "{}",
        stderr(&output)
    );
    Ok(())
}

#[test]
fn check_min_severity_info_keeps_everything() -> AppResult<()> {
    let file = temp_widget("edge-sev-info.html", "<marquee>x</marquee>")?;
    let output = widgetlint(&[
        "check",
        &path_of(&file),
        "--min-severity",
        "info",
        "--fail-on",
        "info"
    ])?;
    assert_eq!(output.status.code(), Some(1), "the info finding survives");
    assert!(stdout(&output).contains("TGW110"));
    Ok(())
}

fn oversized_widget(name: &str) -> AppResult<PathBuf> {
    let mut state = 0x1234_5678_u32;
    let mut body = String::from("<div>");
    for _ in 0..30_000 {
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        body.push((b'a' + ((state >> 16) % 26) as u8) as char);
    }
    body.push_str("</div>");
    temp_widget(name, &body)
}

#[test]
fn size_reports_findings_in_both_formats() -> AppResult<()> {
    let file = oversized_widget("edge-big.html")?;
    let human = widgetlint(&["size", &path_of(&file)])?;
    assert!(!human.status.success(), "over the hard limit must fail");
    assert!(stdout(&human).contains("TGW002"), "{}", stdout(&human));
    let json = widgetlint(&["size", &path_of(&file), "--format", "json"])?;
    let document: serde_json::Value = serde_json::from_str(stdout(&json).trim())
        .map_err(|error| AppError::internal(format!("json: {error}")))?;
    assert_eq!(document["verdict"], "fail");
    assert!(
        document["findings"]
            .as_array()
            .is_some_and(|findings| !findings.is_empty()),
        "the size findings are listed"
    );
    assert!(document["size"]["over_by"].as_u64().unwrap_or(0) > 0);
    Ok(())
}

#[test]
fn size_rejects_oversized_stdin() -> AppResult<()> {
    let output = run_stdin(&["size", "-"], &vec![b' '; 1024 * 1024 + 1])?;
    assert!(!output.status.success());
    assert!(stderr(&output).contains("exceeds"), "{}", stderr(&output));
    Ok(())
}

#[test]
fn sarif_names_stdin_and_maps_severities() -> AppResult<()> {
    let stdin_run = run_stdin(&["check", "-", "--format", "sarif"], b"<input>")?;
    let document: serde_json::Value = serde_json::from_str(stdout(&stdin_run).trim())
        .map_err(|error| AppError::internal(format!("sarif: {error}")))?;
    let text = serde_json::to_string(&document)
        .map_err(|error| AppError::internal(format!("stringify: {error}")))?;
    assert!(text.contains("stdin"), "the artifact is named stdin");

    let config = temp_widget("promote2.toml", "[rules]\nTGW100 = \"error\"\n")?;
    let file = temp_widget("edge-sarif.html", "<input>")?;
    let output = widgetlint(&[
        "check",
        &path_of(&file),
        "--config",
        &path_of(&config),
        "--format",
        "sarif"
    ])?;
    assert!(stdout(&output).contains("\"error\""), "errors map to error");
    let marquee = temp_widget("edge-sarif-info.html", "<marquee>x</marquee>")?;
    let output = widgetlint(&["check", &path_of(&marquee), "--format", "sarif"])?;
    assert!(stdout(&output).contains("\"note\""), "info maps to note");
    Ok(())
}

#[test]
fn check_missing_directory_is_reported() -> AppResult<()> {
    let missing = std::env::temp_dir().join(format!("wl-cli-missing-{}", std::process::id()));
    let output = widgetlint(&["check", &path_of(&missing)])?;
    assert!(!output.status.success(), "a missing directory must fail");
    assert!(
        stderr(&output).contains("cannot read"),
        "{}",
        stderr(&output)
    );
    Ok(())
}

#[test]
fn size_over_budget_reports_warn_verdict() -> AppResult<()> {
    let file = temp_widget("warnverdict.html", "<div>hello</div>")?;
    let output = widgetlint(&["size", &path_of(&file), "--budget", "1", "--format", "json"])?;
    assert_eq!(output.status.code(), Some(1), "over budget must fail");
    let report: serde_json::Value = serde_json::from_str(stdout(&output).trim())
        .map_err(|error| AppError::internal(format!("json: {error}")))?;
    assert_eq!(report["verdict"], "warn");
    Ok(())
}

#[test]
fn check_demoted_rule_severities_follow_config() -> AppResult<()> {
    let file = temp_widget("demote.html", "<script>fetch(\"/x\")</script>")?;
    let warn_config = temp_widget("demote-warn.toml", "[rules]\nTGW400 = \"warn\"\n")?;
    let output = widgetlint(&[
        "check",
        &path_of(&file),
        "--config",
        &path_of(&warn_config),
        "--format",
        "json"
    ])?;
    assert_eq!(
        output.status.code(),
        Some(0),
        "the demoted finding is no longer an error"
    );
    let info_config = temp_widget("demote-info.toml", "[rules]\nTGW400 = \"info\"\n")?;
    let output = widgetlint(&[
        "check",
        &path_of(&file),
        "--config",
        &path_of(&info_config),
        "--format",
        "json"
    ])?;
    assert_eq!(
        output.status.code(),
        Some(0),
        "the demoted finding is info only"
    );
    Ok(())
}

#[test]
fn check_fail_on_info_promotes_quiet_findings() -> AppResult<()> {
    let file = temp_widget("failoninfo.html", "<input>")?;
    let output = widgetlint(&[
        "check",
        &path_of(&file),
        "--format",
        "agent",
        "--fail-on",
        "info"
    ])?;
    assert_eq!(
        output.status.code(),
        Some(1),
        "warn findings fail at --fail-on info too"
    );
    Ok(())
}

#[test]
fn check_reports_a_directory_it_cannot_read() -> AppResult<()> {
    let dir = temp_dir("unreadable")?;
    std::fs::write(dir.join("widget.html"), "<input>")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o000))?;
        let output = widgetlint(&["check", &path_of(&dir)])?;
        if std::fs::read_dir(&dir).is_err() {
            assert!(
                !output.status.success(),
                "an unreadable directory must fail"
            );
            assert!(
                stderr(&output).contains("cannot read"),
                "{}",
                stderr(&output)
            );
        } else {
            assert!(output.status.success(), "a readable directory must pass");
        }
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}
