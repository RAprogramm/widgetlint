// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Byte spans and the line index that resolves them to 1-based positions.

use serde::Serialize;

/// A half-open byte range `[start, end)` in the widget source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Span {
    /// Byte offset of the first byte of the span.
    pub start: u32,
    /// Byte offset just past the last byte of the span.
    pub end:   u32
}

impl Span {
    /// Creates a span from byte offsets.
    #[must_use]
    pub const fn new(start: u32, end: u32) -> Self {
        Self {
            start,
            end
        }
    }

    /// Span length in bytes.
    #[must_use]
    pub const fn len(self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    /// Whether the span covers no bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use wl_core::Span;
    ///
    /// assert!(Span::new(4, 4).is_empty());
    /// assert!(!Span::new(4, 5).is_empty());
    /// ```
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.end <= self.start
    }
}

/// A resolved 1-based line/column position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct LineCol {
    /// 1-based line number.
    pub line: u32,
    /// 1-based column number, counted in Unicode code points.
    pub col:  u32
}

/// Index of line starts for one source document.
///
/// Built once in O(n); every lookup is O(log n). Columns count Unicode code
/// points, not bytes, so diagnostics line up with what an editor shows.
#[derive(Debug, Clone)]
pub struct LineIndex {
    /// Byte offset of each line start; the first entry is always `0`.
    line_starts: Vec<u32>
}

impl LineIndex {
    /// Builds the index over `source`, scanning for `\n`, `\r\n` and `\r`.
    #[must_use]
    pub fn new(source: &str) -> Self {
        let bytes = source.as_bytes();
        let mut line_starts = vec![0_u32];
        let mut index = 0_usize;
        while let Some(&byte) = bytes.get(index) {
            match byte {
                b'\n' => index += 1,
                b'\r' => {
                    index += if bytes.get(index + 1) == Some(&b'\n') {
                        2
                    } else {
                        1
                    };
                }
                _ => {
                    index += 1;
                    continue;
                }
            }
            if let Ok(offset) = u32::try_from(index) {
                line_starts.push(offset);
            } else {
                break;
            }
        }
        Self {
            line_starts
        }
    }

    /// Resolves a byte offset to a 1-based line/column pair.
    ///
    /// Offsets past the end resolve to the last line.
    #[must_use]
    pub fn position(&self, source: &str, offset: u32) -> LineCol {
        let line_index = self.line_starts.partition_point(|&start| start <= offset);
        let line = line_index.max(1);
        let line_start = self.line_starts.get(line - 1).copied().unwrap_or(0) as usize;
        let prefix = &source[line_start..source.len().min(offset as usize)];
        LineCol {
            line: u32::try_from(line).unwrap_or(u32::MAX),
            col:  u32::try_from(prefix.chars().count().saturating_add(1)).unwrap_or(u32::MAX)
        }
    }

    /// Number of lines in the document.
    #[must_use]
    pub const fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Byte offset of every line start; the first entry is always `0`.
    #[must_use]
    pub fn line_starts(&self) -> &[u32] {
        &self.line_starts
    }
}
