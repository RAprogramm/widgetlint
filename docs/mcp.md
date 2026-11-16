<!-- SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com> -->
<!-- SPDX-License-Identifier: MIT -->

# MCP server

`widgetlint mcp` starts a Model Context Protocol server over stdio. It is the
same binary as the CLI: one download, two front ends.

## Client configuration

```json
{
  "mcpServers": {
    "widgetlint": {
      "command": "widgetlint",
      "args": ["mcp"]
    }
  }
}
```

With a release binary:

```json
{
  "mcpServers": {
    "widgetlint": {
      "command": "/usr/local/bin/widgetlint",
      "args": ["mcp"]
    }
  }
}
```

The server writes the protocol to the standard output and nothing else, so it
must not be started from an interactive shell: it waits for a client.

## Tools

| Tool | Parameters | Result |
|---|---|---|
| `check` | `source`, optional `path`, `budget`, `target`, `min_severity`, `min_confidence`, `fail_on` | the full agent report: verdict, size budget, grouped findings with positions, fix hints, next action |
| `verify` | `source`, optional `path`, `fail_on` | the verdict, the reason, the counts and the single next action |
| `explain` | `rule_id` | severity, confidence, title, fix hint and the specification link of one rule |
| `rules` | optional `severity` | the catalog of every known rule |
| `schema` | none | the JSON Schema of the agent report |

Every tool answers with structured content, so a client can read the verdict
without parsing text.

## Behaviour

- The server is stateless: every call carries the widget source.
- Nothing is read from the file system and nothing is written.
- The thresholds of the command line interface are the defaults of the tools:
  `fail_on` is `error`, `min_severity` is `info`, `min_confidence` is
  `heuristic` and the budget is 3800 bytes.
- The report is the same document the CLI prints with `--format agent`, so
  both front ends agree on fingerprints, counts and the next action.

## Example

```json
{
  "method": "tools/call",
  "params": {
    "name": "check",
    "arguments": {
      "source": "<div class=\"row\">tap</div><script>fetch('/x')</script>",
      "fail_on": "warn"
    }
  }
}
```

```json
{
  "verdict": "fail",
  "reason": "1 error(s)",
  "findings": [
    {
      "id": "TGW400",
      "sev": "error",
      "msg": "network API `fetch` is disabled in widgets",
      "where": { "line": 1, "col": 40, "region": "js" }
    }
  ],
  "next_action": "fix the error findings above, then re-run widgetlint check"
}
```

## Security

The server runs the same analysis as the CLI and holds no secret. It is meant
to be started by an MCP client as a child process; the agent that talks to it
can only ask questions about the widget it passes in.
