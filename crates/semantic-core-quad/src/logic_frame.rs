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

/// Primitive truth-table complement operation.
#[inline(always)]
pub fn not(state: QuadState) -> QuadState {
    NOT_LUT[state as usize]
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
}
