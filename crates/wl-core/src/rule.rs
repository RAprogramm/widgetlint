// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Rule metadata as data: TOML compiled into the binary, queried by id.

use std::{collections::HashMap, sync::OnceLock};

use serde::Deserialize;

use crate::severity::{Confidence, Severity};

/// TOML shape of the whole rules file.
#[derive(Debug, Deserialize)]
struct RulesFile {
    rules: Vec<RuleData>
}

/// TOML shape of one rule entry.
#[derive(Debug, Deserialize)]
struct RuleData {
    id:         String,
    severity:   SeverityData,
    confidence: ConfidenceData,
    title:      String,
    #[serde(default)]
    help:       Option<String>,
    spec:       String
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum SeverityData {
    Info,
    Warn,
    Error
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ConfidenceData {
    Certain,
    Heuristic
}

/// Static metadata of one rule: id, default severity, texts, spec link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleMeta {
    /// Stable rule code, for example `TGW100`. Never reused.
    pub id:               &'static str,
    /// Default severity before user overrides.
    pub default_severity: Severity,
    /// How sure the rule is.
    pub confidence:       Confidence,
    /// One-line description.
    pub title:            &'static str,
    /// Optional fix hint.
    pub help:             Option<&'static str>,
    /// URL of the specification section.
    pub spec:             &'static str
}

/// Concatenation of all rule metadata files, embedded at compile time.
const RULES_TOML: &str = concat!(
    include_str!("../../../ruleset/rules/size.toml"),
    include_str!("../../../ruleset/rules/html.toml"),
    include_str!("../../../ruleset/rules/css.toml"),
);

/// Returns the registry of all known rules, parsed once.
fn registry() -> &'static HashMap<&'static str, RuleMeta> {
    static REGISTRY: OnceLock<HashMap<&'static str, RuleMeta>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let parsed: RulesFile = toml::from_str(RULES_TOML).unwrap_or_else(|error| {
            unreachable!("embedded rules TOML must be valid: {error}");
        });
        let mut map = HashMap::with_capacity(parsed.rules.len());
        for rule in parsed.rules {
            let meta = RuleMeta {
                id:               leak(rule.id),
                default_severity: match rule.severity {
                    SeverityData::Info => Severity::Info,
                    SeverityData::Warn => Severity::Warn,
                    SeverityData::Error => Severity::Error
                },
                confidence:       match rule.confidence {
                    ConfidenceData::Certain => Confidence::Certain,
                    ConfidenceData::Heuristic => Confidence::Heuristic
                },
                title:            leak(rule.title),
                help:             rule.help.map(leak),
                spec:             leak(rule.spec)
            };
            map.insert(meta.id, meta);
        }
        map
    })
}

/// Looks up rule metadata by id.
///
/// # Examples
///
/// ```
/// assert!(wl_core::rule::lookup("TGW100").is_some());
/// assert!(wl_core::rule::lookup("TGW999").is_none());
/// ```
#[must_use]
pub fn lookup(id: &str) -> Option<&'static RuleMeta> {
    registry().get(id)
}

/// All known rules, ordered by id for stable output.
#[must_use]
pub fn all() -> Vec<&'static RuleMeta> {
    let mut rules: Vec<&'static RuleMeta> = registry().values().collect();
    rules.sort_by_key(|rule| rule.id);
    rules
}

fn leak(text: String) -> &'static str {
    String::leak(text)
}
