<!-- SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com> -->
<!-- SPDX-License-Identifier: MIT -->

# GitHub Action

The repository ships a composite action that validates Telegram widgets
without installing anything.

```yaml
- uses: RAprogramm/widgetlint@v1
  with:
    path: widgets/
    fail-on: warn
```

## Inputs

| Input | Default | Meaning |
|---|---|---|
| `path` | `.` | widget file or directory to check |
| `fail-on` | `error` | severity that fails the step: `info`, `warn` or `error` |
| `min-severity` | — | drop findings below this severity |
| `min-confidence` | — | drop rules below this confidence: `heuristic` or `certain` |
| `config` | — | path to a `widgetlint.toml` configuration file |
| `version` | `latest` | release version to install |
| `repository` | `RAprogramm/widgetlint` | repository that publishes the releases |

The step fails when a finding at or above `fail-on` remains. The output is
the usual human report, so the log shows the rule id, the position and the
fix hint.

## Example workflow

```yaml
name: widgets
on:
  pull_request:
  push:
    branches: [main]

permissions:
  contents: read

jobs:
  widgets:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: RAprogramm/widgetlint@v1
        with:
          path: widgets/
          fail-on: warn
```

## How the binary is installed

The action downloads the release archive that matches the runner, verifies
the SHA-256 checksum published with the archive and puts the binary on the
path. Nothing is compiled, and nothing is installed system-wide.

Supported runners:

| Runner | Release |
|---|---|
| `ubuntu-latest`, `ubuntu-24.04` | `x86_64-unknown-linux-gnu` |
| `ubuntu-24.04-arm` | `aarch64-unknown-linux-gnu` |
| `macos-latest`, `macos-26` | `aarch64-apple-darwin` |
| `macos-15`, `macos-13` | `x86_64-apple-darwin` |
| `windows-latest`, `windows-2025` | `x86_64-pc-windows-msvc` |
| `windows-11-arm` | `aarch64-pc-windows-msvc` |

## Pinning

For a reproducible pipeline, pin the action to a version tag or to a commit
SHA:

```yaml
- uses: RAprogramm/widgetlint@v1.0.0
```

The repository publishes a workflow dependency lockfile for its own CI; the
action itself is a single file without external dependencies.

## Permissions

The action needs no write permission. It reads the release assets with the
token of the workflow, so `permissions: contents: read` is enough.
