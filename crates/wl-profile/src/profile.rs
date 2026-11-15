// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Root profile document: aggregation, parsing and validation.

use masterror::AppResult;
use serde::Deserialize;

use crate::{
    css::Css, error::ProfileError, gestures::Gestures, html::Html, js::Js, limits::Limits
};

/// A validated widget profile: numeric limits plus the "in"/"out" lists.
///
/// Unknown TOML fields are rejected so a typo can never silently drop a rule.
///
/// # Examples
///
/// ```
/// let source = std::fs::read_to_string(concat!(
///     env!("CARGO_MANIFEST_DIR"),
///     "/../../ruleset/profile-1.toml"
/// ))
/// .expect("profile file");
/// let profile = wl_profile::Profile::from_toml(&source).expect("valid profile");
/// assert_eq!(profile.profile, 1);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    /// Profile number; v1 supports only `1`.
    pub profile:       u32,
    /// Hash of the normalized specification source this profile was built from.
    pub spec_snapshot: String,
    /// URL of the specification source.
    pub spec_url:      String,
    /// Numeric limits (spec section 4.2).
    pub limits:        Limits,
    /// HTML lists (spec section 8.2).
    pub html:          Html,
    /// CSS lists (spec section 8.3).
    pub css:           Css,
    /// JavaScript lists (spec sections 8.5–8.6).
    pub js:            Js,
    /// Gesture timings (spec section "Gestures").
    pub gestures:      Gestures
}

impl Profile {
    /// Parses a profile from TOML and validates it.
    ///
    /// * `text` — TOML document.
    ///
    /// Returns the validated profile or a `BadRequest` [`masterror::AppError`]
    /// describing the first broken invariant.
    ///
    /// # Errors
    ///
    /// [`masterror::AppError`] with `BadRequest` taxonomy when the TOML is
    /// invalid or the profile violates an invariant.
    pub fn from_toml(text: &str) -> AppResult<Self> {
        let profile: Self =
            toml::from_str(text).map_err(|error| ProfileError::new(error.to_string()))?;
        profile.validate()?;
        Ok(profile)
    }

    /// Runs every section validator and the cross-section checks.
    ///
    /// # Errors
    ///
    /// [`masterror::AppError`] with `BadRequest` taxonomy on the first broken
    /// invariant.
    pub fn validate(&self) -> AppResult<()> {
        if self.profile != 1 {
            return Err(ProfileError::new(format!(
                "unsupported profile {}; only profile 1 is known",
                self.profile
            ))
            .into());
        }
        if self.spec_url.is_empty() {
            return Err(ProfileError::new("spec_url must not be empty").into());
        }
        self.limits.validate()?;
        self.html.validate()?;
        self.css.validate()?;
        self.js.validate()?;
        self.gestures.validate()?;
        Ok(())
    }
}
