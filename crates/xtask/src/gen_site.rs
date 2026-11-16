// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Generates the browsable rule catalog for the web: one HTML page per rule
//! and an index with search and filters.
//!
//! The site is a build artifact: it lives under `playground/rules/` and is
//! published next to the playground on GitHub Pages. The source of truth stays
//! the rule registry, so the Markdown pages and the site never diverge.

use std::{collections::BTreeMap, fmt::Write as _, path::Path};

use masterror::AppResult;
use wl_core::RuleMeta;

/// Directory of the generated site, relative to the workspace root.
const SITE_DIR: &str = "playground/rules";

/// Generates the rule site under `playground/rules/`.
///
/// # Errors
///
/// [`masterror::AppError`] with `Internal` taxonomy when a file cannot be
/// written.
pub fn run() -> AppResult<()> {
    run_at(&crate::gen_docs::workspace_root()?)
}

fn run_at(root: &Path) -> AppResult<()> {
    let mut rules = wl_core::rule::all();
    rules.sort_by_key(|rule| rule.id);
    let site = root.join(SITE_DIR);
    write(&site.join("index.html"), &catalog_page(&rules))?;
    write(&site.join("style.css"), STYLE_CSS)?;
    write(&site.join("app.js"), APP_JS)?;
    for rule in &rules {
        write(&site.join(format!("{}.html", rule.id)), &rule_page(rule))?;
    }
    let machine = root.join("llms.txt");
    if machine.exists() {
        let text = std::fs::read_to_string(&machine)?;
        write(&root.join("playground/llms.txt"), &text)?;
    }
    println!("generated the rule site in {SITE_DIR}");
    Ok(())
}

/// Writes one file, creating its parent directory.
fn write(path: &Path, content: &str) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(())
}

/// Escapes text for HTML.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other)
        }
    }
    out
}

/// Rule category derived from the id block.
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

/// Search terms of one rule: the title, the fix hint and the category, so a
/// search for a keyword like `fetch` finds the rule that mentions it.
fn search_terms(rule: &RuleMeta) -> String {
    let mut terms = vec![rule.id, rule.title, category(rule.id)];
    if let Some(help) = rule.help {
        terms.push(help);
    }
    if !rule.keywords.is_empty() {
        terms.push(rule.keywords);
    }
    terms.join(" ")
}

/// Renders the catalog page with every rule.
fn catalog_page(rules: &[&'static RuleMeta]) -> String {
    let mut grouped: BTreeMap<&str, Vec<&'static RuleMeta>> = BTreeMap::new();
    for rule in rules {
        grouped.entry(category(rule.id)).or_default().push(rule);
    }
    let mut page = String::from(
        "<!DOCTYPE html>\n<html lang=\"en\" data-theme=\"auto\">\n<head>\n\
         <meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>widgetlint rules — the Telegram widget rule catalog</title>\n\
         <meta name=\"description\" content=\"Every rule widgetlint enforces on a Telegram Bot API 10.4 widget, with severity, confidence and the fix for each.\">\n\
         <meta name=\"color-scheme\" content=\"light dark\">\n\
         <link rel=\"icon\" href=\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Crect width='32' height='32' rx='8' fill='%232f6feb'/%3E%3Cpath d='M9 16.5l4.5 4.5L23 11.5' stroke='white' stroke-width='3' fill='none' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E\">\n\
         <link rel=\"stylesheet\" href=\"style.css\">\n</head>\n<body>\n\
         <header class=\"masthead\">\n  <a class=\"back\" href=\"../\">← Playground</a>\n  <h1>Rule catalog</h1>\n\
         <p class=\"lead\">Every rule <code>widgetlint</code> enforces on a Telegram Bot API 10.4 widget. \
         <span class=\"error\">error</span> means the server refuses the widget, \
         <span class=\"warn\">warn</span> means the client silently changes or removes behavior, \
         <span class=\"info\">info</span> is advisory.</p>\n</header>\n\
         <main>\n\
         <section class=\"controls\">\n  <input id=\"search\" type=\"search\" placeholder=\"Search by id, title or keyword…\" aria-label=\"Search rules\">\n  <div class=\"chips\" id=\"categories\"></div>\n  <div class=\"chips\" id=\"severities\"></div>\n</section>\n<p class=\"count\" id=\"count\"></p>\n"
    );
    for (name, group) in &grouped {
        let _ = write!(
            page,
            "<section class=\"group\" data-category=\"{}\">\n<h2>{} <span class=\"pill\">{}</span></h2>\n<ul class=\"rules\">\n",
            escape(name),
            escape(name),
            group.len()
        );
        for rule in group {
            let _ = write!(
                page,
                "<li class=\"rule\" data-id=\"{id}\" data-severity=\"{severity}\" data-category=\"{category}\" \
                 data-search=\"{search}\">\n  <a href=\"{id}.html\"><code>{id}</code></a>\n  \
                 <span class=\"sev sev--{severity}\">{severity}</span>\n  <span class=\"title\">{title}</span>\n</li>\n",
                id = escape(rule.id),
                severity = escape(rule.default_severity.as_str()),
                category = escape(category(rule.id)),
                title = escape(rule.title),
                search = escape(&search_terms(rule))
            );
        }
        page.push_str("</ul>\n</section>\n");
    }
    page.push_str(
        "</main>\n\
         <footer>\n  <p><a href=\"../\">Try your widget in the playground</a> · \
         <a href=\"../llms.txt\">llms.txt</a> · \
         <a href=\"https://github.com/RAprogramm/widgetlint\">Source</a></p>\n</footer>\n\
         <script src=\"app.js\"></script>\n</body>\n</html>\n"
    );
    page
}

/// Renders one rule page.
fn rule_page(rule: &RuleMeta) -> String {
    let keywords = if rule.keywords.is_empty() {
        String::new()
    } else {
        format!("\n  <dt>Keywords</dt><dd>{}</dd>", escape(rule.keywords))
    };
    let help = rule.help.map_or_else(String::new, |help| {
        format!(
            "<section class=\"help\"><h2>How to fix</h2><p>{}</p></section>\n",
            escape(help)
        )
    });
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\" data-theme=\"auto\">\n<head>\n\
         <meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>{id} — widgetlint rule</title>\n\
         <meta name=\"description\" content=\"{title}\">\n\
         <meta name=\"color-scheme\" content=\"light dark\">\n\
         <link rel=\"icon\" href=\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Crect width='32' height='32' rx='8' fill='%232f6feb'/%3E%3Cpath d='M9 16.5l4.5 4.5L23 11.5' stroke='white' stroke-width='3' fill='none' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E\">\n\
         <link rel=\"stylesheet\" href=\"style.css\">\n</head>\n<body>\n\
         <header class=\"masthead\">\n  <a class=\"back\" href=\"index.html\">← All rules</a>\n  <h1><code>{id}</code> {title}</h1>\n</header>\n\
         <main>\n\
         <dl class=\"meta\">\n  <dt>Severity</dt><dd><span class=\"sev sev--{severity}\">{severity}</span></dd>\n  \
         <dt>Confidence</dt><dd>{confidence}</dd>\n  <dt>Category</dt><dd>{category}</dd>\n{keywords}</dl>\n\
         {help}<section class=\"spec\"><h2>Specification</h2><p><a href=\"{spec}\">{spec}</a></p></section>\n\
         <p class=\"cta\"><a href=\"../\">Check a widget in the playground →</a></p>\n\
         </main>\n\
         <footer>\n  <p><a href=\"index.html\">Rule catalog</a> · \
         <a href=\"https://github.com/RAprogramm/widgetlint\">Source</a></p>\n</footer>\n\
         <script src=\"app.js\"></script>\n</body>\n</html>\n",
        id = escape(rule.id),
        title = escape(rule.title),
        severity = escape(rule.default_severity.as_str()),
        confidence = escape(rule.confidence.as_str()),
        category = escape(category(rule.id)),
        spec = escape(rule.spec),
        keywords = keywords,
        help = help
    )
}

/// Styles of the rule site.
const STYLE_CSS: &str = include_str!("site/style.css");

/// Behaviour of the rule site.
const APP_JS: &str = include_str!("site/app.js");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_catalog_lists_every_rule() {
        let rules = wl_core::rule::all();
        let page = catalog_page(&rules);
        for rule in &rules {
            assert!(
                page.contains(&format!("data-id=\"{}\"", rule.id)),
                "the catalog misses {}",
                rule.id
            );
        }
        assert!(page.contains("TGW100"));
    }

    #[test]
    fn a_rule_page_carries_the_metadata() {
        let rule = wl_core::rule::lookup("TGW100").expect("TGW100 exists");
        let page = rule_page(rule);
        assert!(page.contains("TGW100"));
        assert!(page.contains("warn"));
        assert!(page.contains("certain"));
        assert!(page.contains("core.telegram.org"));
    }

    #[test]
    fn the_site_is_written_to_disk() -> AppResult<()> {
        let root = std::env::temp_dir().join(format!("xtask-site-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root)?;
        std::fs::write(root.join("llms.txt"), "catalog")?;
        run_at(&root)?;
        assert!(root.join("playground/llms.txt").is_file());
        let index = root.join("playground/rules/index.html");
        assert!(index.is_file(), "the catalog must be written");
        let page = root.join("playground/rules/TGW100.html");
        assert!(page.is_file(), "every rule needs a page");
        let text = std::fs::read_to_string(&index)?;
        assert!(text.contains("TGW100"));
        let _ = std::fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn markup_is_escaped() {
        assert_eq!(escape("<script>"), "&lt;script&gt;");
        assert_eq!(escape("a & b"), "a &amp; b");
        assert_eq!(escape("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(escape("it's"), "it&#39;s");
    }

    #[test]
    fn an_unknown_id_block_has_its_own_category() {
        assert_eq!(category("TGW999"), "Other");
        assert_eq!(category("TGW"), "Other");
    }
}
