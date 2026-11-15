// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Inline suppression directives: `tgw-disable`, `tgw-disable-file`,
//! `tgw-disable-next-line`.
//!
//! The parser is syntax-agnostic: callers pass directive text (HTML comment
//! bodies, JS/CSS line comments) together with the range it applies to.

use std::{cell::Cell, collections::HashSet};

use crate::span::Span;

/// Parsed directive kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectiveKind {
    /// Suppress the listed rules for one following range.
    Disable,
    /// Suppress the listed rules for the whole file.
    DisableFile,
    /// Suppress the listed rules for the following line.
    DisableNextLine
}

/// A parsed inline directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Directive {
    /// What scope the directive has.
    pub kind:  DirectiveKind,
    /// Rule ids it lists; empty means "all rules".
    pub rules: Vec<String>
}

/// Parses directive text such as `tgw-disable TGW100, TGW101`.
///
/// Returns `None` when the text is not a directive.
///
/// # Examples
///
/// ```
/// let d = wl_core::suppress::parse_directive(" tgw-disable-file TGW606 ").unwrap();
/// assert_eq!(d.kind, wl_core::suppress::DirectiveKind::DisableFile);
/// assert_eq!(d.rules, vec!["TGW606".to_owned()]);
/// assert!(wl_core::suppress::parse_directive("tgw-disabled TGW1").is_none());
/// ```
#[must_use]
pub fn parse_directive(text: &str) -> Option<Directive> {
    let trimmed = text.trim();
    let (kind, rest) = if let Some(rest) = trimmed.strip_prefix("tgw-disable-file") {
        (DirectiveKind::DisableFile, rest)
    } else if let Some(rest) = trimmed.strip_prefix("tgw-disable-next-line") {
        (DirectiveKind::DisableNextLine, rest)
    } else {
        let rest = trimmed.strip_prefix("tgw-disable")?;
        (DirectiveKind::Disable, rest)
    };
    if rest.chars().next().is_some_and(|c| !c.is_whitespace()) {
        return None;
    }
    let rules = rest
        .split([',', ' ', '\t', '\n'])
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect();
    Some(Directive {
        kind,
        rules
    })
}

/// A rule set: `None` means every rule, `Some(set)` only the listed ids.
type RuleSet = Option<HashSet<String>>;

/// Collected suppressions for one file.
///
/// The collector also records which directives actually suppressed a
/// finding, so callers can report directives that did nothing.
#[derive(Debug, Clone)]
pub struct Suppressions {
    file_rules: RuleSet,
    file_used:  Cell<bool>,
    ranges:     Vec<ScopedRule>
}

/// One ranged directive with a used marker.
#[derive(Debug, Clone)]
struct ScopedRule {
    span:  Span,
    rules: RuleSet,
    used:  Cell<bool>
}

impl Default for Suppressions {
    /// An empty collector suppresses nothing.
    fn default() -> Self {
        Self {
            file_rules: Some(HashSet::new()),
            file_used:  Cell::new(false),
            ranges:     Vec::new()
        }
    }
}

impl Suppressions {
    /// Records a file-wide directive.
    pub fn add_file(&mut self, rules: &[String]) {
        merge(&mut self.file_rules, rules);
    }

    /// Records a directive scoped to `span`.
    pub fn add_range(&mut self, span: Span, rules: &[String]) {
        let mut set: RuleSet = Some(HashSet::new());
        merge(&mut set, rules);
        self.ranges.push(ScopedRule {
            span,
            rules: set,
            used: Cell::new(false)
        });
    }

    /// Checks whether `rule` at `span` is suppressed.
    ///
    /// A finding without a span can only be suppressed file-wide.
    #[must_use]
    pub fn is_suppressed(&self, rule: &str, span: Option<Span>) -> bool {
        if allows(&self.file_rules, rule) {
            self.file_used.set(true);
            return true;
        }
        let Some(span) = span else {
            return false;
        };
        self.ranges.iter().any(|scoped| {
            let hit = span.start >= scoped.span.start
                && span.end <= scoped.span.end
                && allows(&scoped.rules, rule);
            if hit {
                scoped.used.set(true);
            }
            hit
        })
    }

    /// Number of directives that suppressed no finding at all.
    ///
    /// A file-wide directive counts as one directive, ranged directives count
    /// individually.
    ///
    /// # Examples
    ///
    /// ```
    /// let suppressions = wl_core::Suppressions::default();
    /// assert_eq!(suppressions.unused_count(), 0);
    /// ```
    #[must_use]
    pub fn unused_count(&self) -> usize {
        let file_present = self
            .file_rules
            .as_ref()
            .is_none_or(|rules| !rules.is_empty());
        usize::from(file_present && !self.file_used.get())
            + self
                .ranges
                .iter()
                .filter(|scoped| !scoped.used.get())
                .count()
    }
}

fn allows(set: &RuleSet, rule: &str) -> bool {
    set.as_ref().is_none_or(|rules| rules.contains(rule))
}

fn merge(target: &mut RuleSet, rules: &[String]) {
    if rules.is_empty() {
        *target = None;
        return;
    }
    if let Some(set) = target {
        set.extend(rules.iter().cloned());
    }
}
