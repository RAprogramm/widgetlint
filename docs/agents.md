<!-- SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com> -->
<!-- SPDX-License-Identifier: MIT -->

# AGENTS.md snippet for widget projects

Paste this into the `AGENTS.md` of a repository that ships Telegram widgets.
It tells any coding agent how to validate widget work before finishing.

```markdown
## Telegram widgets

- Widgets are single HTML files (inline `<style>`/`<script>` only, images as
  `data:` URLs) accepted at ≤ 32768 raw and ≤ 4096 zlib-9 bytes.
- After every change to a widget file run:
  `widgetlint check <file> --fail-on warn`
  and fix all reported errors and warnings before finishing the task.
- When you are an AI agent, prefer the machine contract:
  `widgetlint check <file> --format agent --fail-on warn` (compact JSON:
  verdict, fingerprints, one next action), `widgetlint verify <file>` between
  edits, and `--since <fp-file>` to see only findings new since your last run.
- Target `production` unless told otherwise (production means private chats
  only; groups, inline mode and forwards are test-server features).
- Keep the compressed size under the 3800-byte soft budget; run
  `widgetlint size <file>` after large edits.
- Do not add suppression comments (`tgw-disable*`) without a human's explicit
  agreement, and never lower thresholds to make a check pass.
- When unsure about a rule, run `widgetlint explain <TGWxxx>`; the rule pages
  under `docs/rules/` and `llms.txt` of the widgetlint repository document
  every check.
- `alert`/`confirm`/`prompt`, network APIs, external `<script src>`, form
  controls and remote `<img src>` never work in widgets; the linter explains
  the supported alternative for each.
```

## MCP server (optional)

When the agent has no shell, `widgetlint mcp` serves the same checks over the
Model Context Protocol on stdio:

```json
{
  "mcpServers": {
    "widgetlint": { "command": "widgetlint", "args": ["mcp"] }
  }
}
```

The tools are `check` (the full agent report), `verify` (the verdict and the
next action), `explain` (one rule), `rules` (the catalog) and `schema` (the
report contract). The widget source is a tool parameter; the server reads
nothing from disk.
