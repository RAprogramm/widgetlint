// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Generates the rule documentation, `llms.txt` and `llms-full.txt` from the
//! embedded rule registry, so every consumer — humans, IDE agents and MCP
//! documentation servers — reads exactly what the linter enforces.

use std::{
    collections::BTreeMap,
    fmt::Write as _,
    path::{Path, PathBuf}
};

use masterror::{AppError, AppResult};
use wl_core::RuleMeta;

/// SPDX header every generated file carries so the repository passes
/// `reuse lint`.
const SPDX_HEADER: &str = concat!(
    "<!-- SPDX-",
    "FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com> -->\n",
    "<!-- SPDX-",
    "License-Identifier: MIT -->\n\n",
);

/// One generated file: workspace-relative path and content.
struct Generated {
    path:    PathBuf,
    content: String
}

/// Generates every documentation file, or verifies them when `check`.
pub fn run(check: bool) -> AppResult<()> {
    let root = workspace_root()?;
    let files = generate();
    if check {
        let stale: Vec<String> = files
            .iter()
            .filter(|file| {
                std::fs::read_to_string(root.join(&file.path))
                    .map_or(true, |current| current != file.content)
            })
            .map(|file| file.path.display().to_string())
            .collect();
        if stale.is_empty() {
            println!("docs are up to date");
            return Ok(());
        }
        return Err(AppError::bad_request(format!(
            "generated docs are stale: {}; run `cargo run -p xtask -- gen-docs`",
            stale.join(", ")
        )));
    }
    for file in &files {
        let target = root.join(&file.path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                AppError::internal(format!("cannot create '{}': {error}", parent.display()))
            })?;
        }
        std::fs::write(&target, &file.content).map_err(|error| {
            AppError::internal(format!("cannot write '{}': {error}", target.display()))
        })?;
    }
    println!("generated {} documentation files", files.len());
    Ok(())
}

fn workspace_root() -> AppResult<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(PathBuf::from)
        .ok_or_else(|| AppError::internal("cannot locate the workspace root"))
}

fn generate() -> Vec<Generated> {
    let mut rules = wl_core::rule::all();
    rules.sort_by_key(|rule| rule.id);
    let mut files: Vec<Generated> = rules
        .iter()
        .map(|rule| Generated {
            path:    PathBuf::from(format!("docs/rules/{}.md", rule.id)),
            content: rule_page(rule)
        })
        .collect();
    files.push(Generated {
        path:    PathBuf::from("docs/rules/index.md"),
        content: index_page(&rules)
    });
    files.push(Generated {
        path:    PathBuf::from("llms.txt"),
        content: llms_index(&rules)
    });
    files.push(Generated {
        path:    PathBuf::from("llms-full.txt"),
        content: llms_full(&rules)
    });
    for file in &mut files {
        file.content.insert_str(0, SPDX_HEADER);
    }
    files
}

/// Rule category derived from the id block: `TGW0xx` size, `1xx` HTML and
/// SVG, `2xx` CSS, `3xx` document limits, `4xx` JavaScript.
fn category(id: &str) -> &'static str {
    match id.as_bytes().get(3) {
        Some(b'0') => "Size",
        Some(b'1') => "HTML and SVG",
        Some(b'2') => "CSS",
        Some(b'3') => "Document limits",
        Some(b'4') => "JavaScript",
        _ => "Other"
    }
}

fn rule_page(rule: &RuleMeta) -> String {
    let help = rule
        .help
        .map_or_else(String::new, |help| format!("{help}\n\n"));
    format!(
        "# {} — {}\n\n| Field | Value |\n|---|---|\n| Severity | `{}` |\n| Confidence | `{}` |\n\
         | Category | {} |\n\n{help}Spec: <{}>\n",
        rule.id,
        rule.title,
        rule.default_severity.as_str(),
        rule.confidence.as_str(),
        category(rule.id),
        rule.spec
    )
}

fn index_page(rules: &[&'static RuleMeta]) -> String {
    let mut grouped: BTreeMap<&str, Vec<&'static RuleMeta>> = BTreeMap::new();
    for rule in rules {
        grouped.entry(category(rule.id)).or_default().push(rule);
    }
    let mut page = String::from(
        "# Rule index\n\n\
         Every rule the linter enforces, grouped by category. Severity `error` means the server \
         refuses the widget; `warn` means the client silently changes or removes behavior; `info` \
         is advisory.\n"
    );
    for (name, group) in grouped {
        let _ = write!(
            page,
            "\n## {name}\n\n| Rule | Severity | Confidence | Title |\n|---|---|---|---|\n"
        );
        for rule in group {
            let _ = writeln!(
                page,
                "| [{}]({}.md) | {} | {} | {} |",
                rule.id,
                rule.id,
                rule.default_severity.as_str(),
                rule.confidence.as_str(),
                rule.title
            );
        }
    }
    page
}

fn llms_index(rules: &[&'static RuleMeta]) -> String {
    let mut page = String::from(
        "# widgetlint\n\n\
         > Validator, linter and optimizer for Telegram Bot API 10.4 widgets: catches everything \
         the client silently strips and everything the server refuses, before sending.\n\n\
         ## Rule reference\n\n"
    );
    for rule in rules {
        let _ = writeln!(
            page,
            "- [{} — {}](docs/rules/{}.md): {}",
            rule.id, rule.title, rule.id, rule.title
        );
    }
    page.push_str(
        "\n## Optional\n\n\
         - [Rule index](docs/rules/index.md): every rule in one table\n\
         - [Full catalog](llms-full.txt): all rule pages in a single file\n\
         - [Telegram widget specification](https://core.telegram.org/bots/widgets): the upstream \
         source of truth\n"
    );
    page
}

fn llms_full(rules: &[&'static RuleMeta]) -> String {
    let mut page = String::from(
        "# widgetlint — full rule catalog\n\n\
         Generated by `cargo run -p xtask -- gen-docs`; never edit by hand.\n"
    );
    for rule in rules {
        page.push_str("\n---\n\n");
        page.push_str(&rule_page(rule));
    }
    page
}
