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

pub const AND_LUT: [[QuadState; 4]; 4] = make_and_lut();
pub const OR_LUT: [[QuadState; 4]; 4] = make_or_lut();

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
    fn test_commutativity() {
        for a in QuadState::ALL {
            for b in QuadState::ALL {
                assert_eq!(and(a, b), and(b, a), "AND must be commutative");
                assert_eq!(or(a, b), or(b, a), "OR must be commutative");
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
        }

        assert_eq!(and(QuadState::N, QuadState::T), QuadState::N);
        assert_eq!(or(QuadState::N, QuadState::F), QuadState::N);
        assert_eq!(and(QuadState::S, QuadState::T), QuadState::S);
        assert_eq!(or(QuadState::S, QuadState::F), QuadState::S);
    }
}
