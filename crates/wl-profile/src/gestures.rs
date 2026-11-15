// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Gesture timing constants from the widget specification.

use serde::Deserialize;

use crate::error::ProfileError;

/// Gesture recognition parameters (spec section "Gestures").
///
/// # Examples
///
/// ```
/// let profile = wl_profile::profile_1().expect("valid");
/// assert_eq!(profile.gestures.long_press_ms, 500);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gestures {
    /// Hold duration in milliseconds (500).
    pub long_press_ms:           u32,
    /// Movement tolerance during a hold, in pixels (10).
    pub long_press_tolerance_px: u32,
    /// Maximum gap between two taps of a double tap, in milliseconds (300).
    pub double_tap_ms:           u32,
    /// Maximum distance between two taps of a double tap, in pixels (40).
    pub double_tap_px:           u32,
    /// Leading edge that never accepts gestures, in pixels (20).
    pub leading_unclaimable_px:  u32
}

impl Gestures {
    /// Checks that every timing and distance is positive.
    ///
    /// # Errors
    ///
    /// [`ProfileError`] if any gesture parameter is zero.
    pub fn validate(&self) -> Result<(), ProfileError> {
        let named = [
            ("long_press_ms", self.long_press_ms),
            ("long_press_tolerance_px", self.long_press_tolerance_px),
            ("double_tap_ms", self.double_tap_ms),
            ("double_tap_px", self.double_tap_px),
            ("leading_unclaimable_px", self.leading_unclaimable_px)
        ];
        for (name, value) in named {
            if value == 0 {
                return Err(ProfileError::new(format!(
                    "gestures.{name} must be positive"
                )));
            }
        }
        Ok(())
    }
}
