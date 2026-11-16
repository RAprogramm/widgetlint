<!-- SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com> -->
<!-- SPDX-License-Identifier: MIT -->

# Changelog

All notable changes to widgetlint will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-11-16

### Bug Fixes

- Generate the nextest junit report through the default profile ([21753e5](https://github.com/RAprogramm/widgetlint/commit/21753e5c38d92b18ff20973816c08a035fdb3b21))

### CI/CD

- Authenticate codecov uploads with the repository token ([361aa48](https://github.com/RAprogramm/widgetlint/commit/361aa48962b7241247e6f8be448d5d0356e45d04))


## [0.1.1] - 2026-11-16

### Bug Fixes

- Preserve the SPDX header and hide bookkeeping commits in generated changelogs ([5c2b8ea](https://github.com/RAprogramm/widgetlint/commit/5c2b8eaa832bb71d23639e2fe4701713a66180e8))
- Align the wl-mcp lockfile version with the workspace ([d48eb3f](https://github.com/RAprogramm/widgetlint/commit/d48eb3f9bb233e7296940e9709fe8a3fb194e3b6))
- Read the version from the manifest in the version command test ([2d3b4a7](https://github.com/RAprogramm/widgetlint/commit/2d3b4a7b17369cbf1121269cc55c769b92caf8f3))

### Documentation

- List the wl-mcp crate in the repository layout ([eb6311e](https://github.com/RAprogramm/widgetlint/commit/eb6311e3dc00b1ff544da4db50814980c70a8222))

### CI/CD

- Install the cross-compilation target in release builds ([b1773f2](https://github.com/RAprogramm/widgetlint/commit/b1773f20c3ed2919581b3ccdd1cd63b86a79bf8b))
- Make the release pipeline idempotent and move the floating tag to the release job ([7af4f45](https://github.com/RAprogramm/widgetlint/commit/7af4f453105bf32ceccebacf75b2b7c4fd3c42fa))


## [0.1.0] - 2026-11-16

### Features

- Milestone M0 — workspace, profile-1 data, exact zlib-9 size engine ([b7e9333](https://github.com/RAprogramm/widgetlint/commit/b7e93337490807b0ea2c38ec60c93c72aa1d4b6b))
- Wl-core kernel — canonical finding model, rule registry, report contract, inline suppressions; wl-size emits canonical findings ([79a6826](https://github.com/RAprogramm/widgetlint/commit/79a6826ab6de528dcc1ff497c1149963f90e96b4))
- Wl-html — html5ever fragment sink with arena DOM, span scanner and document-order alignment ([963c809](https://github.com/RAprogramm/widgetlint/commit/963c8092a2fa19bac5f5543f8f2771bae550bdfa))
- Add wl-css engine with span-precise scanner and TGW200-216, TGW300/301 ([ddc8536](https://github.com/RAprogramm/widgetlint/commit/ddc853637cc637ec773a42239948f466099bcdff))
- Add wl-js engine on oxc parser with TGW400-406, 408, 411 ([d51c103](https://github.com/RAprogramm/widgetlint/commit/d51c1036f9028a414436930850edf05be50cc04a))
- Add wl-html rules with TGW100-114, 150 and document limits TGW302-304 ([f132cf5](https://github.com/RAprogramm/widgetlint/commit/f132cf5d5a9d621de71a0606f10fa73b0aabcab2))
- Add wl-check pipeline merging all regions with config and suppressions ([5f592df](https://github.com/RAprogramm/widgetlint/commit/5f592df08668084b99efd1570df958c5b05c8084))
- Add check, explain and rules commands with config, suppressions and sarif output ([144d819](https://github.com/RAprogramm/widgetlint/commit/144d819c0249612dfe2cf7f6a7d7d70778d80bbb))
- Add xtask doc generator, generated rule pages, llms.txt, AGENTS.md and refreshed README ([214e94a](https://github.com/RAprogramm/widgetlint/commit/214e94a74d10fbe5db14f896f3312c0f28e53bbe))
- Add agent report, verify, schema and version commands with fingerprints, grouping and since-diffing ([0ddd6ae](https://github.com/RAprogramm/widgetlint/commit/0ddd6aee18d33370be555d7e5df9847292d133fe))
- Add wasm crate and browser playground with pages workflow and size backend features ([bcb6012](https://github.com/RAprogramm/widgetlint/commit/bcb60126e9fbd81a82bcbfb1544cb4c5c43bfd9d))
- Serve the linter over the model context protocol with the mcp subcommand ([571051e](https://github.com/RAprogramm/widgetlint/commit/571051edc7a2365d6335559bab39f0d3ae96d131))
- Ship a composite github action that installs the release binary and checks the widgets ([4848709](https://github.com/RAprogramm/widgetlint/commit/4848709c2e351589c303241cc0111378d2c88770))
- Rebuild the playground as a light and dark interface with samples, a size meter and an inline rule reference ([af67b25](https://github.com/RAprogramm/widgetlint/commit/af67b25a11bd818b9c5a5123b919de8b91d692cd))
- Publish the rule catalog as a website, add search keywords to the registry and fix the release pipeline ([036b4d2](https://github.com/RAprogramm/widgetlint/commit/036b4d25f00f19716aeb1d66caccf85fef2a61f2))
- Publish the action with floating version tags, annotations, outputs and marketplace documentation ([775a8fb](https://github.com/RAprogramm/widgetlint/commit/775a8fb7f6e77e4d48053d1bd49c7e6a24820187))

### Bug Fixes

- Count text and comment nodes in TGW303 and check the 48K document text total in TGW304 ([521e8ec](https://github.com/RAprogramm/widgetlint/commit/521e8ec092054edc3470382a72757bdaefefadea))

### Documentation

- Rebuild the front page with hero screenshot, badges and structured guide, add changelog ([4ad91e0](https://github.com/RAprogramm/widgetlint/commit/4ad91e0f1ccfc393a5313240216556f86a04bf48))
- Point badges and playground links at the renamed widgetlint repository ([b3d8781](https://github.com/RAprogramm/widgetlint/commit/b3d8781faab4e3298d96387f0bf1698c110df54a))
- Add the pipeline guide, the secrets setup and the release and codecov badges ([e05aa1f](https://github.com/RAprogramm/widgetlint/commit/e05aa1fb7d0b236812b0863eba168b90513981c6))
- Point the support links at the sponsorship page of the maintainer profile ([144f944](https://github.com/RAprogramm/widgetlint/commit/144f9446235cc7fa2189e2240f77da3f5acbaefe))

### Testing

- Raise coverage to 95 percent and fix sink reparent and entity scan bugs ([6042963](https://github.com/RAprogramm/widgetlint/commit/60429635f0fd6acac9c18f5c4ccb4228d20f0b66))

### CI/CD

- Pass explicit toolchains to rust-toolchain steps and rely on rust-toolchain.toml elsewhere ([45bd3af](https://github.com/RAprogramm/widgetlint/commit/45bd3afdb08ce056ebdec2c562b1b100c8304f7a))
- Stage jobs with dependencies, normalize line endings and cover xtask and cli edges to pass the coverage gate ([692471c](https://github.com/RAprogramm/widgetlint/commit/692471c2908aea724818e3c09d84b96edc387b70))
- Unify the pipeline into one staged workflow with a git-cliff release, codecov, codspeed and a workflow dependency lockfile ([7e55bda](https://github.com/RAprogramm/widgetlint/commit/7e55bdabe47de69ecd242dabe65e9c6a3bb9f623))
- Measure the engines with CodSpeed through the criterion compatibility layer and OIDC ([421d536](https://github.com/RAprogramm/widgetlint/commit/421d5361205a0e76adc400d6438223002a0a7d43))
- Authenticate the coverage uploads with openid connect and point the playground navigation at the rule catalog ([421672a](https://github.com/RAprogramm/widgetlint/commit/421672ae42c7bd27757ec5d007299a865b858cb1))

### Miscellaneous

- Make the repository reuse-compliant and extend CI with docs, schema, coverage and compliance gates ([96dfa83](https://github.com/RAprogramm/widgetlint/commit/96dfa834f40c69e290ee198bcb3fe62313a91635))

### Bench

- Add criterion benchmarks for the size counter and the checking pipeline ([5f5eab0](https://github.com/RAprogramm/widgetlint/commit/5f5eab08ef60687d83636cf9b22abd6ab0f7fe83))


<!-- generated by git-cliff -->
