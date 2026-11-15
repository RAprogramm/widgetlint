// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! CSS section: properties, units, selectors and at-rules outside the profile.

use serde::Deserialize;

use crate::{error::ProfileError, validate};

/// CSS part of the profile.
///
/// # Examples
///
/// ```
/// let profile = wl_profile::profile_1().expect("valid");
/// assert!(profile.css.out.units_height.iter().any(|unit| unit == "vh"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Css {
    /// Out lists of CSS features.
    pub out: CssOut
}

impl Css {
    /// Validates all CSS lists.
    ///
    /// # Errors
    ///
    /// [`ProfileError`] if any CSS list is empty or contains malformed names.
    pub fn validate(&self) -> Result<(), ProfileError> {
        self.out.validate()
    }
}

/// Out lists of CSS features (spec section 8.3).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CssOut {
    /// Height-relative units, unsupported (TGW200).
    pub units_height:             Vec<String>,
    /// `overflow` values rewritten to `hidden` (TGW201).
    pub overflow_to_hidden:       Vec<String>,
    /// `position` values rewritten to `relative` (TGW202).
    pub position_to_relative:     Vec<String>,
    /// Modern color functions outside the profile (TGW210).
    pub color_functions:          Vec<String>,
    /// Allowed `filter` functions (TGW207).
    pub filter_functions_allowed: Vec<String>,
    /// Allowed `backdrop-filter` functions (TGW207).
    pub backdrop_filter_allowed:  Vec<String>,
    /// Unsupported at-rules (TGW212).
    pub at_rules:                 Vec<String>,
    /// Unsupported selector tokens (TGW213).
    pub selectors:                Vec<String>,
    /// Unsupported `@media` features (TGW214).
    pub media_unsupported:        Vec<String>,
    /// Allowed generic font families (TGW215).
    pub font_generics_allowed:    Vec<String>
}

impl CssOut {
    /// Validates every CSS list.
    ///
    /// # Errors
    ///
    /// [`ProfileError`] if a list is empty, has duplicates or malformed names.
    pub fn validate(&self) -> Result<(), ProfileError> {
        validate::html_css_names(&self.units_height, "css.out.units_height")?;
        validate::html_css_names(&self.overflow_to_hidden, "css.out.overflow_to_hidden")?;
        validate::html_css_names(&self.position_to_relative, "css.out.position_to_relative")?;
        validate::html_css_names(&self.color_functions, "css.out.color_functions")?;
        validate::html_css_names(
            &self.filter_functions_allowed,
            "css.out.filter_functions_allowed"
        )?;
        validate::html_css_names(
            &self.backdrop_filter_allowed,
            "css.out.backdrop_filter_allowed"
        )?;
        validate::html_css_names(&self.at_rules, "css.out.at_rules")?;
        validate::selector_tokens(&self.selectors, "css.out.selectors")?;
        validate::html_css_names(&self.media_unsupported, "css.out.media_unsupported")?;
        validate::html_css_names(&self.font_generics_allowed, "css.out.font_generics_allowed")
    }
}
