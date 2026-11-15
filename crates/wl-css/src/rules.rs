// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! CSS rule checks: profile-driven keyword rules, the unknown-property check
//! over the curated browser table and the stylesheet size limits.

use wl_core::{finding::Finding, rule::lookup, severity::Region, span::Span};
use wl_profile::Profile;

use crate::{
    known_properties,
    scan::{AtRule, CssDoc, Decl, Selector, functions, lengths, words}
};

/// Properties whose value may contain height-relative units (TGW200).
const LENGTH_PROPS: &[&str] = &[
    "height",
    "min-height",
    "max-height",
    "top",
    "bottom",
    "left",
    "right",
    "inset",
    "inset-top",
    "inset-bottom",
    "inset-left",
    "inset-right",
    "margin",
    "margin-top",
    "margin-bottom",
    "margin-left",
    "margin-right",
    "padding",
    "padding-top",
    "padding-bottom",
    "padding-left",
    "padding-right",
    "translate",
    "flex-basis",
    "line-height"
];

/// Properties checked for `overflow` rewrites (TGW201).
const OVERFLOW_PROPS: &[&str] = &["overflow", "overflow-x", "overflow-y"];

/// Logical property prefixes (TGW203).
const LOGICAL_PREFIXES: &[&str] = &[
    "margin-inline",
    "margin-block",
    "padding-inline",
    "padding-block",
    "inset-inline",
    "inset-block",
    "border-inline",
    "border-block",
    "scroll-margin-inline",
    "scroll-margin-block",
    "scroll-padding-inline",
    "scroll-padding-block",
    "contain-intrinsic-inline-size",
    "contain-intrinsic-block-size"
];

/// Logical size properties (TGW203).
const LOGICAL_EXACT: &[&str] = &[
    "inline-size",
    "block-size",
    "min-inline-size",
    "min-block-size",
    "max-inline-size",
    "max-block-size"
];

/// Unsupported `display` values (TGW205).
const UNSUPPORTED_DISPLAY: &[&str] = &[
    "table",
    "inline-table",
    "table-caption",
    "table-column-group",
    "table-header-group",
    "table-footer-group",
    "table-row-group",
    "table-row",
    "table-cell",
    "table-column",
    "list-item",
    "flow-root",
    "run-in",
    "ruby",
    "ruby-base-container",
    "ruby-base",
    "ruby-text-container",
    "ruby-text",
    "math"
];

/// 3D transform functions (TGW206).
const TRANSFORM_3D: &[&str] = &[
    "rotate3d",
    "rotatex",
    "rotatey",
    "translate3d",
    "translatez",
    "scale3d",
    "scalez",
    "matrix3d",
    "perspective"
];

/// Clipping, masking and blending properties (TGW208).
const CLIP_BLEND_PROPS: &[&str] = &[
    "clip",
    "clip-path",
    "mix-blend-mode",
    "background-blend-mode",
    "isolation"
];

/// Text effect properties (TGW209).
const TEXT_EFFECT_PROPS: &[&str] = &[
    "text-shadow",
    "-webkit-line-clamp",
    "writing-mode",
    "text-orientation"
];

/// CSS-wide keywords that may appear in a font list (TGW215).
const CSS_WIDE_KEYWORDS: &[&str] = &["inherit", "initial", "unset", "revert", "revert-layer"];

/// Runs every CSS check over one scanned block and returns findings sorted by
/// position.
pub fn run(doc: &CssDoc<'_>, profile: &Profile, base: u32, out: &mut Vec<Finding>) {
    for decl in &doc.decls {
        check_units(decl, profile, base, out);
        check_overflow(decl, profile, base, out);
        check_position(decl, profile, base, out);
        check_logical(decl, base, out);
        check_columns(decl, base, out);
        check_display(decl, base, out);
        check_transforms(decl, base, out);
        check_filters(decl, profile, base, out);
        check_clip_blend(decl, base, out);
        check_text_effects(decl, base, out);
        check_color_functions(decl, profile, base, out);
        check_env_url(decl, base, out);
        check_fonts(decl, profile, base, out);
        check_unknown_property(decl, base, out);
    }
    for selector in &doc.selectors {
        check_selectors(selector, profile, base, out);
    }
    for at_rule in &doc.at_rules {
        check_at_rules(at_rule, profile, base, out);
        check_media_features(at_rule, profile, base, out);
    }
    check_limits(doc, profile, out);
}

fn push(
    out: &mut Vec<Finding>,
    id: &str,
    span: Option<Span>,
    base: u32,
    message: impl Into<String>
) {
    let Some(meta) = lookup(id) else {
        return;
    };
    let mut finding = Finding::new(meta, message);
    if let Some(span) = span {
        finding = finding.with_span(
            Span::new(
                span.start.saturating_add(base),
                span.end.saturating_add(base)
            ),
            Region::Css
        );
    } else {
        finding.region = Region::Css;
    }
    out.push(finding);
}

fn in_list(list: &[String], name: &str) -> bool {
    list.iter().any(|item| item.eq_ignore_ascii_case(name))
}

const fn shift(span: Span, offset: u32) -> Span {
    Span::new(
        span.start.saturating_add(offset),
        span.end.saturating_add(offset)
    )
}

/// Turns a span relative to a declaration value into a span in the block.
const fn vspan(decl: &Decl<'_>, span: Span) -> Span {
    shift(span, decl.value_span.start)
}

fn check_units(decl: &Decl<'_>, profile: &Profile, base: u32, out: &mut Vec<Finding>) {
    if !LENGTH_PROPS
        .iter()
        .any(|prop| decl.prop.eq_ignore_ascii_case(prop))
    {
        return;
    }
    for (token, span) in lengths(decl.value) {
        let unit = token.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.');
        if in_list(&profile.css.out.units_height, unit) {
            push(
                out,
                "TGW200",
                Some(vspan(decl, span)),
                base,
                format!("height-relative unit `{unit}` is unsupported")
            );
        }
    }
}

fn check_overflow(decl: &Decl<'_>, profile: &Profile, base: u32, out: &mut Vec<Finding>) {
    if !OVERFLOW_PROPS
        .iter()
        .any(|prop| decl.prop.eq_ignore_ascii_case(prop))
    {
        return;
    }
    for (word, span) in words(decl.value) {
        if in_list(&profile.css.out.overflow_to_hidden, word) {
            push(
                out,
                "TGW201",
                Some(vspan(decl, span)),
                base,
                format!("`{word}` is rewritten to `hidden`: widgets do not scroll")
            );
        }
    }
}

fn check_position(decl: &Decl<'_>, profile: &Profile, base: u32, out: &mut Vec<Finding>) {
    if !decl.prop.eq_ignore_ascii_case("position") {
        return;
    }
    for (word, span) in words(decl.value) {
        if in_list(&profile.css.out.position_to_relative, word) {
            push(
                out,
                "TGW202",
                Some(vspan(decl, span)),
                base,
                format!("`position: {word}` is rewritten to `relative`")
            );
        }
    }
}

fn check_logical(decl: &Decl<'_>, base: u32, out: &mut Vec<Finding>) {
    let prop = decl.prop.to_ascii_lowercase();
    let logical = LOGICAL_EXACT.contains(&prop.as_str())
        || LOGICAL_PREFIXES
            .iter()
            .any(|prefix| prop.starts_with(prefix));
    if logical {
        push(
            out,
            "TGW203",
            Some(decl.prop_span),
            base,
            format!("logical property `{}` is unsupported", decl.prop)
        );
    }
}

fn check_columns(decl: &Decl<'_>, base: u32, out: &mut Vec<Finding>) {
    let prop = decl.prop.to_ascii_lowercase();
    let multicolumn = prop == "columns" || (prop.starts_with("column-") && prop != "column-gap");
    if multicolumn {
        push(
            out,
            "TGW204",
            Some(decl.prop_span),
            base,
            format!("multi-column property `{}` is unsupported", decl.prop)
        );
    }
}

fn check_display(decl: &Decl<'_>, base: u32, out: &mut Vec<Finding>) {
    if !decl.prop.eq_ignore_ascii_case("display") {
        return;
    }
    for (word, span) in words(decl.value) {
        if UNSUPPORTED_DISPLAY
            .iter()
            .any(|value| value.eq_ignore_ascii_case(word))
        {
            push(
                out,
                "TGW205",
                Some(vspan(decl, span)),
                base,
                format!("`display: {word}` is unsupported")
            );
        }
    }
}

fn check_transforms(decl: &Decl<'_>, base: u32, out: &mut Vec<Finding>) {
    if !decl.prop.eq_ignore_ascii_case("transform") {
        return;
    }
    for func in functions(decl.value) {
        if TRANSFORM_3D
            .iter()
            .any(|name| name.eq_ignore_ascii_case(func.name))
        {
            push(
                out,
                "TGW206",
                Some(vspan(decl, func.name_span)),
                base,
                format!("3D transform `{}` is unsupported", func.name)
            );
        }
    }
}

fn check_filters(decl: &Decl<'_>, profile: &Profile, base: u32, out: &mut Vec<Finding>) {
    let allowed: Option<&[String]> = if decl.prop.eq_ignore_ascii_case("filter") {
        Some(&profile.css.out.filter_functions_allowed)
    } else if decl.prop.eq_ignore_ascii_case("backdrop-filter") {
        Some(&profile.css.out.backdrop_filter_allowed)
    } else {
        None
    };
    let Some(allowed) = allowed else {
        return;
    };
    for func in functions(decl.value) {
        if func.name.eq_ignore_ascii_case("url") || in_list(allowed, func.name) {
            continue;
        }
        push(
            out,
            "TGW207",
            Some(vspan(decl, func.name_span)),
            base,
            format!("`{}()` is not allowed on `{}`", func.name, decl.prop)
        );
    }
}

fn check_clip_blend(decl: &Decl<'_>, base: u32, out: &mut Vec<Finding>) {
    let prop = decl.prop.to_ascii_lowercase();
    let flagged = CLIP_BLEND_PROPS.contains(&prop.as_str()) || prop.starts_with("mask");
    if flagged {
        push(
            out,
            "TGW208",
            Some(decl.prop_span),
            base,
            format!("`{}` is unsupported", decl.prop)
        );
    }
}

fn check_text_effects(decl: &Decl<'_>, base: u32, out: &mut Vec<Finding>) {
    let prop = decl.prop.to_ascii_lowercase();
    if TEXT_EFFECT_PROPS.contains(&prop.as_str()) {
        push(
            out,
            "TGW209",
            Some(decl.prop_span),
            base,
            format!("`{}` is unsupported", decl.prop)
        );
        return;
    }
    if prop == "background-clip"
        && words(decl.value)
            .into_iter()
            .any(|(word, _)| word.eq_ignore_ascii_case("text"))
    {
        push(
            out,
            "TGW209",
            Some(decl.value_span),
            base,
            "`background-clip: text` is unsupported"
        );
    }
}

fn check_color_functions(decl: &Decl<'_>, profile: &Profile, base: u32, out: &mut Vec<Finding>) {
    for func in functions(decl.value) {
        if in_list(&profile.css.out.color_functions, func.name) {
            push(
                out,
                "TGW210",
                Some(vspan(decl, func.name_span)),
                base,
                format!("color function `{}()` is unsupported", func.name)
            );
        }
    }
}

fn check_env_url(decl: &Decl<'_>, base: u32, out: &mut Vec<Finding>) {
    for func in functions(decl.value) {
        if func.name.eq_ignore_ascii_case("env") {
            let name = func.args.split(',').next().unwrap_or("").trim();
            if !name.starts_with("safe-area-inset-") {
                push(
                    out,
                    "TGW211",
                    Some(vspan(decl, func.args_span)),
                    base,
                    format!("`env({name})` is unsupported: only safe-area-inset-* works")
                );
            }
        }
        if func.name.eq_ignore_ascii_case("url") {
            let target = func.args.trim().trim_matches(['\'', '"']);
            if !target.starts_with("data:") && !target.starts_with('#') {
                push(
                    out,
                    "TGW211",
                    Some(vspan(decl, func.args_span)),
                    base,
                    format!("`url({target})` is unsupported: only data: images and #fragments")
                );
            }
        }
    }
}

fn check_fonts(decl: &Decl<'_>, profile: &Profile, base: u32, out: &mut Vec<Finding>) {
    if !decl.prop.eq_ignore_ascii_case("font-family") {
        return;
    }
    for family in split_commas(decl.value) {
        let name = family.trim().trim_matches(['\'', '"']);
        if name.is_empty()
            || CSS_WIDE_KEYWORDS
                .iter()
                .any(|keyword| keyword.eq_ignore_ascii_case(name))
            || in_list(&profile.css.out.font_generics_allowed, name)
        {
            continue;
        }
        push(
            out,
            "TGW215",
            Some(decl.value_span),
            base,
            format!("font family `{name}` is not available in widgets")
        );
    }
}

fn check_unknown_property(decl: &Decl<'_>, base: u32, out: &mut Vec<Finding>) {
    if decl.prop.starts_with("--") {
        return;
    }
    if !known_properties::is_known(&decl.prop.to_ascii_lowercase()) {
        push(
            out,
            "TGW216",
            Some(decl.prop_span),
            base,
            format!(
                "property `{}` is not recognized by the CSS engine",
                decl.prop
            )
        );
    }
}

fn check_selectors(selector: &Selector<'_>, profile: &Profile, base: u32, out: &mut Vec<Finding>) {
    if selector.nested {
        push(
            out,
            "TGW212",
            Some(selector.span),
            base,
            "nested CSS is unsupported"
        );
    }
    let lower = selector.text.to_ascii_lowercase();
    let mut best: Option<&str> = None;
    for token in &profile.css.out.selectors {
        let token_lower = token.to_ascii_lowercase();
        if lower.contains(&token_lower) && best.is_none_or(|b: &str| token_lower.len() > b.len()) {
            best = Some(token);
        }
    }
    if let Some(token) = best {
        push(
            out,
            "TGW213",
            Some(selector.span),
            base,
            format!("selector `{token}` is unsupported")
        );
    }
}

fn check_at_rules(at_rule: &AtRule<'_>, profile: &Profile, base: u32, out: &mut Vec<Finding>) {
    if in_list(&profile.css.out.at_rules, at_rule.name) {
        push(
            out,
            "TGW212",
            Some(at_rule.name_span),
            base,
            format!("@{} is unsupported", at_rule.name)
        );
    }
}

fn check_media_features(
    at_rule: &AtRule<'_>,
    profile: &Profile,
    base: u32,
    out: &mut Vec<Finding>
) {
    if !at_rule.name.eq_ignore_ascii_case("media") {
        return;
    }
    let mut reported: Vec<&str> = Vec::new();
    for (word, span) in words(at_rule.prelude) {
        let lower = word.to_ascii_lowercase();
        for feature in &profile.css.out.media_unsupported {
            let feature_lower = feature.to_ascii_lowercase();
            if lower.contains(&feature_lower) && !reported.contains(&feature.as_str()) {
                reported.push(feature);
                push(
                    out,
                    "TGW214",
                    Some(shift(span, at_rule.prelude_span.start)),
                    base,
                    format!("@media feature `{feature}` is unsupported")
                );
            }
        }
    }
}

fn check_limits(doc: &CssDoc<'_>, profile: &Profile, out: &mut Vec<Finding>) {
    if doc.rule_count > profile.limits.css_rules {
        push(
            out,
            "TGW300",
            None,
            0,
            format!(
                "stylesheet has {} CSS rules, the limit is {}",
                doc.rule_count, profile.limits.css_rules
            )
        );
    }
    if doc.keyframes_count > profile.limits.css_keyframes {
        push(
            out,
            "TGW301",
            None,
            0,
            format!(
                "stylesheet has {} @keyframes rules, the limit is {}",
                doc.keyframes_count, profile.limits.css_keyframes
            )
        );
    }
}

/// Splits a comma-separated list at top level (parenthesis aware).
fn split_commas(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut parts = Vec::new();
    let mut start = 0_usize;
    let mut parens = 0_usize;
    let mut i = 0_usize;
    while i < bytes.len() {
        match bytes.get(i).copied().unwrap_or(0) {
            b'(' => parens += 1,
            b')' => parens = parens.saturating_sub(1),
            b',' if parens == 0 => {
                parts.push(&text[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    parts.push(&text[start..]);
    parts
}
