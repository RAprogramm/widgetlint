// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Deterministic hashing for stable fingerprints.
//!
//! The standard library hasher is seeded per process, so its output changes
//! between runs. Fingerprints that agents compare across runs need a fixed
//! algorithm; FNV-1a is small, allocation-free and stable across platforms.

/// Computes the 64-bit FNV-1a hash of `data`.
///
/// # Examples
///
/// ```
/// assert_eq!(wl_core::hash::fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
/// assert_eq!(wl_core::hash::fnv1a64(b"test"), 0xf9e6_e6ef_197c_2b25);
/// ```
#[must_use]
pub fn fnv1a64(data: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in data {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Short lowercase hexadecimal form of [`fnv1a64`], eight characters wide.
///
/// # Examples
///
/// ```
/// assert_eq!(
///     wl_core::hash::fnv1a32_hex("TGW100\u{0}<input>".as_bytes()),
///     "efbe2255"
/// );
/// ```
#[must_use]
pub fn fnv1a32_hex(data: &[u8]) -> String {
    let hash = fnv1a64(data);
    let folded = u32::try_from(((hash >> 32) ^ hash) & 0xffff_ffff).unwrap_or_default();
    format!("{folded:08x}")
}
