// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Integration tests for the end-to-end pipeline: region merging, span
//! offsets, suppression directives and configuration overrides.

use std::collections::BTreeMap;

use masterror::AppResult;
use wl_check::{Config, check};
use wl_profile::Profile;

fn profile() -> AppResult<Profile> {
    wl_profile::profile_1()
}

fn ids(source: &str) -> AppResult<Vec<&'static str>> {
    Ok(check(source, &profile()?, &Config::default())?
        .findings
        .iter()
        .map(|finding| finding.rule.id)
        .collect())
}

fn assert_rule(source: &str, id: &str) -> AppResult<()> {
    let found = ids(source)?;
    assert!(
        found.contains(&id),
        "expected {id} in {found:?} for: {source}"
    );
    Ok(())
}

fn assert_clean(source: &str) -> AppResult<()> {
    assert_eq!(
        ids(source)?,
        Vec::<&'static str>::new(),
        "expected clean: {source}"
    );
    Ok(())
}

#[test]
fn merges_findings_from_every_region() -> AppResult<()> {
    let source =
        "<input>\n<style>color: red; transform: preserve-3d</style>\n<script>fetch('/x')</script>";
    let found = ids(source)?;
    assert!(found.contains(&"TGW100"), "html region: {found:?}");
    assert!(found.contains(&"TGW400"), "js region: {found:?}");
    Ok(())
}

#[test]
fn style_block_spans_point_at_the_document() -> AppResult<()> {
    let source = "<style>position: sticky</style>";
    let checked = check(source, &profile()?, &Config::default())?;
    let finding = checked
        .findings
        .iter()
        .find(|finding| finding.rule.id == "TGW202")
        .expect("a css finding");
    let span = finding.span.expect("has span");
    let start = usize::try_from(span.start).expect("fits");
    let end = usize::try_from(span.end).expect("fits");
    assert!(
        source[start..end].contains("sticky"),
        "span text: {}",
        &source[start..end]
    );
    Ok(())
}

#[test]
fn script_block_spans_point_at_the_document() -> AppResult<()> {
    let source = "<script>alert('x')</script>";
    let checked = check(source, &profile()?, &Config::default())?;
    let finding = checked
        .findings
        .iter()
        .find(|finding| finding.rule.id == "TGW402")
        .expect("a js finding");
    let span = finding.span.expect("has span");
    let start = usize::try_from(span.start).expect("fits");
    let end = usize::try_from(span.end).expect("fits");
    assert_eq!(&source[start..end], "alert");
    Ok(())
}

#[test]
fn inline_style_and_handler_attributes_are_checked() -> AppResult<()> {
    assert_rule("<div style=\"position: sticky\"></div>", "TGW202")?;
    assert_rule("<div onclick=\"alert('x')\"></div>", "TGW402")?;
    Ok(())
}

#[test]
fn broken_script_is_reported_as_error() {
    let profile = wl_profile::profile_1().expect("valid profile");
    let result = check("<script>function (</script>", &profile, &Config::default());
    assert!(result.is_err());
}

#[test]
fn disable_file_suppresses_the_whole_document() -> AppResult<()> {
    assert_clean("<!-- tgw-disable-file TGW100 --><input>")?;
    assert_rule("<!-- tgw-disable-file TGW101 --><input>", "TGW100")?;
    Ok(())
}

#[test]
fn disable_covers_the_following_element() -> AppResult<()> {
    let source = "<!-- tgw-disable TGW100 --><input><input>";
    let checked = check(source, &profile()?, &Config::default())?;
    let tgw100 = checked
        .findings
        .iter()
        .filter(|finding| finding.rule.id == "TGW100")
        .count();
    assert_eq!(tgw100, 1, "only the first input stays suppressed");
    Ok(())
}

#[test]
fn disable_next_line_in_script_suppresses_one_line() -> AppResult<()> {
    let source = "<script>\n// tgw-disable-next-line TGW402\nalert('a');\nalert('b');\n</script>";
    let checked = check(source, &profile()?, &Config::default())?;
    let alerts = checked
        .findings
        .iter()
        .filter(|finding| finding.rule.id == "TGW402")
        .count();
    assert_eq!(alerts, 1, "the first alert stays suppressed");
    Ok(())
}

#[test]
fn rule_off_drops_findings() -> AppResult<()> {
    let mut config = Config::default();
    config
        .rules
        .insert("TGW100".to_owned(), wl_check::config::RuleAction::Off);
    let checked = check("<input>", &profile()?, &config)?;
    assert!(
        checked
            .findings
            .iter()
            .all(|finding| finding.rule.id != "TGW100")
    );
    Ok(())
}

#[test]
fn rule_severity_override_replaces_the_default() -> AppResult<()> {
    let mut rules = BTreeMap::new();
    rules.insert("TGW100".to_owned(), wl_check::config::RuleAction::Error);
    let config = Config {
        rules,
        ..Config::default()
    };
    let checked = check("<input>", &profile()?, &config)?;
    let finding = checked
        .findings
        .iter()
        .find(|finding| finding.rule.id == "TGW100")
        .expect("kept finding");
    assert_eq!(finding.severity, wl_core::Severity::Error);
    Ok(())
}

#[test]
fn thresholds_drop_quiet_findings() -> AppResult<()> {
    let source = "<marquee>x</marquee>";
    assert_rule(source, "TGW110")?;
    let mut config = Config::default();
    config.output.min_severity = wl_check::config::SeverityFilter::Error;
    let checked = check(source, &profile()?, &config)?;
    assert!(
        checked
            .findings
            .iter()
            .all(|finding| finding.severity == wl_core::Severity::Error)
    );
    Ok(())
}

#[test]
fn confidence_threshold_drops_heuristic_rules() -> AppResult<()> {
    let source = "<div placeholder='name'>x</div>";
    assert_rule(source, "TGW114")?;
    let mut config = Config::default();
    config.output.min_confidence = wl_check::config::ConfidenceFilter::Certain;
    let checked = check(source, &profile()?, &config)?;
    assert!(
        checked
            .findings
            .iter()
            .all(|finding| finding.rule.confidence == wl_core::Confidence::Certain)
    );
    Ok(())
}

#[test]
fn findings_are_sorted_by_position() -> AppResult<()> {
    let source = "<input><div tabindex='0'><script>alert('x')</script>";
    let checked = check(source, &profile()?, &Config::default())?;
    let starts: Vec<u32> = checked
        .findings
        .iter()
        .filter_map(|finding| finding.span.map(|span| span.start))
        .collect();
    let mut sorted = starts.clone();
    sorted.sort_unstable();
    assert_eq!(starts, sorted);
    Ok(())
}

#[test]
fn size_findings_carry_the_size_region() -> AppResult<()> {
    let checked = check("<div>hello</div>", &profile()?, &Config::default())?;
    assert_eq!(checked.size.raw, 16);
    assert!(
        checked
            .findings
            .iter()
            .all(|finding| finding.rule.id.starts_with("TGW"))
    );
    Ok(())
}
