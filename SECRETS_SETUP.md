<!-- SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com> -->
<!-- SPDX-License-Identifier: MIT -->

# GitHub Secrets Setup

This document lists every secret the CI pipeline of `widgetlint` reads, why it
needs it, and what happens when it is missing.

Repository settings live at
<https://github.com/RAprogramm/widgetlint/settings/secrets/actions>.

## Required secrets

None of the secrets below is mandatory for the pipeline to stay green: every
secret-gated step degrades gracefully when the secret is absent. Adding them
turns the corresponding service on.

### 1. `CODECOV_TOKEN`

Uploads the coverage report and the test results to
[Codecov](https://codecov.io). The repository is private, so the upload needs
a token: tokenless uploads are only supported for public repositories.

How to get it:

1. Sign in at <https://codecov.io>.
2. Add the repository `RAprogramm/widgetlint`.
3. Copy the upload token from the repository settings.
4. Add it as the repository secret `CODECOV_TOKEN`.

Without the secret the coverage gate still runs locally in CI
(`cargo llvm-cov nextest --fail-under-lines 95`), but the report is not
uploaded and no Codecov badge data is produced.

### 2. `CODSPEED_TOKEN`

Uploads the benchmark results to [CodSpeed](https://codspeed.io). The
repository is private, so the token is required for the upload.

How to get it:

1. Sign in at <https://codspeed.io>.
2. Add the repository `RAprogramm/widgetlint`.
3. Copy the API token from the repository settings.
4. Add it as the repository secret `CODSPEED_TOKEN`.

Without the secret the benchmark job reports a failure but does not fail the
workflow: benchmarks never gate a merge.

### 3. `CARGO_REGISTRY_TOKEN`

Publishes the crates to [crates.io](https://crates.io). It is **not used
yet** — the crates are not published, and the release pipeline only builds
binaries and creates a GitHub Release. The name is the crates.io convention;
a fine-grained PAT with the `publish-new-crates` scope for the `widgetlint`
namespace is enough.

Add it only when the crates are published to crates.io.

## Built-in credentials

`GITHUB_TOKEN` is provided by GitHub automatically and needs no setup. The
pipeline grants it exactly the permissions each job needs through explicit
`permissions:` blocks, so the repository setting
*Settings → Actions → General → Workflow permissions* may stay on the
default (read-only) value.

The jobs that need more than read access declare it themselves:

| Job | Permissions |
|---|---|
| `version-bump` | `contents: write` — pushes the version commit and the release tag |
| `create-release` | `contents: write` — creates the GitHub Release and uploads assets |
| `pages-deploy` | `pages: write`, `id-token: write` — publishes the playground |
| `build-release` | `id-token: write`, `attestations: write` — signs the build provenance |

## Testing the setup

1. **Coverage upload:** push any commit to `main` and open the `coverage` job
   in the run — the Codecov steps appear only when the token is set.
2. **Benchmarks:** push any commit to `main` and check the `Benchmarks
   (CodSpeed)` job.
3. **Release:** push a conventional commit (`feat:`, `fix:`) to `main`. The
   pipeline bumps the version, tags it, builds the binaries for Linux, macOS
   and Windows, signs their provenance and publishes the GitHub Release.

## Security notes

- Never commit a token; secrets live only in the repository settings.
- Rotate the tokens periodically and revoke them when they are no longer
  needed.
- Use tokens scoped to a single repository and to the single service they
  belong to; never reuse one token across services.
- The release pipeline authenticates with the built-in `GITHUB_TOKEN` only —
  no long-lived credential is required to publish a release.

## Troubleshooting

### Coverage upload fails

- Check that `CODECOV_TOKEN` exists and has not expired.
- Check that the repository is added to the Codecov organisation.
- Check that the token belongs to the repository, not to another one.

### Benchmark upload fails

- Check that `CODSPEED_TOKEN` exists and that the repository is registered.
- The job is marked `continue-on-error`, so a failure never blocks a merge.

### Release creation fails

- Check that the tag follows semantic versioning (`vX.Y.Z`).
- Check that no release with the same tag exists.
- Check that the `version-bump` job pushed the tag: `git ls-remote --tags`.
