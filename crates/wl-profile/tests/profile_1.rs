// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Positive and negative scenarios for profile loading and validation.

use wl_profile::Profile;

fn embedded() -> String {
    wl_profile::PROFILE_1_TOML.to_owned()
}

fn patched(needle: &str, replacement: &str) -> String {
    embedded().replace(needle, replacement)
}

#[test]
fn embedded_profile_is_valid() {
    let profile = Profile::from_toml(&embedded()).expect("embedded profile must parse");
    assert_eq!(profile.profile, 1);
    assert_eq!(profile.limits.html_bytes, 32768);
    assert_eq!(profile.limits.html_zlib9_bytes, 4096);
    assert_eq!(profile.limits.canvas_pixels, [2048, 2048]);
    assert_eq!(profile.gestures.long_press_ms, 500);
    assert!(
        profile
            .html
            .out
            .form_controls
            .iter()
            .any(|name| name == "input")
    );
    assert!(
        profile
            .js
            .privileged_calls
            .list
            .iter()
            .any(|name| name == "TelegramWidget.request")
    );
}

#[test]
fn rejects_unknown_field() {
    let source = patched("url_chars = 256", "url_chars = 256\nbogus_key = 1");
    let error = Profile::from_toml(&source).expect_err("unknown field must be rejected");
    assert!(error.to_string().contains("bogus_key"), "message: {error}");
}

#[test]
fn rejects_missing_required_field() {
    let source = patched("dom_nodes = 2000\n", "");
    assert!(Profile::from_toml(&source).is_err());
}

#[test]
fn rejects_unsupported_profile_number() {
    let source = patched("profile = 1", "profile = 2");
    let error = Profile::from_toml(&source).expect_err("profile 2 is unknown");
    assert!(error.to_string().contains("profile 2"), "message: {error}");
}

#[test]
fn rejects_zero_limit() {
    let source = patched("timers = 64", "timers = 0");
    let error = Profile::from_toml(&source).expect_err("zero limit must be rejected");
    assert!(error.to_string().contains("timers"), "message: {error}");
}

#[test]
fn rejects_zlib9_not_below_html_bytes() {
    let source = patched("html_zlib9_bytes = 4096", "html_zlib9_bytes = 40000");
    assert!(Profile::from_toml(&source).is_err());
}

#[test]
fn rejects_backdrop_filter_above_heavy_elements() {
    let source = patched("heavy_backdrop_filter = 2", "heavy_backdrop_filter = 99");
    assert!(Profile::from_toml(&source).is_err());
}

#[test]
fn rejects_duplicate_names() {
    let source = patched(
        "form_controls = [\n  \"input\",",
        "form_controls = [\n  \"input\",\n  \"input\","
    );
    let error = Profile::from_toml(&source).expect_err("duplicates must be rejected");
    assert!(error.to_string().contains("duplicates"), "message: {error}");
}

#[test]
fn rejects_uppercase_html_name() {
    let source = patched("\"input\",\n  \"select\",", "\"INPUT\",\n  \"select\",");
    let error = Profile::from_toml(&source).expect_err("uppercase name must be rejected");
    assert!(error.to_string().contains("INPUT"), "message: {error}");
}

#[test]
fn rejects_selector_without_colon() {
    let source = patched("\":has\",", "\"has\",");
    assert!(Profile::from_toml(&source).is_err());
}

#[test]
fn rejects_privileged_call_without_dot() {
    let source = patched("\"TelegramWidget.request\",", "\"request\",");
    assert!(Profile::from_toml(&source).is_err());
}

#[test]
fn rejects_empty_list() {
    let source = embedded().replace(
        "dialogs = [\"alert\", \"confirm\", \"prompt\", \"print\"]",
        "dialogs = []"
    );
    assert!(source != embedded(), "patch must apply");
    assert!(Profile::from_toml(&source).is_err());
}

#[test]
fn rejects_invalid_svg_element_name() {
    let source = patched("\"clipPath\",", "\"clip path!\",");
    let error = Profile::from_toml(&source).expect_err("invalid svg name must be rejected");
    assert!(
        error.to_string().contains("SVG element name"),
        "message: {error}"
    );
}

#[test]
fn rejects_js_identifier_with_whitespace() {
    let source = patched("\"fetch\",", "\"fetch now\",");
    let error = Profile::from_toml(&source).expect_err("whitespace id must be rejected");
    assert!(error.to_string().contains("whitespace"), "message: {error}");
}

#[test]
fn rejects_zero_gesture() {
    let source = patched("long_press_ms = 500", "long_press_ms = 0");
    let error = Profile::from_toml(&source).expect_err("zero gesture must be rejected");
    assert!(
        error.to_string().contains("long_press_ms"),
        "message: {error}"
    );
}
