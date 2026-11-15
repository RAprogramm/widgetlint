// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! JavaScript rule checks driven by the oxc AST and the profile out-lists.

use std::collections::HashSet;

use oxc_ast::ast::{
    Argument, AssignmentExpression, AssignmentTarget, BindingIdentifier, CallExpression,
    Expression, IdentifierReference, Program, StaticMemberExpression
};
use oxc_ast_visit::Visit;
use wl_core::{finding::Finding, rule::lookup, severity::Region, span::Span};
use wl_profile::Profile;

/// Global objects whose properties behave like bare globals (`window.fetch`).
const GLOBAL_PREFIXES: &[&str] = &["window", "globalThis", "self", "top", "parent"];

/// Canvas contexts obtained with these `getContext` arguments are outside the
/// profile (TGW406).
const WEBGL_CONTEXTS: &[&str] = &["webgl", "webgl2", "experimental-webgl"];

/// Element methods that do nothing in widgets (TGW405).
const FOCUS_METHODS: &[&str] = &["focus", "blur", "select"];

/// Deprecated `TelegramWidget` members (TGW411).
const DEPRECATED_THEME_MEMBERS: &[&str] = &["theme", "onTheme"];

/// Collects every bound name so shadowed globals are not reported.
#[derive(Default)]
struct Bindings<'a> {
    names: HashSet<&'a str>
}

impl<'a> Visit<'a> for Bindings<'a> {
    fn visit_binding_identifier(&mut self, it: &BindingIdentifier<'a>) {
        self.names.insert(it.name.as_str());
    }
}

/// Walks the AST once and produces the JavaScript findings.
struct Collector<'a> {
    profile:  &'a Profile,
    base:     u32,
    shadowed: HashSet<&'a str>,
    findings: Vec<Finding>
}

impl<'a> Collector<'a> {
    fn push(&mut self, id: &str, span: Span, message: impl Into<String>) {
        let Some(meta) = lookup(id) else {
            return;
        };
        let shifted = Span::new(
            span.start.saturating_add(self.base),
            span.end.saturating_add(self.base)
        );
        self.findings
            .push(Finding::new(meta, message).with_span(shifted, Region::Js));
    }

    fn is_shadowed(&self, name: &str) -> bool {
        self.shadowed.contains(name)
    }

    fn check_static_member(&mut self, member: &StaticMemberExpression<'a>) {
        let prop = member.property.name.as_str();
        let span = to_span(member.property.span);
        let object = ident_name(&member.object);
        let global = object.is_some_and(|name| GLOBAL_PREFIXES.contains(&name));
        if object == Some("TelegramWidget") {
            if prop == "sendMessage" {
                self.push("TGW408", span, "TelegramWidget.sendMessage does not exist");
            } else if DEPRECATED_THEME_MEMBERS.contains(&prop) {
                self.push(
                    "TGW411",
                    span,
                    format!("`TelegramWidget.{prop}` is deprecated, use onThemeChange")
                );
            }
            return;
        }
        if object == Some("history") {
            self.push(
                "TGW401",
                span,
                format!("`history.{prop}` does not work in widgets")
            );
        }
        if object == Some("navigator") && in_list(&self.profile.js.out.navigator, prop) {
            self.push(
                "TGW403",
                span,
                format!("`navigator.{prop}` is unavailable in widgets")
            );
        }
        if (global || object == Some("navigator")) && in_list(&self.profile.js.out.network, prop) {
            self.push(
                "TGW400",
                span,
                format!("network API `{prop}` is disabled in widgets")
            );
        }
        if global && prop == "open" {
            self.push(
                "TGW401",
                span,
                "window.open does not work, use TelegramWidget.openLink"
            );
        }
        if in_list(&self.profile.js.out.canvas_methods, prop) {
            self.push(
                "TGW406",
                span,
                format!("canvas API `{prop}` is outside the profile")
            );
        }
        if in_list(&self.profile.js.out.misc, prop) {
            self.push(
                "TGW404",
                span,
                format!("`{prop}` is unavailable in widgets")
            );
        }
    }

    fn check_call(&mut self, call: &CallExpression<'a>) {
        match &call.callee {
            Expression::Identifier(id) => {
                let name = id.name.as_str();
                if !self.is_shadowed(name) && in_list(&self.profile.js.out.dialogs, name) {
                    self.push(
                        "TGW402",
                        to_span(id.span),
                        format!("`{name}()` does nothing in widgets")
                    );
                }
            }
            Expression::StaticMemberExpression(member) => {
                let prop = member.property.name.as_str();
                let object = ident_name(&member.object);
                if object.is_some_and(|name| GLOBAL_PREFIXES.contains(&name))
                    && in_list(&self.profile.js.out.dialogs, prop)
                {
                    self.push(
                        "TGW402",
                        to_span(member.property.span),
                        format!("`{prop}()` does nothing in widgets")
                    );
                }
                if FOCUS_METHODS.contains(&prop) {
                    self.push(
                        "TGW405",
                        to_span(member.property.span),
                        format!("`{prop}()` does nothing in widgets")
                    );
                }
                if prop == "getContext" && webgl_argument(call) {
                    self.push(
                        "TGW406",
                        to_span(member.property.span),
                        "WebGL contexts are outside the canvas profile"
                    );
                }
            }
            _ => {}
        }
    }

    fn check_assignment(&mut self, assignment: &AssignmentExpression<'a>) {
        match &assignment.left {
            AssignmentTarget::AssignmentTargetIdentifier(id) => {
                let name = id.name.as_str();
                if name == "location" && !self.is_shadowed(name) {
                    self.push(
                        "TGW401",
                        to_span(id.span),
                        "assigning to `location` does nothing in widgets"
                    );
                }
            }
            AssignmentTarget::StaticMemberExpression(member) => {
                let prop = member.property.name.as_str();
                if in_list(&self.profile.js.out.canvas_props, prop) {
                    self.push(
                        "TGW406",
                        to_span(member.property.span),
                        format!("canvas property `{prop}` is outside the profile")
                    );
                }
                let global_location = member.property.name.as_str() == "location"
                    && ident_name(&member.object)
                        .is_some_and(|name| GLOBAL_PREFIXES.contains(&name));
                if uses_location(&member.object) || global_location {
                    self.push(
                        "TGW401",
                        to_span(member.property.span),
                        "assigning to `location` does nothing in widgets"
                    );
                }
            }
            _ => {}
        }
    }
}

impl<'a> Visit<'a> for Collector<'a> {
    fn visit_static_member_expression(&mut self, it: &StaticMemberExpression<'a>) {
        self.check_static_member(it);
        oxc_ast_visit::walk::walk_static_member_expression(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        self.check_call(it);
        oxc_ast_visit::walk::walk_call_expression(self, it);
    }

    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        self.check_assignment(it);
        oxc_ast_visit::walk::walk_assignment_expression(self, it);
    }

    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        let name = it.name.as_str();
        if !self.is_shadowed(name) {
            let out = &self.profile.js.out;
            if in_list(&out.network, name) {
                self.push(
                    "TGW400",
                    to_span(it.span),
                    format!("network API `{name}` is disabled in widgets")
                );
            } else if name.chars().next().is_some_and(char::is_uppercase)
                && in_list(&out.navigator, name)
            {
                self.push(
                    "TGW403",
                    to_span(it.span),
                    format!("`{name}` is unavailable in widgets")
                );
            } else if in_list(&out.misc, name) {
                self.push(
                    "TGW404",
                    to_span(it.span),
                    format!("`{name}` is unavailable in widgets")
                );
            } else if name == "OffscreenCanvas" {
                self.push(
                    "TGW406",
                    to_span(it.span),
                    "OffscreenCanvas is outside the canvas profile"
                );
            }
        }
        oxc_ast_visit::walk::walk_identifier_reference(self, it);
    }
}

/// Whether the call passes a WebGL context name to `getContext`.
fn webgl_argument(call: &CallExpression<'_>) -> bool {
    call.arguments.first().is_some_and(|argument| {
        let Argument::StringLiteral(text) = argument else {
            return false;
        };
        text.value.as_str().is_some_and(|value| {
            WEBGL_CONTEXTS
                .iter()
                .any(|name| value.eq_ignore_ascii_case(name))
        })
    })
}

fn in_list(list: &[String], name: &str) -> bool {
    list.iter().any(|item| item.eq_ignore_ascii_case(name))
}

/// Identifier name of an expression, if it is a bare reference.
fn ident_name<'a>(expression: &Expression<'a>) -> Option<&'a str> {
    match expression {
        Expression::Identifier(id) => Some(id.name.as_str()),
        _ => None
    }
}

/// Whether a member chain reaches `location` or `window.location`.
fn uses_location(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::Identifier(id) => id.name.as_str() == "location",
        Expression::StaticMemberExpression(member) => {
            let global =
                ident_name(&member.object).is_some_and(|name| GLOBAL_PREFIXES.contains(&name));
            (global && member.property.name.as_str() == "location")
                || uses_location(&member.object)
        }
        Expression::ComputedMemberExpression(member) => uses_location(&member.object),
        _ => false
    }
}

const fn to_span(span: oxc_span::Span) -> Span {
    Span::new(span.start, span.end)
}

/// Runs every JavaScript check over one parsed script and returns findings
/// sorted by position.
pub fn run(program: &Program<'_>, profile: &Profile, base: u32) -> Vec<Finding> {
    let mut bindings = Bindings::default();
    bindings.visit_program(program);
    let mut collector = Collector {
        profile,
        base,
        shadowed: bindings.names,
        findings: Vec::new()
    };
    collector.visit_program(program);
    collector
        .findings
        .sort_by_key(|finding| finding.span.map_or(u32::MAX, |span| span.start));
    collector.findings
}
