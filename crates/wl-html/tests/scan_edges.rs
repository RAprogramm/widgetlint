// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Scanner edge paths: unquoted values, slashes in tags, entity decoding and
//! raw-text regions that run into the end of the document.

use wl_html::parse;

fn attribute(source: &str, name: &str) -> wl_html::Attr {
    let doc = parse(source);
    doc.dom
        .nodes
        .iter()
        .find_map(|node| match &node.kind {
            wl_html::NodeKind::Element(element) => element.attr(name).cloned(),
            _ => None
        })
        .unwrap_or_else(|| unreachable!("no {name} attribute in {source}"))
}

#[test]
fn unquoted_attribute_values_are_scanned() {
    let id = attribute("<div id=x class=y>text</div>", "id");
    assert_eq!(id.value, "x");
    assert!(id.value_span.is_some(), "the value gets an exact span");
    let class = attribute("<div id=x class=y>text</div>", "class");
    assert_eq!(class.value, "y");
}

#[test]
fn slash_inside_a_tag_is_skipped() {
    let id = attribute("<div/ id=x>text</div>", "id");
    assert_eq!(id.value, "x", "the stray slash does not eat the attribute");
}

#[test]
fn entities_decode_in_attribute_values() {
    let title = attribute(
        "<div title=\"&lt;a&gt; &amp; &#65; &#x42; &quot;q&quot; &nbsp;&apos;\">x</div>",
        "title"
    );
    assert_eq!(title.value, "<a> & A B \"q\" \u{a0}'");
    assert!(
        title.value_span.is_some(),
        "decoded values still align with the source"
    );
}

#[test]
fn invalid_entities_stay_literal() {
    let title = attribute("<div title=\"&#x110000; &bogus &#xZZ;\">x</div>", "title");
    assert_eq!(
        title.value, "\u{fffd} &bogus &#xZZ;",
        "out-of-range references become the replacement character, the rest stays literal"
    );
    assert!(title.value_span.is_some(), "literal values still align");
}

#[test]
fn plaintext_runs_to_the_end_of_the_document() {
    let doc = parse("<plaintext>hello <b> not markup");
    assert!(
        !doc.dom.nodes.iter().any(|node| match &node.kind {
            wl_html::NodeKind::Element(element) => element.local() == "b",
            _ => false
        }),
        "everything after plaintext is text"
    );
}

#[test]
fn raw_text_with_a_truncated_close_tag_survives() {
    let doc = parse("<textarea>keep &lt; this</tex");
    assert!(
        doc.dom.nodes.iter().any(|node| match &node.kind {
            wl_html::NodeKind::Element(element) => element.local() == "textarea",
            _ => false
        }),
        "the textarea is still an element"
    );
}

#[test]
fn scanner_decodes_character_references() {
    let events = wl_html::scan::scan("<div title=\"&lt;a&gt; &amp; &#65; &#x42; &nbsp;\">x</div>");
    let title = events.iter().find_map(|event| match event {
        wl_html::ScanEvent::Start(tag) => tag.attrs.iter().find(|attr| attr.name == "title"),
        _ => None
    });
    let title = title.expect("the scanner sees the title attribute");
    assert_eq!(title.value, "<a> & A B \u{a0}");
}

#[test]
fn scanner_keeps_unknown_references_literal() {
    let events = wl_html::scan::scan("<div title=\"&#x110000; &bogus &#65;\">x</div>");
    let title = events.iter().find_map(|event| match event {
        wl_html::ScanEvent::Start(tag) => tag.attrs.iter().find(|attr| attr.name == "title"),
        _ => None
    });
    let title = title.expect("the scanner sees the title attribute");
    assert_eq!(title.value, "&#x110000; &bogus A");
}
