// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Parsing, span alignment and extraction: positive and negative scenarios.

use wl_core::Span;
use wl_html::{
    ScanEvent,
    extract::{comments, event_handlers, inline_styles, script_blocks, style_blocks},
    parse, scan
};

fn slice(source: &str, span: Span) -> &str {
    let start = usize::try_from(span.start).unwrap_or(0);
    let end = usize::try_from(span.end).unwrap_or(source.len());
    &source[start..end]
}

fn first_element(source: &str) -> Option<wl_html::Element> {
    let doc = parse(source);
    doc.dom
        .elements()
        .next()
        .map(|(_, element)| element.clone())
}

#[test]
fn element_and_attribute_spans() {
    let source = "<div id=\"a\" class='b'>x</div>";
    let element = first_element(source).expect("at least one element");
    assert_eq!(element.local(), "div");
    assert_eq!(element.span.expect("span").start, 0);
    assert_eq!(element.span.expect("span").end, 22);
    let id = element.attr("id").expect("id attribute");
    assert_eq!(id.value, "a");
    assert_eq!(id.value_span.expect("value span").start, 9);
    let class = element.attr("class").expect("class attribute");
    assert_eq!(class.value, "b");
    assert_eq!(class.value_span.expect("value span").start, 19);
}

#[test]
fn script_raw_text_is_not_parsed_as_markup() {
    let source = "<script>if (a < b) { f(\"</div>\"); }</script>";
    let doc = parse(source);
    let scripts: Vec<_> = doc
        .dom
        .elements()
        .filter(|(_, el)| el.local() == "script")
        .collect();
    assert_eq!(scripts.len(), 1);
    let (_, script) = scripts[0];
    let content = script.raw_text.expect("raw text span");
    assert_eq!(slice(source, content), "if (a < b) { f(\"</div>\"); }");
    assert!(doc.dom.elements().all(|(_, el)| el.local() != "div"));
}

#[test]
fn style_block_extraction() {
    let source = "<style>.a { color: red }</style>";
    let doc = parse(source);
    let blocks = style_blocks(&doc.dom);
    assert_eq!(blocks.len(), 1);
    assert_eq!(slice(source, blocks[0].content), ".a { color: red }");
}

#[test]
fn script_src_and_inline_content() {
    let doc = parse("<script src=\"https://cdn.example/x.js\"></script><script>var a=1</script>");
    let blocks = script_blocks(&doc.dom);
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].src.as_deref(), Some("https://cdn.example/x.js"));
    assert!(blocks[0].src_span.is_some());
    assert_eq!(
        blocks[1]
            .content
            .map(|s| usize::try_from(s.len()).expect("small")),
        Some(7)
    );
}

#[test]
fn inline_style_and_event_handlers() {
    let doc = parse("<p style=\"color:red\" onclick=\"go()\" ONMOUSEOVER=\"h()\">t</p>");
    let styles = inline_styles(&doc.dom);
    assert_eq!(styles.len(), 1);
    assert_eq!(styles[0].value, "color:red");
    assert!(styles[0].value_span.is_some());
    let handlers = event_handlers(&doc.dom);
    assert_eq!(handlers.len(), 2);
    assert_eq!(handlers[0].name, "onclick");
    assert_eq!(handlers[1].name, "onmouseover");
}

#[test]
fn comments_carry_directive_text_and_spans() {
    let source = "<!-- tgw-disable-file TGW606 --><div></div>";
    let doc = parse(source);
    let list = comments(&doc.dom);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].text.trim(), "tgw-disable-file TGW606");
    let text_span = list[0].text_span.expect("text span");
    assert_eq!(slice(source, text_span).trim(), "tgw-disable-file TGW606");
}

#[test]
fn nested_same_name_elements_get_ordered_spans() {
    let source = "<div><div>inner</div></div>";
    let doc = parse(source);
    let spans: Vec<_> = doc
        .dom
        .elements()
        .map(|(_, el)| el.span.expect("span").start)
        .collect();
    assert_eq!(spans, vec![0, 5]);
}

#[test]
fn uppercase_tags_and_self_closing_are_lowercased() {
    let element = first_element("<DIV/><BR>").expect("at least one element");
    assert_eq!(element.local(), "div");
    let doc = parse("<DIV/><BR>");
    let names: Vec<_> = doc
        .dom
        .elements()
        .map(|(_, el)| el.local().to_owned())
        .collect();
    assert_eq!(names, vec!["div", "br"]);
}

#[test]
fn entity_decoding_in_attribute_values() {
    let element =
        first_element("<a href=\"x?a=1&amp;b=2&#39;&gt;\">t</a>").expect("at least one element");
    assert_eq!(element.attr("href").expect("href").value, "x?a=1&b=2'>");
}

#[test]
fn line_numbers_follow_newlines() {
    let doc = parse("<div>\n  <span>\n    <b>x</b>\n  </span>\n</div>");
    let lines: Vec<_> = doc.dom.elements().map(|(_, el)| el.line).collect();
    assert_eq!(lines, vec![1, 2, 3]);
}

#[test]
fn malformed_markup_never_panics() {
    for source in [
        "<div <p>text",
        "<a href=",
        "<script>unclosed",
        "<textarea><textarea></textarea>",
        "</div></span>",
        "<<<>>>",
        "<!-- unterminated",
        "<div id=>x</div>",
        "<",
        "<!",
        "<?php echo 1; ?>",
        "<style>a{"
    ] {
        let doc = parse(source);
        let _ = doc.dom.elements().count();
    }
}

#[test]
fn scanner_events_cover_whole_source() {
    let source = "<div id=\"x\">text<!--c--></div>";
    let events = scan::scan(source);
    let covered: usize = events
        .iter()
        .map(|event| match event {
            ScanEvent::Start(tag) => usize::try_from(tag.span.len()).expect("small"),
            ScanEvent::End {
                span, ..
            }
            | ScanEvent::Comment {
                span, ..
            }
            | ScanEvent::Doctype {
                span
            }
            | ScanEvent::Text {
                span
            } => usize::try_from(span.len()).expect("small")
        })
        .sum();
    assert_eq!(covered, source.len());
}

#[test]
fn textarea_content_is_raw_text() {
    let source = "<textarea><b>not bold</b></textarea>";
    let doc = parse(source);
    assert!(doc.dom.elements().all(|(_, el)| el.local() != "b"));
    let textarea = doc
        .dom
        .elements()
        .find(|(_, el)| el.local() == "textarea")
        .expect("textarea");
    assert_eq!(
        slice(source, textarea.1.raw_text.expect("raw")),
        "<b>not bold</b>"
    );
}

#[test]
fn html_body_wrapper_is_folded_into_context() {
    let doc = parse("<html><body><p>x</p></body></html>");
    let names: Vec<_> = doc
        .dom
        .elements()
        .map(|(_, el)| el.local().to_owned())
        .collect();
    assert_eq!(names, vec!["p"]);
}
