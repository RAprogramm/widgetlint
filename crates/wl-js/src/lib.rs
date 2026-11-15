// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! JavaScript checks for Telegram widgets, built on the `oxc` parser.
//!
//! The checks cover the disabled and unavailable browser APIs (TGW400 to
//! TGW406) and the `TelegramWidget` contract (TGW408, TGW411). A `<script>`
//! body and an `on*` attribute are both checked with [`check`]; the caller
//! passes the byte offset of the block inside the widget document so spans
//! point at the widget source.
//!
//! Names bound anywhere in the script (variables, parameters, functions) are
//! never reported as disabled globals, so local shadowing stays clean.
//!
//! # Examples
//!
//! ```
//! use wl_js::check;
//!
//! let profile = wl_profile::profile_1().expect("valid profile");
//! let findings = check("fetch('/x')", 0, &profile).expect("valid script");
//! assert_eq!(findings.first().expect("one finding").rule.id, "TGW400");
//! ```
//!
//! ```
//! use wl_js::check;
//!
//! let profile = wl_profile::profile_1().expect("valid profile");
//! let findings =
//!     check("TelegramWidget.sendMessage({ text: 'x' })", 0, &profile).expect("valid script");
//! assert_eq!(findings.first().expect("one finding").rule.id, "TGW408");
//! ```
//!
//! ```
//! use wl_js::check;
//!
//! let profile = wl_profile::profile_1().expect("valid profile");
//! assert!(check("function (", 0, &profile).is_err());
//! ```

pub mod error;
pub mod rules;

use masterror::AppResult;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use wl_core::finding::Finding;
use wl_profile::Profile;

pub use crate::error::JsError;

/// Checks one JavaScript block against the profile and returns findings
/// sorted by position in the block.
///
/// * `js` — the JavaScript text: a `<script>` body or an `on*` attribute value.
/// * `base` — byte offset of `js` inside the widget document; every span is
///   shifted by it, so findings point at the widget source directly.
/// * `profile` — the widget profile providing the out-lists.
///
/// # Errors
///
/// [`masterror::AppError`] with [`JsError`] taxonomy when the script does not
/// parse; the message names the first syntax problem.
pub fn check(js: &str, base: u32, profile: &Profile) -> AppResult<Vec<Finding>> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, js, SourceType::cjs()).parse();
    if let Some(diagnostic) = parsed.diagnostics.first() {
        return Err(JsError::new(format!("JavaScript syntax error: {diagnostic}")).into());
    }
    Ok(rules::run(&parsed.program, profile, base))
}
