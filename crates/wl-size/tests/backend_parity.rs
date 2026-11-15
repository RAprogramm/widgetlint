// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Backend parity gates. The `zlib-sys` backend must reproduce the reference
//! zlib byte-for-byte (values below come from `CPython`
//! `zlib.compress(data, 9)`). The pure-Rust `rust-backend` used by wasm is
//! zlib-rs: compatible in format, within a byte or two in length, so it is
//! gated by a tolerance window and its sizes are reported as approximate.

use masterror::AppResult;
use wl_size::SizeInfo;

fn corpus() -> String {
    let part = "<html><body><div class=\"card\"><p>Hello, widget! The quick brown fox jumps over \
                the lazy dog. ";
    format!("{}{}", part.repeat(8), "</p></div></body></html>")
}

#[cfg(feature = "zlib-sys")]
#[test]
fn golden_compressed_length() -> AppResult<()> {
    let source = corpus();
    let info = SizeInfo::measure(source.as_bytes())?;
    assert_eq!(info.raw, 768);
    assert_eq!(info.zlib9, 119);
    let empty = SizeInfo::measure(b"")?;
    assert_eq!(empty.zlib9, 8);
    Ok(())
}

#[cfg(feature = "rust-backend")]
#[test]
fn rust_backend_within_tolerance() -> AppResult<()> {
    let source = corpus();
    let info = SizeInfo::measure(source.as_bytes())?;
    assert_eq!(info.raw, 768);
    assert!(
        (117..=121).contains(&info.zlib9),
        "zlib-rs drifted away from the reference: {}",
        info.zlib9
    );
    let empty = SizeInfo::measure(b"")?;
    assert!(empty.zlib9 <= 16);
    Ok(())
}
