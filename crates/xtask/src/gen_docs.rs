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
    run_at(&workspace_root()?, check)
}

fn run_at(root: &Path, check: bool) -> AppResult<()> {
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

#[cfg(test)]
mod tests {
    use wl_core::{Confidence, Severity};

    use super::*;

    fn synthetic_rule(id: &'static str, help: Option<&'static str>) -> &'static RuleMeta {
        Box::leak(Box::new(RuleMeta {
            id,
            default_severity: Severity::Warn,
            confidence: Confidence::Certain,
            title: "synthetic rule",
            help,
            spec: "https://core.telegram.org/bots/widgets"
        }))
    }

    #[test]
    fn category_maps_every_id_block() {
        assert_eq!(category("TGW001"), "Size");
        assert_eq!(category("TGW100"), "HTML and SVG");
        assert_eq!(category("TGW200"), "CSS");
        assert_eq!(category("TGW300"), "Document limits");
        assert_eq!(category("TGW400"), "JavaScript");
        assert_eq!(category("TGW"), "Other");
    }

    #[test]
    fn generate_emits_one_page_per_rule_plus_indexes() {
        let rules = wl_core::rule::all();
        let files = generate();
        assert_eq!(files.len(), rules.len() + 3);
        for rule in &rules {
            let expected = PathBuf::from(format!("docs/rules/{}.md", rule.id));
            assert!(
                files.iter().any(|file| file.path == expected),
                "missing page for {}",
                rule.id
            );
        }
        for file in &files {
            assert!(
                file.content.starts_with(SPDX_HEADER),
                "{} lacks the SPDX header",
                file.path.display()
            );
        }
    }

    #[test]
    fn rule_page_renders_fields_and_omits_missing_help() {
        let with_help = synthetic_rule("TGW900", Some("do it differently"));
        let without_help = synthetic_rule("TGW901", None);
        let page = rule_page(with_help);
        assert!(page.contains("# TGW900 — synthetic rule"));
        assert!(page.contains("| Severity | `warn` |"));
        assert!(page.contains("do it differently"));
        assert!(page.contains("Spec: <https://core.telegram.org/bots/widgets>"));
        let bare = rule_page(without_help);
        assert!(
            !bare.contains("do it differently"),
            "missing help must render nothing"
        );
    }

    #[test]
    fn index_and_llms_reference_every_rule() {
        let rules = wl_core::rule::all();
        let index = index_page(&rules);
        let llms = llms_index(&rules);
        let full = llms_full(&rules);
        for rule in &rules {
            assert!(index.contains(rule.id), "index misses {}", rule.id);
            assert!(llms.contains(rule.id), "llms.txt misses {}", rule.id);
            assert!(full.contains(rule.id), "llms-full.txt misses {}", rule.id);
        }
        assert!(index.contains("## CSS"), "index must group by category");
    }

    #[test]
    fn run_at_writes_then_verifies_and_detects_stale() -> AppResult<()> {
        let root = std::env::temp_dir().join(format!("xtask-docs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        run_at(&root, false)?;
        run_at(&root, true)?;
        std::fs::write(root.join("docs/rules/index.md"), "stale")?;
        let error = run_at(&root, true).expect_err("stale docs must be rejected");
        assert!(error.to_string().contains("index.md"), "message: {error}");
        std::fs::remove_dir_all(&root)?;
        Ok(())
    }
}

#[cfg(test)]
mod filesystem_tests {
    use super::*;

    fn root(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("xtask-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        root
    }

    #[test]
    fn a_file_in_place_of_a_directory_stops_the_generation() -> AppResult<()> {
        let root = root("blocked");
        std::fs::write(&root, b"not a directory")?;
        let error = run_at(&root, false).expect_err("a file cannot become a directory");
        assert!(
            error.to_string().contains("cannot create"),
            "message: {error}"
        );
        let _ = std::fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn a_directory_in_place_of_a_file_stops_the_generation() -> AppResult<()> {
        let root = root("blocked-file");
        let target = root.join("docs/rules/index.md");
        std::fs::create_dir_all(&target)?;
        let error = run_at(&root, false).expect_err("a directory cannot be written");
        assert!(
            error.to_string().contains("cannot write"),
            "message: {error}"
        );
        let _ = std::fs::remove_dir_all(&root);
        Ok(())
    }
}
