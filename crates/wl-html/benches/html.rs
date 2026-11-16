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
    reason = "benches must fail loudly when the embedded profile is broken"
)]

//! Benchmarks for HTML parsing with source spans and the HTML rules.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

const INPUTS: [(&str, &str, usize); 3] = [
    (
        "clean",
        "<div class=\"row\"><span>hello</span></div>\n",
        1024
    ),
    (
        "attributes",
        "<button id=\"b\" class=\"btn\" data-x=\"1\" aria-label=\"tap\">tap</button>\n",
        8192
    ),
    (
        "violations",
        "<input type=\"text\"><a href=\"http://x.example\">x</a><iframe></iframe>\n",
        16384
    )
];

fn widget(filler: &str, size: usize) -> String {
    let mut out = String::with_capacity(size);
    while out.len() < size {
        out.push_str(filler);
    }
    out.truncate(size);
    out
}

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("html_parse");
    for (name, filler, size) in INPUTS {
        let source = widget(filler, size);
        group.bench_function(name, |b| {
            b.iter(|| wl_html::parse(black_box(&source)));
        });
    }
    group.finish();
}

fn bench_check(c: &mut Criterion) {
    let profile = wl_profile::profile_1().expect("the embedded profile must parse");
    let mut group = c.benchmark_group("html_check");
    for (name, filler, size) in INPUTS {
        let source = widget(filler, size);
        group.bench_function(name, |b| {
            b.iter(|| wl_html::check(black_box(&source), black_box(&profile)));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_parse, bench_check);
criterion_main!(benches);
