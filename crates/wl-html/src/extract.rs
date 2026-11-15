// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Extraction of embedded regions: CSS and JS sources, inline attributes
//! and comments (which carry the suppression directives).
//!
//! Every extracted region carries the document span of its text, so nested
//! CSS/JS diagnostics map back to the widget source by adding the base
//! offset.

use wl_core::Span;

use crate::dom::{Dom, NodeId};

/// A `<style>` element with its content region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyleBlock {
    /// The `<style>` element.
    pub element: NodeId,
    /// Span of the CSS text inside the element.
    pub content: Span
}

/// A `<script>` element: inline content, an external `src`, or both.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptBlock {
    /// The `<script>` element.
    pub element:  NodeId,
    /// Span of the inline JS text, when present.
    pub content:  Option<Span>,
    /// `src` attribute value, when present.
    pub src:      Option<String>,
    /// Span of the `src` value.
    pub src_span: Option<Span>
}

/// An inline `style=""` or `on*=""` attribute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineAttr {
    /// The element carrying the attribute.
    pub element:    NodeId,
    /// Attribute local name.
    pub name:       String,
    /// Attribute value.
    pub value:      String,
    /// Span of the value inside the source.
    pub value_span: Option<Span>
}

/// A comment with its text and positions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentInfo {
    /// Comment text without the delimiters.
    pub text:      String,
    /// Span of the whole comment.
    pub span:      Option<Span>,
    /// Span of the text inside the delimiters.
    pub text_span: Option<Span>
}

/// All `<style>` elements with content spans.
#[must_use]
pub fn style_blocks(dom: &Dom) -> Vec<StyleBlock> {
    dom.elements()
        .filter(|(_, element)| element.local() == "style")
        .filter_map(|(id, element)| {
            element.raw_text.map(|content| StyleBlock {
                element: id,
                content
            })
        })
        .collect()
}

/// All `<script>` elements.
#[must_use]
pub fn script_blocks(dom: &Dom) -> Vec<ScriptBlock> {
    dom.elements()
        .filter(|(_, element)| element.local() == "script")
        .map(|(id, element)| ScriptBlock {
            element:  id,
            content:  element.raw_text,
            src:      element.attr("src").map(|attr| attr.value.clone()),
            src_span: element.attr("src").and_then(|attr| attr.value_span)
        })
        .collect()
}

/// All `style=""` attributes.
#[must_use]
pub fn inline_styles(dom: &Dom) -> Vec<InlineAttr> {
    collect_attrs(dom, |name| name == "style")
}

/// All `on*=""` event handler attributes.
#[must_use]
pub fn event_handlers(dom: &Dom) -> Vec<InlineAttr> {
    collect_attrs(dom, |name| name.starts_with("on"))
}

/// All comments in document order.
#[must_use]
pub fn comments(dom: &Dom) -> Vec<CommentInfo> {
    let mut out = Vec::new();
    for id in dom.preorder(dom.root) {
        let Some(node) = dom.node(id) else {
            continue;
        };
        if let crate::dom::NodeKind::Comment {
            text,
            span,
            text_span
        } = &node.kind
        {
            out.push(CommentInfo {
                text:      text.clone(),
                span:      *span,
                text_span: *text_span
            });
        }
    }
    out
}

fn collect_attrs(dom: &Dom, mut wanted: impl FnMut(&str) -> bool) -> Vec<InlineAttr> {
    let mut out = Vec::new();
    for (id, element) in dom.elements() {
        for attr in &element.attrs {
            if wanted(attr.local()) {
                out.push(InlineAttr {
                    element:    id,
                    name:       attr.local().to_owned(),
                    value:      attr.value.clone(),
                    value_span: attr.value_span
                });
            }
        }
    }
    out
}
