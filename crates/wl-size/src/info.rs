// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Raw and compressed sizes of one widget document.

use masterror::AppResult;

use crate::counter::ZlibCounter;

/// Sizes of a widget document: raw bytes and zlib level 9 bytes.
///
/// # Examples
///
/// ```
/// let info = wl_size::SizeInfo::measure(b"hello world").expect("zlib works");
/// assert_eq!((info.raw, info.zlib9), (11, 19));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SizeInfo {
    /// Raw document size in bytes.
    pub raw:   usize,
    /// Size after zlib level 9 compression in bytes.
    pub zlib9: usize
}

impl SizeInfo {
    /// Measures `data`: raw length and zlib-9 compressed length.
    ///
    /// * `data` — widget HTML bytes.
    ///
    /// # Errors
    ///
    /// Propagates internal zlib errors from [`ZlibCounter::compressed_len`].
    pub fn measure(data: &[u8]) -> AppResult<Self> {
        let mut counter = ZlibCounter::new();
        let zlib9 = counter.compressed_len(data)?;
        Ok(Self {
            raw: data.len(),
            zlib9
        })
    }
}
