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

    // #1732 (FA-05-002) review follow-up: minimum_semcode_revision() is now
    // an exhaustive match over Opcode (no wildcard `_`), so adding a future
    // opcode variant without updating it is a compile-time error and no
    // variant can acquire a minimum revision implicitly. This test is a
    // readable sample across every group in that match, not a substitute for
    // the compiler's own exhaustiveness guarantee.
    #[test]
    fn minimum_semcode_revision_matches_expected_values() {
        use semcode_format::Opcode;

        // QTruth: the only family currently assigned a non-baseline minimum revision
        for op in [
            Opcode::QTruthAnd,
            Opcode::QTruthOr,
            Opcode::QTruthNot,
            Opcode::QTruthImpl,
        ] {
            assert_eq!(op.minimum_semcode_revision(), 19, "{op:?} must be rev 19");
        }

        // Baseline representatives, one per group
        for op in [
            Opcode::LoadI32,      // loads/constants
            Opcode::AddI32,       // arithmetic
            Opcode::QAnd,         // legacy lattice logic, next to QTruth
            Opcode::BoolNot,      // legacy lattice logic
            Opcode::CmpEq,        // comparisons
            Opcode::Jmp,          // control flow
            Opcode::Call,         // calls
            Opcode::MakeTuple,    // tuple/record/ADT
            Opcode::AdtGet,       // tuple/record/ADT
            Opcode::LoadText,     // text (capability-gated)
            Opcode::LoadF64,      // f64/fx (capability-gated)
            Opcode::MakeSequence, // sequence (capability-gated)
            Opcode::MakeClosure,  // closures (capability-gated)
            Opcode::MapGet,       // map (capability-gated)
            Opcode::RngSeed,      // PRNG (capability-gated)
            Opcode::GateRead,     // gate host-effect (capability-gated)
            Opcode::ClockRead,    // host-boundary (capability-gated)
        ] {
            assert_eq!(op.minimum_semcode_revision(), 1, "{op:?} must be rev 1");
        }
    }
}
