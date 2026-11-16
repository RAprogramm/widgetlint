// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Direct tests of the MCP tools: the same code path an MCP client reaches,
//! without the transport.

use masterror::AppResult;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::Value;
use wl_mcp::{
    WidgetLintServer,
    params::{
        CheckParams, ConfidenceArg, ExplainParams, RulesParams, SeverityArg, TargetArg,
        VerifyParams
    }
};

/// Builds a server with the embedded profile.
fn server() -> AppResult<WidgetLintServer> {
    WidgetLintServer::new()
}

/// Converts a tool error into the crate error type.
fn call<T>(result: Result<T, rmcp::ErrorData>) -> AppResult<T> {
    result.map_err(|error| masterror::AppError::bad_request(error.to_string()))
}

/// Extracts the structured content of a tool result.
fn structured(result: &rmcp::model::CallToolResult) -> AppResult<Value> {
    result
        .structured_content
        .clone()
        .ok_or_else(|| masterror::AppError::internal("no structured content"))
}

#[tokio::test]
async fn the_check_tool_reports_the_findings_of_a_widget() -> AppResult<()> {
    let server = server()?;
    let params = CheckParams {
        source:         "<input>".to_owned(),
        path:           Some("widget.html".to_owned()),
        budget:         None,
        target:         None,
        min_severity:   None,
        min_confidence: None,
        fail_on:        Some(SeverityArg::Warn)
    };
    let report = structured(&call(server.check(Parameters(params)).await)?)?;
    assert_eq!(report["verdict"], "fail");
    assert_eq!(report["input"], "widget.html");
    assert_eq!(report["findings"][0]["id"], "TGW100");
    assert_eq!(
        report["findings"][0]["fix"]["hint"],
        "widgets have no input fields; use buttons and state instead"
    );
    assert_eq!(report["tool"]["name"], "widgetlint");
    Ok(())
}

#[tokio::test]
async fn the_check_tool_honours_the_thresholds() -> AppResult<()> {
    let server = server()?;
    let params = CheckParams {
        source:         "<input>".to_owned(),
        path:           None,
        budget:         Some(4096),
        target:         Some(TargetArg::Test),
        min_severity:   Some(SeverityArg::Error),
        min_confidence: Some(ConfidenceArg::Certain),
        fail_on:        None
    };
    let report = structured(&call(server.check(Parameters(params)).await)?)?;
    assert_eq!(report["counts"]["warn"], 0);
    assert_eq!(report["budget"]["target"], 4096);
    Ok(())
}

#[tokio::test]
async fn the_check_tool_rejects_a_broken_script() -> AppResult<()> {
    let server = server()?;
    let params = CheckParams {
        source:         "<script>function (</script>".to_owned(),
        path:           None,
        budget:         None,
        target:         None,
        min_severity:   None,
        min_confidence: None,
        fail_on:        None
    };
    assert!(server.check(Parameters(params)).await.is_err());
    Ok(())
}

#[tokio::test]
async fn the_verify_tool_returns_the_next_action() -> AppResult<()> {
    let server = server()?;
    let params = VerifyParams {
        source:  "<b>ok</b>".to_owned(),
        path:    None,
        fail_on: None
    };
    let verdict = structured(&call(server.verify(Parameters(params)).await)?)?;
    assert_eq!(verdict["verdict"], "pass");
    assert_eq!(verdict["counts"]["error"], 0);
    assert!(verdict["next_action"].is_string());
    Ok(())
}

#[tokio::test]
async fn the_explain_tool_returns_the_rule_metadata() -> AppResult<()> {
    let server = server()?;
    let params = ExplainParams {
        rule_id: "TGW100".to_owned()
    };
    let rule = structured(&call(server.explain(Parameters(params)).await)?)?;
    assert_eq!(rule["id"], "TGW100");
    assert_eq!(rule["severity"], "warn");
    assert_eq!(rule["confidence"], "certain");
    assert!(
        rule["spec"]
            .as_str()
            .is_some_and(|spec| spec.contains("widgets"))
    );
    Ok(())
}

#[tokio::test]
async fn the_explain_tool_rejects_an_unknown_rule() -> AppResult<()> {
    let server = server()?;
    let params = ExplainParams {
        rule_id: "TGW000".to_owned()
    };
    assert!(server.explain(Parameters(params)).await.is_err());
    Ok(())
}

#[tokio::test]
async fn the_rules_tool_filters_by_severity() -> AppResult<()> {
    let server = server()?;
    let all = structured(&call(
        server.rules(Parameters(RulesParams::default())).await
    )?)?;
    let errors = structured(&call(
        server
            .rules(Parameters(RulesParams {
                severity: Some(SeverityArg::Error)
            }))
            .await
    )?)?;
    let all_count = all["rules"].as_array().map_or(0, Vec::len);
    let error_count = errors["rules"].as_array().map_or(0, Vec::len);
    assert!(all_count > error_count);
    assert!(
        errors["rules"]
            .as_array()
            .is_some_and(|rules| rules.iter().all(|rule| rule["severity"] == "error"))
    );
    Ok(())
}

#[tokio::test]
async fn the_schema_tool_returns_the_agent_contract() -> AppResult<()> {
    let server = server()?;
    let schema = structured(&call(server.schema().await)?)?;
    assert!(schema["$schema"].is_string());
    assert!(schema["properties"]["verdict"].is_object());
    Ok(())
}

#[test]
fn the_severity_parameter_converts_into_every_threshold() {
    use wl_check::config::SeverityFilter;
    use wl_core::Severity;

    assert_eq!(
        SeverityFilter::from(SeverityArg::Info),
        SeverityFilter::Info
    );
    assert_eq!(
        SeverityFilter::from(SeverityArg::Warn),
        SeverityFilter::Warn
    );
    assert_eq!(
        SeverityFilter::from(SeverityArg::Error),
        SeverityFilter::Error
    );
    assert_eq!(Severity::from(SeverityArg::Info), Severity::Info);
    assert_eq!(Severity::from(SeverityArg::Warn), Severity::Warn);
    assert_eq!(Severity::from(SeverityArg::Error), Severity::Error);
}

#[test]
fn the_confidence_parameter_converts_into_every_threshold() {
    use wl_check::config::ConfidenceFilter;
    use wl_core::Confidence;

    assert_eq!(
        ConfidenceFilter::from(ConfidenceArg::Heuristic),
        ConfidenceFilter::Heuristic
    );
    assert_eq!(
        ConfidenceFilter::from(ConfidenceArg::Certain),
        ConfidenceFilter::Certain
    );
    assert_eq!(
        Confidence::from(ConfidenceArg::Heuristic),
        Confidence::Heuristic
    );
    assert_eq!(
        Confidence::from(ConfidenceArg::Certain),
        Confidence::Certain
    );
}

#[test]
fn the_target_parameter_converts_into_every_target() {
    use wl_check::config::Target;

    assert_eq!(Target::from(TargetArg::Production), Target::Production);
    assert_eq!(Target::from(TargetArg::Test), Target::Test);
}
