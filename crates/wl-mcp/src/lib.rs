// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Model Context Protocol server for the widgetlint pipeline.
//!
//! The crate exposes the linter as MCP tools so that an agent without a shell
//! can check a Telegram widget, read a rule and iterate on the findings:
//!
//! - `check` — the full agent report of one widget source;
//! - `verify` — the verdict and the next action only;
//! - `explain` — one rule by id;
//! - `rules` — the catalog of every known rule;
//! - `schema` — the JSON Schema of the agent report.
//!
//! The server speaks the protocol over stdio and never touches the file
//! system: every widget arrives as a tool parameter.
//!
//! # Examples
//!
//! ```no_run
//! # async fn run() -> masterror::AppResult<()> {
//! wl_mcp::run().await
//! # }
//! ```

pub mod params;
pub mod server;

use masterror::AppResult;
use rmcp::ServiceExt;

pub use crate::server::WidgetLintServer;

/// Runs the server on the standard input and output until the client stops.
///
/// # Errors
///
/// [`masterror::AppError`] with `Internal` taxonomy when the transport cannot
/// be established or the session ends with an error.
pub async fn run() -> AppResult<()> {
    let service = WidgetLintServer::new()?
        .serve(rmcp::transport::stdio())
        .await
        .map_err(|error| masterror::AppError::internal(format!("mcp transport: {error}")))?;
    service
        .waiting()
        .await
        .map_err(|error| masterror::AppError::internal(format!("mcp session: {error}")))?;
    Ok(())
}
