// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Agent report contract: verdicts, fingerprints, grouping, truncation,
//! since-diffing, suppression statistics and text sanitization.

use std::collections::BTreeSet;

use masterror::AppResult;
use wl_check::{AgentOptions, Config, check};
use wl_core::Severity;
use wl_profile::Profile;

fn profile() -> AppResult<Profile> {
    wl_profile::profile_1()
}

fn report_for(
    source: &str,
    options: &AgentOptions<'_>
) -> AppResult<wl_check::agent::AgentReport> {
    let checked = check(source, &profile()?, &Config::default())?;
    Ok(wl_check::agent::report(
        "widget.html",
        source,
        &checked,
        options
    ))
}

fn warn_options() -> AgentOptions<'static> {
    AgentOptions {
        fail_on: Severity::Warn,
        ..AgentOptions::default()
    }
}

#[test]
fn clean_widget_passes() -> AppResult<()> {
    let report = report_for("<b>hello</b>", &AgentOptions::default())?;
    assert_eq!(report.verdict, "pass");
    assert_eq!(report.reason, "no findings");
    assert!(report.findings.is_empty());
    assert_eq!(report.truncated, 0);
    assert_eq!(
        report.next_action,
        "no findings: the widget matches profile 1"
    );
    Ok(())
}

#[test]
fn warn_fails_only_at_warn_threshold() -> AppResult<()> {
    let strict = report_for("<input>", &warn_options())?;
    assert_eq!(strict.verdict, "fail");
    assert_eq!(strict.counts.warn, 1);
    assert_eq!(
        strict.next_action,
        "warnings are silently changed or removed by the client; fix them or accept the behavior"
    );
    let lenient = report_for("<input>", &AgentOptions::default())?;
    assert_eq!(lenient.verdict, "pass");
    Ok(())
}

#[test]
fn fingerprints_are_stable_and_context_sensitive() -> AppResult<()> {
    let first = report_for("<div><input></div>", &warn_options())?;
    let second = report_for("<div><input></div>", &warn_options())?;
    let fp_a = first.findings.first().expect("finding").fp.clone();
    let fp_b = second.findings.first().expect("finding").fp.clone();
    assert_eq!(fp_a, fp_b, "same input must produce the same fingerprint");
    assert_eq!(fp_a.len(), 8);
    let moved = report_for("<section>\n  <input>\n</section>", &warn_options())?;
    let fp_c = moved.findings.first().expect("finding").fp.clone();
    assert_ne!(fp_a, fp_c, "different context must change the fingerprint");
    Ok(())
}

#[test]
fn repeats_group_into_one_finding() -> AppResult<()> {
    let report = report_for("<input><input><input>", &warn_options())?;
    let tgw100: Vec<_> = report
        .findings
        .iter()
        .filter(|finding| finding.id == "TGW100")
        .collect();
    assert_eq!(tgw100.len(), 1, "identical lines group together");
    assert_eq!(tgw100.first().expect("group").repeat, 3);
    Ok(())
}

#[test]
fn max_findings_truncates_and_counts() -> AppResult<()> {
    let source = "<input>\n<style>color: red; transform: preserve-3d; position: sticky</style>";
    let full = report_for(source, &warn_options())?;
    let total = full.findings.len();
    assert!(total > 1, "fixture must produce several groups");
    let options = AgentOptions {
        max_findings: 1,
        ..warn_options()
    };
    let cut = report_for(source, &options)?;
    assert_eq!(cut.findings.len(), 1);
    assert_eq!(cut.truncated, total - 1);
    assert_eq!(cut.counts.warn + cut.counts.error, total);
    Ok(())
}

#[test]
fn since_splits_new_and_resolved() -> AppResult<()> {
    let current = report_for("<input>", &warn_options())?;
    let fp = current.findings.first().expect("finding").fp.clone();
    let gone = "deadbeef";
    let known: BTreeSet<String> = [gone.to_owned(), fp.clone()].into_iter().collect();
    let options = AgentOptions {
        since: Some(&known),
        ..warn_options()
    };
    let diff = report_for("<input>", &options)?;
    assert!(
        diff.findings.is_empty(),
        "known fingerprints must be hidden"
    );
    assert_eq!(diff.resolved, vec![gone.to_owned()]);
    let fresh: BTreeSet<String> = std::iter::once(gone.to_owned()).collect();
    let options = AgentOptions {
        since: Some(&fresh),
        ..warn_options()
    };
    let diff = report_for("<input>", &options)?;
    assert!(diff.findings.iter().any(|finding| finding.fp == fp));
    assert_eq!(diff.resolved, vec![gone.to_owned()]);
    Ok(())
}

#[test]
fn suppression_statistics_are_reported() -> AppResult<()> {
    let used = report_for(
        "<!-- tgw-disable-file TGW100 --><input>",
        &AgentOptions::default()
    )?;
    assert_eq!(used.suppressions.used, 1);
    assert_eq!(used.suppressions.ignored, 0);
    let ignored = report_for(
        "<!-- tgw-disable-file TGW999 --><b>x</b>",
        &AgentOptions::default()
    )?;
    assert_eq!(ignored.suppressions.used, 0);
    assert_eq!(ignored.suppressions.ignored, 1);
    Ok(())
}

#[test]
fn budget_block_reports_overrun_and_action() -> AppResult<()> {
    let options = AgentOptions {
        limit: 1,
        ..AgentOptions::default()
    };
    let report = report_for("<b>hello</b>", &options)?;
    assert!(report.budget.over_by > 0);
    assert_eq!(report.budget.limit, 1);
    assert!(report.next_action.starts_with("reduce the compressed size"));
    Ok(())
}

#[test]
fn report_text_carries_no_control_characters() -> AppResult<()> {
    let checked = check("<input>", &profile()?, &Config::default())?;
    let report =
        wl_check::agent::report("we\u{1b}[31ird.html", "<input>", &checked, &warn_options());
    assert!(!report.input.contains('\u{1b}'));
    let json = serde_json::to_string(&report).expect("serializable");
    assert!(!json.contains('\u{1b}'), "no ANSI escapes in agent output");
    assert!(!json.contains('\n'), "single-line report");
    Ok(())
}

#[test]
fn caveats_cover_static_scope() -> AppResult<()> {
    let report = report_for("<input>", &warn_options())?;
    assert!(
        report
            .caveats
            .iter()
            .any(|caveat| caveat.contains("dynamic limits"))
    );
    assert!(
        report
            .caveats
            .iter()
            .any(|caveat| caveat.contains("static analysis"))
    );
    Ok(())
}

#[test]
fn schema_is_valid_json_with_the_report_fields() -> AppResult<()> {
    let schema: serde_json::Value =
        serde_json::from_str(&wl_check::agent::schema_json()?).expect("valid json");
    for field in [
        "verdict",
        "findings",
        "next_action",
        "caveats",
        "suppressions"
    ] {
        assert!(
            schema.pointer(&format!("/properties/{field}")).is_some(),
            "schema must document {field}"
        );
    }
    Ok(())
}
