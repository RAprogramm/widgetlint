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
    reason = "benches must fail loudly when the embedded profile or a script is broken"
)]

//! Benchmarks for the JavaScript rules on the oxc parser.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

/// Repeats whole statements only, so the script always parses.
fn script(filler: &str, size: usize) -> String {
    let mut out = String::with_capacity(size);
    while out.len() + filler.len() <= size {
        out.push_str(filler);
    }
    out
}

fn bench_check(c: &mut Criterion) {
    let profile = wl_profile::profile_1().expect("the embedded profile must parse");
    let mut group = c.benchmark_group("js_check");
    for (name, filler, size) in [
        ("state", "state.count = state.count + 1;\n", 1024_usize),
        (
            "functions",
            "function tap(e) { const v = e.target.value; if (v > 1) { state.x = v * 2; } }\n",
            8192
        ),
        (
            "violations",
            "fetch('http://x.example'); eval('1'); document.cookie = 'a=1';\n",
            16384
        )
    ] {
        let js = script(filler, size);
        wl_js::check(&js, 0, &profile).expect("the bench script must parse");
        group.bench_function(name, |b| {
            b.iter(|| wl_js::check(black_box(&js), black_box(0), black_box(&profile)));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_check);
criterion_main!(benches);
