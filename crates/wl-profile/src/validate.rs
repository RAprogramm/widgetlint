// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Shared validation helpers for profile name lists.

use crate::error::ProfileError;

/// Checks that `list` is non-empty and free of duplicates.
pub fn non_empty_unique(list: &[String], field: &str) -> Result<(), ProfileError> {
    if list.is_empty() {
        return Err(ProfileError::new(format!("{field} must not be empty")));
    }
    let mut sorted = list.to_vec();
    sorted.sort();
    if sorted.windows(2).any(|pair| pair.first() == pair.last()) {
        return Err(ProfileError::new(format!("{field} contains duplicates")));
    }
    Ok(())
}

/// Checks HTML/CSS names: lowercase ASCII letters, digits, `-` and `_`.
pub fn html_css_names(list: &[String], field: &str) -> Result<(), ProfileError> {
    non_empty_unique(list, field)?;
    for name in list {
        let valid = !name.is_empty()
            && name.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
            });
        if !valid {
            return Err(ProfileError::new(format!(
                "{field}: '{name}' is not a lowercase ASCII name"
            )));
        }
    }
    Ok(())
}

/// Checks SVG element names: ASCII letters in both cases, digits, `-` and `_`.
pub fn svg_names(list: &[String], field: &str) -> Result<(), ProfileError> {
    non_empty_unique(list, field)?;
    for name in list {
        let valid = !name.is_empty()
            && name.bytes().all(|byte| {
                byte.is_ascii_alphabetic() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
            });
        if !valid {
            return Err(ProfileError::new(format!(
                "{field}: '{name}' is not an SVG element name"
            )));
        }
    }
    Ok(())
}

/// Checks CSS selector tokens: start with `:` and contain no whitespace.
pub fn selector_tokens(list: &[String], field: &str) -> Result<(), ProfileError> {
    non_empty_unique(list, field)?;
    for name in list {
        if !name.starts_with(':') || name.chars().any(char::is_whitespace) {
            return Err(ProfileError::new(format!(
                "{field}: '{name}' is not a selector token"
            )));
        }
    }
    Ok(())
}

/// Checks JS identifiers: non-empty, unique, without whitespace.
pub fn js_identifiers(list: &[String], field: &str) -> Result<(), ProfileError> {
    non_empty_unique(list, field)?;
    for name in list {
        if name.chars().any(char::is_whitespace) {
            return Err(ProfileError::new(format!(
                "{field}: '{name}' contains whitespace"
            )));
        }
    }
    Ok(())
}

/// Checks dotted qualified calls such as `TelegramWidget.request`.
pub fn qualified_calls(list: &[String], field: &str) -> Result<(), ProfileError> {
    non_empty_unique(list, field)?;
    for name in list {
        if !name.contains('.') || name.chars().any(char::is_whitespace) {
            return Err(ProfileError::new(format!(
                "{field}: '{name}' is not a qualified call"
            )));
        }
    }
    Ok(())
}
