// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

#![allow(
    clippy::expect_used,
    clippy::literal_string_with_formatting_args,
    reason = "benches must fail loudly and embed CSS literals whose braces are not format arguments"
)]

//! Benchmarks for the end-to-end checking pipeline.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use wl_check::{Config, check};

fn widget(filler: &str, size: usize) -> String {
    let mut out = String::with_capacity(size);
    while out.len() < size {
        out.push_str(filler);
    }
    out.truncate(size);
    out
}

fn bench_check(c: &mut Criterion) {
    let profile = wl_profile::profile_1().expect("the embedded profile must parse");
    let config = Config::default();
    let mut group = c.benchmark_group("check");
    for (name, filler, size) in [
        ("clean", "<div>hello</div>\n", 1024_usize),
        (
            "styled",
            "<style>.a{color:red}</style>\n<button class=\"a\">tap</button>\n",
            8192
        ),
        (
            "mixed",
            "<span class=\"s\">v</span>\n<script>state.x=1</script>\n",
            24576
        )
    ] {
        let source = widget(filler, size);
        group.bench_function(name, |b| {
            b.iter(|| check(black_box(&source), black_box(&profile), black_box(&config)));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_check);
criterion_main!(benches);
