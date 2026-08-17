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

    // #1734 (FA-05-004): `expected_opcode_byte` is an exhaustive match over
    // every `Opcode` variant (no wildcard `_`), so adding a future variant
    // without giving it an arm here is a compile-time error - the same
    // compiler-enforced completeness guarantee `minimum_semcode_revision`
    // already relies on. `ALL_OPCODES` below is a second, independently
    // hand-maintained list (kept in the same declaration order as the
    // `Opcode` enum in `local_format.rs`, for easy visual diffing) driving
    // the loop that actually exercises `expected_opcode_byte` per variant.
    //
    // IMPORTANT, and a known residual gap (adversarial review, #1734): only
    // `expected_opcode_byte`'s exhaustiveness is compiler-enforced.
    // `ALL_OPCODES`'s length is a second, independently hand-typed literal
    // with no structural link to the real enum's variant count - the
    // length/uniqueness checks below catch a *duplicated* entry masking a
    // missing one (verified: swapping the array's last entry for a repeat
    // of an earlier one fails the dedup check), but if a brand-new `Opcode`
    // variant is added, the compiler forces a new arm into
    // `expected_opcode_byte` yet does NOT force a corresponding new entry
    // into `ALL_OPCODES` - the array and this test would keep compiling and
    // passing green while silently covering only the old variant count.
    // Closing that fully needs enum-variant reflection (e.g. `strum`'s
    // `EnumIter`), which is a new dependency and out of scope for this
    // test-only issue; anyone adding an `Opcode` variant MUST update
    // `ALL_OPCODES` and its length below by hand, in the same change that
    // updates `expected_opcode_byte`. `sm-vm`'s pre-existing
    // `OPCODE_PROFILE_OPCODES: [Opcode; 73]` has this same characteristic.
    fn expected_opcode_byte(op: semcode_format::Opcode) -> u8 {
        use semcode_format::Opcode;
        match op {
            Opcode::LoadQ => 0x01,
            Opcode::LoadBool => 0x02,
            Opcode::LoadI32 => 0x03,
            Opcode::AddI32 => 0x07,
            Opcode::SubI32 => 0x08,
            Opcode::MulI32 => 0x09,
            Opcode::DivI32 => 0x0a,
            Opcode::ModI32 => 0x0b,
            Opcode::ConcatText => 0x0c,
            Opcode::LoadU32 => 0x06,
            Opcode::LoadVar => 0x04,
            Opcode::StoreVar => 0x05,
            Opcode::QAnd => 0x10,
            Opcode::QOr => 0x11,
            Opcode::QNot => 0x12,
            Opcode::QImpl => 0x13,
            Opcode::BoolAnd => 0x14,
            Opcode::BoolOr => 0x15,
            Opcode::BoolNot => 0x16,
            Opcode::QTruthAnd => 0x17,
            Opcode::QTruthOr => 0x18,
            Opcode::QTruthNot => 0x19,
            Opcode::QTruthImpl => 0x1a,
            Opcode::CmpEq => 0x20,
            Opcode::CmpNe => 0x21,
            Opcode::CmpI32Lt => 0x22,
            Opcode::CmpI32Le => 0x23,
            Opcode::Jmp => 0x30,
            Opcode::JmpIf => 0x31,
            Opcode::Call => 0x40,
            Opcode::Ret => 0x41,
            Opcode::Assert => 0x42,
            Opcode::MakeTuple => 0x43,
            Opcode::TupleGet => 0x44,
            Opcode::MakeRecord => 0x45,
            Opcode::RecordGet => 0x46,
            Opcode::MakeAdt => 0x47,
            Opcode::AdtTag => 0x48,
            Opcode::AdtGet => 0x49,
            Opcode::LoadF64 => 0x50,
            Opcode::AddF64 => 0x51,
            Opcode::SubF64 => 0x52,
            Opcode::MulF64 => 0x53,
            Opcode::DivF64 => 0x54,
            Opcode::LoadFx => 0x55,
            Opcode::AddFx => 0x56,
            Opcode::SubFx => 0x57,
            Opcode::MulFx => 0x58,
            Opcode::DivFx => 0x59,
            Opcode::LoadText => 0x5a,
            Opcode::MakeSequence => 0x5b,
            Opcode::SequenceGet => 0x5c,
            Opcode::MakeClosure => 0x5d,
            Opcode::ClosureCall => 0x5e,
            Opcode::SequenceLen => 0x5f,
            Opcode::SequenceIsEmpty => 0x67,
            Opcode::SequenceContains => 0x68,
            Opcode::SequencePush => 0x69,
            Opcode::SequencePrepend => 0x6a,
            Opcode::SequencePop => 0x6b,
            Opcode::MapEmpty => 0x70,
            Opcode::MapContains => 0x71,
            Opcode::MapGet => 0x72,
            Opcode::MapSet => 0x73,
            Opcode::RngSeed => 0x74,
            Opcode::RngNextI32 => 0x75,
            Opcode::GateRead => 0x60,
            Opcode::GateWrite => 0x61,
            Opcode::PulseEmit => 0x62,
            Opcode::StateQuery => 0x63,
            Opcode::StateUpdate => 0x64,
            Opcode::EventPost => 0x65,
            Opcode::ClockRead => 0x66,
        }
    }

    #[test]
    fn test_opcode_byte_values_frozen() {
        use semcode_format::Opcode;

        const ALL_OPCODES: [Opcode; 73] = [
            Opcode::LoadQ,
            Opcode::LoadBool,
            Opcode::LoadI32,
            Opcode::AddI32,
            Opcode::SubI32,
            Opcode::MulI32,
            Opcode::DivI32,
            Opcode::ModI32,
            Opcode::ConcatText,
            Opcode::LoadU32,
            Opcode::LoadVar,
            Opcode::StoreVar,
            Opcode::QAnd,
            Opcode::QOr,
            Opcode::QNot,
            Opcode::QImpl,
            Opcode::BoolAnd,
            Opcode::BoolOr,
            Opcode::BoolNot,
            Opcode::QTruthAnd,
            Opcode::QTruthOr,
            Opcode::QTruthNot,
            Opcode::QTruthImpl,
            Opcode::CmpEq,
            Opcode::CmpNe,
            Opcode::CmpI32Lt,
            Opcode::CmpI32Le,
            Opcode::Jmp,
            Opcode::JmpIf,
            Opcode::Call,
            Opcode::Ret,
            Opcode::Assert,
            Opcode::MakeTuple,
            Opcode::TupleGet,
            Opcode::MakeRecord,
            Opcode::RecordGet,
            Opcode::MakeAdt,
            Opcode::AdtTag,
            Opcode::AdtGet,
            Opcode::LoadF64,
            Opcode::AddF64,
            Opcode::SubF64,
            Opcode::MulF64,
            Opcode::DivF64,
            Opcode::LoadFx,
            Opcode::AddFx,
            Opcode::SubFx,
            Opcode::MulFx,
            Opcode::DivFx,
            Opcode::LoadText,
            Opcode::MakeSequence,
            Opcode::SequenceGet,
            Opcode::MakeClosure,
            Opcode::ClosureCall,
            Opcode::SequenceLen,
            Opcode::SequenceIsEmpty,
            Opcode::SequenceContains,
            Opcode::SequencePush,
            Opcode::SequencePrepend,
            Opcode::SequencePop,
            Opcode::MapEmpty,
            Opcode::MapContains,
            Opcode::MapGet,
            Opcode::MapSet,
            Opcode::RngSeed,
            Opcode::RngNextI32,
            Opcode::GateRead,
            Opcode::GateWrite,
            Opcode::PulseEmit,
            Opcode::StateQuery,
            Opcode::StateUpdate,
            Opcode::EventPost,
            Opcode::ClockRead,
        ];

        // Independent cross-check that ALL_OPCODES itself wasn't
        // transcribed with a missing or duplicated variant: 73 entries,
        // all distinct. This 73 is a hand-typed literal, not derived from
        // the real Opcode enum - if you're here because this assert just
        // failed after adding a new Opcode variant, update ALL_OPCODES
        // (and expected_opcode_byte above) to include it, then update this
        // literal to match.
        assert_eq!(
            ALL_OPCODES.len(),
            73,
            "ALL_OPCODES must list every Opcode variant exactly once - if you added a new \
             Opcode variant, add it to ALL_OPCODES and expected_opcode_byte, then update \
             this expected count"
        );
        let mut seen = std::collections::HashSet::new();
        for op in ALL_OPCODES {
            assert!(
                seen.insert(op),
                "{op:?} appears more than once in ALL_OPCODES"
            );
        }

        // Bidirectional freeze proof for every variant: the enum
        // discriminant must still match its independently hand-pinned
        // expected byte, and decoding that byte must still round-trip to
        // the same variant.
        for op in ALL_OPCODES {
            let expected = expected_opcode_byte(op);
            assert_eq!(
                op as u8, expected,
                "{op:?}'s byte value drifted from its frozen pin 0x{expected:02x}"
            );
            assert_eq!(
                Opcode::from_byte(expected),
                Ok(op),
                "Opcode::from_byte(0x{expected:02x}) no longer round-trips to {op:?}"
            );
        }
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
