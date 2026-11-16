<!-- SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com> -->
<!-- SPDX-License-Identifier: MIT -->

# Changelog

All notable changes to `widgetlint` are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.1.0] — 2026-11-16

First release: the full M1 milestone.

### Added

- `widgetlint check` — HTML, CSS and JavaScript rules with exact spans,
  50 rules driven by a single TOML ruleset (`ruleset/`).
- `widgetlint size` — raw and zlib level 9 compressed size with byte-for-byte
  parity against the Python `zlib` reference; hard limit, soft budget and
  near-limit warnings (TGW001–005).
- `widgetlint verify` — minimal pass/fail verdict with `next_action` and
  `--since` fingerprint diffing for edit loops.
- `--format agent` — compact machine report for LLM fix loops: grouped
  repeats, stable fingerprints, caveats, suppression statistics.
- `--format json` and `--format sarif` (GitHub Code Scanning) output.
- `explain`, `rules`, `schema`, `version` commands.
- Config file `widgetlint.toml`: thresholds, rule severity overrides,
  path include/exclude filters.
- Inline suppressions: `tgw-disable`, `tgw-disable-file`,
  `tgw-disable-next-line` in HTML comments, style and script blocks.
- Browser playground (`playground/`) on a standalone WASM crate with the
  pure-Rust zlib backend; sizes there are labelled approximate.
- Generated rule reference (`docs/rules/`), `llms.txt` and `llms-full.txt`
  for AI agents, plus a ready `AGENTS.md` snippet in `docs/agents.md`.
- CI: fmt, clippy pedantic+nursery, nextest, doctests, coverage gate at 95%
  lines, docs freshness, REUSE 3.3, actionlint, yamllint; GitHub Pages
  deployment for the playground.
