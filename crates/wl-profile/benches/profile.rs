// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

#![allow(
    missing_docs,
    clippy::significant_drop_tightening,
    reason = "the CodSpeed criterion_group! and criterion_main! macros expand to undocumented \
              functions that hold the Criterion value until the end of the group"
)]

//! Benchmarks for loading and validating the embedded profile.

use criterion::{Criterion, criterion_group, criterion_main};

fn bench_profile(c: &mut Criterion) {
    c.bench_function("profile_1_load", |b| {
        b.iter(wl_profile::profile_1);
    });
}

criterion_group!(benches, bench_profile);
criterion_main!(benches);
