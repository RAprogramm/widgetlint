// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

#![allow(
    clippy::expect_used,
    reason = "benches must fail loudly on zlib errors"
)]

//! Benchmarks for the exact zlib level 9 size counter.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use wl_size::ZlibCounter;

fn widget_html(size: usize) -> Vec<u8> {
    let mut source = Vec::with_capacity(size);
    let filler = b"<div class=\"row\">text</div>\n";
    while source.len() < size {
        source.extend_from_slice(filler);
    }
    source.truncate(size);
    source
}

fn bench_compress(c: &mut Criterion) {
    let mut group = c.benchmark_group("zlib9");
    for size in [1024_usize, 8192, 32768] {
        let data = widget_html(size);
        let mut counter = ZlibCounter::new();
        group.bench_function(format!("len_{size}"), |b| {
            b.iter(|| {
                counter
                    .compressed_len(black_box(&data))
                    .expect("zlib must succeed")
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_compress);
criterion_main!(benches);
