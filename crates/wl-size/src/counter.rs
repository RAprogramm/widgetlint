// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Reusable zlib level 9 size counter.

use flate2::{Compress, Compression, FlushCompress, Status};
use masterror::{AppError, AppResult};

/// Compression level fixed by the widget specification.
pub const ZLIB_LEVEL: u32 = 9;

/// Upper bound on the zlib output size, mirroring the `deflateBound` formula
/// plus the zlib wrapper (2 header bytes and 4 adler32 bytes).
///
/// # Examples
///
/// ```
/// assert!(wl_size::counter::deflate_bound(0) >= 8);
/// assert!(wl_size::counter::deflate_bound(32768) > 32768);
/// ```
#[must_use]
pub const fn deflate_bound(input_len: usize) -> usize {
    input_len
        .saturating_add(input_len.saturating_add(7) >> 3)
        .saturating_add(input_len.saturating_add(63) >> 6)
        .saturating_add(12)
}

/// Reusable counter producing the exact zlib level 9 compressed size.
///
/// One instance keeps the compressor and the output buffer, so repeated calls
/// pay no allocation cost beyond growth. The counter is pure: no I/O, no
/// clocks, no global state.
///
/// # Examples
///
/// ```
/// let mut counter = wl_size::ZlibCounter::new();
/// assert_eq!(
///     counter.compressed_len(b"hello world").expect("zlib works"),
///     19
/// );
/// assert_eq!(counter.compressed_len(b"").expect("zlib works"), 8);
/// ```
#[derive(Debug)]
pub struct ZlibCounter {
    compress: Compress,
    out:      Vec<u8>
}

impl Default for ZlibCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl ZlibCounter {
    /// Creates a counter configured for the zlib format at level 9.
    #[must_use]
    pub fn new() -> Self {
        Self {
            compress: Compress::new(Compression::new(ZLIB_LEVEL), true),
            out:      Vec::new()
        }
    }

    /// Returns the compressed size of `data` in bytes.
    ///
    /// * `data` — input bytes (widget HTML).
    ///
    /// # Errors
    ///
    /// [`masterror::AppError`] with `Internal` taxonomy if deflate fails or
    /// makes no progress.
    pub fn compressed_len(&mut self, data: &[u8]) -> AppResult<usize> {
        self.compress.reset();
        self.out.clear();
        self.out.resize(deflate_bound(data.len()), 0);
        loop {
            let produced = usize::try_from(self.compress.total_out())
                .map_err(|_| AppError::internal("zlib output counter overflow"))?;
            let consumed = usize::try_from(self.compress.total_in())
                .map_err(|_| AppError::internal("zlib input counter overflow"))?;
            let output_window = self
                .out
                .get_mut(produced..)
                .ok_or_else(|| AppError::internal("zlib output window out of bounds"))?;
            let input_window = data
                .get(consumed..)
                .ok_or_else(|| AppError::internal("zlib input window out of bounds"))?;
            let status = self
                .compress
                .compress(input_window, output_window, FlushCompress::Finish)
                .map_err(|_| AppError::internal("zlib deflate failed"))?;
            if status == Status::StreamEnd {
                return usize::try_from(self.compress.total_out())
                    .map_err(|_| AppError::internal("zlib output counter overflow"));
            }
            if self.compress.total_out() == produced as u64
                && self.compress.total_in() == consumed as u64
            {
                return Err(AppError::internal("zlib made no progress"));
            }
        }
    }
}
