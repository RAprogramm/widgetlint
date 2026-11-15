# widgetlint

Validator, linter and optimizer for Telegram Bot API 10.4 widgets.

Widgets are HTML/CSS/JS blocks rendered inside a message bubble in a QuickJS
sandbox with no network. The server accepts at most 32768 bytes of HTML and
4096 bytes after zlib level 9 compression; the client silently drops anything
outside the widget profile. `widgetlint` catches all of it before sending.

## Status

Milestone M0: exact compressed-size counter (`size`) with byte-for-byte parity
with the Python `zlib` reference, and the profile-1 data loader. The static
linter (`check`), the optimizer (`pack`) and the simulator (`sim`) follow per
the roadmap in `widget-linter-spec.md`.

## Usage

```console
$ widgetlint size widgets/order.html
widgets/order.html: 31204 B -> 4231 B (zlib-9)    limit 4096    over by +135

$ widgetlint size widgets/order.html --format json
{"schema_version":1,"size":{"raw":31204,"zlib9":4231,"limit":4096,"budget":3800,"over_by":135},"verdict":"fail"}
```

Exit codes: `0` no findings, `1` findings at or above the threshold,
`2` usage or input error, `3` internal error.

## Development

```console
cargo build --workspace
cargo nextest run --workspace
cargo +nightly fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery
uv run --python 3.12 tools/zlib_parity.py   # size parity against real zlib
```

Design documents: `widget-linter-spec.md`, `widget-linter-engineering.md`,
`widget-linter-ai-native.md`.

## License

MIT — see [LICENSE](LICENSE).
