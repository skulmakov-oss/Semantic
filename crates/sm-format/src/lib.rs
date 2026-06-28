#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
mod local_format;

#[cfg(feature = "std")]
pub mod semcode_decode;

#[cfg(feature = "std")]
pub mod semcode_format {
    pub use crate::local_format::*;
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn facade_exports_resolve() {
        // Just verify that the constants and decoding types are reachable.
        let _magic = semcode_format::MAGIC0;
        let _cap = semcode_format::CAP_DEBUG_SYMBOLS;
        // Verify DecodeError is reachable
        let _ = std::mem::size_of::<semcode_decode::DecodeError>();
    }
}
