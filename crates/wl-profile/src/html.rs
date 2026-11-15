// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! HTML section: elements and attributes outside the widget profile.

use serde::Deserialize;

use crate::{error::ProfileError, validate};

/// HTML part of the profile: out lists, degradation map, partial in list.
///
/// # Examples
///
/// ```
/// let profile = wl_profile::profile_1().expect("valid");
/// assert!(
///     profile
///         .html
///         .out
///         .form_controls
///         .iter()
///         .any(|name| name == "input")
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Html {
    /// Elements the client removes together with their content.
    pub out:        HtmlOut,
    /// Elements that change meaning instead of being removed.
    pub degrade:    HtmlDegrade,
    /// Known supported elements; the source list is incomplete by design.
    pub in_partial: HtmlInPartial
}

impl Html {
    /// Validates all HTML lists.
    ///
    /// # Errors
    ///
    /// [`ProfileError`] if any HTML list is empty or contains malformed names.
    pub fn validate(&self) -> Result<(), ProfileError> {
        self.out.validate()?;
        validate::html_css_names(
            std::slice::from_ref(&self.degrade.form),
            "html.degrade.form"
        )?;
        validate::html_css_names(
            std::slice::from_ref(&self.degrade.label),
            "html.degrade.label"
        )?;
        validate::html_css_names(&self.in_partial.elements, "html.in_partial.elements")
    }
}

/// Out lists of HTML elements and attributes (spec section 8.2).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HtmlOut {
    /// Form controls removed with content (TGW100).
    pub form_controls: Vec<String>,
    /// Media and embedding elements (TGW102).
    pub media_embed:   Vec<String>,
    /// Service elements (TGW103).
    pub service:       Vec<String>,
    /// Legacy presentational elements (TGW108).
    pub legacy:        Vec<String>,
    /// Unsupported global attributes (TGW109).
    pub attrs:         Vec<String>
}

impl HtmlOut {
    /// Validates every out list.
    ///
    /// # Errors
    ///
    /// [`ProfileError`] if a list is empty, has duplicates or malformed names.
    pub fn validate(&self) -> Result<(), ProfileError> {
        validate::html_css_names(&self.form_controls, "html.out.form_controls")?;
        validate::html_css_names(&self.media_embed, "html.out.media_embed")?;
        validate::html_css_names(&self.service, "html.out.service")?;
        validate::html_css_names(&self.legacy, "html.out.legacy")?;
        validate::html_css_names(&self.attrs, "html.out.attrs")
    }
}

/// Elements that degrade instead of being removed (TGW101).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HtmlDegrade {
    /// How `<form>` is rendered (expected: `block`).
    pub form:  String,
    /// How `<label>` is rendered (expected: `inline`).
    pub label: String
}

/// Partial list of supported elements (unknown names are informational).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HtmlInPartial {
    /// Known supported elements; the source list ends with "…".
    pub elements: Vec<String>
}
