// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Integration tests for the JavaScript rule engine: every rule gets positive
//! and negative scenarios, plus shadowing, span, offset and parse-error checks.

use masterror::AppResult;
use wl_js::check;
use wl_profile::Profile;

fn profile() -> AppResult<Profile> {
    wl_profile::profile_1()
}

fn ids(js: &str) -> AppResult<Vec<&'static str>> {
    Ok(check(js, 0, &profile()?)?
        .iter()
        .map(|finding| finding.rule.id)
        .collect())
}

fn assert_rule(js: &str, id: &str) -> AppResult<()> {
    let found = ids(js)?;
    assert!(found.contains(&id), "expected {id} in {found:?} for: {js}");
    Ok(())
}

fn assert_clean(js: &str) -> AppResult<()> {
    assert_eq!(ids(js)?, Vec::<&'static str>::new(), "expected clean: {js}");
    Ok(())
}

#[test]
fn tgw400_flags_network_apis() -> AppResult<()> {
    assert_rule("fetch('/x')", "TGW400")?;
    assert_rule("const xhr = new XMLHttpRequest()", "TGW400")?;
    assert_rule("const ws = new WebSocket('wss://x')", "TGW400")?;
    assert_rule("window.fetch('/x')", "TGW400")?;
    assert_rule("navigator.sendBeacon('/x', data)", "TGW400")?;
    assert_rule("new EventSource('/x')", "TGW400")?;
    assert_rule("new BroadcastChannel('x')", "TGW400")?;
    Ok(())
}

#[test]
fn tgw400_accepts_widget_messaging_and_shadowing() -> AppResult<()> {
    assert_clean("TelegramWidget.requestSendData({ data: 'x' })")?;
    assert_clean("function fetch(url) { return url } fetch('/x')")?;
    assert_clean("const data = { worker: 1 }; console.log(data.worker)")?;
    Ok(())
}

#[test]
fn tgw401_flags_navigation() -> AppResult<()> {
    assert_rule("location = 'https://x.example'", "TGW401")?;
    assert_rule("location.href = 'https://x.example'", "TGW401")?;
    assert_rule("window.location = 'https://x.example'", "TGW401")?;
    assert_rule("window.location.href = 'https://x.example'", "TGW401")?;
    assert_rule("history.pushState({}, '', '/x')", "TGW401")?;
    assert_rule("window.open('https://x.example')", "TGW401")?;
    Ok(())
}

#[test]
fn tgw401_accepts_widget_links() -> AppResult<()> {
    assert_clean("TelegramWidget.openLink('https://x.example')")?;
    assert_clean("const path = location.pathname")?;
    assert_clean("const w = { open(url) {} }; w.open('/x')")?;
    Ok(())
}

#[test]
fn tgw402_flags_dialogs() -> AppResult<()> {
    assert_rule("alert('hello')", "TGW402")?;
    assert_rule("const ok = confirm('sure?')", "TGW402")?;
    assert_rule("prompt('name?')", "TGW402")?;
    assert_rule("window.print()", "TGW402")?;
    Ok(())
}

#[test]
fn tgw402_accepts_in_widget_messages() -> AppResult<()> {
    assert_clean("statusEl.textContent = 'hello'")?;
    assert_clean("function confirm(value) { return value } confirm(1)")?;
    Ok(())
}

#[test]
fn tgw403_flags_navigator_apis() -> AppResult<()> {
    assert_rule("navigator.clipboard.writeText('x')", "TGW403")?;
    assert_rule("navigator.share({ title: 'x' })", "TGW403")?;
    assert_rule("navigator.vibrate(5)", "TGW403")?;
    assert_rule(
        "navigator.geolocation.getCurrentPosition(() => {})",
        "TGW403"
    )?;
    assert_rule("new Notification('x')", "TGW403")?;
    Ok(())
}

#[test]
fn tgw403_accepts_widget_alternatives() -> AppResult<()> {
    assert_clean("TelegramWidget.copyText('x')")?;
    assert_clean("const nav = { share: true }; console.log(nav.share)")?;
    Ok(())
}

#[test]
fn tgw404_flags_dom_and_file_apis() -> AppResult<()> {
    assert_rule("el.attachShadow({ mode: 'open' })", "TGW404")?;
    assert_rule(
        "customElements.define('x-c', class extends HTMLElement {})",
        "TGW404"
    )?;
    assert_rule("const blob = new Blob(['x'])", "TGW404")?;
    assert_rule("const reader = new FileReader()", "TGW404")?;
    assert_rule("URL.createObjectURL(blob)", "TGW404")?;
    assert_rule("el.requestFullscreen()", "TGW404")?;
    Ok(())
}

#[test]
fn tgw404_accepts_plain_dom() -> AppResult<()> {
    assert_clean("document.querySelector('.x').classList.add('on')")?;
    assert_clean("const url = new URL('https://x.example')")?;
    Ok(())
}

#[test]
fn tgw405_flags_focus_methods() -> AppResult<()> {
    assert_rule("input.focus()", "TGW405")?;
    assert_rule("input.blur()", "TGW405")?;
    assert_rule("selectEl.select()", "TGW405")?;
    assert_clean("input.setAttribute('readonly', '')")?;
    Ok(())
}

#[test]
fn tgw406_flags_canvas_outside_profile() -> AppResult<()> {
    assert_rule("ctx.getImageData(0, 0, 1, 1)", "TGW406")?;
    assert_rule("canvas.toDataURL()", "TGW406")?;
    assert_rule("canvas.toBlob(() => {})", "TGW406")?;
    assert_rule("ctx.filter = 'blur(2px)'", "TGW406")?;
    assert_rule("ctx.shadowBlur = 4", "TGW406")?;
    assert_rule("ctx.globalCompositeOperation = 'multiply'", "TGW406")?;
    assert_rule("const gl = canvas.getContext('webgl')", "TGW406")?;
    assert_rule("new OffscreenCanvas(1, 1)", "TGW406")?;
    Ok(())
}

#[test]
fn tgw406_accepts_supported_canvas() -> AppResult<()> {
    assert_clean("const ctx = canvas.getContext('2d'); ctx.fillStyle = 'red'")?;
    assert_clean("const kept = items.filter((x) => x.ok)")?;
    assert_clean("const picked = items.select")?;
    Ok(())
}

#[test]
fn tgw408_flags_send_message() -> AppResult<()> {
    assert_rule("TelegramWidget.sendMessage({ text: 'x' })", "TGW408")?;
    assert_clean("TelegramWidget.requestSendData({ data: 'x' })")?;
    Ok(())
}

#[test]
fn tgw411_flags_deprecated_theme_members() -> AppResult<()> {
    assert_rule("const t = TelegramWidget.theme", "TGW411")?;
    assert_rule("TelegramWidget.onTheme = (t) => {}", "TGW411")?;
    assert_clean("TelegramWidget.onThemeChange = (t) => {}")?;
    Ok(())
}

#[test]
fn parse_errors_are_reported() -> AppResult<()> {
    let result = check("function (", 0, &profile()?);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn spans_point_at_the_offending_token() -> AppResult<()> {
    let js = "window.alert('x')";
    let findings = check(js, 0, &profile()?)?;
    let finding = findings.first().expect("one finding");
    assert_eq!(finding.rule.id, "TGW402");
    let span = finding.span.expect("has span");
    let start = usize::try_from(span.start).expect("fits");
    let end = usize::try_from(span.end).expect("fits");
    assert_eq!(&js[start..end], "alert");
    Ok(())
}

#[test]
fn base_offset_shifts_spans() -> AppResult<()> {
    let js = "alert('x')";
    let findings = check(js, 100, &profile()?)?;
    let span = findings
        .first()
        .expect("one finding")
        .span
        .expect("has span");
    assert_eq!(span.start, 100);
    assert_eq!(span.end, 105);
    Ok(())
}

#[test]
fn clean_script_has_no_findings() -> AppResult<()> {
    assert_clean(
        "const btn = document.getElementById('b');
         btn.addEventListener('click', () => {
             TelegramWidget.requestSendData({ data: 'ping' });
         });
         TelegramWidget.ready();
         TelegramWidget.onThemeChange = (theme) => {
             document.body.classList.toggle('dark', theme === 'dark');
         };"
    )
}
