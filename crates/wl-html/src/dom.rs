// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Arena DOM: every node lives in one `Vec`, handles are `u32` indices.
//!
//! The arena avoids per-node heap allocations and keeps traversal cache
//! friendly. Text nodes are not stored: rules never need text content, and
//! the scanner keeps the exact source positions.

use html5ever::QualName;
use wl_core::Span;

/// Handle of a node inside [`Dom`].
pub type NodeId = u32;

/// One attribute of an element, value already entity-decoded by the parser.
#[derive(Debug, Clone)]
pub struct Attr {
    /// Expanded attribute name.
    pub name:       QualName,
    /// Attribute value.
    pub value:      String,
    /// Span of the whole attribute, when aligned with the source.
    pub span:       Option<Span>,
    /// Span of the value without quotes, when aligned with the source.
    pub value_span: Option<Span>
}

impl Attr {
    /// Local attribute name.
    #[must_use]
    pub fn local(&self) -> &str {
        &self.name.local
    }

    /// Whether the attribute has this local name.
    #[must_use]
    pub fn is(&self, name: &str) -> bool {
        self.local() == name
    }
}

/// An element node.
#[derive(Debug, Clone)]
pub struct Element {
    /// Expanded tag name.
    pub name:      QualName,
    /// Attributes in source order.
    pub attrs:     Vec<Attr>,
    /// Span of the whole start tag, when aligned with the source.
    pub span:      Option<Span>,
    /// Span of the tag name inside the start tag.
    pub name_span: Option<Span>,
    /// Content span for raw-text elements (`script`, `style`, ...).
    pub raw_text:  Option<Span>,
    /// 1-based line reported by the parser when the element was created.
    pub line:      u32
}

impl Element {
    /// Local tag name.
    #[must_use]
    pub fn local(&self) -> &str {
        &self.name.local
    }

    /// First attribute with this local name.
    #[must_use]
    pub fn attr(&self, name: &str) -> Option<&Attr> {
        self.attrs.iter().find(|attr| attr.is(name))
    }

    /// Whether an attribute with this local name exists.
    #[must_use]
    pub fn has_attr(&self, name: &str) -> bool {
        self.attr(name).is_some()
    }
}

/// What a node is.
#[derive(Debug, Clone)]
pub enum NodeKind {
    /// The document root.
    Document,
    /// A template contents fragment.
    Fragment,
    /// An element.
    Element(Element),
    /// A comment, with its text and source positions.
    Comment {
        /// Comment text without the delimiters.
        text:      String,
        /// Span of the whole comment, when aligned with the source.
        span:      Option<Span>,
        /// Span of the text inside the delimiters.
        text_span: Option<Span>
    }
}

/// One arena slot.
#[derive(Debug)]
pub struct Node {
    /// Node payload.
    pub kind:     NodeKind,
    /// Parent node, `None` for roots.
    pub parent:   Option<NodeId>,
    /// Children in document order.
    pub children: Vec<NodeId>
}

impl Node {
    /// The element carried by this node, when it is one.
    #[must_use]
    pub const fn element(&self) -> Option<&Element> {
        match &self.kind {
            NodeKind::Element(element) => Some(element),
            _ => None
        }
    }
}

/// The whole parsed document.
#[derive(Debug)]
pub struct Dom {
    /// Arena of nodes; index `0` is the document.
    pub nodes:        Vec<Node>,
    /// Document node id.
    pub document:     NodeId,
    /// Fragment context node (the synthetic `<div>`).
    pub root:         NodeId,
    /// HTML5 parse errors reported while building the tree.
    pub parse_errors: u32
}

impl Dom {
    /// Node by id.
    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id as usize)
    }

    /// Element carried by the node, when it is one.
    #[must_use]
    pub fn element(&self, id: NodeId) -> Option<&Element> {
        self.node(id).and_then(Node::element)
    }

    /// Children of a node in document order.
    #[must_use]
    pub fn children(&self, id: NodeId) -> &[NodeId] {
        self.node(id).map_or(&[], |node| &node.children)
    }

    /// Parent of a node.
    #[must_use]
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.node(id).and_then(|node| node.parent)
    }

    /// Preorder walk starting at `root`, inclusive.
    #[must_use]
    pub fn preorder(&self, root: NodeId) -> Preorder<'_> {
        Preorder {
            dom:   self,
            stack: vec![root]
        }
    }

    /// All content elements in document order, excluding the synthetic root.
    #[must_use]
    pub fn elements(&self) -> Elements<'_> {
        let stack = self.children(self.root).iter().rev().copied().collect();
        Elements {
            dom: self,
            stack
        }
    }
}

/// Preorder node iterator.
#[derive(Debug)]
pub struct Preorder<'a> {
    dom:   &'a Dom,
    stack: Vec<NodeId>
}

impl Iterator for Preorder<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        let id = self.stack.pop()?;
        self.stack
            .extend(self.dom.children(id).iter().rev().copied());
        Some(id)
    }
}

/// Document-order element iterator over the parsed content.
#[derive(Debug)]
pub struct Elements<'a> {
    dom:   &'a Dom,
    stack: Vec<NodeId>
}

impl<'a> Iterator for Elements<'a> {
    type Item = (NodeId, &'a Element);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let id = self.stack.pop()?;
            self.stack
                .extend(self.dom.children(id).iter().rev().copied());
            if let Some(element) = self.dom.element(id) {
                return Some((id, element));
            }
        }
    }
}
