// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! HTML5 fragment parsing with exact source spans.
//!
//! [`html5ever`] guarantees HTML5-correct tree structure but emits no
//! positions. This crate pairs it with a lightweight scanner ([`scan`]) that
//! records byte spans, and aligns the two in document order ([`align`]), so
//! every element, attribute and comment gets both a spec-correct tree and
//! exact source positions.

pub mod align;
pub mod dom;
pub mod extract;
pub mod scan;
pub mod sink;

use html5ever::{
    QualName,
    driver::{self, ParseOpts}
};
use tendril::TendrilSink;

pub use crate::{
    dom::{Attr, Dom, Element, Node, NodeId, NodeKind, Preorder},
    extract::{CommentInfo, InlineAttr, ScriptBlock, StyleBlock},
    scan::{ScanAttr, ScanEvent, StartTag},
    sink::HtmlSink
};

/// Parsed document: aligned arena DOM plus the raw scanner events.
#[derive(Debug)]
pub struct HtmlDoc {
    /// The aligned DOM.
    pub dom:    Dom,
    /// Scanner events in document order.
    pub events: Vec<ScanEvent>
}

/// Parses `source` as an HTML5 fragment in a `<div>` context and aligns
/// source spans onto the tree.
///
/// # Examples
///
/// ```
/// let doc = wl_html::parse("<div id='x'>hi</div>");
/// let (id, element) = doc.dom.elements().next().expect("one element");
/// assert_eq!(element.local(), "div");
/// assert_eq!(element.attr("id").expect("id").value, "x");
/// assert!(element.span.is_some());
/// let _ = id;
/// ```
#[must_use]
pub fn parse(source: &str) -> HtmlDoc {
    let events = scan::scan(source);
    let sink = sink::HtmlSink::new();
    let context = QualName::new(None, html5ever::ns!(html), html5ever::local_name!("div"));
    let mut parser =
        driver::parse_fragment(sink, ParseOpts::default(), context, Vec::new(), false);
    parser.process(source.into());
    let mut dom = parser.finish();
    align::align(&mut dom, &events);
    HtmlDoc {
        dom,
        events
    }
}
