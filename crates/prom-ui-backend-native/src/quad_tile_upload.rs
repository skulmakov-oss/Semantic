//! Deterministic GPU transport representation for the semantic quad tile.
//!
//! `semantic_core_quad::QuadTile128` remains the canonical semantic storage.
//! `GpuQuadTile128` is a transport representation only. Word order is the
//! least-significant 32-bit word first. This module creates no WGPU object and
//! does not authorize raw byte casting. Pod/Zeroable derivation and byte-slice
//! upload helpers remain deferred to the final transport qualification slice.

use semantic_core_quad::QuadTile128;

/// Portable word representation of a 128-lane quad tile's two planes.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct GpuQuadTile128 {
    /// Truth-plane words, least-significant word first.
    pub t: [u32; 4],
    /// Falsity-plane words, least-significant word first.
    pub f: [u32; 4],
}

const _: () = {
    assert!(core::mem::size_of::<GpuQuadTile128>() == 32);
    assert!(core::mem::align_of::<GpuQuadTile128>() == 16);
    assert!(core::mem::offset_of!(GpuQuadTile128, t) == 0);
    assert!(core::mem::offset_of!(GpuQuadTile128, f) == 16);
};

/// Splits a `u128` into least-significant 32-bit words first.
pub const fn split_u128(value: u128) -> [u32; 4] {
    [
        value as u32,
        (value >> 32) as u32,
        (value >> 64) as u32,
        (value >> 96) as u32,
    ]
}

/// Joins least-significant-first 32-bit words into a `u128`.
pub const fn join_u128(words: [u32; 4]) -> u128 {
    (words[0] as u128)
        | ((words[1] as u128) << 32)
        | ((words[2] as u128) << 64)
        | ((words[3] as u128) << 96)
}

impl From<QuadTile128> for GpuQuadTile128 {
    fn from(tile: QuadTile128) -> Self {
        Self {
            t: split_u128(tile.true_plane()),
            f: split_u128(tile.false_plane()),
        }
    }
}

impl From<GpuQuadTile128> for QuadTile128 {
    fn from(tile: GpuQuadTile128) -> Self {
        QuadTile128::from_planes(join_u128(tile.t), join_u128(tile.f))
    }
}
