// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Loading and validation of widget profile data.
//!
//! A profile is a TOML document with numeric limits and the "in"/"out" lists
//! from the Telegram widget specification. The embedded profile 1 is compiled
//! into the binary; external profiles are parsed with [`Profile::from_toml`].
//!
//! # Examples
//!
//! ```
//! let profile = wl_profile::profile_1().expect("embedded profile is valid");
//! assert_eq!(profile.profile, 1);
//! assert_eq!(profile.limits.html_zlib9_bytes, 4096);
//! ```

pub mod css;
pub mod error;
pub mod gestures;
pub mod html;
pub mod js;
pub mod limits;
pub mod profile;
mod validate;

use masterror::AppResult;

pub use crate::{error::ProfileError, profile::Profile};

/// TOML source of the embedded profile 1, compiled into the binary.
pub const PROFILE_1_TOML: &str = include_str!("../../../ruleset/profile-1.toml");

/// Returns the embedded profile 1, parsed and validated.
///
/// Fails only if the embedded data is broken, which is a bug rather than user
/// input.
///
/// # Errors
///
/// [`masterror::AppError`] with `BadRequest` taxonomy if the embedded data is
/// broken.
///
/// # Examples
///
/// ```
/// let profile = wl_profile::profile_1().expect("valid");
/// assert!(profile.limits.html_bytes > profile.limits.html_zlib9_bytes);
/// ```
pub fn profile_1() -> AppResult<Profile> {
    Profile::from_toml(PROFILE_1_TOML)
}
