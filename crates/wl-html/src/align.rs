// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Aligns scanner events with the parsed DOM in document order.
//!
//! Elements are matched per tag name in order of appearance; comments are
//! matched in order. Anything that cannot be matched keeps no span and falls
//! back to the line number recorded by the sink.

use std::collections::{HashMap, VecDeque};

use crate::{
    dom::{Attr, Dom, NodeId, NodeKind},
    scan::{ScanAttr, ScanEvent, StartTag}
};

/// Attaches source spans to elements, attributes and comments in `dom`.
pub fn align(dom: &mut Dom, events: &[ScanEvent]) {
    let mut starts: HashMap<&str, VecDeque<&StartTag>> = HashMap::new();
    for event in events {
        if let ScanEvent::Start(tag) = event {
            starts.entry(tag.name.as_str()).or_default().push_back(tag);
        }
    }
    let names: Vec<(NodeId, String)> = dom
        .elements()
        .map(|(id, element)| (id, element.local().to_ascii_lowercase()))
        .collect();
    for (id, name) in names {
        let Some(tag) = starts.get_mut(name.as_str()).and_then(VecDeque::pop_front) else {
            continue;
        };
        let Some(node) = dom.nodes.get_mut(id as usize) else {
            continue;
        };
        let NodeKind::Element(element) = &mut node.kind else {
            continue;
        };
        element.span = Some(tag.span);
        element.name_span = Some(tag.name_span);
        element.raw_text = tag.raw_text;
        align_attrs(&mut element.attrs, &tag.attrs);
    }
    align_comments(dom, events);
}

fn align_attrs(attrs: &mut [Attr], scan: &[ScanAttr]) {
    let mut used = vec![false; scan.len()];
    for attr in attrs {
        for (slot, candidate) in used.iter_mut().zip(scan) {
            if !*slot && candidate.name == attr.local() {
                *slot = true;
                attr.span = Some(candidate.span);
                attr.value_span = candidate.value_span;
                break;
            }
        }
    }
}

fn align_comments(dom: &mut Dom, events: &[ScanEvent]) {
    let nodes: Vec<NodeId> = dom
        .preorder(dom.root)
        .filter(|id| {
            matches!(
                dom.node(*id).map(|node| &node.kind),
                Some(NodeKind::Comment { .. })
            )
        })
        .collect();
    let comments = events
        .iter()
        .filter(|event| matches!(event, ScanEvent::Comment { .. }));
    for (id, event) in nodes.iter().copied().zip(comments) {
        let ScanEvent::Comment {
            span,
            text_span
        } = *event
        else {
            continue;
        };
        if let Some(NodeKind::Comment {
            span: target,
            text_span: target_text,
            ..
        }) = dom.nodes.get_mut(id as usize).map(|node| &mut node.kind)
        {
            *target = Some(span);
            *target_text = Some(text_span);
        }
    }
}
