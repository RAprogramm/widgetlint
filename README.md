# widgetlint

Validator and linter for Telegram Bot API 10.4 widgets.

Widgets are HTML/CSS/JS blocks rendered inside a message bubble in a QuickJS
sandbox with no network. The server accepts at most 32768 bytes of HTML and
4096 bytes after zlib level 9 compression; the client silently drops anything
outside the widget profile. `widgetlint` catches all of it before sending.

## Status

Milestone M1: `check` (HTML, CSS and JavaScript rules with exact spans),
`size` (exact compressed-size counter with byte-for-byte parity against the
Python `zlib` reference), `verify`, `explain`, `rules`, `schema` and
`version`. The agent report (`--format agent`) is the machine contract for
LLM fix loops. The optimizer (`pack`) and the simulator (`sim`) follow per the
roadmap in `widget-linter-spec.md`.

## Install

```console
cargo install --path crates/wl-cli
```

## Usage

```console
$ widgetlint check widgets/order.html
widgets/order.html
  warn   TGW100  <input> is removed together with its content    14:3
         help: widgets have no input fields; use buttons and state instead
  error  TGW400  network API `fetch` is disabled in widgets    41:5
         help: widgets have no network access; ask the bot with TelegramWidget.requestSendData
2 errors, 3 warnings, 0 info

$ widgetlint check widgets/order.html --format json
[{"schema_version":1,"input":{"path":"widgets/order.html","bytes":31204}, ... }]

$ widgetlint check widgets/ --format sarif --fail-on warn > widgetlint.sarif

$ widgetlint size widgets/order.html
widgets/order.html: 31204 B -> 4231 B (zlib-9)    limit 4096    budget 3800    fail (+135)

$ widgetlint explain TGW100
TGW100  warn  certain
form controls are removed together with their content
help: widgets have no input fields; use buttons and state instead
spec: https://core.telegram.org/bots/widgets#html
```

`check` accepts files, directories and `-` for stdin; reads `widgetlint.toml`
from the working directory (thresholds, rule on/off and severity overrides,
path filters). Inline suppressions: `<!-- tgw-disable TGW100 -->`,
`<!-- tgw-disable-file TGW606 -->`, `// tgw-disable-next-line TGW400`.

## AI agents

```console
$ widgetlint verify widgets/order.html
{"input":"widgets/order.html","next_action":"fix the error findings above, then re-run widgetlint check","reason":"1 error(s), 5 warning(s)","schema_version":1,"verdict":"fail"}

$ widgetlint check widgets/order.html --format agent --fail-on warn
{"schema_version":1,"verdict":"fail","reason":"1 error(s), 5 warning(s)","budget":{...},"findings":[{"fp":"1dffa306","id":"TGW400","sev":"error","msg":"...","fix":{"applicability":"manual","hint":"..."},"repeat":1,...}],...}
```

`verify` is the minimal verdict between edits; `--format agent` is the full
compact report: grouped repeats, fingerprints (`fp`), caveats, suppression
statistics and one `next_action`. `--since <fp-file>` reports only findings
new since the previous run plus resolved fingerprints; `--max-findings` caps
the report size. The JSON Schema is published by `widgetlint schema`;
`widgetlint version --json` carries the tool and ruleset identity. A ready
`AGENTS.md` snippet for widget projects lives in [docs/agents.md](docs/agents.md).

Exit codes: `0` no findings at or above the threshold, `1` findings present,
`2` usage or input error, `3` internal error. The threshold is `--fail-on`.

## Documentation

- Rule reference: [docs/rules/index.md](docs/rules/index.md) — one page per
  rule, generated from the rule registry.
- AI agents: [llms.txt](llms.txt) and [llms-full.txt](llms-full.txt), plus the
  ready `AGENTS.md` snippet in [docs/agents.md](docs/agents.md).
- Machine output: `check --format json|sarif|agent`, `rules --format json`,
  `schema`, `version --json`.

## Development

```console
cargo build --workspace
cargo nextest run --workspace
cargo test --doc --workspace
cargo +nightly fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery
uv run --python 3.12 tools/zlib_parity.py   # size parity against real zlib
cargo run -p xtask -- gen-docs              # regenerate rule docs and llms.txt
```

Contributors and coding agents: see [AGENTS.md](AGENTS.md).
Design documents: `widget-linter-spec.md`, `widget-linter-engineering.md`,
`widget-linter-ai-native.md`.

## License

MIT — see [LICENSE](LICENSE).
