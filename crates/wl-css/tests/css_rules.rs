// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Integration tests for the CSS rule engine: every rule gets positive and
//! negative scenarios, plus span, offset and malformed-input robustness.

use masterror::AppResult;
use wl_css::check;
use wl_profile::Profile;

fn profile() -> AppResult<Profile> {
    wl_profile::profile_1()
}

fn ids(css: &str) -> AppResult<Vec<&'static str>> {
    Ok(check(css, 0, &profile()?)
        .iter()
        .map(|finding| finding.rule.id)
        .collect())
}

fn assert_rule(css: &str, id: &str) -> AppResult<()> {
    assert_eq!(ids(css)?, vec![id], "expected exactly {id} for: {css}");
    Ok(())
}

fn assert_clean(css: &str) -> AppResult<()> {
    assert_eq!(
        ids(css)?,
        Vec::<&'static str>::new(),
        "expected clean: {css}"
    );
    Ok(())
}

#[test]
fn tgw200_flags_height_units() -> AppResult<()> {
    assert_rule(".box { height: 50vh }", "TGW200")?;
    assert_rule(".box { min-height: 10dvh; }", "TGW200")?;
    assert_rule(".box { top: calc(100svh - 8px) }", "TGW200")?;
    Ok(())
}

#[test]
fn tgw200_accepts_supported_units() -> AppResult<()> {
    assert_clean(".box { height: 50px }")?;
    assert_clean(".box { height: 50% }")?;
    assert_clean(".box { width: 50vh }")?;
    assert_clean(".box { height: calc(100% - 8px) }")?;
    Ok(())
}

#[test]
fn tgw201_flags_scrollable_overflow() -> AppResult<()> {
    assert_rule(".box { overflow: auto }", "TGW201")?;
    assert_rule(".box { overflow-y: scroll }", "TGW201")?;
    Ok(())
}

#[test]
fn tgw201_accepts_hidden_overflow() -> AppResult<()> {
    assert_clean(".box { overflow: hidden }")?;
    assert_clean(".box { overflow-wrap: anywhere }")?;
    Ok(())
}

#[test]
fn tgw202_flags_sticky() -> AppResult<()> {
    assert_rule(".box { position: sticky; top: 0 }", "TGW202")?;
    assert_clean(".box { position: relative }")?;
    Ok(())
}

#[test]
fn tgw203_flags_logical_properties() -> AppResult<()> {
    assert_rule(".box { margin-inline-start: 4px }", "TGW203")?;
    assert_rule(".box { inline-size: 100px }", "TGW203")?;
    assert_rule(".box { inset-block-end: 0 }", "TGW203")?;
    assert_clean(".box { margin-left: 4px }")?;
    assert_clean(".box { width: 100px }")?;
    Ok(())
}

#[test]
fn tgw204_flags_multicolumn() -> AppResult<()> {
    assert_rule(".box { column-count: 2 }", "TGW204")?;
    assert_rule(".box { columns: 12em }", "TGW204")?;
    assert_clean(".box { column-gap: 8px }")?;
    assert_clean(".box { gap: 8px }")?;
    Ok(())
}

#[test]
fn tgw205_flags_unsupported_display() -> AppResult<()> {
    assert_rule(".box { display: table }", "TGW205")?;
    assert_rule(".box { display: list-item }", "TGW205")?;
    assert_clean(".box { display: flex }")?;
    assert_clean(".box { display: inline-block }")?;
    Ok(())
}

#[test]
fn tgw206_flags_3d_transforms() -> AppResult<()> {
    assert_rule(".box { transform: rotateX(10deg) }", "TGW206")?;
    assert_rule(".box { transform: translate3d(0, 0, 5px) }", "TGW206")?;
    assert_clean(".box { transform: rotate(10deg) translate(2px, 3px) }")?;
    Ok(())
}

#[test]
fn tgw207_flags_unsupported_filter_functions() -> AppResult<()> {
    assert_rule(".box { filter: blur(2px) saturate(2) }", "TGW207")?;
    assert_rule(".box { backdrop-filter: blur(4px) contrast(2) }", "TGW207")?;
    assert_clean(".box { filter: blur(2px) brightness(1.2) drop-shadow(0 0 1px red) }")?;
    assert_clean(".box { backdrop-filter: blur(4px) brightness(0.9) }")?;
    assert_clean(".box { filter: none }")?;
    Ok(())
}

#[test]
fn tgw208_flags_clip_mask_blend() -> AppResult<()> {
    assert_rule(".box { clip-path: circle(50%) }", "TGW208")?;
    assert_rule(".box { mask-image: linear-gradient(red, blue) }", "TGW208")?;
    assert_rule(".box { mix-blend-mode: multiply }", "TGW208")?;
    assert_clean(".box { overflow: hidden }")?;
    Ok(())
}

#[test]
fn tgw209_flags_text_effects() -> AppResult<()> {
    assert_rule(".box { text-shadow: 0 1px 2px red }", "TGW209")?;
    assert_rule(".box { background-clip: text }", "TGW209")?;
    assert_rule(".box { writing-mode: vertical-rl }", "TGW209")?;
    assert_clean(".box { background-clip: border-box }")?;
    assert_clean(".box { text-transform: uppercase }")?;
    Ok(())
}

#[test]
fn tgw210_flags_modern_color_functions() -> AppResult<()> {
    assert_rule("a { color: oklch(70% 0.1 200) }", "TGW210")?;
    assert_rule("a { color: color-mix(in srgb, red 50%, blue) }", "TGW210")?;
    assert_clean("a { color: rgb(10 20 30) }")?;
    assert_clean("a { color: hsl(120deg 50% 50%) }")?;
    Ok(())
}

#[test]
fn tgw211_flags_env_and_url_arguments() -> AppResult<()> {
    assert_rule(".box { padding-top: env(vendor-inset, 10px) }", "TGW211")?;
    assert_rule(
        ".box { background: url(https://evil.example/x.png) }",
        "TGW211"
    )?;
    assert_clean(".box { padding-top: env(safe-area-inset-top, 0px) }")?;
    assert_clean(".box { background: url(data:image/png;base64,AAAA) }")?;
    assert_clean(".box { fill: url(#grad) }")?;
    Ok(())
}

#[test]
fn tgw212_flags_unsupported_at_rules_and_nesting() -> AppResult<()> {
    assert_rule("@import url(other.css);", "TGW212")?;
    assert_rule("@font-face { font-weight: 400 }", "TGW212")?;
    assert_rule("@container (width > 10px) { a { color: red } }", "TGW212")?;
    assert_rule("a { b { color: red } }", "TGW212")?;
    assert_rule("a { &:hover { color: red } }", "TGW212")?;
    assert_clean("@media (width < 100px) { a { color: red } }")?;
    assert_clean("@keyframes spin { from { opacity: 0 } to { opacity: 1 } }")?;
    Ok(())
}

#[test]
fn tgw213_flags_unsupported_selector_pseudo_classes() -> AppResult<()> {
    assert_rule("a:focus { color: red }", "TGW213")?;
    assert_rule("div:has(p) { color: red }", "TGW213")?;
    assert_rule("p::first-line { color: red }", "TGW213")?;
    assert_rule("a:focus-visible { color: red }", "TGW213")?;
    assert_clean("a:hover { color: red }")?;
    assert_clean(".card > .title { color: red }")?;
    Ok(())
}

#[test]
fn tgw214_flags_unsupported_media_features() -> AppResult<()> {
    assert_rule("@media (hover: hover) { a { color: red } }", "TGW214")?;
    assert_rule(
        "@media (min-resolution: 2dppx) { a { color: red } }",
        "TGW214"
    )?;
    assert_rule("@media print { a { color: red } }", "TGW214")?;
    assert_clean("@media (width < 100px) { a { color: red } }")?;
    assert_clean("@media (prefers-color-scheme: dark) { a { color: red } }")?;
    Ok(())
}

#[test]
fn tgw215_flags_named_font_families() -> AppResult<()> {
    assert_rule("body { font-family: Roboto, sans-serif }", "TGW215")?;
    assert_rule("body { font-family: 'Roboto' }", "TGW215")?;
    assert_clean("body { font-family: system-ui, sans-serif }")?;
    assert_clean("body { font-family: ui-monospace, monospace }")?;
    assert_clean("body { font-family: inherit }")?;
    Ok(())
}

#[test]
fn tgw216_flags_unknown_properties() -> AppResult<()> {
    assert_rule("a { colr: red }", "TGW216")?;
    assert_rule("a { text-shaddow: 0 0 1px red }", "TGW216")?;
    assert_clean("a { color: red }")?;
    assert_clean("a { content: \"x\" }")?;
    assert_clean("a { float: left }")?;
    assert_clean("a { object-fit: cover }")?;
    assert_clean("a { --brand: red }")?;
    assert_clean("a { -webkit-transform: rotate(1deg) }")?;
    Ok(())
}

#[test]
fn tgw300_flags_rule_count() -> AppResult<()> {
    let rule = "r0 { color: red }";
    assert!(ids(&rule.repeat(401))?.contains(&"TGW300"));
    assert_eq!(ids(&rule.repeat(400))?, Vec::<&'static str>::new());
    Ok(())
}

#[test]
fn tgw301_flags_keyframes_count() -> AppResult<()> {
    let keyframes = "@keyframes k { from { opacity: 0 } to { opacity: 1 } }";
    assert!(ids(&keyframes.repeat(33))?.contains(&"TGW301"));
    assert_eq!(ids(&keyframes.repeat(32))?, Vec::<&'static str>::new());
    Ok(())
}

#[test]
fn spans_point_at_the_offending_token() -> AppResult<()> {
    let css = ".box { height: 50vh }";
    let findings = check(css, 0, &profile()?);
    let span = findings
        .first()
        .expect("one finding")
        .span
        .expect("has span");
    let start = usize::try_from(span.start).expect("fits");
    let end = usize::try_from(span.end).expect("fits");
    assert_eq!(&css[start..end], "50vh");
    Ok(())
}

#[test]
fn base_offset_shifts_spans() -> AppResult<()> {
    let css = ".box { height: 50vh }";
    let findings = check(css, 100, &profile()?);
    let span = findings
        .first()
        .expect("one finding")
        .span
        .expect("has span");
    assert_eq!(span.start, 115);
    assert_eq!(span.end, 119);
    Ok(())
}

#[test]
fn inline_style_attributes_are_supported() -> AppResult<()> {
    assert_rule("color: red; overflow: auto", "TGW201")?;
    assert_rule("height: 100vh", "TGW200")?;
    assert_clean("color: red; background: blue")?;
    Ok(())
}

#[test]
fn comments_and_strings_are_ignored() -> AppResult<()> {
    assert_clean("/* height: 50vh */ a { color: red }")?;
    assert_clean("a { content: \"overflow: auto\" }")?;
    assert_rule("a { content: '}'; height: 50vh }", "TGW200")?;
    Ok(())
}

#[test]
fn malformed_css_never_panics() {
    for css in [
        "",
        "}",
        "{",
        "a {",
        "a { color",
        "a { : red }",
        "@media",
        "@",
        "a { content: \"unterminated }",
        "a { /* unterminated",
        "a { color: rgb(",
        "::::",
        ";;;",
        "@keyframes",
        "a{b{c{d{color:red}}}}"
    ] {
        let _ = check(css, 0, &profile().expect("profile"));
    }
}

#[test]
fn findings_are_sorted_by_position() -> AppResult<()> {
    let css = "a { height: 50vh; overflow: auto; color: oklch(0% 0 0) }";
    let findings = check(css, 0, &profile()?);
    assert_eq!(findings.len(), 3);
    let starts: Vec<u32> = findings
        .iter()
        .map(|f| f.span.expect("span").start)
        .collect();
    let mut sorted = starts.clone();
    sorted.sort_unstable();
    assert_eq!(starts, sorted);
    Ok(())
}

#[test]
fn scanner_survives_strings_escapes_and_stray_semicolons() -> AppResult<()> {
    assert_clean(";p { color: red }")?;
    assert_clean("p:not(\"a\\\\b\") { color: red }")?;
    assert_clean("p { transform: translate(1px, 2px) }")?;
    assert_rule("p { font-family: local(x), serif }", "TGW215")?;
    Ok(())
}

#[test]
fn a_quoted_string_inside_a_function_call_does_not_shift_the_arguments() -> AppResult<()> {
    let plain = ids(".box { transform: translate(1px, 20px) }")?;
    let quoted = ids(".box { transform: translate(1px, \"a)b\") }")?;
    assert_eq!(plain, quoted);
    Ok(())
}
