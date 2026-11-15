// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Domain error for profile loading and validation.

use masterror::{AppCode, AppErrorKind};

/// Profile parsing or validation failure.
///
/// The generated [`From`] conversion turns it into [`masterror::AppError`]
/// with a `BadRequest` taxonomy; the message is safe to show to a user.
///
/// # Examples
///
/// ```
/// let error = wl_profile::ProfileError::new("limits.html_bytes must be positive");
/// assert!(error.to_string().contains("html_bytes"));
/// ```
#[derive(Debug, masterror::Error)]
#[error("{reason}")]
#[app_error(kind = AppErrorKind::BadRequest, code = AppCode::BadRequest, message)]
pub struct ProfileError {
    /// Human-readable description of the failed invariant.
    reason: String
}

impl ProfileError {
    /// Creates an error from a validation message.
    ///
    /// * `reason` — description of the failed invariant.
    #[must_use]
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into()
        }
    }
}
