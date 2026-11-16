// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Positive and negative scenarios for the core model.

use wl_core::{
    Finding, Region, Report, Suppressions,
    rule::lookup,
    severity::Severity,
    span::{LineIndex, Span},
    suppress::{DirectiveKind, parse_directive}
};

#[test]
fn rule_registry_has_size_and_html_rules() {
    let tgw100 = lookup("TGW100").expect("TGW100 exists");
    assert_eq!(tgw100.default_severity, Severity::Warn);
    assert!(tgw100.spec.contains("widgets"));
    assert!(lookup("TGW001").is_some());
    assert!(lookup("TGW999").is_none());
    assert!(wl_core::rule::all().iter().any(|rule| rule.id == "TGW100"));
}

#[test]
fn line_index_resolves_crlf_and_unicode() {
    let source = "ab\r\nпривет\rx";
    let index = LineIndex::new(source);
    assert_eq!(index.line_count(), 3);
    let second = index.position(source, 6);
    assert_eq!((second.line, second.col), (2, 2));
    let third = index.position(
        source,
        u32::try_from(source.len() - 1).expect("small literal")
    );
    assert_eq!(third.line, 3);
}

#[test]
fn line_index_handles_empty_and_offset_past_end() {
    let index = LineIndex::new("");
    let pos = index.position("", 0);
    assert_eq!((pos.line, pos.col), (1, 1));
    let source = "one\ntwo";
    let index = LineIndex::new(source);
    let past = index.position(source, 99);
    assert_eq!(past.line, 2);
}

#[test]
fn report_renders_spans_and_summary() {
    let source = "<div>\n  <input>\n</div>";
    let meta = lookup("TGW100").expect("rule");
    let finding = Finding::new(meta, "input removed").with_span(Span::new(8, 15), Region::Html);
    let report = Report::build(Some("w.html"), source, None, &[finding]);
    assert_eq!(report.summary.warn, 1);
    let json = report.to_json().expect("json");
    assert!(json.contains("\"schema_version\":1"));
    assert!(json.contains("\"line\":2"));
    assert!(json.contains("\"region\":\"html\""));
}

#[test]
fn report_without_findings_has_empty_summary() {
    let report = Report::build(None, "", None, &[]);
    assert_eq!(report.summary.warn, 0);
    assert!(report.diagnostics.is_empty());
    assert!(report.size.is_none());
}

#[test]
fn directives_parse_and_suppress() {
    let all = parse_directive("tgw-disable").expect("directive");
    assert_eq!(all.kind, DirectiveKind::Disable);
    assert_eq!(all.rules, Vec::<String>::new());
    let listed = parse_directive("tgw-disable TGW100, TGW101").expect("directive");
    assert_eq!(listed.rules.len(), 2);
    assert!(parse_directive("not a directive").is_none());
    assert!(parse_directive("tgw-disablexyz").is_none());

    let mut suppressions = Suppressions::default();
    suppressions.add_range(Span::new(0, 20), &["TGW100".to_owned()]);
    assert!(suppressions.is_suppressed("TGW100", Some(Span::new(2, 9))));
    assert!(!suppressions.is_suppressed("TGW101", Some(Span::new(2, 9))));
    assert!(!suppressions.is_suppressed("TGW100", None));
    assert!(!suppressions.is_suppressed("TGW100", Some(Span::new(5, 25))));
}

#[test]
fn file_wide_disable_covers_spanless_findings() {
    let mut suppressions = Suppressions::default();
    suppressions.add_file(&[]);
    assert!(suppressions.is_suppressed("TGW002", None));
}

#[test]
fn empty_spans_are_recognized() {
    assert!(Span::new(4, 4).is_empty());
    assert!(!Span::new(4, 5).is_empty());
}

#[test]
fn finding_builders_attach_help_and_span() {
    let rule = lookup("TGW100").expect("TGW100 exists");
    let finding = Finding::new(rule, "boom").with_help("use buttons");
    assert_eq!(finding.help.as_deref(), Some("use buttons"));
    let spanned = finding.with_span(Span::new(0, 6), Region::Html);
    assert_eq!(spanned.span, Some(Span::new(0, 6)));
    assert_eq!(spanned.region, Region::Html);
}

#[test]
fn region_names_are_stable() {
    assert_eq!(Region::Html.as_str(), "html");
    assert_eq!(Region::Css.as_str(), "css");
    assert_eq!(Region::Js.as_str(), "js");
    assert_eq!(Region::Size.as_str(), "size");
}

#[test]
fn report_summary_counts_every_severity() {
    let rule = lookup("TGW100").expect("TGW100 exists");
    let mut error = Finding::new(rule, "e");
    error.severity = Severity::Error;
    let warn = Finding::new(rule, "w");
    let mut info = Finding::new(rule, "i");
    info.severity = Severity::Info;
    let report = Report::build(Some("w.html"), "<input>", None, &[error, warn, info]);
    assert_eq!(report.summary.error, 1);
    assert_eq!(report.summary.warn, 1);
    assert_eq!(report.summary.info, 1);
}
