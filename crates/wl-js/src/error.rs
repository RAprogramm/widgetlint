// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Domain error for JavaScript parsing.

use masterror::{AppCode, AppErrorKind};

/// JavaScript source that the parser could not read.
///
/// The generated [`From`] conversion turns it into [`masterror::AppError`]
/// with a `BadRequest` taxonomy; the message is safe to show to a user.
///
/// # Examples
///
/// ```
/// let error = wl_js::JsError::new("unexpected token");
/// assert!(error.to_string().contains("token"));
/// ```
#[derive(Debug, masterror::Error)]
#[error("{reason}")]
#[app_error(kind = AppErrorKind::BadRequest, code = AppCode::BadRequest, message)]
pub struct JsError {
    /// Human-readable description of the parse failure.
    reason: String
}

impl JsError {
    /// Creates an error from a parse failure message.
    ///
    /// * `reason` — description of the syntax problem.
    #[must_use]
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into()
        }
    }
}
