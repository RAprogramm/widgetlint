// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! JavaScript section: forbidden globals, canvas API, events, privileged calls.

use serde::Deserialize;

use crate::{error::ProfileError, validate};

/// JavaScript part of the profile.
///
/// # Examples
///
/// ```
/// let profile = wl_profile::profile_1().expect("valid");
/// assert!(profile.js.out.network.iter().any(|name| name == "fetch"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Js {
    /// Out lists of JavaScript APIs and events.
    pub out:              JsOut,
    /// Calls that require a real tap, one per tap.
    pub privileged_calls: PrivilegedCalls
}

impl Js {
    /// Validates all JavaScript lists.
    ///
    /// # Errors
    ///
    /// [`ProfileError`] if any JavaScript list is empty or malformed.
    pub fn validate(&self) -> Result<(), ProfileError> {
        self.out.validate()?;
        self.privileged_calls.validate()
    }
}

/// Out lists of JavaScript APIs (spec sections 8.5–8.6).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsOut {
    /// Network APIs that throw `NetworkDisabledError` (TGW400).
    pub network:        Vec<String>,
    /// Dialog functions with fixed results (TGW402).
    pub dialogs:        Vec<String>,
    /// Unsupported `navigator` members (TGW403).
    pub navigator:      Vec<String>,
    /// Miscellaneous unsupported APIs (TGW404).
    pub misc:           Vec<String>,
    /// Unsupported canvas methods (TGW406).
    pub canvas_methods: Vec<String>,
    /// Unsupported canvas properties (TGW406).
    pub canvas_props:   Vec<String>,
    /// Unsupported event types (TGW500–502).
    pub events:         Vec<String>
}

impl JsOut {
    /// Validates every JavaScript list.
    ///
    /// # Errors
    ///
    /// [`ProfileError`] if a list is empty or a name contains whitespace.
    pub fn validate(&self) -> Result<(), ProfileError> {
        validate::js_identifiers(&self.network, "js.out.network")?;
        validate::js_identifiers(&self.dialogs, "js.out.dialogs")?;
        validate::js_identifiers(&self.navigator, "js.out.navigator")?;
        validate::js_identifiers(&self.misc, "js.out.misc")?;
        validate::js_identifiers(&self.canvas_methods, "js.out.canvas_methods")?;
        validate::js_identifiers(&self.canvas_props, "js.out.canvas_props")?;
        validate::js_identifiers(&self.events, "js.out.events")
    }
}

/// Privileged calls requiring a synchronous tap handler (TGW503–505).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrivilegedCalls {
    /// Qualified call names such as `TelegramWidget.request`.
    pub list: Vec<String>
}

impl PrivilegedCalls {
    /// Validates the privileged call list.
    ///
    /// # Errors
    ///
    /// [`ProfileError`] if the list is empty or an entry is not a qualified
    /// dotted call.
    pub fn validate(&self) -> Result<(), ProfileError> {
        validate::qualified_calls(&self.list, "js.privileged_calls.list")
    }
}
