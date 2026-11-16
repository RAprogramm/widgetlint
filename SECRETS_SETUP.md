<!-- SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com> -->
<!-- SPDX-License-Identifier: MIT -->

# Repository Secrets

**`widgetlint` needs no repository secrets.** The repository is public, so
every service authenticates through OpenID Connect and the built-in
`GITHUB_TOKEN`:

| Service | Authentication | Secret needed |
|---|---|---|
| Codecov (coverage and test results) | OIDC (`use_oidc: true`) | none |
| CodSpeed (benchmarks) | OIDC | none |
| GitHub Pages (the playground and the rule catalog) | `GITHUB_TOKEN` + `id-token` | none |
| Releases, tags, version bumps | `GITHUB_TOKEN` with `contents: write` | none |
| Provenance attestations | `GITHUB_TOKEN` with `id-token` and `attestations` | none |

The workflow grants each job exactly the permissions it needs through
`permissions:` blocks, so the repository setting
*Settings → Actions → General → Workflow permissions* may stay at its default
value.

## Optional secrets

### `CARGO_REGISTRY_TOKEN`

Only needed when the crates are published to [crates.io](https://crates.io).
The release pipeline of the current version builds the binaries and creates
the GitHub Release; it does not publish to a registry.

When publishing starts, add a fine-grained personal access token with the
`publish-new-crates` scope for the `widgetlint` namespace as
`CARGO_REGISTRY_TOKEN`.

### `CODECOV_TOKEN`

Not needed. Codecov accepts OIDC uploads for public repositories. If the
organization ever disables tokenless uploads, add the repository upload token
as `CODECOV_TOKEN` and set `use_oidc: false` in the coverage job.

### `CODSPEED_TOKEN`

Not needed. The benchmarks authenticate with OIDC.

## How the authentication works

The coverage job asks for an `id-token` and the Codecov actions exchange it
for a short-lived upload token:

```yaml
permissions:
  contents: read
  id-token: write
```

The token is issued for the current run and expires with it, so there is no
long-lived credential in the repository and nothing to rotate.

## Verifying the setup

Push any commit to `main` and open the run:

- the **Coverage** job must show two successful uploads to Codecov;
- the **Benchmarks (CodSpeed)** job must publish the benchmark results;
- the **Playground deploy** job must publish the site;
- the **Create GitHub Release** job must publish the release with the binaries
  for Linux, macOS and Windows.
