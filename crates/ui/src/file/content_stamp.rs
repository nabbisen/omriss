//! A fingerprint of a file's bytes, for noticing that another program changed it
//! (RFC-068 §7.6).
//!
//! omriss keeps the stamp of the bytes it last read from the file or wrote to it. Before a
//! save it reads the file again and compares: a different stamp means someone else
//! changed it, whatever its timestamp says (a `git checkout`, `rsync --times` or
//! `touch -d` can leave the old time on new bytes, and two writes in one second share
//! one). Timestamps are not consulted.
//!
//! The stamp is the length and a 64-bit hash of the bytes, from the standard library's
//! hasher. This is change detection, not security: nobody is crafting a second file
//! that collides, and an accident has odds of 2^-64 per change. Nothing but equality
//! of two stamps taken in the same run is ever relied on, so the hasher's algorithm
//! not being specified across Rust versions does not matter.

use std::hash::Hasher;

/// The length and hash of a run of bytes. Equal bytes have equal stamps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentStamp {
    len: u64,
    hash: u64,
}

impl ContentStamp {
    /// The stamp of `bytes`.
    pub fn of(bytes: &[u8]) -> Self {
        // `DefaultHasher::new()` is keyed identically in every instance in a process.
        let mut hasher = std::hash::DefaultHasher::new();
        hasher.write(bytes);
        Self {
            len: bytes.len() as u64,
            hash: hasher.finish(),
        }
    }
}
