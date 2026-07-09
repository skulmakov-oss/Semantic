#![allow(clippy::new_without_default, clippy::zero_repeat_side_effects)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod logic_frame;

use core::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub const LSB_MASK_32: u64 = 0x5555_5555_5555_5555;
pub const MSB_MASK_32: u64 = 0xAAAA_AAAA_AAAA_AAAA;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuadBoundsError {
    index: usize,
    lanes: usize,
}

impl QuadBoundsError {
    pub const fn new(index: usize, lanes: usize) -> Self {
        Self { index, lanes }
    }

    pub const fn index(self) -> usize {
        self.index
    }

    pub const fn lanes(self) -> usize {
        self.lanes
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum QuadState {
    N = 0b00,
    F = 0b01,
    T = 0b10,
    S = 0b11,
}

impl QuadState {
    pub const ALL: [Self; 4] = [Self::N, Self::F, Self::T, Self::S];

    pub const fn bits(self) -> u8 {
        self as u8
    }

    pub const fn from_bits(bits: u8) -> Option<Self> {
        match bits {
            0b00 => Some(Self::N),
            0b01 => Some(Self::F),
            0b10 => Some(Self::T),
            0b11 => Some(Self::S),
            _ => None,
        }
    }

    pub const fn from_bits_unchecked(bits: u8) -> Self {
        match bits & 0b11 {
            0b00 => Self::N,
            0b01 => Self::F,
            0b10 => Self::T,
            _ => Self::S,
        }
    }

    pub const fn true_plane(self) -> bool {
        (self.bits() & 0b10) != 0
    }

    pub const fn false_plane(self) -> bool {
        (self.bits() & 0b01) != 0
    }

    pub const fn is_null(self) -> bool {
        self.bits() == Self::N.bits()
    }

    pub const fn is_known(self) -> bool {
        self.bits() == Self::F.bits() || self.bits() == Self::T.bits()
    }

    pub const fn is_conflict(self) -> bool {
        self.bits() == Self::S.bits()
    }

    pub const fn inverse(self) -> Self {
        Self::from_bits_unchecked(((self.bits() & 0b01) << 1) | ((self.bits() & 0b10) >> 1))
    }

    pub const fn join(self, other: Self) -> Self {
        Self::from_bits_unchecked(self.bits() | other.bits())
    }

    pub const fn meet(self, other: Self) -> Self {
        Self::from_bits_unchecked(self.bits() & other.bits())
    }

    pub const fn raw_xor(self, other: Self) -> Self {
        Self::from_bits_unchecked(self.bits() ^ other.bits())
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct QuadroReg32(u64);

impl QuadroReg32 {
    pub const LANES: usize = 32;

    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub fn try_get(self, index: usize) -> Option<QuadState> {
        (index < Self::LANES).then(|| self.get_unchecked(index))
    }

    pub fn try_set(&mut self, index: usize, state: QuadState) -> Result<(), QuadBoundsError> {
        if index >= Self::LANES {
            return Err(QuadBoundsError::new(index, Self::LANES));
        }
        self.set_unchecked(index, state);
        Ok(())
    }

    pub fn get_unchecked(self, index: usize) -> QuadState {
        debug_assert!(index < Self::LANES);
        let shift = index * 2;
        QuadState::from_bits_unchecked(((self.0 >> shift) & 0b11) as u8)
    }

    pub fn set_unchecked(&mut self, index: usize, state: QuadState) {
        debug_assert!(index < Self::LANES);
        let shift = index * 2;
        let mask = !(0b11u64 << shift);
        self.0 = (self.0 & mask) | ((state.bits() as u64) << shift);
    }

    pub const fn join(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn meet(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub const fn inverse(self) -> Self {
        Self(((self.0 & LSB_MASK_32) << 1) | ((self.0 & MSB_MASK_32) >> 1))
    }

    pub const fn raw_delta(self, other: Self) -> u64 {
        self.0 ^ other.0
    }

    pub fn masks(self) -> QuadMasks32 {
        let mut n = 0u64;
        let mut f = 0u64;
        let mut t = 0u64;
        let mut s = 0u64;
        let mut lane = 0usize;
        while lane < Self::LANES {
            let bit = 1u64 << lane;
            match self.get_unchecked(lane) {
                QuadState::N => n |= bit,
                QuadState::F => f |= bit,
                QuadState::T => t |= bit,
                QuadState::S => s |= bit,
            }
            lane += 1;
        }
        QuadMasks32 {
            n: QuadMask32::new_unchecked(n),
            f: QuadMask32::new_unchecked(f),
            t: QuadMask32::new_unchecked(t),
            s: QuadMask32::new_unchecked(s),
        }
    }

    pub fn mask_known(self) -> QuadMask32 {
        self.masks().known()
    }

    pub fn mask_null(self) -> QuadMask32 {
        self.masks().null()
    }

    pub fn mask_conflict(self) -> QuadMask32 {
        self.masks().conflict()
    }

    pub fn mask_true(self) -> QuadMask32 {
        let mut raw = 0u64;
        let mut lane = 0usize;
        while lane < Self::LANES {
            if self.get_unchecked(lane).true_plane() {
                raw |= 1u64 << lane;
            }
            lane += 1;
        }
        QuadMask32::new_unchecked(raw)
    }

    pub fn mask_false(self) -> QuadMask32 {
        let mut raw = 0u64;
        let mut lane = 0usize;
        while lane < Self::LANES {
            if self.get_unchecked(lane).false_plane() {
                raw |= 1u64 << lane;
            }
            lane += 1;
        }
        QuadMask32::new_unchecked(raw)
    }

    pub fn set_by_mask(&mut self, mask: QuadMask32, state: QuadState) {
        for lane in mask.iter() {
            self.set_unchecked(lane, state);
        }
    }

    pub fn clear_by_mask(&mut self, mask: QuadMask32) {
        self.set_by_mask(mask, QuadState::N);
    }

    pub fn force_super(&mut self, mask: QuadMask32) {
        self.set_by_mask(mask, QuadState::S);
    }

    pub fn map_not_scalar(mut self) -> Self {
        let mut lane = 0;
        while lane < Self::LANES {
            let state = self.get_unchecked(lane);
            self.set_unchecked(lane, logic_frame::not(state));
            lane += 1;
        }
        self
    }

    pub fn map_and_scalar(mut self, other: Self) -> Self {
        let mut lane = 0;
        while lane < Self::LANES {
            let a = self.get_unchecked(lane);
            let b = other.get_unchecked(lane);
            self.set_unchecked(lane, logic_frame::and(a, b));
            lane += 1;
        }
        self
    }

    pub fn map_or_scalar(mut self, other: Self) -> Self {
        let mut lane = 0;
        while lane < Self::LANES {
            let a = self.get_unchecked(lane);
            let b = other.get_unchecked(lane);
            self.set_unchecked(lane, logic_frame::or(a, b));
            lane += 1;
        }
        self
    }

    pub fn map_xor_scalar(mut self, other: Self) -> Self {
        let mut lane = 0;
        while lane < Self::LANES {
            let a = self.get_unchecked(lane);
            let b = other.get_unchecked(lane);
            self.set_unchecked(lane, logic_frame::xor(a, b));
            lane += 1;
        }
        self
    }

    pub fn map_implies_scalar(mut self, other: Self) -> Self {
        let mut lane = 0;
        while lane < Self::LANES {
            let a = self.get_unchecked(lane);
            let b = other.get_unchecked(lane);
            self.set_unchecked(lane, logic_frame::implies(a, b));
            lane += 1;
        }
        self
    }

    pub fn map_nand_scalar(mut self, other: Self) -> Self {
        let mut lane = 0;
        while lane < Self::LANES {
            let a = self.get_unchecked(lane);
            let b = other.get_unchecked(lane);
            self.set_unchecked(lane, logic_frame::nand(a, b));
            lane += 1;
        }
        self
    }

    pub fn map_nor_scalar(mut self, other: Self) -> Self {
        let mut lane = 0;
        while lane < Self::LANES {
            let a = self.get_unchecked(lane);
            let b = other.get_unchecked(lane);
            self.set_unchecked(lane, logic_frame::nor(a, b));
            lane += 1;
        }
        self
    }

    // SWAR NOT swaps the falsity and truth bit planes.
    pub const fn map_not_swar(self) -> Self {
        self.inverse()
    }

    // SWAR XOR is raw-code XOR across packed quadit lanes.
    pub const fn map_xor_swar(self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }

    // SWAR AND computes truth-plane intersection and falsity-plane union.
    pub const fn map_and_swar(self, other: Self) -> Self {
        let af = self.0 & LSB_MASK_32;
        let at = self.0 & MSB_MASK_32;
        let bf = other.0 & LSB_MASK_32;
        let bt = other.0 & MSB_MASK_32;

        Self((at & bt) | (af | bf))
    }

    // SWAR OR computes truth-plane union and falsity-plane intersection.
    pub const fn map_or_swar(self, other: Self) -> Self {
        let af = self.0 & LSB_MASK_32;
        let at = self.0 & MSB_MASK_32;
        let bf = other.0 & LSB_MASK_32;
        let bt = other.0 & MSB_MASK_32;

        Self((at | bt) | (af & bf))
    }
}

impl fmt::Debug for QuadroReg32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("QReg32[")?;
        for lane in 0..Self::LANES {
            let ch = match self.get_unchecked(lane) {
                QuadState::N => "N",
                QuadState::F => "F",
                QuadState::T => "T",
                QuadState::S => "S",
            };
            f.write_str(ch)?;
        }
        f.write_str("]")
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct QuadMask32(u64);

impl QuadMask32 {
    pub const VALID_MASK: u64 = 0xFFFF_FFFF;

    pub const fn try_new(raw: u64) -> Option<Self> {
        if raw & !Self::VALID_MASK == 0 {
            Some(Self(raw))
        } else {
            None
        }
    }

    pub const fn new_unchecked(raw: u64) -> Self {
        Self(raw & Self::VALID_MASK)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub fn expand2(self) -> u64 {
        let mut out = 0u64;
        let mut raw = self.0;
        let mut lane = 0usize;
        while raw != 0 {
            if raw & 1 != 0 {
                out |= 0b11u64 << (lane * 2);
            }
            raw >>= 1;
            lane += 1;
        }
        out
    }

    pub const fn count(self) -> u32 {
        self.0.count_ones()
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn and(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub const fn or(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn xor(self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }

    pub const fn not_valid(self) -> Self {
        Self(!self.0 & Self::VALID_MASK)
    }

    pub fn iter(self) -> QuadMask32Iter {
        QuadMask32Iter { remaining: self.0 }
    }
}

pub struct QuadMask32Iter {
    remaining: u64,
}

impl Iterator for QuadMask32Iter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let idx = self.remaining.trailing_zeros() as usize;
        self.remaining &= self.remaining - 1;
        Some(idx)
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuadMasks32 {
    pub n: QuadMask32,
    pub f: QuadMask32,
    pub t: QuadMask32,
    pub s: QuadMask32,
}

impl QuadMasks32 {
    pub const fn known(self) -> QuadMask32 {
        self.t.or(self.f)
    }

    pub const fn conflict(self) -> QuadMask32 {
        self.s
    }

    pub const fn null(self) -> QuadMask32 {
        self.n
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct QuadTile128 {
    t: u128,
    f: u128,
}

impl QuadTile128 {
    pub const LANES: usize = 128;

    pub const fn new() -> Self {
        Self { t: 0, f: 0 }
    }

    pub const fn from_planes(t: u128, f: u128) -> Self {
        Self { t, f }
    }

    pub const fn true_plane(self) -> u128 {
        self.t
    }

    pub const fn false_plane(self) -> u128 {
        self.f
    }

    pub fn try_get(self, index: usize) -> Option<QuadState> {
        (index < Self::LANES).then(|| self.get_unchecked(index))
    }

    pub fn try_set(&mut self, index: usize, state: QuadState) -> Result<(), QuadBoundsError> {
        if index >= Self::LANES {
            return Err(QuadBoundsError::new(index, Self::LANES));
        }
        self.set_unchecked(index, state);
        Ok(())
    }

    pub fn get_unchecked(self, index: usize) -> QuadState {
        debug_assert!(index < Self::LANES);
        let bit = 1u128 << index;
        let t = (self.t & bit) != 0;
        let f = (self.f & bit) != 0;
        QuadState::from_bits_unchecked(((t as u8) << 1) | (f as u8))
    }

    pub fn set_unchecked(&mut self, index: usize, state: QuadState) {
        debug_assert!(index < Self::LANES);
        let bit = 1u128 << index;
        self.t &= !bit;
        self.f &= !bit;
        if state.true_plane() {
            self.t |= bit;
        }
        if state.false_plane() {
            self.f |= bit;
        }
    }

    pub const fn join(self, other: Self) -> Self {
        Self {
            t: self.t | other.t,
            f: self.f | other.f,
        }
    }

    pub const fn meet(self, other: Self) -> Self {
        Self {
            t: self.t & other.t,
            f: self.f & other.f,
        }
    }

    pub const fn inverse(self) -> Self {
        Self {
            t: self.f,
            f: self.t,
        }
    }

    pub const fn raw_delta(self, other: Self) -> Self {
        Self {
            t: self.t ^ other.t,
            f: self.f ^ other.f,
        }
    }

    pub const fn known_mask(self) -> QuadMask128 {
        QuadMask128(self.t ^ self.f)
    }

    pub const fn conflict_mask(self) -> QuadMask128 {
        QuadMask128(self.t & self.f)
    }

    pub const fn null_mask(self) -> QuadMask128 {
        QuadMask128(!(self.t | self.f))
    }

    pub const fn true_mask(self) -> QuadMask128 {
        QuadMask128(self.t)
    }

    pub const fn false_mask(self) -> QuadMask128 {
        QuadMask128(self.f)
    }

    pub fn set_by_mask(&mut self, mask: QuadMask128, state: QuadState) {
        for lane in mask.iter() {
            self.set_unchecked(lane, state);
        }
    }

    pub fn from_regs(regs: [QuadroReg32; 4]) -> Self {
        let mut out = Self::new();
        let mut lane = 0usize;
        while lane < Self::LANES {
            let reg_index = lane / 32;
            let reg_lane = lane % 32;
            out.set_unchecked(lane, regs[reg_index].get_unchecked(reg_lane));
            lane += 1;
        }
        out
    }

    pub fn to_regs(self) -> [QuadroReg32; 4] {
        let mut regs = [QuadroReg32::new(); 4];
        let mut lane = 0usize;
        while lane < Self::LANES {
            let reg_index = lane / 32;
            let reg_lane = lane % 32;
            regs[reg_index].set_unchecked(reg_lane, self.get_unchecked(lane));
            lane += 1;
        }
        regs
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct QuadMask128(u128);

impl QuadMask128 {
    pub const fn raw(self) -> u128 {
        self.0
    }

    pub const fn count(self) -> u32 {
        self.0.count_ones()
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn and(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub const fn or(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn xor(self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }

    pub const fn not_valid(self) -> Self {
        Self(!self.0)
    }

    pub fn iter(self) -> QuadMask128Iter {
        QuadMask128Iter { remaining: self.0 }
    }
}

pub struct QuadMask128Iter {
    remaining: u128,
}

impl Iterator for QuadMask128Iter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let idx = self.remaining.trailing_zeros() as usize;
        self.remaining &= self.remaining - 1;
        Some(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateDelta32 {
    pub entered_true: QuadMask32,
    pub left_true: QuadMask32,
    pub entered_false: QuadMask32,
    pub left_false: QuadMask32,
    pub entered_super: QuadMask32,
    pub left_super: QuadMask32,
    pub changed: QuadMask32,
    pub became_known: QuadMask32,
    pub became_unknown: QuadMask32,
    pub became_conflicted: QuadMask32,
    pub resolved_conflict: QuadMask32,
}

impl StateDelta32 {
    pub fn from_regs(prev: QuadroReg32, current: QuadroReg32) -> Self {
        let prev_true = prev.mask_true();
        let prev_false = prev.mask_false();
        let curr_true = current.mask_true();
        let curr_false = current.mask_false();
        let prev_known = prev.mask_known();
        let curr_known = current.mask_known();
        let prev_conflict = prev.mask_conflict();
        let curr_conflict = current.mask_conflict();
        let changed = prev_true.xor(curr_true).or(prev_false.xor(curr_false));

        Self {
            entered_true: curr_true.and(prev_true.not_valid()),
            left_true: prev_true.and(curr_true.not_valid()),
            entered_false: curr_false.and(prev_false.not_valid()),
            left_false: prev_false.and(curr_false.not_valid()),
            entered_super: curr_conflict.and(prev_conflict.not_valid()),
            left_super: prev_conflict.and(curr_conflict.not_valid()),
            changed,
            became_known: curr_known.and(prev_known.not_valid()),
            became_unknown: prev_known.and(curr_known.not_valid()),
            became_conflicted: curr_conflict.and(prev_conflict.not_valid()),
            resolved_conflict: prev_conflict.and(curr_conflict.not_valid()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateDelta128 {
    pub entered_true: QuadMask128,
    pub left_true: QuadMask128,
    pub entered_false: QuadMask128,
    pub left_false: QuadMask128,
    pub entered_super: QuadMask128,
    pub left_super: QuadMask128,
    pub changed: QuadMask128,
    pub became_known: QuadMask128,
    pub became_unknown: QuadMask128,
    pub became_conflicted: QuadMask128,
    pub resolved_conflict: QuadMask128,
}

impl StateDelta128 {
    pub fn from_tiles(prev: QuadTile128, current: QuadTile128) -> Self {
        let prev_true = prev.true_mask();
        let prev_false = prev.false_mask();
        let curr_true = current.true_mask();
        let curr_false = current.false_mask();
        let prev_known = prev.known_mask();
        let curr_known = current.known_mask();
        let prev_conflict = prev.conflict_mask();
        let curr_conflict = current.conflict_mask();
        let changed = prev_true.xor(curr_true).or(prev_false.xor(curr_false));

        Self {
            entered_true: curr_true.and(prev_true.not_valid()),
            left_true: prev_true.and(curr_true.not_valid()),
            entered_false: curr_false.and(prev_false.not_valid()),
            left_false: prev_false.and(curr_false.not_valid()),
            entered_super: curr_conflict.and(prev_conflict.not_valid()),
            left_super: prev_conflict.and(curr_conflict.not_valid()),
            changed,
            became_known: curr_known.and(prev_known.not_valid()),
            became_unknown: prev_known.and(curr_known.not_valid()),
            became_conflicted: curr_conflict.and(prev_conflict.not_valid()),
            resolved_conflict: prev_conflict.and(curr_conflict.not_valid()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuadroBank<const N: usize> {
    regs: [QuadroReg32; N],
}

impl<const N: usize> QuadroBank<N> {
    pub const fn new() -> Self {
        Self {
            regs: [QuadroReg32::new(); N],
        }
    }

    pub const fn from_array(regs: [QuadroReg32; N]) -> Self {
        Self { regs }
    }

    pub const fn as_array(&self) -> &[QuadroReg32; N] {
        &self.regs
    }

    pub const fn as_slice(&self) -> &[QuadroReg32] {
        &self.regs
    }

    pub fn get(&self, index: usize) -> Option<QuadroReg32> {
        self.regs.get(index).copied()
    }

    pub fn set(&mut self, index: usize, reg: QuadroReg32) -> Result<(), QuadBoundsError> {
        match self.regs.get_mut(index) {
            Some(slot) => {
                *slot = reg;
                Ok(())
            }
            None => Err(QuadBoundsError::new(index, N)),
        }
    }

    pub fn join_inplace(&mut self, other: &Self) {
        for (dst, src) in self.regs.iter_mut().zip(other.regs.iter().copied()) {
            *dst = dst.join(src);
        }
    }

    pub fn meet_inplace(&mut self, other: &Self) {
        for (dst, src) in self.regs.iter_mut().zip(other.regs.iter().copied()) {
            *dst = dst.meet(src);
        }
    }

    pub fn inverse_inplace(&mut self) {
        for reg in &mut self.regs {
            *reg = reg.inverse();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuadTileBank<const N: usize> {
    tiles: [QuadTile128; N],
}

impl<const N: usize> QuadTileBank<N> {
    pub const fn new() -> Self {
        Self {
            tiles: [QuadTile128::new(); N],
        }
    }

    pub const fn from_array(tiles: [QuadTile128; N]) -> Self {
        Self { tiles }
    }

    pub const fn as_array(&self) -> &[QuadTile128; N] {
        &self.tiles
    }

    pub const fn as_slice(&self) -> &[QuadTile128] {
        &self.tiles
    }

    pub fn get(&self, index: usize) -> Option<QuadTile128> {
        self.tiles.get(index).copied()
    }

    pub fn set(&mut self, index: usize, tile: QuadTile128) -> Result<(), QuadBoundsError> {
        match self.tiles.get_mut(index) {
            Some(slot) => {
                *slot = tile;
                Ok(())
            }
            None => Err(QuadBoundsError::new(index, N)),
        }
    }

    pub fn join_inplace(&mut self, other: &Self) {
        for (dst, src) in self.tiles.iter_mut().zip(other.tiles.iter().copied()) {
            *dst = dst.join(src);
        }
    }

    pub fn meet_inplace(&mut self, other: &Self) {
        for (dst, src) in self.tiles.iter_mut().zip(other.tiles.iter().copied()) {
            *dst = dst.meet(src);
        }
    }

    pub fn inverse_inplace(&mut self) {
        for tile in &mut self.tiles {
            *tile = tile.inverse();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::format;
    use alloc::vec::Vec;

    fn reg_filled(state: QuadState) -> QuadroReg32 {
        let mut reg = QuadroReg32::new();
        for lane in 0..QuadroReg32::LANES {
            reg.set_unchecked(lane, state);
        }
        reg
    }

    fn tile_filled(state: QuadState) -> QuadTile128 {
        let mut tile = QuadTile128::new();
        for lane in 0..QuadTile128::LANES {
            tile.set_unchecked(lane, state);
        }
        tile
    }

    #[test]
    fn quad_state_encoding_is_frozen() {
        assert_eq!(QuadState::N.bits(), 0b00);
        assert_eq!(QuadState::F.bits(), 0b01);
        assert_eq!(QuadState::T.bits(), 0b10);
        assert_eq!(QuadState::S.bits(), 0b11);
    }

    #[test]
    fn quad_state_inverse_truth_table() {
        assert_eq!(QuadState::N.inverse(), QuadState::N);
        assert_eq!(QuadState::F.inverse(), QuadState::T);
        assert_eq!(QuadState::T.inverse(), QuadState::F);
        assert_eq!(QuadState::S.inverse(), QuadState::S);
    }

    #[test]
    fn quad_state_join_truth_table() {
        for lhs in QuadState::ALL {
            for rhs in QuadState::ALL {
                assert_eq!(lhs.join(rhs).bits(), lhs.bits() | rhs.bits());
            }
        }
    }

    #[test]
    fn quad_state_meet_truth_table() {
        for lhs in QuadState::ALL {
            for rhs in QuadState::ALL {
                assert_eq!(lhs.meet(rhs).bits(), lhs.bits() & rhs.bits());
            }
        }
    }

    #[test]
    fn quad_state_from_bits_rejects_invalid() {
        assert_eq!(QuadState::from_bits(4), None);
        assert_eq!(QuadState::from_bits(255), None);
    }

    #[test]
    fn reg32_get_set_all_lanes_all_states() {
        let mut reg = QuadroReg32::new();
        for lane in 0..QuadroReg32::LANES {
            for state in QuadState::ALL {
                reg.try_set(lane, state).unwrap();
                assert_eq!(reg.try_get(lane), Some(state));
            }
        }
    }

    #[test]
    fn reg32_join_matches_quad_truth_table() {
        for lhs in QuadState::ALL {
            for rhs in QuadState::ALL {
                let reg = reg_filled(lhs).join(reg_filled(rhs));
                for lane in 0..QuadroReg32::LANES {
                    assert_eq!(reg.get_unchecked(lane), lhs.join(rhs));
                }
            }
        }
    }

    #[test]
    fn reg32_meet_matches_quad_truth_table() {
        for lhs in QuadState::ALL {
            for rhs in QuadState::ALL {
                let reg = reg_filled(lhs).meet(reg_filled(rhs));
                for lane in 0..QuadroReg32::LANES {
                    assert_eq!(reg.get_unchecked(lane), lhs.meet(rhs));
                }
            }
        }
    }

    #[test]
    fn reg32_inverse_matches_quad_truth_table() {
        for state in QuadState::ALL {
            let reg = reg_filled(state).inverse();
            for lane in 0..QuadroReg32::LANES {
                assert_eq!(reg.get_unchecked(lane), state.inverse());
            }
        }
    }

    #[test]
    fn reg32_raw_roundtrip() {
        let raw = 0xDEAD_BEEF_F00D_BAADu64;
        assert_eq!(QuadroReg32::from_raw(raw).raw(), raw);
    }

    #[test]
    fn reg32_out_of_bounds_rejected() {
        let mut reg = QuadroReg32::new();
        assert_eq!(reg.try_get(32), None);
        assert_eq!(
            reg.try_set(32, QuadState::T),
            Err(QuadBoundsError::new(32, 32))
        );
    }

    #[test]
    fn reg32_debug_is_deterministic() {
        let mut reg = QuadroReg32::new();
        reg.set_unchecked(0, QuadState::T);
        reg.set_unchecked(1, QuadState::F);
        reg.set_unchecked(2, QuadState::S);
        let text = format!("{reg:?}");
        assert!(text.starts_with("QReg32["));
        assert!(text.contains("TFS"));
        assert!(text.ends_with(']'));
    }

    #[test]
    fn mask32_rejects_msb_aligned_bits() {
        assert_eq!(QuadMask32::try_new(1u64 << 40), None);
        assert_eq!(
            QuadMask32::try_new(0xFFFF_FFFF),
            Some(QuadMask32(0xFFFF_FFFF))
        );
    }

    #[test]
    fn mask32_expand2_expands_to_two_bit_slots() {
        let mask = QuadMask32::new_unchecked(0b1011);
        assert_eq!(mask.expand2(), 0b11 | (0b11 << 2) | (0b11 << 6));
    }

    #[test]
    fn mask32_iter_returns_lane_indices() {
        let lanes: Vec<_> = QuadMask32::new_unchecked(0b10101).iter().collect();
        assert_eq!(lanes, [0, 2, 4]);
    }

    #[test]
    fn mask32_count_matches_popcount() {
        let mask = QuadMask32::new_unchecked(0xF0F0_F00F);
        assert_eq!(mask.count(), mask.raw().count_ones());
    }

    #[test]
    fn reg32_masks_all_n() {
        let masks = reg_filled(QuadState::N).masks();
        assert_eq!(masks.n.raw(), 0xFFFF_FFFF);
        assert!(masks.f.is_empty());
        assert!(masks.t.is_empty());
        assert!(masks.s.is_empty());
    }

    #[test]
    fn reg32_masks_all_f() {
        let masks = reg_filled(QuadState::F).masks();
        assert_eq!(masks.f.raw(), 0xFFFF_FFFF);
    }

    #[test]
    fn reg32_masks_all_t() {
        let masks = reg_filled(QuadState::T).masks();
        assert_eq!(masks.t.raw(), 0xFFFF_FFFF);
    }

    #[test]
    fn reg32_masks_all_s() {
        let masks = reg_filled(QuadState::S).masks();
        assert_eq!(masks.s.raw(), 0xFFFF_FFFF);
    }

    #[test]
    fn reg32_masks_mixed_pattern() {
        let mut reg = reg_filled(QuadState::S);
        reg.set_unchecked(0, QuadState::N);
        reg.set_unchecked(1, QuadState::F);
        reg.set_unchecked(2, QuadState::T);
        reg.set_unchecked(3, QuadState::S);
        let masks = reg.masks();
        assert_eq!(masks.n.raw(), 0b0001);
        assert_eq!(masks.f.raw(), 0b0010);
        assert_eq!(masks.t.raw(), 0b0100);
        assert_eq!(masks.s.raw() & 0b1111, 0b1000);
    }

    #[test]
    fn reg32_set_by_mask_changes_only_selected_lanes() {
        let mut reg = reg_filled(QuadState::F);
        reg.set_by_mask(QuadMask32::new_unchecked(0b1010), QuadState::T);
        assert_eq!(reg.get_unchecked(0), QuadState::F);
        assert_eq!(reg.get_unchecked(1), QuadState::T);
        assert_eq!(reg.get_unchecked(2), QuadState::F);
        assert_eq!(reg.get_unchecked(3), QuadState::T);
    }

    #[test]
    fn tile128_get_set_all_lanes_all_states() {
        let mut tile = QuadTile128::new();
        for lane in 0..QuadTile128::LANES {
            for state in QuadState::ALL {
                tile.try_set(lane, state).unwrap();
                assert_eq!(tile.try_get(lane), Some(state));
            }
        }
    }

    #[test]
    fn tile128_join_matches_quad_truth_table() {
        for lhs in QuadState::ALL {
            for rhs in QuadState::ALL {
                let tile = tile_filled(lhs).join(tile_filled(rhs));
                for lane in 0..QuadTile128::LANES {
                    assert_eq!(tile.get_unchecked(lane), lhs.join(rhs));
                }
            }
        }
    }

    #[test]
    fn tile128_meet_matches_quad_truth_table() {
        for lhs in QuadState::ALL {
            for rhs in QuadState::ALL {
                let tile = tile_filled(lhs).meet(tile_filled(rhs));
                for lane in 0..QuadTile128::LANES {
                    assert_eq!(tile.get_unchecked(lane), lhs.meet(rhs));
                }
            }
        }
    }

    #[test]
    fn tile128_inverse_matches_quad_truth_table() {
        for state in QuadState::ALL {
            let tile = tile_filled(state).inverse();
            for lane in 0..QuadTile128::LANES {
                assert_eq!(tile.get_unchecked(lane), state.inverse());
            }
        }
    }

    #[test]
    fn tile128_known_mask() {
        let tile = QuadTile128::from_planes(0b0011, 0b0101);
        assert_eq!(tile.known_mask().raw(), 0b0110);
    }

    #[test]
    fn tile128_conflict_mask() {
        let tile = QuadTile128::from_planes(0b0111, 0b0101);
        assert_eq!(tile.conflict_mask().raw(), 0b0101);
    }

    #[test]
    fn tile128_null_mask() {
        let tile = QuadTile128::from_planes(0b0011, 0b0101);
        assert_eq!(tile.null_mask().raw() & 0b1111, 0b1000);
    }

    #[test]
    fn mask128_count() {
        let mask = QuadMask128(0xF0F0);
        assert_eq!(mask.count(), 8);
    }

    #[test]
    fn mask128_iter() {
        let lanes: Vec<_> = QuadMask128(0b1001_0010).iter().collect();
        assert_eq!(lanes, [1, 4, 7]);
    }

    #[test]
    fn mask128_boolean_ops() {
        let lhs = QuadMask128(0b1100);
        let rhs = QuadMask128(0b1010);
        assert_eq!(lhs.and(rhs).raw(), 0b1000);
        assert_eq!(lhs.or(rhs).raw(), 0b1110);
        assert_eq!(lhs.xor(rhs).raw(), 0b0110);
    }

    #[test]
    fn reg32_tile128_roundtrip_all_n() {
        let regs = [reg_filled(QuadState::N); 4];
        assert_eq!(QuadTile128::from_regs(regs).to_regs(), regs);
    }

    #[test]
    fn reg32_tile128_roundtrip_all_f() {
        let regs = [reg_filled(QuadState::F); 4];
        assert_eq!(QuadTile128::from_regs(regs).to_regs(), regs);
    }

    #[test]
    fn reg32_tile128_roundtrip_all_t() {
        let regs = [reg_filled(QuadState::T); 4];
        assert_eq!(QuadTile128::from_regs(regs).to_regs(), regs);
    }

    #[test]
    fn reg32_tile128_roundtrip_all_s() {
        let regs = [reg_filled(QuadState::S); 4];
        assert_eq!(QuadTile128::from_regs(regs).to_regs(), regs);
    }

    #[test]
    fn reg32_tile128_roundtrip_mixed_pattern() {
        let mut regs = [QuadroReg32::new(); 4];
        for lane in 0..128usize {
            let reg_index = lane / 32;
            let reg_lane = lane % 32;
            regs[reg_index].set_unchecked(reg_lane, QuadState::ALL[lane % 4]);
        }
        assert_eq!(QuadTile128::from_regs(regs).to_regs(), regs);
    }

    #[test]
    fn delta32_all_4x4_transitions() {
        for prev_state in QuadState::ALL {
            for curr_state in QuadState::ALL {
                let mut prev = QuadroReg32::new();
                let mut curr = QuadroReg32::new();
                prev.set_unchecked(0, prev_state);
                curr.set_unchecked(0, curr_state);
                let delta = StateDelta32::from_regs(prev, curr);
                assert_eq!(
                    delta.entered_true.raw() & 1,
                    (!prev_state.true_plane() && curr_state.true_plane()) as u64
                );
                assert_eq!(
                    delta.left_true.raw() & 1,
                    (prev_state.true_plane() && !curr_state.true_plane()) as u64
                );
                assert_eq!(
                    delta.entered_false.raw() & 1,
                    (!prev_state.false_plane() && curr_state.false_plane()) as u64
                );
                assert_eq!(
                    delta.left_false.raw() & 1,
                    (prev_state.false_plane() && !curr_state.false_plane()) as u64
                );
                assert_eq!(
                    delta.changed.raw() & 1,
                    (prev_state.bits() != curr_state.bits()) as u64
                );
                assert_eq!(
                    delta.became_known.raw() & 1,
                    (!prev_state.is_known() && curr_state.is_known()) as u64
                );
                assert_eq!(
                    delta.became_unknown.raw() & 1,
                    (prev_state.is_known() && !curr_state.is_known()) as u64
                );
                assert_eq!(
                    delta.became_conflicted.raw() & 1,
                    (!prev_state.is_conflict() && curr_state.is_conflict()) as u64
                );
                assert_eq!(
                    delta.resolved_conflict.raw() & 1,
                    (prev_state.is_conflict() && !curr_state.is_conflict()) as u64
                );
            }
        }
    }

    #[test]
    fn delta32_changed_detects_any_plane_change() {
        let mut prev = QuadroReg32::new();
        let mut curr = QuadroReg32::new();
        prev.set_unchecked(0, QuadState::T);
        curr.set_unchecked(0, QuadState::S);
        let delta = StateDelta32::from_regs(prev, curr);
        assert_eq!(delta.changed.raw() & 1, 1);
    }

    #[test]
    fn delta32_no_msb_leakage() {
        let delta = StateDelta32::from_regs(reg_filled(QuadState::N), reg_filled(QuadState::S));
        assert_eq!(delta.changed.raw() & !0xFFFF_FFFF, 0);
    }

    #[test]
    fn delta32_became_known() {
        let prev = QuadroReg32::new();
        let mut curr = QuadroReg32::new();
        curr.set_unchecked(0, QuadState::T);
        let delta = StateDelta32::from_regs(prev, curr);
        assert_eq!(delta.became_known.raw() & 1, 1);
    }

    #[test]
    fn delta32_became_unknown() {
        let mut prev = QuadroReg32::new();
        prev.set_unchecked(0, QuadState::T);
        let curr = QuadroReg32::new();
        let delta = StateDelta32::from_regs(prev, curr);
        assert_eq!(delta.became_unknown.raw() & 1, 1);
    }

    #[test]
    fn delta32_became_conflicted() {
        let mut prev = QuadroReg32::new();
        let mut curr = QuadroReg32::new();
        prev.set_unchecked(0, QuadState::T);
        curr.set_unchecked(0, QuadState::S);
        let delta = StateDelta32::from_regs(prev, curr);
        assert_eq!(delta.became_conflicted.raw() & 1, 1);
    }

    #[test]
    fn delta32_resolved_conflict() {
        let mut prev = QuadroReg32::new();
        let mut curr = QuadroReg32::new();
        prev.set_unchecked(0, QuadState::S);
        curr.set_unchecked(0, QuadState::F);
        let delta = StateDelta32::from_regs(prev, curr);
        assert_eq!(delta.resolved_conflict.raw() & 1, 1);
    }

    #[test]
    fn delta128_all_4x4_transitions_per_lane() {
        for prev_state in QuadState::ALL {
            for curr_state in QuadState::ALL {
                let mut prev = QuadTile128::new();
                let mut curr = QuadTile128::new();
                prev.set_unchecked(7, prev_state);
                curr.set_unchecked(7, curr_state);
                let delta = StateDelta128::from_tiles(prev, curr);
                assert_eq!(
                    (delta.changed.raw() >> 7) & 1,
                    (prev_state.bits() != curr_state.bits()) as u128
                );
            }
        }
    }

    #[test]
    fn delta128_changed() {
        let delta = StateDelta128::from_tiles(tile_filled(QuadState::N), tile_filled(QuadState::T));
        assert_eq!(delta.changed.count(), 128);
    }

    #[test]
    fn delta128_conflict_transition() {
        let mut prev = QuadTile128::new();
        let mut curr = QuadTile128::new();
        prev.set_unchecked(0, QuadState::T);
        curr.set_unchecked(0, QuadState::S);
        let delta = StateDelta128::from_tiles(prev, curr);
        assert_eq!(delta.became_conflicted.raw() & 1, 1);
    }

    #[test]
    fn delta128_known_unknown_transition() {
        let mut prev = QuadTile128::new();
        prev.set_unchecked(0, QuadState::F);
        let curr = QuadTile128::new();
        let delta = StateDelta128::from_tiles(prev, curr);
        assert_eq!(delta.became_unknown.raw() & 1, 1);
    }

    #[test]
    fn bank_new_all_zero() {
        let bank = QuadroBank::<4>::new();
        assert!(bank.as_slice().iter().all(|reg| reg.raw() == 0));
    }

    #[test]
    fn bank_get_set() {
        let mut bank = QuadroBank::<2>::new();
        bank.set(1, reg_filled(QuadState::T)).unwrap();
        assert_eq!(bank.get(1), Some(reg_filled(QuadState::T)));
    }

    #[test]
    fn bank_join_matches_per_reg() {
        let mut lhs =
            QuadroBank::<2>::from_array([reg_filled(QuadState::T), reg_filled(QuadState::F)]);
        let rhs = QuadroBank::<2>::from_array([reg_filled(QuadState::F), reg_filled(QuadState::T)]);
        lhs.join_inplace(&rhs);
        assert_eq!(lhs.get(0), Some(reg_filled(QuadState::S)));
        assert_eq!(lhs.get(1), Some(reg_filled(QuadState::S)));
    }

    #[test]
    fn bank_meet_matches_per_reg() {
        let mut lhs =
            QuadroBank::<2>::from_array([reg_filled(QuadState::T), reg_filled(QuadState::S)]);
        let rhs = QuadroBank::<2>::from_array([reg_filled(QuadState::F), reg_filled(QuadState::T)]);
        lhs.meet_inplace(&rhs);
        assert_eq!(lhs.get(0), Some(reg_filled(QuadState::N)));
        assert_eq!(lhs.get(1), Some(reg_filled(QuadState::T)));
    }

    #[test]
    fn bank_inverse_matches_per_reg() {
        let mut bank = QuadroBank::<1>::from_array([reg_filled(QuadState::F)]);
        bank.inverse_inplace();
        assert_eq!(bank.get(0), Some(reg_filled(QuadState::T)));
    }

    #[test]
    fn tile_bank_join_matches_per_tile() {
        let mut lhs = QuadTileBank::<1>::from_array([tile_filled(QuadState::F)]);
        let rhs = QuadTileBank::<1>::from_array([tile_filled(QuadState::T)]);
        lhs.join_inplace(&rhs);
        assert_eq!(lhs.get(0), Some(tile_filled(QuadState::S)));
    }

    #[test]
    fn tile_bank_meet_matches_per_tile() {
        let mut lhs = QuadTileBank::<1>::from_array([tile_filled(QuadState::S)]);
        let rhs = QuadTileBank::<1>::from_array([tile_filled(QuadState::T)]);
        lhs.meet_inplace(&rhs);
        assert_eq!(lhs.get(0), Some(tile_filled(QuadState::T)));
    }

    #[test]
    fn tile_bank_inverse_matches_per_tile() {
        let mut bank = QuadTileBank::<1>::from_array([tile_filled(QuadState::F)]);
        bank.inverse_inplace();
        assert_eq!(bank.get(0), Some(tile_filled(QuadState::T)));
    }

    macro_rules! bank_tail_case {
        ($name:ident, $len:expr) => {
            #[test]
            fn $name() {
                let mut lhs = QuadroBank::<$len>::new();
                let rhs = QuadroBank::<$len>::from_array([reg_filled(QuadState::T); $len]);
                lhs.join_inplace(&rhs);
                assert!(lhs
                    .as_slice()
                    .iter()
                    .all(|reg| *reg == reg_filled(QuadState::T)));
                lhs.meet_inplace(&rhs);
                lhs.inverse_inplace();
                assert!(lhs
                    .as_slice()
                    .iter()
                    .all(|reg| *reg == reg_filled(QuadState::F)));
            }
        };
    }

    bank_tail_case!(bank_tail_len_0, 0);
    bank_tail_case!(bank_tail_len_1, 1);
    bank_tail_case!(bank_tail_len_2, 2);
    bank_tail_case!(bank_tail_len_3, 3);
    bank_tail_case!(bank_tail_len_4, 4);
    bank_tail_case!(bank_tail_len_5, 5);
    bank_tail_case!(bank_tail_len_7, 7);
    bank_tail_case!(bank_tail_len_8, 8);
    bank_tail_case!(bank_tail_len_15, 15);
    bank_tail_case!(bank_tail_len_16, 16);
    bank_tail_case!(bank_tail_len_31, 31);
    bank_tail_case!(bank_tail_len_32, 32);
    bank_tail_case!(bank_tail_len_33, 33);
    bank_tail_case!(bank_tail_len_37, 37);
    bank_tail_case!(bank_tail_len_64, 64);
    bank_tail_case!(bank_tail_len_127, 127);
    bank_tail_case!(bank_tail_len_128, 128);
    bank_tail_case!(bank_tail_len_129, 129);

    const RAW_SAMPLES: [u64; 7] = [
        0x0000_0000_0000_0000,
        0xFFFF_FFFF_FFFF_FFFF,
        0x5555_5555_5555_5555,
        0xAAAA_AAAA_AAAA_AAAA,
        0x0123_4567_89AB_CDEF,
        0xE4E4_E4E4_E4E4_E4E4,
        0xBADC_0FFE_DEAD_BEEF,
    ];

    #[test]
    fn map_not_swar_matches_scalar_samples() {
        for raw in RAW_SAMPLES {
            let reg = QuadroReg32::from_raw(raw);
            assert_eq!(reg.map_not_swar(), reg.map_not_scalar());
        }
    }

    #[test]
    fn map_xor_swar_matches_scalar_samples() {
        for a_raw in RAW_SAMPLES {
            for b_raw in RAW_SAMPLES {
                let a = QuadroReg32::from_raw(a_raw);
                let b = QuadroReg32::from_raw(b_raw);
                assert_eq!(a.map_xor_swar(b), a.map_xor_scalar(b));
            }
        }
    }

    #[test]
    fn map_not_swar_matches_scalar_repeated_states() {
        for state in QuadState::ALL {
            let reg = reg_filled(state);
            assert_eq!(reg.map_not_swar(), reg.map_not_scalar());
        }
    }

    #[test]
    fn map_xor_swar_matches_scalar_repeated_state_pairs() {
        for a_state in QuadState::ALL {
            for b_state in QuadState::ALL {
                let a = reg_filled(a_state);
                let b = reg_filled(b_state);
                assert_eq!(a.map_xor_swar(b), a.map_xor_scalar(b));
            }
        }
    }
    #[test]
    fn map_and_swar_matches_scalar_samples() {
        for a_raw in RAW_SAMPLES {
            for b_raw in RAW_SAMPLES {
                let a = QuadroReg32::from_raw(a_raw);
                let b = QuadroReg32::from_raw(b_raw);
                assert_eq!(a.map_and_swar(b), a.map_and_scalar(b));
            }
        }
    }

    #[test]
    fn map_or_swar_matches_scalar_samples() {
        for a_raw in RAW_SAMPLES {
            for b_raw in RAW_SAMPLES {
                let a = QuadroReg32::from_raw(a_raw);
                let b = QuadroReg32::from_raw(b_raw);
                assert_eq!(a.map_or_swar(b), a.map_or_scalar(b));
            }
        }
    }

    #[test]
    fn map_and_swar_matches_scalar_repeated_state_pairs() {
        for a_state in QuadState::ALL {
            for b_state in QuadState::ALL {
                let a = reg_filled(a_state);
                let b = reg_filled(b_state);
                assert_eq!(a.map_and_swar(b), a.map_and_scalar(b));
            }
        }
    }

    #[test]
    fn map_or_swar_matches_scalar_repeated_state_pairs() {
        for a_state in QuadState::ALL {
            for b_state in QuadState::ALL {
                let a = reg_filled(a_state);
                let b = reg_filled(b_state);
                assert_eq!(a.map_or_swar(b), a.map_or_scalar(b));
            }
        }
    }
}
