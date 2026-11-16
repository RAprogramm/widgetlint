// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! End-to-end test of the Model Context Protocol server: a real client talks
//! to the real binary over stdio.

use std::process::Stdio;

use masterror::{AppError, AppResult};
use rmcp::{ServiceExt, model::CallToolRequestParams, transport::TokioChildProcess};
use serde_json::{Map, Value, json};
use tokio::process::Command;

/// Spawns the server and connects a client to it.
async fn connect() -> AppResult<rmcp::service::RunningService<rmcp::RoleClient, ()>> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_widgetlint"));
    command.arg("mcp");
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    let child = TokioChildProcess::new(command)
        .map_err(|error| AppError::internal(format!("child process: {error}")))?;
    let client = ()
        .serve(child)
        .await
        .map_err(|error| AppError::internal(format!("mcp client: {error}")))?;
    Ok(client)
}

/// Builds the arguments of a tool call.
fn arguments(source: &str) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("source".to_owned(), json!(source));
    map
}

#[tokio::test]
async fn the_server_exposes_every_tool() -> AppResult<()> {
    let client = connect().await?;
    let tools = client
        .list_all_tools()
        .await
        .map_err(|error| AppError::internal(format!("tools/list: {error}")))?;
    let names: Vec<&str> = tools.iter().map(|tool| tool.name.as_ref()).collect();
    for expected in ["check", "verify", "explain", "rules", "schema"] {
        assert!(
            names.contains(&expected),
            "the tool '{expected}' is missing from {names:?}"
        );
    }
    client
        .cancel()
        .await
        .map_err(|error| AppError::internal(format!("cancel: {error}")))?;
    Ok(())
}

#[tokio::test]
async fn the_check_tool_reports_a_broken_widget() -> AppResult<()> {
    let client = connect().await?;
    let mut map = arguments("<input>");
    map.insert("fail_on".to_owned(), json!("warn"));
    let result = client
        .call_tool(CallToolRequestParams::new("check").with_arguments(map))
        .await
        .map_err(|error| AppError::internal(format!("tools/call: {error}")))?;
    let report = result
        .structured_content
        .ok_or_else(|| AppError::internal("the tool returned no structured content"))?;
    assert_eq!(report["verdict"], "fail");
    assert_eq!(report["findings"][0]["id"], "TGW100");
    assert_eq!(report["counts"]["warn"], 1);
    client
        .cancel()
        .await
        .map_err(|error| AppError::internal(format!("cancel: {error}")))?;
    Ok(())
}

#[tokio::test]
async fn the_check_tool_keeps_warnings_below_the_error_threshold() -> AppResult<()> {
    let client = connect().await?;
    let result = client
        .call_tool(CallToolRequestParams::new("check").with_arguments(arguments("<input>")))
        .await
        .map_err(|error| AppError::internal(format!("tools/call: {error}")))?;
    let report = result
        .structured_content
        .ok_or_else(|| AppError::internal("the tool returned no structured content"))?;
    assert_eq!(report["verdict"], "pass");
    assert_eq!(report["findings"][0]["id"], "TGW100");
    client
        .cancel()
        .await
        .map_err(|error| AppError::internal(format!("cancel: {error}")))?;
    Ok(())
}

#[tokio::test]
async fn the_verify_tool_returns_the_next_action() -> AppResult<()> {
    let client = connect().await?;
    let result = client
        .call_tool(CallToolRequestParams::new("verify").with_arguments(arguments("<b>ok</b>")))
        .await
        .map_err(|error| AppError::internal(format!("tools/call: {error}")))?;
    let verdict = result
        .structured_content
        .ok_or_else(|| AppError::internal("the tool returned no structured content"))?;
    assert_eq!(verdict["verdict"], "pass");
    assert!(verdict["next_action"].is_string());
    client
        .cancel()
        .await
        .map_err(|error| AppError::internal(format!("cancel: {error}")))?;
    Ok(())
}

#[tokio::test]
async fn the_explain_tool_rejects_an_unknown_rule() -> AppResult<()> {
    let client = connect().await?;
    let mut map = Map::new();
    map.insert("rule_id".to_owned(), json!("TGW999"));
    let outcome = client
        .call_tool(CallToolRequestParams::new("explain").with_arguments(map))
        .await;
    assert!(outcome.is_err(), "an unknown rule id must be rejected");
    client
        .cancel()
        .await
        .map_err(|error| AppError::internal(format!("cancel: {error}")))?;
    Ok(())
}

#[tokio::test]
async fn the_rules_tool_lists_the_catalog() -> AppResult<()> {
    let client = connect().await?;
    let result = client
        .call_tool(CallToolRequestParams::new("rules").with_arguments(Map::new()))
        .await
        .map_err(|error| AppError::internal(format!("tools/call: {error}")))?;
    let catalog = result
        .structured_content
        .ok_or_else(|| AppError::internal("the tool returned no structured content"))?;
    let rules = catalog["rules"]
        .as_array()
        .ok_or_else(|| AppError::internal("the catalog has no rules array"))?;
    assert!(rules.len() > 10, "the catalog looks empty: {}", rules.len());
    assert!(rules.iter().any(|rule| rule["id"] == "TGW100"));
    client
        .cancel()
        .await
        .map_err(|error| AppError::internal(format!("cancel: {error}")))?;
    Ok(())
}
