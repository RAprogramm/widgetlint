// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! CSS checks for Telegram widgets: profile-driven keyword rules (TGW200 to
//! TGW215), an unknown-property check built on a curated browser property
//! table (TGW216) and the stylesheet size limits (TGW300, TGW301).
//!
//! Every finding carries an exact byte span inside the CSS block it was found
//! in. A `<style>` body and a `style=""` attribute are both checked with
//! [`check`]; the caller passes the byte offset of the block inside the widget
//! document so spans point at the widget source.
//!
//! # Examples
//!
//! ```
//! use wl_css::check;
//!
//! let profile = wl_profile::profile_1().expect("valid profile");
//! let findings = check(".box { height: 50vh }", 0, &profile);
//! assert_eq!(findings.len(), 1);
//! assert_eq!(findings.first().expect("one finding").rule.id, "TGW200");
//! ```
//!
//! ```
//! use wl_css::check;
//!
//! let profile = wl_profile::profile_1().expect("valid profile");
//! let findings = check("a { color: oklch(70% 0.1 200) }", 0, &profile);
//! assert_eq!(findings.first().expect("one finding").rule.id, "TGW210");
//! ```

pub mod known_properties;
mod rules;
pub mod scan;

use wl_core::finding::Finding;
use wl_profile::Profile;

/// Checks one CSS block against the profile and returns findings sorted by
/// position in the block.
///
/// * `css` — the CSS text: a `<style>` body or a `style=""` attribute value.
/// * `base` — byte offset of `css` inside the widget document; every span is
///   shifted by it, so findings point at the widget source directly.
/// * `profile` — the widget profile providing the out-lists and the limits.
///
/// Malformed CSS never fails: the scanner degrades to fewer items instead.
#[must_use]
pub fn check(css: &str, base: u32, profile: &Profile) -> Vec<Finding> {
    let doc = scan::scan(css);
    let mut findings = Vec::new();
    rules::run(&doc, profile, base, &mut findings);
    findings.sort_by_key(|finding| finding.span.map_or(u32::MAX, |span| span.start));
    findings
}
