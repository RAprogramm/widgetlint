// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The agent report: a compact, machine-first view of one checked document.
//!
//! The contract is designed for LLM fix loops (see `widget-linter-ai-native.md`
//! section 4): a verdict with one next action, findings grouped by fingerprint,
//! hard size numbers, caveats about what was *not* checked, and suppression
//! statistics. Texts are sanitized of control characters and capped so that
//! widget content can never smuggle instructions into the report.
//!
//! # Examples
//!
//! ```
//! use wl_check::{AgentOptions, Config, check};
//! use wl_core::Severity;
//!
//! let profile = wl_profile::profile_1().expect("valid profile");
//! let source = "<input>";
//! let checked = check(source, &profile, &Config::default()).expect("valid widget");
//! let options = AgentOptions {
//!     fail_on: Severity::Warn,
//!     ..AgentOptions::default()
//! };
//! let report = wl_check::agent::report("widget.html", source, &checked, &options);
//! assert_eq!(report.verdict, "fail");
//! assert_eq!(report.findings.first().expect("one finding").id, "TGW100");
//! ```

use std::{
    collections::{BTreeSet, HashMap},
    fmt::Write as _
};

use masterror::{AppError, AppResult};
use schemars::JsonSchema;
use serde::Serialize;
use wl_core::{Finding, LineIndex, Severity, hash::fnv1a32_hex};

use crate::Checked;

/// Findings kept per report before truncation kicks in.
pub const DEFAULT_MAX_FINDINGS: usize = 20;
/// Recommended cap on agent fix-loop iterations.
pub const MAX_RECOMMENDED_ITERATIONS: u32 = 5;
/// Longest kept length of any free text in the report.
const MAX_TEXT_LEN: usize = 200;
/// How many positions a grouped finding lists.
const GROUP_POSITIONS: usize = 3;

/// Tunables for [`report`].
#[derive(Debug, Clone)]
pub struct AgentOptions<'a> {
    /// Severity at or above which the verdict is `fail`.
    pub fail_on:      Severity,
    /// Maximum grouped findings kept in the report.
    pub max_findings: usize,
    /// Fix-loop iteration echoed back to the agent.
    pub iteration:    u32,
    /// Fingerprints from the previous run; new and resolved findings split.
    pub since:        Option<&'a BTreeSet<String>>,
    /// Hard compressed limit of the profile.
    pub limit:        u32,
    /// Soft compressed budget.
    pub budget:       u32,
    /// Tool version reported to the agent.
    pub tool_version: &'a str
}

impl Default for AgentOptions<'_> {
    fn default() -> Self {
        Self {
            fail_on:      Severity::Error,
            max_findings: DEFAULT_MAX_FINDINGS,
            iteration:    1,
            since:        None,
            limit:        4096,
            budget:       3800,
            tool_version: env!("CARGO_PKG_VERSION")
        }
    }
}

/// The serialized agent report.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AgentReport {
    /// Report contract version; additive changes only within a major.
    pub schema_version: u32,
    /// `pass` or `fail` against the `fail_on` threshold.
    pub verdict:        String,
    /// One-line explanation of the verdict.
    pub reason:         String,
    /// Input the report describes.
    pub input:          String,
    /// Compressed-size numbers.
    pub budget:         BudgetBlock,
    /// Counts per severity plus suppressed findings.
    pub counts:         Counts,
    /// Grouped findings, most important first.
    pub findings:       Vec<AgentFinding>,
    /// Findings hidden by `max_findings`.
    pub truncated:      usize,
    /// Fingerprints present in `since` but gone now.
    pub resolved:       Vec<String>,
    /// The single action the agent should take next.
    pub next_action:    String,
    /// Fix-loop guidance.
    pub stop_hint:      StopHint,
    /// What the tool did not check.
    pub caveats:        Vec<String>,
    /// Suppression statistics.
    pub suppressions:   SuppressionStats,
    /// Tool identity.
    pub tool:           ToolInfo
}

/// Compressed-size numbers of the checked document.
#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
pub struct BudgetBlock {
    /// zlib level 9 size in bytes.
    pub zlib9:   usize,
    /// Hard compressed limit.
    pub limit:   u32,
    /// Bytes over the limit, zero when within it.
    pub over_by: usize,
    /// Soft budget.
    pub target:  u32
}

/// Counts per severity.
#[derive(Debug, Clone, Copy, Default, Serialize, JsonSchema)]
pub struct Counts {
    /// Error findings.
    pub error:      usize,
    /// Warning findings.
    pub warn:       usize,
    /// Info findings.
    pub info:       usize,
    /// Findings removed by inline directives.
    pub suppressed: usize
}

/// One grouped finding.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AgentFinding {
    /// Stable fingerprint: rule plus normalized context; identical across runs.
    pub fp:      String,
    /// Rule id.
    pub id:      String,
    /// Severity as lowercase string.
    pub sev:     String,
    /// Confidence as lowercase string.
    pub conf:    String,
    /// Concrete message, sanitized.
    pub msg:     String,
    /// Why it matters: the rule title.
    pub why:     String,
    /// First position of the group, `None` for document-wide findings.
    pub r#where: Option<WhereBlock>,
    /// Fix guidance.
    pub fix:     FixBlock,
    /// How many findings share this fingerprint.
    pub repeat:  usize,
    /// First few 1-based lines of the group.
    pub lines:   Vec<u32>
}

/// Resolved 1-based position.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct WhereBlock {
    /// 1-based line.
    pub line:   u32,
    /// 1-based column in code points.
    pub col:    u32,
    /// Region name: `html`, `css`, `js` or `size`.
    pub region: String
}

/// Fix guidance for one finding.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct FixBlock {
    /// Applicability level; automatic edits arrive with the `fix` command.
    pub applicability: String,
    /// Human-readable fix hint.
    pub hint:          Option<String>
}

/// Fix-loop guidance.
#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
pub struct StopHint {
    /// Iteration echoed from the caller.
    pub iteration:       u32,
    /// Iterations after which a human should take over.
    pub max_recommended: u32
}

/// Suppression statistics.
#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
pub struct SuppressionStats {
    /// Findings removed by directives.
    pub used:    usize,
    /// Directives that suppressed nothing.
    pub ignored: usize
}

/// Tool identity.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ToolInfo {
    /// Tool name.
    pub name:    String,
    /// Tool version.
    pub version: String,
    /// Ruleset fingerprint; equal fingerprints enforce identical rules.
    pub ruleset: String
}

/// Builds the agent report for one checked document.
///
/// * `path` — input name as given by the caller.
/// * `source` — the checked text, used for fingerprints and positions.
/// * `checked` — pipeline result for the same text.
/// * `options` — thresholds, limits and previous-run fingerprints.
///
/// # Examples
///
/// ```
/// use wl_check::{AgentOptions, Config, check};
///
/// let profile = wl_profile::profile_1().expect("valid profile");
/// let checked = check("<b>x</b>", &profile, &Config::default()).expect("valid widget");
/// let report = wl_check::agent::report("-", "<b>x</b>", &checked, &AgentOptions::default());
/// assert_eq!(report.verdict, "pass");
/// ```
#[must_use]
pub fn report(
    path: &str,
    source: &str,
    checked: &Checked,
    options: &AgentOptions<'_>
) -> AgentReport {
    let index = LineIndex::new(source);
    let mut counts = Counts {
        suppressed: checked.suppressed,
        ..Counts::default()
    };
    for finding in &checked.findings {
        match finding.severity {
            Severity::Error => counts.error += 1,
            Severity::Warn => counts.warn += 1,
            Severity::Info => counts.info += 1
        }
    }
    let mut groups = group(&checked.findings, source, &index);
    let current: BTreeSet<String> = groups.iter().map(|group| group.fp.clone()).collect();
    if let Some(since) = options.since {
        groups.retain(|group| !since.contains(&group.fp));
    }
    let kept = options.max_findings.min(groups.len());
    let truncated: usize = groups.iter().skip(kept).map(|group| group.repeat).sum();
    let resolved = options.since.map_or_else(Vec::new, |since| {
        since.difference(&current).cloned().collect()
    });
    let failed = checked
        .findings
        .iter()
        .any(|finding| severity_rank(finding.severity) <= severity_rank(options.fail_on));
    let over_by = checked
        .size
        .zlib9
        .saturating_sub(usize::try_from(options.limit).unwrap_or(usize::MAX));
    let mut caveats = vec![
        "dynamic limits (CPU time, memory, rendered height) are not verified".to_owned(),
        "runtime behavior is not simulated; findings come from static analysis only".to_owned(),
    ];
    if checked
        .findings
        .iter()
        .any(|finding| finding.rule.id == "TGW216")
    {
        caveats.push(
            "unknown CSS properties are matched against a curated table; verify new properties \
             against the current client build"
                .to_owned()
        );
    }
    if checked
        .findings
        .iter()
        .any(|finding| finding.rule.confidence == wl_core::Confidence::Heuristic)
    {
        caveats.push(
            "findings with `conf: heuristic` are guesses and may be false positives".to_owned()
        );
    }
    AgentReport {
        schema_version: wl_core::SCHEMA_VERSION,
        verdict: if failed { "fail" } else { "pass" }.to_owned(),
        reason: reason(&counts),
        input: clean(path),
        budget: BudgetBlock {
            zlib9: checked.size.zlib9,
            limit: options.limit,
            over_by,
            target: options.budget
        },
        counts,
        findings: groups
            .into_iter()
            .take(kept)
            .map(AgentFinding::from)
            .collect(),
        truncated,
        resolved,
        next_action: next_action(over_by, &checked.findings),
        stop_hint: StopHint {
            iteration:       options.iteration,
            max_recommended: MAX_RECOMMENDED_ITERATIONS
        },
        caveats,
        suppressions: SuppressionStats {
            used:    checked.suppressed,
            ignored: checked.unused_directives
        },
        tool: ToolInfo {
            name:    "widgetlint".to_owned(),
            version: options.tool_version.to_owned(),
            ruleset: format!("fnv1a64:{}", wl_core::rule::fingerprint())
        }
    }
}

/// Publishes the JSON Schema of [`AgentReport`] for CI validation and agents.
///
/// # Errors
///
/// [`masterror::AppError`] with `Internal` taxonomy if serialization fails.
///
/// # Examples
///
/// ```
/// let schema = wl_check::agent::schema_json().expect("valid schema");
/// assert!(schema.contains("\"AgentReport\""));
/// ```
pub fn schema_json() -> AppResult<String> {
    let schema = schemars::schema_for!(AgentReport);
    serde_json::to_string_pretty(&schema)
        .map_err(|error| AppError::internal(format!("agent schema: {error}")))
}

/// One grouped finding under construction.
struct Group {
    fp:      String,
    id:      String,
    sev:     Severity,
    conf:    String,
    msg:     String,
    why:     String,
    r#where: Option<WhereBlock>,
    hint:    Option<String>,
    repeat:  usize,
    lines:   Vec<u32>,
    first:   u32
}

impl From<Group> for AgentFinding {
    fn from(group: Group) -> Self {
        Self {
            fp:      group.fp,
            id:      group.id,
            sev:     group.sev.as_str().to_owned(),
            conf:    group.conf,
            msg:     group.msg,
            why:     group.why,
            r#where: group.r#where,
            fix:     FixBlock {
                applicability: "manual".to_owned(),
                hint:          group.hint
            },
            repeat:  group.repeat,
            lines:   group.lines
        }
    }
}

/// Groups findings by fingerprint, most severe first, then document order.
fn group(findings: &[Finding], source: &str, index: &LineIndex) -> Vec<Group> {
    let mut order: HashMap<String, usize> = HashMap::new();
    let mut groups: Vec<Group> = Vec::new();
    for finding in findings {
        let position = finding.span.map(|span| index.position(source, span.start));
        let fp = fingerprint(finding, source, index);
        let line = position.map_or(0, |position| position.line);
        let entry = order.get(&fp).and_then(|&i| groups.get_mut(i));
        if let Some(existing) = entry {
            existing.repeat += 1;
            if existing.lines.len() < GROUP_POSITIONS && !existing.lines.contains(&line) {
                existing.lines.push(line);
            }
            continue;
        }
        order.insert(fp.clone(), groups.len());
        groups.push(Group {
            fp,
            id: finding.rule.id.to_owned(),
            sev: finding.severity,
            conf: finding.rule.confidence.as_str().to_owned(),
            msg: clean(&finding.message),
            why: clean(finding.rule.title),
            r#where: position.map(|position| WhereBlock {
                line:   position.line,
                col:    position.col,
                region: finding.region.as_str().to_owned()
            }),
            hint: finding.effective_help().map(clean),
            repeat: 1,
            lines: if line == 0 { Vec::new() } else { vec![line] },
            first: finding.span.map_or(u32::MAX, |span| span.start)
        });
    }
    groups.sort_by_key(|group| (severity_rank(group.sev), group.first));
    groups
}

/// Fingerprint: rule id plus the normalized line the finding points at.
///
/// Document-wide findings hash their digit-masked message instead.
fn fingerprint(finding: &Finding, source: &str, index: &LineIndex) -> String {
    let context = finding.span.map_or_else(
        || {
            finding
                .message
                .chars()
                .map(|c| if c.is_ascii_digit() { '#' } else { c })
                .collect()
        },
        |span| clean(line_at(source, index, span.start))
    );
    fnv1a32_hex(format!("{}\u{0}{context}", finding.rule.id).as_bytes())
}

/// The full line containing `offset`.
fn line_at<'a>(source: &'a str, index: &LineIndex, offset: u32) -> &'a str {
    let starts = index.line_starts();
    let line = usize::try_from(index.position(source, offset).line).unwrap_or(1);
    let from = starts
        .get(line.saturating_sub(1))
        .and_then(|&start| usize::try_from(start).ok())
        .unwrap_or(0);
    let to = starts
        .get(line)
        .and_then(|&start| usize::try_from(start).ok())
        .unwrap_or(source.len());
    source.get(from..to).unwrap_or_default()
}

/// Collapses whitespace, drops control characters and caps the length so that
/// widget content cannot smuggle instructions or bloat the report.
fn clean(text: &str) -> String {
    let mut out = String::new();
    let mut pending_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() || ch.is_control() {
            pending_space = !out.is_empty();
            continue;
        }
        if pending_space {
            out.push(' ');
            pending_space = false;
        }
        out.push(ch);
    }
    if out.chars().count() > MAX_TEXT_LEN {
        let cut: String = out.chars().take(MAX_TEXT_LEN).collect();
        let mut capped = cut;
        let _ = write!(capped, "...");
        return capped;
    }
    out
}

const fn severity_rank(severity: Severity) -> u8 {
    match severity {
        Severity::Error => 0,
        Severity::Warn => 1,
        Severity::Info => 2
    }
}

fn reason(counts: &Counts) -> String {
    if counts.error + counts.warn + counts.info == 0 {
        return "no findings".to_owned();
    }
    let mut parts: Vec<String> = Vec::new();
    if counts.error > 0 {
        parts.push(format!("{} error(s)", counts.error));
    }
    if counts.warn > 0 {
        parts.push(format!("{} warning(s)", counts.warn));
    }
    if counts.info > 0 {
        parts.push(format!("{} info(s)", counts.info));
    }
    parts.join(", ")
}

fn next_action(over_by: usize, findings: &[Finding]) -> String {
    if over_by > 0 {
        return "reduce the compressed size below the hard limit, then re-run widgetlint check"
            .to_owned();
    }
    let mut has_error = false;
    let mut has_warn = false;
    let mut has_info = false;
    for finding in findings {
        match finding.severity {
            Severity::Error => has_error = true,
            Severity::Warn => has_warn = true,
            Severity::Info => has_info = true
        }
    }
    if has_error {
        return "fix the error findings above, then re-run widgetlint check".to_owned();
    }
    if has_warn {
        return "warnings are silently changed or removed by the client; fix them or accept the \
                behavior"
            .to_owned();
    }
    if has_info {
        return "no blocking findings; info items are advisory".to_owned();
    }
    "no findings: the widget matches profile 1".to_owned()
}
