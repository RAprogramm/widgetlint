// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

#![allow(
    missing_docs,
    clippy::significant_drop_tightening,
    reason = "the CodSpeed criterion_group! and criterion_main! macros expand to undocumented \
              functions that hold the Criterion value until the end of the group"
)]
#![allow(
    clippy::expect_used,
    clippy::literal_string_with_formatting_args,
    reason = "benches must fail loudly and embed CSS literals whose braces are not format arguments"
)]

//! Benchmarks for the span-precise CSS scanner and rules.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

fn stylesheet(filler: &str, size: usize) -> String {
    let mut out = String::with_capacity(size);
    while out.len() < size {
        out.push_str(filler);
    }
    out.truncate(size);
    out
}

fn bench_check(c: &mut Criterion) {
    let profile = wl_profile::profile_1().expect("the embedded profile must parse");
    let mut group = c.benchmark_group("css_check");
    for (name, filler, size) in [
        ("simple", ".a{color:red;margin:0}\n", 1024_usize),
        (
            "nested",
            "@media (max-width:600px){.row>.cell:hover{padding:4px 8px;display:flex}}\n",
            8192
        ),
        (
            "violations",
            ".x{position:fixed;background:url(http://x.example/a.png);unknown-prop:1}\n",
            16384
        )
    ] {
        let css = stylesheet(filler, size);
        group.bench_function(name, |b| {
            b.iter(|| wl_css::check(black_box(&css), black_box(0), black_box(&profile)));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_check);
criterion_main!(benches);
