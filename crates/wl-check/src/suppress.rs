// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Collects inline suppression directives from a widget document.
//!
//! Three sources are scanned: HTML comments, `<style>`/`<script>` bodies and
//! inline `style=""`/`on*=""` values. `tgw-disable-file` covers the whole
//! document; `tgw-disable` inside an HTML comment covers the element that
//! follows the comment; every other scope falls back to the line after the
//! directive, which matches `tgw-disable-next-line`.
//!
//! # Examples
//!
//! ```
//! let profile = wl_profile::profile_1().expect("valid profile");
//! let source = "<!-- tgw-disable-file TGW100 --><input>";
//! let doc = wl_html::parse(source);
//! let suppressions = wl_check::suppressions(source, &doc);
//! let finding = wl_html::check(source, &profile)
//!     .into_iter()
//!     .next()
//!     .expect("one finding");
//! assert!(suppressions.is_suppressed(finding.rule.id, finding.span));
//! ```

use wl_core::{DirectiveKind, LineIndex, Span, Suppressions, suppress::parse_directive};
use wl_html::{
    Dom, HtmlDoc,
    dom::NodeKind,
    extract::{comments, event_handlers, inline_styles, script_blocks, style_blocks}
};

use crate::slice;

/// Builds the suppression map for one document.
#[must_use]
pub fn suppressions(source: &str, doc: &HtmlDoc) -> Suppressions {
    let index = LineIndex::new(source);
    let mut out = Suppressions::default();
    for comment in comments(&doc.dom) {
        let Some(text_span) = comment.text_span else {
            continue;
        };
        let Some(directive) = parse_directive(&comment.text) else {
            continue;
        };
        match directive.kind {
            DirectiveKind::DisableFile => out.add_file(&directive.rules),
            DirectiveKind::Disable => {
                if let Some(range) = next_element_range(&doc.dom, text_span.end) {
                    out.add_range(range, &directive.rules);
                }
            }
            DirectiveKind::DisableNextLine => {
                if let Some(range) = next_line(&index, source, text_span.start) {
                    out.add_range(range, &directive.rules);
                }
            }
        }
    }
    for block in embedded_texts(source, doc) {
        for (offset, directive) in scan(&block.text) {
            let absolute = block.span.start.saturating_add(offset);
            match directive.kind {
                DirectiveKind::DisableFile => out.add_file(&directive.rules),
                DirectiveKind::Disable | DirectiveKind::DisableNextLine => {
                    if let Some(range) = next_line(&index, source, absolute) {
                        out.add_range(range, &directive.rules);
                    }
                }
            }
        }
    }
    out
}

/// A text region that can carry JS or CSS directives.
struct Embedded {
    span: Span,
    text: String
}

fn embedded_texts(source: &str, doc: &HtmlDoc) -> Vec<Embedded> {
    let mut out = Vec::new();
    for block in style_blocks(&doc.dom) {
        out.push(Embedded {
            span: block.content,
            text: slice(source, block.content).to_owned()
        });
    }
    for block in script_blocks(&doc.dom) {
        if let Some(span) = block.content {
            out.push(Embedded {
                span,
                text: slice(source, span).to_owned()
            });
        }
    }
    for attr in inline_styles(&doc.dom)
        .into_iter()
        .chain(event_handlers(&doc.dom))
    {
        if let Some(span) = attr.value_span {
            out.push(Embedded {
                span,
                text: attr.value
            });
        }
    }
    out
}

/// Finds directives inside a code or style text, keyed by byte offset.
fn scan(text: &str) -> Vec<(u32, wl_core::Directive)> {
    let bytes = text.as_bytes();
    let needle = b"tgw-disable";
    let mut out = Vec::new();
    let mut start = 0_usize;
    while let Some(found) = find(bytes, needle, start) {
        let line_end = bytes
            .get(found..)
            .and_then(|rest| rest.iter().position(|byte| *byte == b'\n'))
            .map_or(text.len(), |offset| found + offset);
        let body = text[found..line_end]
            .trim_end()
            .trim_end_matches("*/")
            .to_owned();
        if let Some(directive) = parse_directive(&body) {
            out.push((u32::try_from(found).unwrap_or(u32::MAX), directive));
        }
        start = line_end.max(found + 1);
    }
    out
}

fn find(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    let start = haystack.get(from..)?;
    start
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|position| position + from)
}

/// Span covering the first element that starts at or after `offset`,
/// including its whole subtree.
fn next_element_range(dom: &Dom, offset: u32) -> Option<Span> {
    let (id, element) = dom
        .elements()
        .filter(|(_, element)| element.span.is_some_and(|span| span.start >= offset))
        .min_by_key(|(_, element)| element.span.map_or(u32::MAX, |span| span.start))?;
    let start = element.span?.start;
    let end = subtree_end(dom, id);
    Some(Span::new(start, end))
}

fn subtree_end(dom: &Dom, root: wl_html::NodeId) -> u32 {
    let mut end = 0_u32;
    for id in dom.preorder(root) {
        if let Some(NodeKind::Element(element)) = dom.node(id).map(|node| &node.kind) {
            end = end.max(element.span.map_or(0, |span| span.end));
        }
    }
    end
}

/// Range of the line after the one containing `offset`.
fn next_line(index: &LineIndex, source: &str, offset: u32) -> Option<Span> {
    let position = index.position(source, offset);
    let starts = index.line_starts();
    let next = usize::try_from(position.line)
        .ok()
        .and_then(|line| starts.get(line))?;
    let end = starts
        .get(usize::try_from(position.line).ok()? + 1)
        .copied()
        .unwrap_or(u32::MAX);
    (end > *next).then_some(Span::new(*next, end))
}
