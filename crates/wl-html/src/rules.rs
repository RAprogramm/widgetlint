// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! HTML rule checks driven by the profile out-lists and the static limits.

use wl_core::{finding::Finding, rule::lookup, severity::Region, span::Span};
use wl_profile::Profile;

use crate::{
    HtmlDoc,
    dom::{Attr, Element},
    scan::ScanEvent
};

/// Legacy presentational attributes (TGW108).
const LEGACY_ATTRS: &[&str] = &[
    "align",
    "valign",
    "bgcolor",
    "border",
    "cellpadding",
    "cellspacing",
    "color",
    "face",
    "nowrap"
];

/// `data:` image MIME types the client renders (TGW106).
const SUPPORTED_IMAGE_MIME: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/jpg",
    "image/gif",
    "image/webp"
];

/// Attributes that hint at text input intent on any element (TGW114).
const INPUT_HINT_ATTRS: &[&str] = &[
    "placeholder",
    "pattern",
    "maxlength",
    "minlength",
    "autocomplete",
    "required"
];

/// Structural names always understood even though the in list is partial
/// (TGW110).
const STRUCTURAL_ELEMENTS: &[&str] = &[
    "html", "head", "body", "script", "style", "title", "wbr", "noscript", "thead", "tbody",
    "tfoot", "tr", "td", "th", "caption", "col", "colgroup", "svg", "g", "path", "rect", "circle",
    "ellipse", "line", "polyline", "polygon", "text", "tspan", "defs", "use", "symbol", "font"
];

/// Runs every HTML check over one parsed document and returns findings sorted
/// by position.
#[must_use]
pub fn run(doc: &HtmlDoc, source: &str, profile: &Profile) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut element_count = 0_u32;
    for (_, element) in doc.dom.elements() {
        element_count += 1;
        check_element(element, profile, &mut findings);
        for attr in &element.attrs {
            check_attribute(element, attr, profile, &mut findings);
        }
    }
    check_limits(doc, source, profile, element_count, &mut findings);
    findings.sort_by_key(|finding| finding.span.map_or(u32::MAX, |span| span.start));
    findings
}

fn push(findings: &mut Vec<Finding>, id: &str, span: Option<Span>, message: impl Into<String>) {
    let Some(meta) = lookup(id) else {
        return;
    };
    let mut finding = Finding::new(meta, message);
    if let Some(span) = span {
        finding = finding.with_span(span, Region::Html);
    } else {
        finding.region = Region::Html;
    }
    findings.push(finding);
}

fn in_list(list: &[String], name: &str) -> bool {
    list.iter().any(|item| item.eq_ignore_ascii_case(name))
}

fn name_span(element: &Element) -> Option<Span> {
    element.name_span.or(element.span)
}

fn check_element(element: &Element, profile: &Profile, findings: &mut Vec<Finding>) {
    let name = element.local();
    let span = name_span(element);
    let out = &profile.html.out;
    if in_list(&out.form_controls, name) {
        push(
            findings,
            "TGW100",
            span,
            format!("<{name}> is removed together with its content")
        );
    }
    if name.eq_ignore_ascii_case("form") {
        push(
            findings,
            "TGW101",
            span,
            format!(
                "`<form>` degrades to a `{}` box, form semantics do not work",
                profile.html.degrade.form
            )
        );
    }
    if name.eq_ignore_ascii_case("label") {
        push(
            findings,
            "TGW101",
            span,
            format!(
                "`<label>` degrades to an `{}` box and does not activate controls",
                profile.html.degrade.label
            )
        );
    }
    if in_list(&out.media_embed, name) {
        push(
            findings,
            "TGW102",
            span,
            format!("<{name}> is unsupported in widgets")
        );
    }
    if in_list(&out.service, name) {
        push(
            findings,
            "TGW103",
            span,
            format!("<{name}> is unsupported in widgets")
        );
    }
    if name.eq_ignore_ascii_case("script") && element.has_attr("src") {
        push(
            findings,
            "TGW104",
            element.attr("src").and_then(|attr| attr.span),
            "external <script src> never executes in widgets"
        );
    }
    check_image(element, span, findings);
    if in_list(&out.legacy, name) {
        push(
            findings,
            "TGW108",
            span,
            format!("<{name}> is a legacy presentational element")
        );
    }
    if in_list(&profile.html.svg.elements, name) {
        push(
            findings,
            "TGW150",
            span,
            format!("<{name}> is an unsupported SVG element")
        );
    }
    if name.eq_ignore_ascii_case("a") {
        check_link(element, findings);
    }
    if !is_known_element(name, profile) {
        push(
            findings,
            "TGW110",
            span,
            format!("<{name}> is unknown to the widget engine and renders as an inline box")
        );
    }
}

fn check_image(element: &Element, span: Option<Span>, findings: &mut Vec<Finding>) {
    if !element.local().eq_ignore_ascii_case("img") {
        return;
    }
    let Some(src) = element.attr("src") else {
        return;
    };
    let value = src.value.trim();
    if !value.to_ascii_lowercase().starts_with("data:") {
        push(
            findings,
            "TGW105",
            src.span,
            "only `data:` images render; the request is never made"
        );
        return;
    }
    let mime = value[5..]
        .split([';', ','])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !SUPPORTED_IMAGE_MIME.contains(&mime.as_str()) {
        push(
            findings,
            "TGW106",
            src.span.or(span),
            format!("`data:` image type `{mime}` is not rendered, use PNG, JPEG, GIF or WebP")
        );
    }
}

fn check_link(element: &Element, findings: &mut Vec<Finding>) {
    let Some(href) = element.attr("href") else {
        return;
    };
    let value = href.value.trim();
    if value.is_empty()
        || value.starts_with('#')
        || value.to_ascii_lowercase().starts_with("https://")
    {
        return;
    }
    push(
        findings,
        "TGW111",
        href.span,
        format!("`href=\"{value}\"` is ignored: only https: links and #fragments work")
    );
}

/// Whether the name is covered by a dedicated rule or a profile list, so
/// TGW110 stays silent about it.
fn is_known_element(name: &str, profile: &Profile) -> bool {
    let out = &profile.html.out;
    STRUCTURAL_ELEMENTS
        .iter()
        .any(|known| known.eq_ignore_ascii_case(name))
        || in_list(&profile.html.in_partial.elements, name)
        || in_list(&out.form_controls, name)
        || in_list(&out.media_embed, name)
        || in_list(&out.service, name)
        || in_list(&out.legacy, name)
        || in_list(&profile.html.svg.elements, name)
        || name.eq_ignore_ascii_case("form")
        || name.eq_ignore_ascii_case("label")
}

fn check_attribute(
    element: &Element,
    attr: &Attr,
    profile: &Profile,
    findings: &mut Vec<Finding>
) {
    let local = attr.local();
    if local.eq_ignore_ascii_case("popover") {
        push(
            findings,
            "TGW102",
            attr.span,
            "the `popover` attribute is unsupported in widgets"
        );
    }
    if LEGACY_ATTRS.contains(&local) {
        push(
            findings,
            "TGW108",
            attr.span,
            format!("the `{local}` attribute is a legacy presentational attribute")
        );
    }
    if in_list(&profile.html.out.attrs, local) {
        push(
            findings,
            "TGW109",
            attr.span,
            format!("the `{local}` attribute has no effect in widgets")
        );
    }
    if local.starts_with("aria-") {
        push(
            findings,
            "TGW113",
            attr.span,
            format!("`{local}` is kept but has no effect; repeat it in the caption")
        );
    }
    if INPUT_HINT_ATTRS.contains(&local) {
        push(
            findings,
            "TGW114",
            attr.span,
            format!("`{local}` implies text input, but widgets have no keyboard")
        );
    }
    if let Some(span) = attr.span
        && u64::from(span.len()) > u64::from(profile.limits.attribute_units) * 2
    {
        push(
            findings,
            "TGW302",
            Some(span),
            format!(
                "attribute `{local}` on <{}> exceeds the {} UTF-16 unit limit",
                element.local(),
                profile.limits.attribute_units
            )
        );
    }
}

fn check_limits(
    doc: &HtmlDoc,
    source: &str,
    profile: &Profile,
    element_count: u32,
    findings: &mut Vec<Finding>
) {
    if element_count > profile.limits.dom_nodes {
        push(
            findings,
            "TGW303",
            None,
            format!(
                "document has {element_count} elements, the limit is {}",
                profile.limits.dom_nodes
            )
        );
    }
    let longest = longest_text(doc, source);
    if longest > u64::from(profile.limits.text_value_units) {
        push(
            findings,
            "TGW304",
            None,
            format!(
                "a text value has {longest} UTF-16 units, the limit is {}",
                profile.limits.text_value_units
            )
        );
    }
}

/// Longest text run between markup, in UTF-16 units.
fn longest_text(doc: &HtmlDoc, source: &str) -> u64 {
    doc.events
        .iter()
        .filter_map(|event| match event {
            ScanEvent::Text {
                span
            } => Some(*span),
            _ => None
        })
        .map(|span| {
            let from = usize::try_from(span.start).unwrap_or(source.len());
            let to = usize::try_from(span.end).unwrap_or(source.len());
            utf16_units(source.get(from..to).unwrap_or_default())
        })
        .max()
        .unwrap_or_default()
}

fn utf16_units(text: &str) -> u64 {
    text.chars()
        .map(|ch| u64::from(u32::from(ch) > 0xFFFF))
        .sum::<u64>()
        + u64::try_from(text.chars().count()).unwrap_or(u64::MAX)
}
