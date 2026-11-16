// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! `TreeSink` edge paths: doctypes, processing instructions, templates and the
//! adoption-agency reparenting that html5ever performs on misnested markup.

use html5ever::{
    Attribute, LocalName, QualName, driver,
    driver::ParseOpts,
    interface::tree_builder::{ElementFlags, NodeOrText, TreeSink}
};
use tendril::TendrilSink;
use wl_html::{HtmlSink, NodeKind, parse};

fn qn(name: &str) -> QualName {
    QualName::new(None, html5ever::ns!(html), LocalName::from(name))
}

fn element(sink: &HtmlSink, name: &str) -> u32 {
    sink.create_element(qn(name), Vec::new(), ElementFlags::default())
}

fn parse_document(source: &str) -> wl_html::Dom {
    driver::parse_document(HtmlSink::default(), ParseOpts::default()).one(source)
}

fn count_nodes(dom: &wl_html::Dom, predicate: impl Fn(&wl_html::Node) -> bool) -> usize {
    dom.nodes.iter().filter(|node| predicate(node)).count()
}

fn has_element(dom: &wl_html::Dom, name: &str) -> bool {
    dom.nodes.iter().any(|node| match &node.kind {
        wl_html::NodeKind::Element(element) => element.local() == name,
        _ => false
    })
}

#[test]
fn default_sink_finishes_to_an_empty_document() {
    let dom = HtmlSink::default().finish();
    assert_eq!(dom.nodes.len(), 1);
    assert!(matches!(dom.nodes[0].kind, NodeKind::Document));
    assert_eq!(dom.root, 0);
}

#[test]
fn doctype_is_consumed_without_a_node() {
    let dom = parse_document("<!doctype html><title>t</title><p>x");
    assert!(has_element(&dom, "p"), "paragraph survives");
    assert_eq!(
        count_nodes(&dom, |node| matches!(node.kind, NodeKind::Comment { .. })),
        0,
        "the doctype must not become a comment"
    );
}

#[test]
fn processing_instruction_becomes_a_comment() {
    let dom = parse_document("<?pi hello><p>x");
    let text = dom
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            NodeKind::Comment {
                text, ..
            } => Some(text.clone()),
            _ => None
        })
        .find(|text| text.contains("pi"))
        .expect("the pi became a comment");
    assert!(text.contains("hello"), "the pi data is kept: {text}");
}

#[test]
fn template_gets_a_contents_fragment() {
    let doc = parse("<template><b>x</b></template>");
    assert!(
        count_nodes(&doc.dom, |node| matches!(node.kind, NodeKind::Fragment)) > 0,
        "templates own a fragment node"
    );
    assert!(has_element(&doc.dom, "b"), "template content is parsed");
}

#[test]
fn misnested_formatting_is_repaired() {
    for source in [
        "<b class=\"c\">one<i>two</b>three</i>",
        "<em><b>x</em>y</b>",
        "<b>x<table><tr><td>y</table>",
        "<i><div>a</i>b</div>"
    ] {
        let doc = parse(source);
        assert!(
            doc.dom.parse_errors > 0,
            "{source} is not spec-clean and html5ever must report it"
        );
        assert!(doc.dom.nodes.len() > 2, "{source} still produces a tree");
    }
}

#[test]
fn table_reconstruction_repairs_rows() {
    let doc = parse("<table><td>a<tr><td>b");
    assert!(
        has_element(&doc.dom, "tr"),
        "html5ever inserts the missing row"
    );
    assert!(doc.dom.parse_errors > 0, "the markup is not spec-clean");
}

#[test]
fn create_pi_is_recorded_as_a_comment() {
    let sink = HtmlSink::new();
    let id = sink.create_pi("php".into(), "echo 1".into());
    let dom = sink.finish();
    let text = match &dom.nodes[id as usize].kind {
        NodeKind::Comment {
            text, ..
        } => text.clone(),
        other => unreachable!("expected a comment, got {other:?}")
    };
    assert_eq!(text, "php echo 1");
}

#[test]
fn append_before_sibling_inserts_and_detaches() {
    let sink = HtmlSink::new();
    let parent = element(&sink, "div");
    let first = element(&sink, "a");
    let second = element(&sink, "b");
    sink.append(&parent, NodeOrText::AppendNode(first));
    sink.append(&parent, NodeOrText::AppendNode(second));
    sink.append_before_sibling(&first, NodeOrText::AppendNode(second));
    let dom = sink.finish();
    assert_eq!(dom.children(parent), vec![second, first]);
    assert_eq!(dom.parent(second), Some(parent));
}

#[test]
fn append_before_sibling_ignores_text_and_orphans() {
    let sink = HtmlSink::new();
    let orphan = element(&sink, "i");
    let moved = element(&sink, "b");
    sink.append_before_sibling(&orphan, NodeOrText::AppendNode(moved));
    sink.append_before_sibling(&orphan, NodeOrText::AppendText("x".into()));
    let dom = sink.finish();
    assert!(
        dom.nodes.iter().all(|node| node.parent.is_none()),
        "nothing attaches without a parent"
    );
}

#[test]
fn append_based_on_parent_node_falls_back_to_sibling() {
    let sink = HtmlSink::new();
    let parent = element(&sink, "div");
    let prev = element(&sink, "p");
    let orphan = element(&sink, "table");
    let child = element(&sink, "td");
    sink.append(&parent, NodeOrText::AppendNode(prev));
    sink.append_based_on_parent_node(&orphan, &prev, NodeOrText::AppendNode(child));
    let dom = sink.finish();
    assert_eq!(dom.children(parent), vec![child, prev]);
}

#[test]
fn append_based_on_parent_node_appends_to_parented_element() {
    let sink = HtmlSink::new();
    let parent = element(&sink, "div");
    let prev = element(&sink, "p");
    let orphan = element(&sink, "table");
    sink.append(&parent, NodeOrText::AppendNode(prev));
    sink.append_based_on_parent_node(&parent, &prev, NodeOrText::AppendNode(orphan));
    let dom = sink.finish();
    assert!(
        dom.children(parent).contains(&orphan),
        "parented element appends"
    );
}

#[test]
fn append_with_unknown_handles_is_harmless() {
    let sink = HtmlSink::new();
    let node = element(&sink, "div");
    sink.append(&999, NodeOrText::AppendNode(node));
    sink.append(&0, NodeOrText::AppendNode(999));
    let dom = sink.finish();
    assert_eq!(
        dom.children(0),
        vec![999],
        "the missing node is still listed"
    );
}

#[test]
fn template_contents_is_cached_on_demand() {
    let sink = HtmlSink::new();
    let plain = element(&sink, "div");
    let first = sink.get_template_contents(&plain);
    let second = sink.get_template_contents(&plain);
    assert_eq!(first, second, "the fragment is created once");
    let dom = sink.finish();
    assert!(matches!(dom.nodes[first as usize].kind, NodeKind::Fragment));
}

#[test]
fn second_html_tag_merges_missing_attributes() {
    let dom = parse_document("<html lang=\"en\"><html lang=\"fr\" dir=\"rtl\"><p>x");
    let html = dom
        .nodes
        .iter()
        .find_map(|node| match &node.kind {
            NodeKind::Element(element) if element.local() == "html" => Some(element),
            _ => None
        })
        .expect("the html element exists");
    assert_eq!(html.attr("lang").expect("lang").value, "en", "first wins");
    assert_eq!(
        html.attr("dir").expect("dir").value,
        "rtl",
        "missing attr merged"
    );
}

#[test]
fn add_attrs_if_missing_ignores_non_elements() {
    let sink = HtmlSink::new();
    sink.add_attrs_if_missing(
        &0,
        vec![Attribute {
            name:  qn("x"),
            value: "v".into()
        }]
    );
    let dom = sink.finish();
    assert!(matches!(dom.nodes[0].kind, NodeKind::Document));
}

#[test]
fn reparent_children_moves_children_and_survives_bogus_ids() {
    let sink = HtmlSink::new();
    let from = element(&sink, "div");
    let to = element(&sink, "section");
    let child = element(&sink, "b");
    sink.append(&from, NodeOrText::AppendNode(child));
    sink.reparent_children(&from, &to);
    sink.reparent_children(&999, &to);
    let dom = sink.finish();
    assert!(dom.children(from).is_empty(), "the source is emptied");
    assert_eq!(dom.children(to), vec![child]);
    assert_eq!(dom.parent(child), Some(to));
}

#[test]
fn remove_from_parent_without_parent_is_harmless() {
    let sink = HtmlSink::new();
    sink.remove_from_parent(&0);
    sink.remove_from_parent(&999);
    let dom = sink.finish();
    assert_eq!(dom.nodes.len(), 1);
}
