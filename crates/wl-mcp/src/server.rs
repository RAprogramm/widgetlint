// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The Model Context Protocol server: the widgetlint pipeline as tools.
//!
//! The server is stateless and side-effect free: every tool receives the
//! widget source as a parameter and returns data. Nothing is read from disk
//! and nothing is written.

#![allow(
    clippy::unused_async_trait_impl,
    reason = "the MCP macros expand into an async trait implementation the lint cannot inspect"
)]

use rmcp::{
    ErrorData as McpError, handler::server::wrapper::Parameters, model::CallToolResult, tool,
    tool_router
};
use serde_json::Value;
use wl_check::{AgentOptions, Config, agent, check};
use wl_core::Severity;
use wl_profile::Profile;

use crate::params::{
    CheckParams, ConfidenceArg, ExplainParams, RulesParams, SeverityArg, VerifyParams
};

/// The widgetlint server: one tool per capability of the linter.
#[derive(Clone)]
pub struct WidgetLintServer {
    profile: Profile,
    config:  Config
}

impl WidgetLintServer {
    /// Builds a server with the embedded profile 1 and the default
    /// configuration.
    ///
    /// # Errors
    ///
    /// [`masterror::AppError`] with `Internal` taxonomy when the embedded
    /// profile cannot be parsed; the profile is compiled into the binary, so
    /// the error means a broken build.
    pub fn new() -> masterror::AppResult<Self> {
        let profile = wl_profile::profile_1()?;
        Ok(Self {
            profile,
            config: Config::default()
        })
    }

    /// Builds the effective configuration for one call.
    fn config_for(
        &self,
        budget: Option<u32>,
        target: Option<crate::params::TargetArg>,
        min_severity: Option<SeverityArg>,
        min_confidence: Option<ConfidenceArg>
    ) -> Config {
        let mut config = self.config.clone();
        if let Some(budget) = budget {
            config.size.budget = budget;
        }
        if let Some(target) = target {
            config.target = target.into();
        }
        if let Some(min_severity) = min_severity {
            config.output.min_severity = min_severity.into();
        }
        if let Some(min_confidence) = min_confidence {
            config.output.min_confidence = min_confidence.into();
        }
        config
    }

    /// Builds the agent report of one document.
    fn report(
        &self,
        source: &str,
        path: &str,
        config: &Config,
        fail_on: Option<SeverityArg>
    ) -> masterror::AppResult<wl_check::agent::AgentReport> {
        let checked = check(source, &self.profile, config)?;
        let options = AgentOptions {
            fail_on:      fail_on.map_or(SeverityArg::Error, SeverityArg::from).into(),
            max_findings: AgentOptions::default().max_findings,
            iteration:    AgentOptions::default().iteration,
            since:        None,
            limit:        self.profile.limits.html_zlib9_bytes,
            budget:       config.size.budget,
            tool_version: env!("CARGO_PKG_VERSION")
        };
        Ok(agent::report(path, source, &checked, &options))
    }
}

#[tool_router(server_handler)]
impl WidgetLintServer {
    /// Checks a Telegram widget and returns the full agent report.
    ///
    /// # Errors
    ///
    /// [`rmcp::ErrorData`] with an internal error code when the widget cannot
    /// be parsed or the report cannot be serialized.
    #[tool(
        description = "Check a Telegram Bot API 10.4 widget and return the full machine-readable report: verdict, size budget, grouped findings with exact positions, fix hints and the next action."
    )]
    pub async fn check(
        &self,
        Parameters(params): Parameters<CheckParams>
    ) -> Result<CallToolResult, McpError> {
        let config = self.config_for(
            params.budget,
            params.target,
            params.min_severity,
            params.min_confidence
        );
        let path = params.path.as_deref().unwrap_or("widget.html");
        let report = self
            .report(&params.source, path, &config, params.fail_on)
            .map_err(|error| McpError::internal_error(error.to_string(), None))?;
        let value = serde_json::to_value(&report)
            .map_err(|error| McpError::internal_error(format!("agent report: {error}"), None))?;
        Ok(CallToolResult::structured(value))
    }

    /// Checks a widget and returns only the verdict and the next action.
    ///
    /// # Errors
    ///
    /// [`rmcp::ErrorData`] with an internal error code when the widget cannot
    /// be parsed or the report cannot be serialized.
    #[tool(
        description = "Check a Telegram widget and return only the verdict, the reason and the single next action to take. Use it to decide whether a widget may be sent."
    )]
    pub async fn verify(
        &self,
        Parameters(params): Parameters<VerifyParams>
    ) -> Result<CallToolResult, McpError> {
        let config = self.config_for(None, None, None, None);
        let path = params.path.as_deref().unwrap_or("widget.html");
        let report = self
            .report(&params.source, path, &config, params.fail_on)
            .map_err(|error| McpError::internal_error(error.to_string(), None))?;
        let value = serde_json::json!({
            "schema_version": report.schema_version,
            "verdict": report.verdict,
            "reason": report.reason,
            "next_action": report.next_action,
            "counts": {
                "error": report.counts.error,
                "warn": report.counts.warn,
                "info": report.counts.info,
                "suppressed": report.counts.suppressed
            },
            "budget": {
                "zlib9": report.budget.zlib9,
                "limit": report.budget.limit,
                "over_by": report.budget.over_by,
                "target": report.budget.target
            },
            "tool": report.tool
        });
        Ok(CallToolResult::structured(value))
    }

    /// Explains one rule.
    ///
    /// # Errors
    ///
    /// [`rmcp::ErrorData`] with invalid parameters when the rule id is
    /// unknown.
    #[tool(
        description = "Explain one rule by its id, for example TGW100: severity, confidence, what it detects, how to fix it and the specification link."
    )]
    pub async fn explain(
        &self,
        Parameters(params): Parameters<ExplainParams>
    ) -> Result<CallToolResult, McpError> {
        let rule = wl_core::rule::lookup(&params.rule_id).ok_or_else(|| {
            McpError::invalid_params(format!("unknown rule id '{}'", params.rule_id), None)
        })?;
        let value = serde_json::json!({
            "id": rule.id,
            "severity": rule.default_severity.as_str(),
            "confidence": rule.confidence.as_str(),
            "title": rule.title,
            "help": rule.help,
            "keywords": rule.keywords,
            "spec": rule.spec
        });
        Ok(CallToolResult::structured(value))
    }

    /// Lists every known rule.
    ///
    /// # Errors
    ///
    /// [`rmcp::ErrorData`] with an internal error code when the catalog
    /// cannot be serialized.
    #[tool(
        description = "List every rule the linter knows, with its id, severity, confidence and title. Use it to pick the right rule id before calling explain."
    )]
    pub async fn rules(
        &self,
        Parameters(params): Parameters<RulesParams>
    ) -> Result<CallToolResult, McpError> {
        let wanted = params.severity.map(Severity::from);
        let rules: Vec<Value> = wl_core::rule::all()
            .into_iter()
            .filter(|rule| wanted.is_none_or(|severity| rule.default_severity == severity))
            .map(|rule| {
                serde_json::json!({
                    "id": rule.id,
                    "severity": rule.default_severity.as_str(),
                    "confidence": rule.confidence.as_str(),
                    "title": rule.title
                })
            })
            .collect();
        let value = serde_json::json!({ "rules": rules });
        Ok(CallToolResult::structured(value))
    }

    /// Returns the JSON Schema of the agent report.
    ///
    /// # Errors
    ///
    /// [`rmcp::ErrorData`] with an internal error code when the schema cannot
    /// be built or parsed.
    #[tool(
        description = "Return the JSON Schema of the agent report, the stable contract of the check and verify tools."
    )]
    pub async fn schema(&self) -> Result<CallToolResult, McpError> {
        let schema = wl_check::agent::schema_json()
            .map_err(|error| McpError::internal_error(error.to_string(), None))?;
        let value: Value = serde_json::from_str(&schema)
            .map_err(|error| McpError::internal_error(format!("agent schema: {error}"), None))?;
        Ok(CallToolResult::structured(value))
    }
}
