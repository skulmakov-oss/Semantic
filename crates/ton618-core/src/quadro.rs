//! Packed quad-state primitives for the Pulsar substrate.
//!
//! The raw storage model is intentionally small:
//! - one `u64` stores 32 quadits;
//! - each quadit uses 2 bits;
//! - `00 = N`, `01 = F`, `10 = T`, `11 = S`.
//!
//! The module keeps the scalar path authoritative and treats any bulk or
//! masked update as a checked boundary.

use core::convert::TryFrom;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

pub const QUADIT_WIDTH: usize = 2;
pub const QUADIT_COUNT: usize = 32;
pub const LSB_MASK: u64 = 0x5555_5555_5555_5555;
pub const MSB_MASK: u64 = 0xAAAA_AAAA_AAAA_AAAA;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuadState {
    N = 0b00,
    F = 0b01,
    T = 0b10,
    S = 0b11,
}

impl QuadState {
    pub const fn bits(self) -> u8 {
        self as u8
    }

    pub const fn inverse(self) -> Self {
        match self {
            Self::N => Self::N,
            Self::F => Self::T,
            Self::T => Self::F,
            Self::S => Self::S,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuadroError {
    IndexOutOfRange { index: usize },
    InvalidState { state: u8 },
    MisalignedMask { mask: u64 },
    LengthMismatch { expected: usize, actual: usize },
}

impl core::fmt::Display for QuadroError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::IndexOutOfRange { index } => write!(f, "quadit index out of range: {index}"),
            Self::InvalidState { state } => write!(f, "invalid quad-state: {state:#04b}"),
            Self::MisalignedMask { mask } => write!(f, "mask is not LSB-aligned: {mask:#018x}"),
            Self::LengthMismatch { expected, actual } => {
                write!(f, "length mismatch: expected {expected}, got {actual}")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for QuadroError {}

impl TryFrom<u8> for QuadState {
    type Error = QuadroError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b00 => Ok(Self::N),
            0b01 => Ok(Self::F),
            0b10 => Ok(Self::T),
            0b11 => Ok(Self::S),
            _ => Err(QuadroError::InvalidState { state: value }),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QuadroReg(u64);

impl QuadroReg {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub fn try_get(self, index: usize) -> Result<QuadState, QuadroError> {
        let shift = Self::lane_shift(index)?;
        let bits = ((self.0 >> shift) & 0b11) as u8;
        QuadState::try_from(bits)
    }

    pub fn try_set(&mut self, index: usize, state: u8) -> Result<(), QuadroError> {
        let shift = Self::lane_shift(index)?;
        let state = QuadState::try_from(state)?;
        let lane_mask = 0b11u64 << shift;
        self.0 = (self.0 & !lane_mask) | ((state.bits() as u64) << shift);
        Ok(())
    }

    pub fn merge(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn intersect(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub fn inverse(self) -> Self {
        let lsb = self.0 & LSB_MASK;
        let msb = self.0 & MSB_MASK;
        Self((lsb << 1) | (msb >> 1))
    }

    pub fn raw_delta(self, next: Self) -> u64 {
        self.0 ^ next.0
    }

    pub fn mask_null(self) -> u64 {
        let f = self.0 & LSB_MASK;
        let t = (self.0 & MSB_MASK) >> 1;
        !(f | t) & LSB_MASK
    }

    pub fn mask_strict_false(self) -> u64 {
        let f = self.0 & LSB_MASK;
        let t = (self.0 & MSB_MASK) >> 1;
        f & !t
    }

    pub fn mask_strict_true(self) -> u64 {
        let f = self.0 & LSB_MASK;
        let t = (self.0 & MSB_MASK) >> 1;
        t & !f
    }

    pub fn mask_super(self) -> u64 {
        let f = self.0 & LSB_MASK;
        let t = (self.0 & MSB_MASK) >> 1;
        f & t
    }

    pub fn mask_non_null(self) -> u64 {
        self.mask_strict_false() | self.mask_strict_true() | self.mask_super()
    }

    pub fn masks_all(self) -> QuadMasks {
        QuadMasks {
            null: self.mask_null(),
            strict_false: self.mask_strict_false(),
            strict_true: self.mask_strict_true(),
            super_: self.mask_super(),
            non_null: self.mask_non_null(),
        }
    }

    pub fn try_set_by_mask(&mut self, mask: u64, state: u8) -> Result<(), QuadroError> {
        if mask & !LSB_MASK != 0 {
            return Err(QuadroError::MisalignedMask { mask });
        }
        let state = QuadState::try_from(state)?;
        let clear = mask | (mask << 1);
        let value = match state {
            QuadState::N => 0,
            QuadState::F => mask,
            QuadState::T => mask << 1,
            QuadState::S => clear,
        };
        self.0 = (self.0 & !clear) | value;
        Ok(())
    }

    pub fn clear_by_mask(&mut self, mask: u64) -> Result<(), QuadroError> {
        self.try_set_by_mask(mask, QuadState::N.bits())
    }

    pub fn force_super(&mut self, mask: u64) -> Result<(), QuadroError> {
        self.try_set_by_mask(mask, QuadState::S.bits())
    }

    pub fn calc_delta(self, next: Self) -> StateDelta {
        let old = self.masks_all();
        let new = next.masks_all();
        StateDelta {
            entered_true: new.strict_true & !old.strict_true,
            left_true: old.strict_true & !new.strict_true,
            entered_false: new.strict_false & !old.strict_false,
            left_false: old.strict_false & !new.strict_false,
            entered_super: new.super_ & !old.super_,
            left_super: old.super_ & !new.super_,
        }
    }

    pub fn popcount_quadits(self) -> usize {
        self.mask_non_null().count_ones() as usize
    }

    pub fn iter_mask_indices(mask: u64) -> Result<MaskIndexIter, QuadroError> {
        iter_mask_indices(mask)
    }

    fn lane_shift(index: usize) -> Result<u32, QuadroError> {
        if index >= QUADIT_COUNT {
            return Err(QuadroError::IndexOutOfRange { index });
        }
        Ok((index * QUADIT_WIDTH) as u32)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QuadMasks {
    pub null: u64,
    pub strict_false: u64,
    pub strict_true: u64,
    pub super_: u64,
    pub non_null: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StateDelta {
    pub entered_true: u64,
    pub left_true: u64,
    pub entered_false: u64,
    pub left_false: u64,
    pub entered_super: u64,
    pub left_super: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaskIndexIter {
    remaining: u64,
}

impl MaskIndexIter {
    fn new(mask: u64) -> Self {
        Self { remaining: mask }
    }
}

impl Iterator for MaskIndexIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let bit = self.remaining.trailing_zeros() as usize;
        self.remaining &= self.remaining - 1;
        Some(bit / QUADIT_WIDTH)
    }
}

pub fn iter_mask_indices(mask: u64) -> Result<MaskIndexIter, QuadroError> {
    if mask & !LSB_MASK != 0 {
        return Err(QuadroError::MisalignedMask { mask });
    }
    Ok(MaskIndexIter::new(mask))
}

#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeltaSoA {
    pub entered_true: Vec<u64>,
    pub left_true: Vec<u64>,
    pub entered_false: Vec<u64>,
    pub left_false: Vec<u64>,
    pub entered_super: Vec<u64>,
    pub left_super: Vec<u64>,
}

#[cfg(feature = "alloc")]
impl DeltaSoA {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entered_true: Vec::with_capacity(capacity),
            left_true: Vec::with_capacity(capacity),
            entered_false: Vec::with_capacity(capacity),
            left_false: Vec::with_capacity(capacity),
            entered_super: Vec::with_capacity(capacity),
            left_super: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.entered_true.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&mut self, delta: StateDelta) {
        self.entered_true.push(delta.entered_true);
        self.left_true.push(delta.left_true);
        self.entered_false.push(delta.entered_false);
        self.left_false.push(delta.left_false);
        self.entered_super.push(delta.entered_super);
        self.left_super.push(delta.left_super);
    }
}

#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QuadroBank {
    regs: Vec<QuadroReg>,
}

#[cfg(feature = "alloc")]
impl QuadroBank {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_regs(regs: impl Into<Vec<QuadroReg>>) -> Self {
        Self { regs: regs.into() }
    }

    pub fn len(&self) -> usize {
        self.regs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.regs.is_empty()
    }

    pub fn as_slice(&self) -> &[QuadroReg] {
        &self.regs
    }

    pub fn as_mut_slice(&mut self) -> &mut [QuadroReg] {
        &mut self.regs
    }

    pub fn push(&mut self, reg: QuadroReg) {
        self.regs.push(reg);
    }

    pub fn merge_inplace(&mut self, other: &Self) -> Result<(), QuadroError> {
        self.ensure_same_len(other.len())?;
        for (left, right) in self.regs.iter_mut().zip(other.regs.iter()) {
            *left = left.merge(*right);
        }
        Ok(())
    }

    pub fn intersect_inplace(&mut self, other: &Self) -> Result<(), QuadroError> {
        self.ensure_same_len(other.len())?;
        for (left, right) in self.regs.iter_mut().zip(other.regs.iter()) {
            *left = left.intersect(*right);
        }
        Ok(())
    }

    pub fn inverse_inplace(&mut self) {
        for reg in self.regs.iter_mut() {
            *reg = reg.inverse();
        }
    }

    pub fn calc_deltas_soa(&self, next: &Self) -> Result<DeltaSoA, QuadroError> {
        self.ensure_same_len(next.len())?;
        let mut deltas = DeltaSoA::with_capacity(self.len());
        for (left, right) in self.regs.iter().copied().zip(next.regs.iter().copied()) {
            deltas.push(left.calc_delta(right));
        }
        Ok(deltas)
    }

    fn ensure_same_len(&self, actual: usize) -> Result<(), QuadroError> {
        if self.len() != actual {
            return Err(QuadroError::LengthMismatch {
                expected: self.len(),
                actual,
            });
        }
        Ok(())
    }
}
