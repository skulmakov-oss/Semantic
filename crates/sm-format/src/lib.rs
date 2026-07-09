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

    #[test]
    fn test_opcode_byte_values_frozen() {
        use semcode_format::Opcode;

        // Ensure existing opcodes haven't shifted
        assert_eq!(Opcode::QAnd as u8, 0x10);
        assert_eq!(Opcode::QOr as u8, 0x11);
        assert_eq!(Opcode::QNot as u8, 0x12);
        assert_eq!(Opcode::QImpl as u8, 0x13);
        assert_eq!(Opcode::BoolAnd as u8, 0x14);
        assert_eq!(Opcode::BoolOr as u8, 0x15);
        assert_eq!(Opcode::BoolNot as u8, 0x16);

        // Ensure explicit QTruth opcodes match reserved slots
        assert_eq!(Opcode::QTruthAnd as u8, 0x17);
        assert_eq!(Opcode::QTruthOr as u8, 0x18);
        assert_eq!(Opcode::QTruthNot as u8, 0x19);
        assert_eq!(Opcode::QTruthImpl as u8, 0x1a);

        // Ensure roundtrip from_byte works
        assert_eq!(Opcode::from_byte(0x10), Ok(Opcode::QAnd));
        assert_eq!(Opcode::from_byte(0x17), Ok(Opcode::QTruthAnd));
        assert_eq!(Opcode::from_byte(0x18), Ok(Opcode::QTruthOr));
        assert_eq!(Opcode::from_byte(0x19), Ok(Opcode::QTruthNot));
        assert_eq!(Opcode::from_byte(0x1a), Ok(Opcode::QTruthImpl));
    }
}
