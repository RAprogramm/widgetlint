// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Exact widget size engine.
//!
//! The server measures a widget as `zlib.compress(html, 9)`: zlib format,
//! level 9, default parameters. This crate reproduces that number byte-for-byte
//! by compressing with the real zlib C library (`flate2` with the `zlib`
//! backend) and evaluates it against the profile limits.
//!
//! # Examples
//!
//! ```
//! let compressed = wl_size::compressed_size(b"hello world").expect("zlib works");
//! assert_eq!(compressed, 19);
//! ```

pub mod counter;
pub mod info;
pub mod status;

use masterror::AppResult;

pub use crate::{
    counter::{ZLIB_LEVEL, ZlibCounter},
    info::SizeInfo,
    status::{DEFAULT_BUDGET, DEFAULT_MARGIN, Severity, SizeFinding, SizeRule, evaluate}
};

/// Returns the zlib level 9 compressed size of `data` in bytes.
///
/// * `data` — widget HTML bytes.
///
/// # Errors
///
/// [`masterror::AppError`] with `Internal` taxonomy on a zlib failure.
///
/// # Examples
///
/// ```
/// assert_eq!(wl_size::compressed_size(b"").expect("zlib works"), 8);
/// ```
pub fn compressed_size(data: &[u8]) -> AppResult<usize> {
    ZlibCounter::new().compressed_len(data)
}
