// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Suppression edge paths: plain comments, next-line comments, directives in
//! style blocks, dangling directives and scripts without content.

use masterror::AppResult;
use wl_check::{Config, check};

fn ids(source: &str) -> AppResult<Vec<&'static str>> {
    let profile = wl_profile::profile_1()?;
    Ok(check(source, &profile, &Config::default())?
        .findings
        .iter()
        .map(|finding| finding.rule.id)
        .collect())
}

#[test]
fn plain_comments_are_not_directives() -> AppResult<()> {
    assert_eq!(ids("<!-- hello --><input>")?, ["TGW100"]);
    Ok(())
}

#[test]
fn disable_next_line_in_an_html_comment_covers_the_next_line() -> AppResult<()> {
    let source = "<!-- tgw-disable-next-line TGW100 -->\n<input>";
    assert!(ids(source)?.is_empty(), "the next line is covered");
    let shifted = "<div>\n  <!-- tgw-disable-next-line TGW100 -->\n  <input>\n</div>";
    assert!(ids(shifted)?.is_empty(), "indented directives work too");
    Ok(())
}

#[test]
fn disable_at_the_end_of_the_document_is_harmless() -> AppResult<()> {
    assert_eq!(ids("<input><!-- tgw-disable TGW100 -->")?, ["TGW100"]);
    Ok(())
}

#[test]
fn disable_file_inside_a_style_block_covers_the_document() -> AppResult<()> {
    let source = "<style>/* tgw-disable-file TGW216 */ not-a-property: 1;</style>";
    assert!(
        ids(source)?.is_empty(),
        "the file directive applies globally"
    );
    Ok(())
}

#[test]
fn disable_next_line_inside_a_style_block_covers_one_line() -> AppResult<()> {
    let source = "<style>\n/* tgw-disable-next-line TGW216 */\nnot-a-property: 1;\nother-prop: 2;\n</style>";
    let findings = ids(source)?;
    assert_eq!(
        findings,
        ["TGW216"],
        "only the second unknown property survives"
    );
    Ok(())
}

#[test]
fn external_scripts_without_content_are_skipped() -> AppResult<()> {
    let source = "<script src=\"a.js\"></script>\n<input>";
    assert_eq!(ids(source)?, ["TGW104", "TGW100"]);
    Ok(())
}

#[test]
fn malformed_directives_in_style_blocks_are_ignored() -> AppResult<()> {
    let source = "<style>/* tgw-disable */</style><input>";
    assert_eq!(ids(source)?, ["TGW100"]);
    Ok(())
}

#[test]
fn unused_directives_are_counted() -> AppResult<()> {
    let profile = wl_profile::profile_1()?;
    let used = check(
        "<!-- tgw-disable-file TGW100 --><input>",
        &profile,
        &Config::default()
    )?;
    assert_eq!(used.suppressed, 1);
    assert_eq!(used.unused_directives, 0);
    let unused = check(
        "<!-- tgw-disable-file TGW100 --><div>ok</div>",
        &profile,
        &Config::default()
    )?;
    assert_eq!(unused.suppressed, 0);
    assert_eq!(unused.unused_directives, 1);
    Ok(())
}

#[test]
fn disable_next_line_on_the_last_line_is_harmless() -> AppResult<()> {
    let source = "<input>\n<!-- tgw-disable-next-line TGW100 -->";
    assert_eq!(ids(source)?, ["TGW100"], "there is no next line to cover");
    Ok(())
}
