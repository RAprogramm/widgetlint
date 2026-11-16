// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Integration tests for the HTML rule engine: every rule gets positive and
//! negative scenarios, plus span, limit and malformed-input robustness.

use masterror::AppResult;
use wl_html::check;
use wl_profile::Profile;

fn profile() -> AppResult<Profile> {
    wl_profile::profile_1()
}

fn ids(html: &str) -> AppResult<Vec<&'static str>> {
    Ok(check(html, &profile()?)
        .iter()
        .map(|finding| finding.rule.id)
        .collect())
}

fn assert_rule(html: &str, id: &str) -> AppResult<()> {
    let found = ids(html)?;
    assert!(
        found.contains(&id),
        "expected {id} in {found:?} for: {html}"
    );
    Ok(())
}

fn assert_clean(html: &str) -> AppResult<()> {
    assert_eq!(
        ids(html)?,
        Vec::<&'static str>::new(),
        "expected clean: {html}"
    );
    Ok(())
}

#[test]
fn tgw100_flags_form_controls() -> AppResult<()> {
    assert_rule("<input type='text'>", "TGW100")?;
    assert_rule("<select><option>a</option></select>", "TGW100")?;
    assert_rule("<textarea></textarea>", "TGW100")?;
    assert_rule("<fieldset><legend>x</legend></fieldset>", "TGW100")?;
    Ok(())
}

#[test]
fn tgw100_accepts_supported_elements() -> AppResult<()> {
    assert_clean("<div><span>text</span></div>")?;
    assert_clean("<button>press</button>")?;
    Ok(())
}

#[test]
fn tgw101_flags_form_and_label() -> AppResult<()> {
    assert_rule("<form><div>x</div></form>", "TGW101")?;
    assert_rule("<label>name</label>", "TGW101")?;
    Ok(())
}

#[test]
fn tgw102_flags_media_and_embedding() -> AppResult<()> {
    assert_rule("<video src='x'></video>", "TGW102")?;
    assert_rule("<iframe src='https://x.example'></iframe>", "TGW102")?;
    assert_rule("<dialog open>x</dialog>", "TGW102")?;
    assert_rule("<div popover>x</div>", "TGW102")?;
    assert_rule("<progress value='1'></progress>", "TGW102")?;
    Ok(())
}

#[test]
fn tgw103_flags_service_elements() -> AppResult<()> {
    assert_rule("<link rel='stylesheet' href='x'>", "TGW103")?;
    assert_rule("<meta charset='utf-8'>", "TGW103")?;
    assert_rule("<template><div>x</div></template>", "TGW103")?;
    assert_rule("<math><mi>x</mi></math>", "TGW103")?;
    Ok(())
}

#[test]
fn tgw104_flags_external_scripts() -> AppResult<()> {
    assert_rule("<script src='https://cdn.example/x.js'></script>", "TGW104")?;
    assert_clean("<script>console.log('x')</script>")?;
    Ok(())
}

#[test]
fn tgw105_flags_remote_images() -> AppResult<()> {
    assert_rule("<img src='https://x.example/a.png'>", "TGW105")?;
    assert_rule("<img src='/local.png'>", "TGW105")?;
    assert_clean("<img src='data:image/png;base64,AAAA'>")?;
    Ok(())
}

#[test]
fn tgw106_flags_unsupported_data_image_types() -> AppResult<()> {
    assert_rule("<img src='data:image/svg+xml;base64,AAAA'>", "TGW106")?;
    assert_rule("<img src='data:image/avif;base64,AAAA'>", "TGW106")?;
    assert_clean("<img src='data:image/webp;base64,AAAA'>")?;
    assert_clean("<img src='data:image/jpeg;base64,AAAA'>")?;
    Ok(())
}

#[test]
fn tgw108_flags_legacy_markup() -> AppResult<()> {
    assert_rule("<center>x</center>", "TGW108")?;
    assert_rule("<font color='red'>x</font>", "TGW108")?;
    assert_rule("<div align='center'>x</div>", "TGW108")?;
    assert_rule("<table border='1'>x</table>", "TGW108")?;
    assert_clean("<div style='text-align:center'>x</div>")?;
    Ok(())
}

#[test]
fn tgw109_flags_unsupported_attributes() -> AppResult<()> {
    assert_rule("<div contenteditable='true'>x</div>", "TGW109")?;
    assert_rule("<div draggable='true'>x</div>", "TGW109")?;
    assert_rule("<div tabindex='0'>x</div>", "TGW109")?;
    assert_clean("<div class='card'>x</div>")?;
    Ok(())
}

#[test]
fn tgw110_flags_unknown_elements() -> AppResult<()> {
    assert_rule("<marquee>x</marquee>", "TGW110")?;
    assert_rule("<custom-widget>x</custom-widget>", "TGW110")?;
    assert_clean("<div><canvas></canvas><details><summary>x</summary></details></div>")?;
    Ok(())
}

#[test]
fn tgw111_flags_link_schemes() -> AppResult<()> {
    assert_rule("<a href='http://x.example'>x</a>", "TGW111")?;
    assert_rule("<a href='javascript:alert(1)'>x</a>", "TGW111")?;
    assert_rule("<a href='tg://resolve?domain=x'>x</a>", "TGW111")?;
    assert_rule("<a href='/relative'>x</a>", "TGW111")?;
    assert_clean("<a href='https://x.example'>x</a>")?;
    assert_clean("<a href='#section'>x</a>")?;
    assert_clean("<a>no href</a>")?;
    Ok(())
}

#[test]
fn tgw113_flags_aria_attributes() -> AppResult<()> {
    assert_rule("<div role='button' aria-label='send'>x</div>", "TGW113")?;
    assert_clean("<div class='x'>y</div>")?;
    Ok(())
}

#[test]
fn tgw114_flags_input_hints() -> AppResult<()> {
    assert_rule("<div placeholder='name'>x</div>", "TGW114")?;
    assert_rule("<div autocomplete='on'>x</div>", "TGW114")?;
    assert_clean("<div title='name'>x</div>")?;
    Ok(())
}

#[test]
fn tgw150_flags_unsupported_svg() -> AppResult<()> {
    assert_rule("<svg><clipPath id='c'></clipPath></svg>", "TGW150")?;
    assert_rule("<svg><feGaussianBlur/></svg>", "TGW150")?;
    assert_rule("<svg><foreignObject></foreignObject></svg>", "TGW150")?;
    assert_clean("<svg viewBox='0 0 10 10'><path d='M0 0h10v10z'/></svg>")?;
    Ok(())
}

#[test]
fn tgw302_flags_huge_attributes() -> AppResult<()> {
    let big = "x".repeat(8192 * 2 + 2);
    assert_rule(&format!("<div title='{big}'>x</div>"), "TGW302")?;
    assert_clean(&format!("<div title='{}'>x</div>", "x".repeat(100)))?;
    Ok(())
}

#[test]
fn tgw303_flags_element_count() -> AppResult<()> {
    assert_rule(&"<i></i>".repeat(2001), "TGW303")?;
    assert_eq!(ids(&"<i></i>".repeat(2000))?, Vec::<&'static str>::new());
    Ok(())
}

#[test]
fn tgw304_flags_huge_text() -> AppResult<()> {
    let big = "x".repeat(32768 * 2 + 2);
    assert_rule(&format!("<div>{big}</div>"), "TGW304")?;
    assert_clean("<div>small text</div>")?;
    Ok(())
}

#[test]
fn tgw303_counts_text_and_comment_nodes() -> AppResult<()> {
    let at_limit = format!("{}<!--c-->tail", "<i></i>".repeat(1998));
    assert_eq!(
        ids(&at_limit)?,
        Vec::<&'static str>::new(),
        "2000 nodes is at the limit"
    );
    let over = format!("{}<!--c-->tail", "<i></i>".repeat(1999));
    assert_rule(&over, "TGW303")?;
    Ok(())
}

#[test]
fn tgw304_flags_document_text_total() -> AppResult<()> {
    let chunk = "x".repeat(300);
    let heavy = format!("<div>{chunk}</div>").repeat(200);
    assert_rule(&heavy, "TGW304")?;
    let at_limit = format!("<div>{chunk}</div>").repeat(160);
    assert_eq!(
        ids(&at_limit)?,
        Vec::<&'static str>::new(),
        "48000 units is at the limit"
    );
    let attrs = format!("<div data-x='{chunk}'></div>").repeat(200);
    assert_rule(&attrs, "TGW304")?;
    Ok(())
}

#[test]
fn spans_point_at_the_offending_token() -> AppResult<()> {
    let html = "<div tabindex='0'>x</div>";
    let findings = check(html, &profile()?);
    let finding = findings.first().expect("one finding");
    assert_eq!(finding.rule.id, "TGW109");
    let span = finding.span.expect("has span");
    let start = usize::try_from(span.start).expect("fits");
    let end = usize::try_from(span.end).expect("fits");
    assert_eq!(&html[start..end], "tabindex='0'");
    Ok(())
}

#[test]
fn malformed_html_never_panics() -> AppResult<()> {
    for html in [
        "",
        "<",
        "<div",
        "<div>",
        "</div>",
        "<img src=",
        "<a href=>x</a>",
        "<svg><clipPath>",
        "<<<<>>>>",
        "<script><style><div>"
    ] {
        let _ = check(html, &profile()?);
    }
    Ok(())
}

#[test]
fn clean_widget_document_has_no_findings() -> AppResult<()> {
    assert_clean(
        "<div class='card'>
             <img src='data:image/png;base64,AAAA' alt=''>
             <p>Hello</p>
             <a href='https://example.org'>open</a>
             <button id='b'>press</button>
             <script>
                 document.getElementById('b').addEventListener('click', function () {
                     TelegramWidget.requestSendData({ data: 'ping' });
                 });
             </script>
         </div>"
    )
}
