// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! `html5ever` [`TreeSink`] implementation filling the arena [`Dom`].
//!
//! The sink records structure and the line number html5ever reports for each
//! token; exact byte spans are attached later by the scanner alignment.

use std::{
    borrow::Cow,
    cell::{Cell, Ref, RefCell},
    collections::HashMap
};

use html5ever::{
    Attribute, QualName,
    interface::tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink}
};
use tendril::StrTendril;

use crate::dom::{Attr, Dom, Element, Node, NodeId, NodeKind};

/// Builds the arena DOM for a parsed fragment.
#[derive(Debug)]
pub struct HtmlSink {
    nodes:        RefCell<Vec<Node>>,
    templates:    RefCell<HashMap<NodeId, NodeId>>,
    root:         Cell<Option<NodeId>>,
    line:         Cell<u32>,
    parse_errors: Cell<u32>
}

impl Default for HtmlSink {
    fn default() -> Self {
        Self::new()
    }
}

impl HtmlSink {
    /// Creates a sink with an empty document node.
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes:        RefCell::new(vec![Node {
                kind:     NodeKind::Document,
                parent:   None,
                children: Vec::new()
            }]),
            templates:    RefCell::new(HashMap::new()),
            root:         Cell::new(None),
            line:         Cell::new(1),
            parse_errors: Cell::new(0)
        }
    }

    fn push(&self, kind: NodeKind) -> NodeId {
        let mut nodes = self.nodes.borrow_mut();
        let id = u32::try_from(nodes.len()).unwrap_or(u32::MAX);
        nodes.push(Node {
            kind,
            parent: None,
            children: Vec::new()
        });
        id
    }

    fn detach(nodes: &mut [Node], id: NodeId) {
        let Some(parent) = nodes.get(id as usize).and_then(|node| node.parent) else {
            return;
        };
        if let Some(parent) = nodes.get_mut(parent as usize) {
            parent.children.retain(|child| *child != id);
        }
        if let Some(node) = nodes.get_mut(id as usize) {
            node.parent = None;
        }
    }

    fn attr_from_token(attr: Attribute) -> Attr {
        Attr {
            name:       attr.name,
            value:      attr.value.to_string(),
            span:       None,
            value_span: None
        }
    }
}

impl TreeSink for HtmlSink {
    type Handle = NodeId;
    type Output = Dom;
    type ElemName<'a> = Ref<'a, QualName>;

    fn finish(self) -> Self::Output {
        let nodes = self.nodes.into_inner();
        let document = nodes
            .first()
            .unwrap_or_else(|| unreachable!("the document node always exists"));
        let root = document
            .children
            .first()
            .copied()
            .unwrap_or_else(|| self.root.get().unwrap_or(0));
        Dom {
            nodes,
            document: 0,
            root,
            parse_errors: self.parse_errors.get()
        }
    }

    fn parse_error(&self, _msg: Cow<'static, str>) {
        self.parse_errors.set(self.parse_errors.get() + 1);
    }

    fn get_document(&self) -> Self::Handle {
        0
    }

    fn elem_name<'a>(&'a self, target: &'a NodeId) -> Self::ElemName<'a> {
        Ref::map(self.nodes.borrow(), |nodes| {
            let node = nodes
                .get(*target as usize)
                .unwrap_or_else(|| unreachable!("handles are always valid ids"));
            match &node.kind {
                NodeKind::Element(element) => &element.name,
                _ => unreachable!("elem_name is only called on elements")
            }
        })
    }

    fn create_element(
        &self,
        name: QualName,
        attrs: Vec<Attribute>,
        flags: ElementFlags
    ) -> Self::Handle {
        let element = Element {
            name,
            attrs: attrs.into_iter().map(Self::attr_from_token).collect(),
            span: None,
            name_span: None,
            raw_text: None,
            line: self.line.get()
        };
        let id = self.push(NodeKind::Element(element));
        if self.root.get().is_none() {
            self.root.set(Some(id));
        }
        if flags.template {
            let fragment = self.push(NodeKind::Fragment);
            self.templates.borrow_mut().insert(id, fragment);
        }
        id
    }

    fn create_comment(&self, text: StrTendril) -> Self::Handle {
        self.push(NodeKind::Comment {
            text:      text.to_string(),
            span:      None,
            text_span: None
        })
    }

    fn create_pi(&self, target: StrTendril, data: StrTendril) -> Self::Handle {
        self.push(NodeKind::Comment {
            text:      format!("{target} {data}"),
            span:      None,
            text_span: None
        })
    }

    fn append(&self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        let NodeOrText::AppendNode(node) = child else {
            return;
        };
        let mut nodes = self.nodes.borrow_mut();
        if let Some(parent) = nodes.get_mut(*parent as usize) {
            parent.children.push(node);
        }
        if let Some(node) = nodes.get_mut(node as usize) {
            node.parent = Some(*parent);
        }
    }

    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>
    ) {
        let has_parent = self
            .nodes
            .borrow()
            .get(*element as usize)
            .is_some_and(|node| node.parent.is_some());
        if has_parent {
            self.append(element, child);
        } else {
            self.append_before_sibling(prev_element, child);
        }
    }

    fn append_doctype_to_document(
        &self,
        _name: StrTendril,
        _public_id: StrTendril,
        _system_id: StrTendril
    ) {
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        if let Some(fragment) = self.templates.borrow().get(target).copied() {
            return fragment;
        }
        let fragment = self.push(NodeKind::Fragment);
        self.templates.borrow_mut().insert(*target, fragment);
        fragment
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    fn set_quirks_mode(&self, _mode: QuirksMode) {}

    fn append_before_sibling(&self, sibling: &Self::Handle, new_node: NodeOrText<Self::Handle>) {
        let NodeOrText::AppendNode(node) = new_node else {
            return;
        };
        let mut nodes = self.nodes.borrow_mut();
        Self::detach(&mut nodes, node);
        let Some(parent) = nodes.get(*sibling as usize).and_then(|s| s.parent) else {
            return;
        };
        let Some(index) = nodes
            .get(parent as usize)
            .and_then(|p| p.children.iter().position(|child| *child == *sibling))
        else {
            return;
        };
        if let Some(parent) = nodes.get_mut(parent as usize) {
            parent.children.insert(index, node);
        }
        if let Some(node) = nodes.get_mut(node as usize) {
            node.parent = Some(parent);
        }
    }

    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<Attribute>) {
        let mut nodes = self.nodes.borrow_mut();
        let Some(NodeKind::Element(element)) =
            nodes.get_mut(*target as usize).map(|n| &mut n.kind)
        else {
            return;
        };
        for attr in attrs {
            if element
                .attrs
                .iter()
                .any(|existing| existing.name == attr.name)
            {
                continue;
            }
            element.attrs.push(Self::attr_from_token(attr));
        }
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        let mut nodes = self.nodes.borrow_mut();
        Self::detach(&mut nodes, *target);
    }

    fn reparent_children(&self, node: &Self::Handle, new_parent: &Self::Handle) {
        let mut nodes = self.nodes.borrow_mut();
        let Some(children) = nodes
            .get_mut(*node as usize)
            .map(|n| std::mem::take(&mut n.children))
        else {
            return;
        };
        for child in children.iter().copied() {
            if let Some(child) = nodes.get_mut(child as usize) {
                child.parent = Some(*new_parent);
            }
        }
        if let Some(new_parent) = nodes.get_mut(*new_parent as usize) {
            new_parent.children.extend_from_slice(&children);
        }
    }

    fn set_current_line(&self, line_number: u64) {
        self.line.set(u32::try_from(line_number).unwrap_or(1));
    }
}
