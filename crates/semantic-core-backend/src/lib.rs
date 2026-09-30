#![cfg_attr(not(feature = "std"), no_std)]

mod arm;
mod scalar;
mod x86;

use semantic_core_quad::{QuadTile128, QuadroReg32};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use arm::detect_arm_caps;
pub use x86::detect_x86_caps;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    Scalar,
    Auto,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendCaps {
    pub has_popcnt: bool,
    pub has_bmi1: bool,
    pub has_bmi2: bool,
    pub has_avx2: bool,
    pub has_avx512: bool,
    pub has_neon: bool,
    pub has_sve: bool,
}

impl BackendCaps {
    pub const fn scalar() -> Self {
        Self {
            has_popcnt: false,
            has_bmi1: false,
            has_bmi2: false,
            has_avx2: false,
            has_avx512: false,
            has_neon: false,
            has_sve: false,
        }
    }
}

impl Default for BackendCaps {
    fn default() -> Self {
        Self::scalar()
    }
}

pub fn detect_backend_caps() -> BackendCaps {
    let mut caps = BackendCaps::scalar();
    let x86_caps = detect_x86_caps();
    caps.has_popcnt = caps.has_popcnt || x86_caps.has_popcnt;
    caps.has_bmi1 = caps.has_bmi1 || x86_caps.has_bmi1;
    caps.has_bmi2 = caps.has_bmi2 || x86_caps.has_bmi2;
    caps.has_avx2 = caps.has_avx2 || x86_caps.has_avx2;
    caps.has_avx512 = caps.has_avx512 || x86_caps.has_avx512;
    let arm_caps = detect_arm_caps();
    caps.has_neon = caps.has_neon || arm_caps.has_neon;
    caps.has_sve = caps.has_sve || arm_caps.has_sve;
    caps
}

pub const fn select_backend(kind: BackendKind, _caps: BackendCaps) -> BackendKind {
    match kind {
        BackendKind::Scalar | BackendKind::Auto => BackendKind::Scalar,
    }
}

pub fn join_reg32(kind: BackendKind, dst: &mut [QuadroReg32], src: &[QuadroReg32]) {
    match select_backend(kind, BackendCaps::scalar()) {
        BackendKind::Scalar | BackendKind::Auto => scalar::ScalarBackend::join_reg32(dst, src),
    }
}

pub fn meet_reg32(kind: BackendKind, dst: &mut [QuadroReg32], src: &[QuadroReg32]) {
    match select_backend(kind, BackendCaps::scalar()) {
        BackendKind::Scalar | BackendKind::Auto => scalar::ScalarBackend::meet_reg32(dst, src),
    }
}

pub fn inverse_reg32(kind: BackendKind, dst: &mut [QuadroReg32]) {
    match select_backend(kind, BackendCaps::scalar()) {
        BackendKind::Scalar | BackendKind::Auto => scalar::ScalarBackend::inverse_reg32(dst),
    }
}

pub fn join_tile128(kind: BackendKind, dst: &mut [QuadTile128], src: &[QuadTile128]) {
    match select_backend(kind, BackendCaps::scalar()) {
        BackendKind::Scalar | BackendKind::Auto => scalar::ScalarBackend::join_tile128(dst, src),
    }
}

pub fn meet_tile128(kind: BackendKind, dst: &mut [QuadTile128], src: &[QuadTile128]) {
    match select_backend(kind, BackendCaps::scalar()) {
        BackendKind::Scalar | BackendKind::Auto => scalar::ScalarBackend::meet_tile128(dst, src),
    }
}

pub fn inverse_tile128(kind: BackendKind, dst: &mut [QuadTile128]) {
    match select_backend(kind, BackendCaps::scalar()) {
        BackendKind::Scalar | BackendKind::Auto => scalar::ScalarBackend::inverse_tile128(dst),
    }
}

pub(crate) trait CoreBackend {
    fn join_reg32(dst: &mut [QuadroReg32], src: &[QuadroReg32]);
    fn meet_reg32(dst: &mut [QuadroReg32], src: &[QuadroReg32]);
    fn inverse_reg32(dst: &mut [QuadroReg32]);
    fn join_tile128(dst: &mut [QuadTile128], src: &[QuadTile128]);
    fn meet_tile128(dst: &mut [QuadTile128], src: &[QuadTile128]);
    fn inverse_tile128(dst: &mut [QuadTile128]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use semantic_core_quad::{QuadState, QuadroReg32};

    fn reg_filled(state: QuadState) -> QuadroReg32 {
        let mut reg = QuadroReg32::new();
        for lane in 0..QuadroReg32::LANES {
            reg.set_unchecked(lane, state);
        }
        reg
    }

    #[test]
    fn backend_caps_default_scalar() {
        assert_eq!(BackendCaps::default(), BackendCaps::scalar());
    }

    #[test]
    fn scalar_backend_matches_direct_ops() {
        let mut dst = [reg_filled(QuadState::T), reg_filled(QuadState::F)];
        let src = [reg_filled(QuadState::F), reg_filled(QuadState::T)];
        join_reg32(BackendKind::Scalar, &mut dst, &src);
        assert_eq!(dst[0], reg_filled(QuadState::S));
        assert_eq!(dst[1], reg_filled(QuadState::S));
        inverse_reg32(BackendKind::Scalar, &mut dst);
        assert_eq!(dst[0], reg_filled(QuadState::S));
    }
}
