// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! End-to-end tests of the xtask binary: dispatch, exit codes and stderr text.

use std::process::Command;

#[test]
fn unknown_task_exits_with_failure() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("definitely-not-a-task")
        .output()
        .expect("the xtask binary must spawn");
    assert!(!output.status.success(), "an unknown task must fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("definitely-not-a-task"), "stderr: {stderr}");
}

#[test]
fn missing_task_exits_with_failure() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .output()
        .expect("the xtask binary must spawn");
    assert!(!output.status.success(), "a missing task name must fail");
}

#[test]
fn check_docs_exits_with_success() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("check-docs")
        .output()
        .expect("the xtask binary must spawn");
    assert!(output.status.success(), "committed docs must be fresh");
}

#[test]
fn gen_site_writes_the_rule_catalog() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("gen-site")
        .output()
        .expect("the xtask binary must spawn");
    assert!(output.status.success(), "the site must be generated");
    let root = std::env::current_dir().expect("the working directory must be readable");
    let mut dir = root.as_path();
    while !dir.join("ruleset").is_dir() {
        dir = dir.parent().expect("the workspace root must be reachable");
    }
    assert!(dir.join("playground/rules/index.html").is_file());
    assert!(dir.join("playground/rules/TGW100.html").is_file());
}
