// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The `mcp` subcommand: the Model Context Protocol server over stdio.

use masterror::AppResult;

use crate::cli::McpArgs;

/// Runs the MCP server until the client closes the session.
///
/// * `args` — arguments of the subcommand; the server takes none.
///
/// # Errors
///
/// [`masterror::AppError`] with `Internal` taxonomy when the async runtime
/// cannot be created or the session fails.
pub fn run(_args: &McpArgs) -> AppResult<bool> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|error| masterror::AppError::internal(format!("async runtime: {error}")))?;
    runtime.block_on(wl_mcp::run())?;
    Ok(true)
}
