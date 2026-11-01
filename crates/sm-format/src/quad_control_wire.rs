//! Candidate *operand* framing for four-state control operations.
//!
//! This module does not register SemCode opcodes, a header revision, or verifier
//! admission. In particular, rev27 (SEMCOD26) remains EVR0 structural-only and
//! fail-closed. These deterministic wire shapes are independently testable
//! while the next admitted revision is decided by the format owner.

/// Register followed by four absolute instruction-stream offsets, ordered N,F,T,S.
/// Opcode byte is not part of this payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuadBranch4Payload {
    pub condition: u16,
    pub on_n: u32,
    pub on_f: u32,
    pub on_t: u32,
    pub on_s: u32,
}

/// Register-only payload for a four-state assertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuadAssertPayload {
    pub condition: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuadControlWireError {
    WrongLength { expected: usize, got: usize },
}

impl QuadBranch4Payload {
    pub const LEN: usize = 18;

    pub fn encode(self) -> [u8; Self::LEN] {
        let mut out = [0u8; Self::LEN];
        out[0..2].copy_from_slice(&self.condition.to_le_bytes());
        out[2..6].copy_from_slice(&self.on_n.to_le_bytes());
        out[6..10].copy_from_slice(&self.on_f.to_le_bytes());
        out[10..14].copy_from_slice(&self.on_t.to_le_bytes());
        out[14..18].copy_from_slice(&self.on_s.to_le_bytes());
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, QuadControlWireError> {
        if bytes.len() != Self::LEN {
            return Err(QuadControlWireError::WrongLength {
                expected: Self::LEN,
                got: bytes.len(),
            });
        }
        Ok(Self {
            condition: u16::from_le_bytes([bytes[0], bytes[1]]),
            on_n: u32::from_le_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]),
            on_f: u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]),
            on_t: u32::from_le_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]),
            on_s: u32::from_le_bytes([bytes[14], bytes[15], bytes[16], bytes[17]]),
        })
    }

    /// All four addresses require independent verifier checks, even when only
    /// one state is possible at runtime. No unchecked fallthrough is implied.
    pub const fn successors(self) -> [u32; 4] {
        [self.on_n, self.on_f, self.on_t, self.on_s]
    }
}

impl QuadAssertPayload {
    pub const LEN: usize = 2;

    pub const fn encode(self) -> [u8; Self::LEN] {
        self.condition.to_le_bytes()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, QuadControlWireError> {
        if bytes.len() != Self::LEN {
            return Err(QuadControlWireError::WrongLength {
                expected: Self::LEN,
                got: bytes.len(),
            });
        }
        Ok(Self {
            condition: u16::from_le_bytes([bytes[0], bytes[1]]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_quad_branch4_operand_bytes_are_little_endian_and_ordered() {
        let v = QuadBranch4Payload {
            condition: 0x3412,
            on_n: 0x01020304,
            on_f: 0x11121314,
            on_t: 0x21222324,
            on_s: 0x31323334,
        };
        let golden = [
            0x12, 0x34, 0x04, 0x03, 0x02, 0x01, 0x14, 0x13, 0x12, 0x11, 0x24, 0x23, 0x22, 0x21,
            0x34, 0x33, 0x32, 0x31,
        ];
        assert_eq!(v.encode(), golden);
        assert_eq!(QuadBranch4Payload::decode(&golden), Ok(v));
        assert_eq!(v.successors(), [v.on_n, v.on_f, v.on_t, v.on_s]);
    }

    #[test]
    fn branch4_rejects_short_and_trailing_payloads() {
        let v = QuadBranch4Payload {
            condition: u16::MAX,
            on_n: u32::MAX,
            on_f: 0,
            on_t: 1,
            on_s: u32::MAX,
        };
        let golden = v.encode();
        assert_eq!(QuadBranch4Payload::decode(&golden), Ok(v));
        for length in 0..QuadBranch4Payload::LEN {
            assert_eq!(
                QuadBranch4Payload::decode(&golden[..length]),
                Err(QuadControlWireError::WrongLength {
                    expected: QuadBranch4Payload::LEN,
                    got: length,
                })
            );
        }
        let mut extended = golden.to_vec();
        extended.push(0);
        assert_eq!(
            QuadBranch4Payload::decode(&extended),
            Err(QuadControlWireError::WrongLength {
                expected: 18,
                got: 19
            })
        );
    }

    #[test]
    fn quad_assert_operand_is_exactly_one_u16_register() {
        let v = QuadAssertPayload { condition: 0xabcd };
        assert_eq!(v.encode(), [0xcd, 0xab]);
        assert_eq!(QuadAssertPayload::decode(&v.encode()), Ok(v));
        assert_eq!(
            QuadAssertPayload::decode(&[0xcd]),
            Err(QuadControlWireError::WrongLength {
                expected: 2,
                got: 1
            })
        );
        assert_eq!(
            QuadAssertPayload::decode(&[0xcd, 0xab, 0]),
            Err(QuadControlWireError::WrongLength {
                expected: 2,
                got: 3
            })
        );
    }
}

#[cfg(test)]
mod opcode_boundary_tests {
    use crate::semcode_format::{Opcode, SemcodeFormatError};

    #[test]
    fn candidate_wire_payload_does_not_register_executable_opcodes() {
        for byte in [0x32, 0x33] {
            assert_eq!(
                Opcode::from_byte(byte),
                Err(SemcodeFormatError::UnknownOpcode(byte)),
                "operand codec must not silently widen admitted opcode vocabulary"
            );
        }
    }
}
