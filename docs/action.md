<!-- SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com> -->
<!-- SPDX-License-Identifier: MIT -->

# GitHub Action

The repository ships a composite action that validates Telegram widgets
without installing anything.

```yaml
- uses: RAprogramm/widgetlint@v0.1
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

## Outputs

| Output | Meaning |
|---|---|
| `verdict` | `pass` when no finding reached the `fail-on` threshold, `fail` otherwise |
| `findings` | number of findings reported for the checked widgets |

```yaml
- uses: RAprogramm/widgetlint@v0.1
  id: lint
- run: echo "the widget verdict is ${{ steps.lint.outputs.verdict }}"
```

## What the step does

1. Downloads the release archive that matches the runner and **verifies its
   SHA-256 checksum** before unpacking it.
2. Runs `widgetlint check` over the given path.
3. Reports every finding as a **GitHub annotation**, so the problems appear
   inline in the pull request diff:
   `::warning file=widgets/card.html,line=12,col=3::TGW400 network API fetch is disabled in widgets`.
4. Writes a **table of findings** to the job summary.
5. Fails the step when a finding at or above `fail-on` remains.

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
      - uses: RAprogramm/widgetlint@v0.1
        with:
          path: widgets/
          fail-on: warn
```

## Version tags

The release pipeline publishes every version as a full semver tag and keeps a
floating tag next to it:

| Tag | Points at | Use it when |
|---|---|---|
| `v0.1.0` | the exact release | you want a reproducible pipeline |
| `v0.1` | the newest `0.1.x` release | you want the fixes of the branch |
| `v1` | the newest `1.x` release | the project is stable |

The floating tag is a convention this repository maintains itself: the release
job moves it to the new version on every release.

```yaml
- uses: RAprogramm/widgetlint@v0.1     # floating minor tag
- uses: RAprogramm/widgetlint@v0.1.0   # exact version
- uses: RAprogramm/widgetlint@<sha>   # exact commit
```

## Supported runners

| Runner | Release |
|---|---|
| `ubuntu-latest`, `ubuntu-24.04` | `x86_64-unknown-linux-gnu` |
| `ubuntu-24.04-arm` | `aarch64-unknown-linux-gnu` |
| `macos-latest`, `macos-26` | `aarch64-apple-darwin` |
| `macos-15`, `macos-13` | `x86_64-apple-darwin` |
| `windows-latest`, `windows-2025` | `x86_64-pc-windows-msvc` |
| `windows-11-arm` | `aarch64-pc-windows-msvc` |

## Permissions

The action needs no write permission. It reads the release assets with the
token of the workflow, so `permissions: contents: read` is enough.

## Publishing to the Marketplace

The action lives in the root of a public repository, which is what the
Marketplace requires. The first release publishes it:

1. Open the release of the first version.
2. Tick **Publish this Action to the GitHub Marketplace**.
3. Pick the primary category *Continuous Integration* and the secondary one
   *Testing*.

The Marketplace agreement of the account owner has to be accepted once
before the checkbox becomes available.
