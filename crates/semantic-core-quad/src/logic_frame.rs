use crate::QuadState;

/// Generated scalar LUT truth-table layer for the Quad Logic Frame v1.
///
/// NOT operations:
/// - NOT(N) = N
/// - NOT(F) = T
/// - NOT(T) = F
/// - NOT(S) = S
pub const NOT_LUT: [QuadState; 4] = [
    QuadState::N, // N -> N
    QuadState::T, // F -> T
    QuadState::F, // T -> F
    QuadState::S, // S -> S
];

const fn make_and_lut() -> [[QuadState; 4]; 4] {
    let mut lut = [[QuadState::N; 4]; 4];
    let mut i = 0;
    while i < 4 {
        let mut j = 0;
        while j < 4 {
            let a_t = (i >> 1) & 1;
            let a_f = i & 1;
            let b_t = (j >> 1) & 1;
            let b_f = j & 1;
            let r_t = a_t & b_t;
            let r_f = a_f | b_f;
            lut[i as usize][j as usize] = QuadState::from_bits_unchecked((r_t << 1) | r_f);
            j += 1;
        }
        i += 1;
    }
    lut
}

const fn make_or_lut() -> [[QuadState; 4]; 4] {
    let mut lut = [[QuadState::N; 4]; 4];
    let mut i = 0;
    while i < 4 {
        let mut j = 0;
        while j < 4 {
            let a_t = (i >> 1) & 1;
            let a_f = i & 1;
            let b_t = (j >> 1) & 1;
            let b_f = j & 1;
            let r_t = a_t | b_t;
            let r_f = a_f & b_f;
            lut[i as usize][j as usize] = QuadState::from_bits_unchecked((r_t << 1) | r_f);
            j += 1;
        }
        i += 1;
    }
    lut
}

const fn make_xor_lut() -> [[QuadState; 4]; 4] {
    let mut lut = [[QuadState::N; 4]; 4];
    let mut i = 0;
    while i < 4 {
        let mut j = 0;
        while j < 4 {
            let a_bits = i as u8;
            let b_bits = j as u8;
            lut[i as usize][j as usize] = QuadState::from_bits_unchecked(a_bits ^ b_bits);
            j += 1;
        }
        i += 1;
    }
    lut
}

const fn make_implies_lut() -> [[QuadState; 4]; 4] {
    let mut lut = [[QuadState::N; 4]; 4];
    let mut i = 0;
    while i < 4 {
        let mut j = 0;
        while j < 4 {
            let b_state = QuadState::from_bits_unchecked(j as u8);
            let not_a = NOT_LUT[i as usize];
            lut[i as usize][j as usize] = not_a.join(b_state);
            j += 1;
        }
        i += 1;
    }
    lut
}

const fn make_nand_lut() -> [[QuadState; 4]; 4] {
    let mut lut = [[QuadState::N; 4]; 4];
    let mut i = 0;
    while i < 4 {
        let mut j = 0;
        while j < 4 {
            let and_res = AND_LUT[i as usize][j as usize];
            lut[i as usize][j as usize] = NOT_LUT[and_res as usize];
            j += 1;
        }
        i += 1;
    }
    lut
}

const fn make_nor_lut() -> [[QuadState; 4]; 4] {
    let mut lut = [[QuadState::N; 4]; 4];
    let mut i = 0;
    while i < 4 {
        let mut j = 0;
        while j < 4 {
            let or_res = OR_LUT[i as usize][j as usize];
            lut[i as usize][j as usize] = NOT_LUT[or_res as usize];
            j += 1;
        }
        i += 1;
    }
    lut
}

pub const AND_LUT: [[QuadState; 4]; 4] = make_and_lut();
pub const OR_LUT: [[QuadState; 4]; 4] = make_or_lut();
pub const XOR_LUT: [[QuadState; 4]; 4] = make_xor_lut();
pub const IMPLIES_LUT: [[QuadState; 4]; 4] = make_implies_lut();
pub const NAND_LUT: [[QuadState; 4]; 4] = make_nand_lut();
pub const NOR_LUT: [[QuadState; 4]; 4] = make_nor_lut();

// EQUIV is intentionally not exposed as `equiv`.
// Quad Logic Frame v1 marks equivalence as deferred / separately named.
// Do not add an unqualified `equiv(a, b)` API without a dedicated spec update.
pub const EQUIV_POLICY: &str = "deferred_or_separately_named";

/// Primitive truth-table complement operation.
#[inline]
pub fn not(state: QuadState) -> QuadState {
    NOT_LUT[state as usize]
}

#[inline]
pub fn and(a: QuadState, b: QuadState) -> QuadState {
    AND_LUT[a as usize][b as usize]
}

#[inline]
pub fn or(a: QuadState, b: QuadState) -> QuadState {
    OR_LUT[a as usize][b as usize]
}

#[inline]
pub fn xor(a: QuadState, b: QuadState) -> QuadState {
    XOR_LUT[a as usize][b as usize]
}

#[inline]
pub fn implies(a: QuadState, b: QuadState) -> QuadState {
    IMPLIES_LUT[a as usize][b as usize]
}

#[inline]
pub fn nand(a: QuadState, b: QuadState) -> QuadState {
    NAND_LUT[a as usize][b as usize]
}

#[inline]
pub fn nor(a: QuadState, b: QuadState) -> QuadState {
    NOR_LUT[a as usize][b as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_table() {
        assert_eq!(not(QuadState::N), QuadState::N, "NOT(Null) must be Null");
        assert_eq!(not(QuadState::F), QuadState::T, "NOT(False) must be True");
        assert_eq!(not(QuadState::T), QuadState::F, "NOT(True) must be False");
        assert_eq!(not(QuadState::S), QuadState::S, "NOT(Super) must be Super");
    }

    #[test]
    fn test_and_or_plane_formulas() {
        for a in QuadState::ALL {
            for b in QuadState::ALL {
                let a_t = (a as u8 >> 1) & 1;
                let a_f = a as u8 & 1;
                let b_t = (b as u8 >> 1) & 1;
                let b_f = b as u8 & 1;

                let and_t = a_t & b_t;
                let and_f = a_f | b_f;
                let exp_and = QuadState::from_bits_unchecked((and_t << 1) | and_f);
                assert_eq!(and(a, b), exp_and, "AND plane formula mismatch");

                let or_t = a_t | b_t;
                let or_f = a_f & b_f;
                let exp_or = QuadState::from_bits_unchecked((or_t << 1) | or_f);
                assert_eq!(or(a, b), exp_or, "OR plane formula mismatch");
            }
        }
    }

    #[test]
    fn test_xor_table() {
        for a in QuadState::ALL {
            for b in QuadState::ALL {
                assert_eq!(xor(a, b), a.raw_xor(b), "XOR raw match mismatch");
            }
        }
    }

    #[test]
    fn test_implies_table() {
        for a in QuadState::ALL {
            for b in QuadState::ALL {
                assert_eq!(
                    implies(a, b),
                    not(a).join(b),
                    "IMPLIES derived policy mismatch"
                );
            }
        }

        assert_ne!(
            implies(QuadState::T, QuadState::F),
            implies(QuadState::F, QuadState::T),
            "Implication must be directional"
        );

        // Exact checks
        assert_eq!(implies(QuadState::T, QuadState::F), QuadState::F);
        assert_eq!(implies(QuadState::F, QuadState::T), QuadState::T);
        assert_eq!(implies(QuadState::F, QuadState::N), QuadState::T);
        assert_eq!(implies(QuadState::N, QuadState::F), QuadState::F);
        assert_eq!(implies(QuadState::T, QuadState::T), QuadState::S);
        assert_eq!(implies(QuadState::S, QuadState::N), QuadState::S);
    }

    #[test]
    fn test_nand_nor_table() {
        for a in QuadState::ALL {
            for b in QuadState::ALL {
                assert_eq!(nand(a, b), not(and(a, b)), "NAND derived policy mismatch");
                assert_eq!(nor(a, b), not(or(a, b)), "NOR derived policy mismatch");
            }
        }

        // Exact checks
        assert_eq!(nand(QuadState::T, QuadState::T), QuadState::F);
        assert_eq!(nand(QuadState::F, QuadState::T), QuadState::T);
        assert_eq!(nand(QuadState::N, QuadState::T), QuadState::N);

        assert_eq!(nor(QuadState::F, QuadState::F), QuadState::T);
        assert_eq!(nor(QuadState::T, QuadState::F), QuadState::F);
        assert_eq!(nor(QuadState::N, QuadState::F), QuadState::N);
    }

    #[test]
    fn test_commutativity() {
        for a in QuadState::ALL {
            for b in QuadState::ALL {
                assert_eq!(and(a, b), and(b, a), "AND must be commutative");
                assert_eq!(or(a, b), or(b, a), "OR must be commutative");
                assert_eq!(xor(a, b), xor(b, a), "XOR must be commutative");
                assert_eq!(nand(a, b), nand(b, a), "NAND must be commutative");
                assert_eq!(nor(a, b), nor(b, a), "NOR must be commutative");
            }
        }
    }

    #[test]
    fn test_identity_checks() {
        // T AND x behaves as truth-table pass-through where defined by plane formula
        // F OR x behaves as truth-table pass-through where defined by plane formula
        for x in QuadState::ALL {
            assert_eq!(and(QuadState::T, x), x, "T AND x should be x");
            assert_eq!(or(QuadState::F, x), x, "F OR x should be x");
            assert_eq!(xor(x, x), QuadState::N, "x XOR x should be N");
            assert_eq!(xor(QuadState::N, x), x, "N XOR x should be x");
        }

        assert_eq!(and(QuadState::N, QuadState::T), QuadState::N);
        assert_eq!(or(QuadState::N, QuadState::F), QuadState::N);
        assert_eq!(and(QuadState::S, QuadState::T), QuadState::S);
        assert_eq!(or(QuadState::S, QuadState::F), QuadState::S);

        // Exact checks for XOR
        assert_eq!(xor(QuadState::F, QuadState::T), QuadState::S);
        assert_eq!(xor(QuadState::T, QuadState::S), QuadState::F);
        assert_eq!(xor(QuadState::F, QuadState::S), QuadState::T);
        assert_eq!(xor(QuadState::N, QuadState::S), QuadState::S);
    }

    #[test]
    fn test_equiv_policy_is_deferred() {
        assert_eq!(EQUIV_POLICY, "deferred_or_separately_named", "EQUIV policy must remain deferred until explicitly standardized");
    }
}
