// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Numeric widget limits (spec section 4.2).

use serde::Deserialize;

use crate::error::ProfileError;

/// Hard numeric limits enforced by the server and the clients.
///
/// # Examples
///
/// ```
/// let profile = wl_profile::profile_1().expect("valid");
/// assert_eq!(profile.limits.dom_nodes, 2000);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    /// Maximum HTML size in UTF-8 bytes (32768).
    pub html_bytes:            u32,
    /// Maximum size after zlib level 9 compression (4096).
    pub html_zlib9_bytes:      u32,
    /// Maximum size of `chat_data` / `public_data` (1024).
    pub state_bytes:           u32,
    /// Maximum size of a request payload (64).
    pub request_data_bytes:    u32,
    /// Maximum size of a bot answer payload (1024).
    pub answer_data_bytes:     u32,
    /// Maximum number of CSS rules (400).
    pub css_rules:             u32,
    /// Maximum number of `@keyframes` (32).
    pub css_keyframes:         u32,
    /// Maximum number of DOM nodes (2000).
    pub dom_nodes:             u32,
    /// Maximum attribute / inline-style length in UTF-16 units (8192).
    pub attribute_units:       u32,
    /// Maximum text value length in UTF-16 units (32768).
    pub text_value_units:      u32,
    /// Maximum document text length in UTF-16 units (49152).
    pub text_document_units:   u32,
    /// Maximum total canvas area as `[width, height]` pixels (2048x2048).
    pub canvas_pixels:         [u32; 2],
    /// Maximum number of live timers (64).
    pub timers:                u32,
    /// Maximum handlers per event type (16).
    pub handlers_per_event:    u32,
    /// Maximum simultaneous "heavy" elements (16).
    pub heavy_elements:        u32,
    /// Maximum heavy elements with `backdrop-filter` (2).
    pub heavy_backdrop_filter: u32,
    /// Web Storage quota per area in bytes (4096).
    pub storage_bytes:         u32,
    /// Maximum `copyText` length in UTF-16 units (256).
    pub copy_text_units:       u32,
    /// Maximum URL length for `openLink` / `openTelegramLink` (256).
    pub url_chars:             u32
}

impl Limits {
    /// Checks that every limit is positive and cross-relations hold.
    ///
    /// Returns [`ProfileError`] describing the first broken invariant.
    ///
    /// # Errors
    ///
    /// [`ProfileError`] if a limit is zero or a cross-relation between limits
    /// is violated.
    ///
    /// # Examples
    ///
    /// ```
    /// let profile = wl_profile::profile_1().expect("valid");
    /// profile.limits.validate().expect("limits are consistent");
    /// ```
    pub fn validate(&self) -> Result<(), ProfileError> {
        let named = [
            ("html_bytes", self.html_bytes),
            ("html_zlib9_bytes", self.html_zlib9_bytes),
            ("state_bytes", self.state_bytes),
            ("request_data_bytes", self.request_data_bytes),
            ("answer_data_bytes", self.answer_data_bytes),
            ("css_rules", self.css_rules),
            ("css_keyframes", self.css_keyframes),
            ("dom_nodes", self.dom_nodes),
            ("attribute_units", self.attribute_units),
            ("text_value_units", self.text_value_units),
            ("text_document_units", self.text_document_units),
            ("canvas_width", self.canvas_pixels[0]),
            ("canvas_height", self.canvas_pixels[1]),
            ("timers", self.timers),
            ("handlers_per_event", self.handlers_per_event),
            ("heavy_elements", self.heavy_elements),
            ("heavy_backdrop_filter", self.heavy_backdrop_filter),
            ("storage_bytes", self.storage_bytes),
            ("copy_text_units", self.copy_text_units),
            ("url_chars", self.url_chars)
        ];
        for (name, value) in named {
            if value == 0 {
                return Err(ProfileError::new(format!("limits.{name} must be positive")));
            }
        }
        if self.html_zlib9_bytes >= self.html_bytes {
            return Err(ProfileError::new(
                "html_zlib9_bytes must be below html_bytes"
            ));
        }
        if self.text_document_units < self.text_value_units {
            return Err(ProfileError::new(
                "text_document_units must be at least text_value_units"
            ));
        }
        if self.heavy_backdrop_filter > self.heavy_elements {
            return Err(ProfileError::new(
                "heavy_backdrop_filter must not exceed heavy_elements"
            ));
        }
        Ok(())
    }
}
