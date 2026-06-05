#![allow(clippy::new_without_default, clippy::explicit_auto_deref)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "alloc", feature = "std"))]
extern crate alloc;

#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::vec;
#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::vec::Vec;

use core::cmp::Ordering;

use semantic_core_quad::QuadState;
use semantic_core_runtime::{
    CoreAdmissionProfile, CoreProfileError, CoreTrap, FuelMeter, FunctionId, SymbolId,
};
use static_assertions::const_assert_eq;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreEnginePolicy {
    DeterministicReference,
    Auto,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fx(i32);

impl Fx {
    pub const FRACTION_BITS: i32 = 16;
    pub const SCALE: i32 = 1 << Self::FRACTION_BITS;

    pub const fn from_raw(raw: i32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> i32 {
        self.0
    }

    pub fn add_checked(self, other: Self) -> Result<Self, CoreTrap> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or(CoreTrap::IntegerOverflow)
    }

    pub fn sub_checked(self, other: Self) -> Result<Self, CoreTrap> {
        self.0
            .checked_sub(other.0)
            .map(Self)
            .ok_or(CoreTrap::IntegerOverflow)
    }

    pub fn mul_checked(self, other: Self) -> Result<Self, CoreTrap> {
        let wide = (self.0 as i64) * (other.0 as i64);
        let scaled = wide >> Self::FRACTION_BITS;
        if scaled < i32::MIN as i64 || scaled > i32::MAX as i64 {
            return Err(CoreTrap::IntegerOverflow);
        }
        Ok(Self(scaled as i32))
    }

    pub fn div_checked(self, other: Self) -> Result<Self, CoreTrap> {
        if other.0 == 0 {
            return Err(CoreTrap::DivisionByZero);
        }
        let wide = (self.0 as i64) << Self::FRACTION_BITS;
        let scaled = wide / other.0 as i64;
        if scaled < i32::MIN as i64 || scaled > i32::MAX as i64 {
            return Err(CoreTrap::IntegerOverflow);
        }
        Ok(Self(scaled as i32))
    }

    pub const fn cmp(self, other: Self) -> Ordering {
        if self.0 < other.0 {
            Ordering::Less
        } else if self.0 > other.0 {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TupleRef(pub u32);

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RecordRef(pub u32);

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AdtRef(pub u32);

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegId(pub u16);

impl RegId {
    pub const fn raw(self) -> u16 {
        self.0
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreValue {
    Unit,
    Quad(QuadState),
    Bool(bool),
    I32(i32),
    U32(u32),
    Fx(Fx),
    Tuple(TupleRef),
    Record(RecordRef),
    Adt(AdtRef),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CoreOpcode {
    Nop = 0,
    Trap = 1,
    Assert = 2,
    LoadUnit = 3,
    LoadQuad = 4,
    LoadBool = 5,
    LoadI32 = 6,
    LoadU32 = 7,
    LoadFx = 8,
    QNot = 9,
    QJoin = 10,
    QMeet = 11,
    QImpl = 12,
    QEq = 13,
    QNe = 14,
    QIsKnown = 15,
    QIsConflict = 16,
    QIsNull = 17,
    BoolNot = 18,
    BoolAnd = 19,
    BoolOr = 20,
    BoolEq = 21,
    BoolNe = 22,
    I32Add = 23,
    I32Sub = 24,
    I32Mul = 25,
    I32Div = 26,
    I32Eq = 27,
    I32Lt = 28,
    I32Le = 29,
    I32Gt = 30,
    I32Ge = 31,
    U32Add = 32,
    U32Sub = 33,
    U32Mul = 34,
    U32Div = 35,
    U32Eq = 36,
    U32Lt = 37,
    U32Le = 38,
    U32Gt = 39,
    U32Ge = 40,
    FxAdd = 41,
    FxSub = 42,
    FxMul = 43,
    FxDiv = 44,
    FxEq = 45,
    FxLt = 46,
    FxLe = 47,
    FxGt = 48,
    FxGe = 49,
    Move = 50,
    Jump = 51,
    JumpIfTrue = 52,
    JumpIfFalse = 53,
    Call = 54,
    Ret = 55,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame<const R: usize> {
    regs: [CoreValue; R],
    pc: usize,
    fuel_used: u64,
}

impl<const R: usize> Frame<R> {
    pub const fn new() -> Self {
        Self {
            regs: [CoreValue::Unit; R],
            pc: 0,
            fuel_used: 0,
        }
    }

    pub fn get(&self, reg: RegId) -> Result<CoreValue, CoreTrap> {
        self.regs
            .get(reg.0 as usize)
            .copied()
            .ok_or(CoreTrap::InvalidRegister)
    }

    pub fn set(&mut self, reg: RegId, value: CoreValue) -> Result<(), CoreTrap> {
        match self.regs.get_mut(reg.0 as usize) {
            Some(slot) => {
                *slot = value;
                Ok(())
            }
            None => Err(CoreTrap::InvalidRegister),
        }
    }

    pub const fn pc(&self) -> usize {
        self.pc
    }

    pub const fn fuel_used(&self) -> u64 {
        self.fuel_used
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "op", rename_all = "snake_case"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instr {
    Nop,
    Trap,
    Assert {
        cond: RegId,
    },
    LoadUnit {
        dst: RegId,
    },
    LoadQuad {
        dst: RegId,
        value: QuadState,
    },
    LoadBool {
        dst: RegId,
        value: bool,
    },
    LoadI32 {
        dst: RegId,
        value: i32,
    },
    LoadU32 {
        dst: RegId,
        value: u32,
    },
    LoadFx {
        dst: RegId,
        value: Fx,
    },
    QNot {
        dst: RegId,
        src: RegId,
    },
    QJoin {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    QMeet {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    QImpl {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    QEq {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    QNe {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    QIsKnown {
        dst: RegId,
        src: RegId,
    },
    QIsConflict {
        dst: RegId,
        src: RegId,
    },
    QIsNull {
        dst: RegId,
        src: RegId,
    },
    BoolNot {
        dst: RegId,
        src: RegId,
    },
    BoolAnd {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    BoolOr {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    BoolEq {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    BoolNe {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    I32Add {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    I32Sub {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    I32Mul {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    I32Div {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    I32Eq {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    I32Lt {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    I32Le {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    I32Gt {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    I32Ge {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    U32Add {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    U32Sub {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    U32Mul {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    U32Div {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    U32Eq {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    U32Lt {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    U32Le {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    U32Gt {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    U32Ge {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    FxAdd {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    FxSub {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    FxMul {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    FxDiv {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    FxEq {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    FxLt {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    FxLe {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    FxGt {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    FxGe {
        dst: RegId,
        lhs: RegId,
        rhs: RegId,
    },
    Move {
        dst: RegId,
        src: RegId,
    },
    Jump {
        target: u32,
    },
    JumpIfTrue {
        cond: RegId,
        target: u32,
    },
    JumpIfFalse {
        cond: RegId,
        target: u32,
    },
    Call {
        dst: RegId,
        function: FunctionId,
        arg_base: RegId,
        arg_count: u16,
    },
    Ret {
        src: RegId,
    },
}

const INSTR_SIZE_BYTES: usize = 12;
const_assert_eq!(core::mem::size_of::<Instr>(), INSTR_SIZE_BYTES);

impl Instr {
    pub const fn opcode(&self) -> CoreOpcode {
        match self {
            Self::Nop => CoreOpcode::Nop,
            Self::Trap => CoreOpcode::Trap,
            Self::Assert { .. } => CoreOpcode::Assert,
            Self::LoadUnit { .. } => CoreOpcode::LoadUnit,
            Self::LoadQuad { .. } => CoreOpcode::LoadQuad,
            Self::LoadBool { .. } => CoreOpcode::LoadBool,
            Self::LoadI32 { .. } => CoreOpcode::LoadI32,
            Self::LoadU32 { .. } => CoreOpcode::LoadU32,
            Self::LoadFx { .. } => CoreOpcode::LoadFx,
            Self::QNot { .. } => CoreOpcode::QNot,
            Self::QJoin { .. } => CoreOpcode::QJoin,
            Self::QMeet { .. } => CoreOpcode::QMeet,
            Self::QImpl { .. } => CoreOpcode::QImpl,
            Self::QEq { .. } => CoreOpcode::QEq,
            Self::QNe { .. } => CoreOpcode::QNe,
            Self::QIsKnown { .. } => CoreOpcode::QIsKnown,
            Self::QIsConflict { .. } => CoreOpcode::QIsConflict,
            Self::QIsNull { .. } => CoreOpcode::QIsNull,
            Self::BoolNot { .. } => CoreOpcode::BoolNot,
            Self::BoolAnd { .. } => CoreOpcode::BoolAnd,
            Self::BoolOr { .. } => CoreOpcode::BoolOr,
            Self::BoolEq { .. } => CoreOpcode::BoolEq,
            Self::BoolNe { .. } => CoreOpcode::BoolNe,
            Self::I32Add { .. } => CoreOpcode::I32Add,
            Self::I32Sub { .. } => CoreOpcode::I32Sub,
            Self::I32Mul { .. } => CoreOpcode::I32Mul,
            Self::I32Div { .. } => CoreOpcode::I32Div,
            Self::I32Eq { .. } => CoreOpcode::I32Eq,
            Self::I32Lt { .. } => CoreOpcode::I32Lt,
            Self::I32Le { .. } => CoreOpcode::I32Le,
            Self::I32Gt { .. } => CoreOpcode::I32Gt,
            Self::I32Ge { .. } => CoreOpcode::I32Ge,
            Self::U32Add { .. } => CoreOpcode::U32Add,
            Self::U32Sub { .. } => CoreOpcode::U32Sub,
            Self::U32Mul { .. } => CoreOpcode::U32Mul,
            Self::U32Div { .. } => CoreOpcode::U32Div,
            Self::U32Eq { .. } => CoreOpcode::U32Eq,
            Self::U32Lt { .. } => CoreOpcode::U32Lt,
            Self::U32Le { .. } => CoreOpcode::U32Le,
            Self::U32Gt { .. } => CoreOpcode::U32Gt,
            Self::U32Ge { .. } => CoreOpcode::U32Ge,
            Self::FxAdd { .. } => CoreOpcode::FxAdd,
            Self::FxSub { .. } => CoreOpcode::FxSub,
            Self::FxMul { .. } => CoreOpcode::FxMul,
            Self::FxDiv { .. } => CoreOpcode::FxDiv,
            Self::FxEq { .. } => CoreOpcode::FxEq,
            Self::FxLt { .. } => CoreOpcode::FxLt,
            Self::FxLe { .. } => CoreOpcode::FxLe,
            Self::FxGt { .. } => CoreOpcode::FxGt,
            Self::FxGe { .. } => CoreOpcode::FxGe,
            Self::Move { .. } => CoreOpcode::Move,
            Self::Jump { .. } => CoreOpcode::Jump,
            Self::JumpIfTrue { .. } => CoreOpcode::JumpIfTrue,
            Self::JumpIfFalse { .. } => CoreOpcode::JumpIfFalse,
            Self::Call { .. } => CoreOpcode::Call,
            Self::Ret { .. } => CoreOpcode::Ret,
        }
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreFunction {
    pub name_id: SymbolId,
    pub regs: u16,
    pub instrs: Vec<Instr>,
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreProgram {
    pub functions: Vec<CoreFunction>,
    pub entry: FunctionId,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreConfig {
    pub fuel: u64,
    pub max_call_depth: u16,
    engine_policy: CoreEnginePolicy,
    pub validate_before_execute: bool,
    pub profile: CoreAdmissionProfile,
}

impl Default for CoreConfig {
    fn default() -> Self {
        let profile = CoreAdmissionProfile::default();
        Self {
            fuel: profile.max_fuel,
            max_call_depth: profile.max_call_depth,
            engine_policy: CoreEnginePolicy::Auto,
            validate_before_execute: true,
            profile,
        }
    }
}

impl CoreConfig {
    pub fn engine_policy(&self) -> CoreEnginePolicy {
        self.engine_policy
    }

    pub fn with_engine_policy(mut self, policy: CoreEnginePolicy) -> Self {
        self.engine_policy = policy;
        self
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreStatus {
    Returned,
    Trapped(CoreTrap),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreResult {
    pub status: CoreStatus,
    pub return_value: CoreValue,
    pub fuel_used: u64,
}

impl CoreResult {
    pub const fn returned(return_value: CoreValue, fuel_used: u64) -> Self {
        Self {
            status: CoreStatus::Returned,
            return_value,
            fuel_used,
        }
    }

    pub const fn trapped(trap: CoreTrap, fuel_used: u64) -> Self {
        Self {
            status: CoreStatus::Trapped(trap),
            return_value: CoreValue::Unit,
            fuel_used,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreResultDigest(pub u64);

impl CoreResultDigest {
    pub fn from_result(result: &CoreResult) -> Self {
        const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
        const PRIME: u64 = 0x1000_0000_01b3;

        fn mix(mut acc: u64, value: u64) -> u64 {
            acc ^= value;
            acc = acc.wrapping_mul(PRIME);
            acc
        }

        fn encode_value(value: CoreValue) -> [u64; 3] {
            match value {
                CoreValue::Unit => [0, 0, 0],
                CoreValue::Quad(q) => [1, q.bits() as u64, 0],
                CoreValue::Bool(v) => [2, v as u64, 0],
                CoreValue::I32(v) => [3, v as u32 as u64, 0],
                CoreValue::U32(v) => [4, v as u64, 0],
                CoreValue::Fx(v) => [5, v.raw() as u32 as u64, 0],
                CoreValue::Tuple(v) => [6, v.0 as u64, 0],
                CoreValue::Record(v) => [7, v.0 as u64, 0],
                CoreValue::Adt(v) => [8, v.0 as u64, 0],
            }
        }

        let status = match result.status {
            CoreStatus::Returned => 0,
            CoreStatus::Trapped(trap) => 1 + trap.code() as u64,
        };
        let mut acc = mix(OFFSET, status);
        for word in encode_value(result.return_value) {
            acc = mix(acc, word);
        }
        acc = mix(acc, result.fuel_used);
        Self(acc)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreValidationError {
    Profile(CoreProfileError),
    EmptyProgram,
    InvalidEntry(FunctionId),
    TooManyFunctions {
        count: usize,
        max: u16,
    },
    TooManyRegisters {
        function: FunctionId,
        regs: u16,
        max: u16,
    },
    TooManyInstructions {
        function: FunctionId,
        instrs: usize,
        max: u32,
    },
    InvalidRegister {
        function: FunctionId,
        reg: RegId,
    },
    InvalidJump {
        function: FunctionId,
        target: u32,
    },
    InvalidCall {
        function: FunctionId,
        target: FunctionId,
    },
    MissingReturn {
        function: FunctionId,
    },
    InvalidArgWindow {
        function: FunctionId,
        base: RegId,
        count: u16,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreLoadError {
    UnsupportedFormat,
}

pub trait SemCodeSource {
    #[cfg(any(feature = "alloc", feature = "std"))]
    fn to_core_program(&self) -> Result<CoreProgram, CoreLoadError>;
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn load_core_program_from_bytes(_bytes: &[u8]) -> Result<CoreProgram, CoreLoadError> {
    Err(CoreLoadError::UnsupportedFormat)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn validate_program(
    program: &CoreProgram,
    profile: CoreAdmissionProfile,
) -> Result<(), CoreValidationError> {
    profile.validate().map_err(CoreValidationError::Profile)?;
    if program.functions.is_empty() {
        return Err(CoreValidationError::EmptyProgram);
    }
    if program.functions.len() > profile.max_functions as usize {
        return Err(CoreValidationError::TooManyFunctions {
            count: program.functions.len(),
            max: profile.max_functions,
        });
    }
    if program.entry.0 as usize >= program.functions.len() {
        return Err(CoreValidationError::InvalidEntry(program.entry));
    }

    for (idx, function) in program.functions.iter().enumerate() {
        let function_id = FunctionId(idx as u16);
        if function.regs > profile.max_registers {
            return Err(CoreValidationError::TooManyRegisters {
                function: function_id,
                regs: function.regs,
                max: profile.max_registers,
            });
        }
        if function.instrs.len() > profile.max_instrs_per_function as usize {
            return Err(CoreValidationError::TooManyInstructions {
                function: function_id,
                instrs: function.instrs.len(),
                max: profile.max_instrs_per_function,
            });
        }
        let mut has_return = false;
        for instr in &function.instrs {
            validate_instr(
                program,
                function_id,
                function.regs,
                function.instrs.len(),
                instr,
            )?;
            if matches!(instr, Instr::Ret { .. }) {
                has_return = true;
            }
        }
        if !has_return {
            return Err(CoreValidationError::MissingReturn {
                function: function_id,
            });
        }
    }

    Ok(())
}

#[cfg(any(feature = "alloc", feature = "std"))]
fn validate_instr(
    program: &CoreProgram,
    function: FunctionId,
    regs: u16,
    instr_len: usize,
    instr: &Instr,
) -> Result<(), CoreValidationError> {
    let check_reg = |reg: RegId| -> Result<(), CoreValidationError> {
        if reg.0 < regs {
            Ok(())
        } else {
            Err(CoreValidationError::InvalidRegister { function, reg })
        }
    };
    let check_jump = |target: u32| -> Result<(), CoreValidationError> {
        if (target as usize) < instr_len {
            Ok(())
        } else {
            Err(CoreValidationError::InvalidJump { function, target })
        }
    };
    match instr {
        Instr::Nop | Instr::Trap => {}
        Instr::Assert { cond } => check_reg(*cond)?,
        Instr::LoadUnit { dst }
        | Instr::LoadQuad { dst, .. }
        | Instr::LoadBool { dst, .. }
        | Instr::LoadI32 { dst, .. }
        | Instr::LoadU32 { dst, .. }
        | Instr::LoadFx { dst, .. } => check_reg(*dst)?,
        Instr::QNot { dst, src }
        | Instr::QIsKnown { dst, src }
        | Instr::QIsConflict { dst, src }
        | Instr::QIsNull { dst, src }
        | Instr::BoolNot { dst, src }
        | Instr::Move { dst, src } => {
            check_reg(*dst)?;
            check_reg(*src)?;
        }
        Instr::QJoin { dst, lhs, rhs }
        | Instr::QMeet { dst, lhs, rhs }
        | Instr::QImpl { dst, lhs, rhs }
        | Instr::QEq { dst, lhs, rhs }
        | Instr::QNe { dst, lhs, rhs }
        | Instr::BoolAnd { dst, lhs, rhs }
        | Instr::BoolOr { dst, lhs, rhs }
        | Instr::BoolEq { dst, lhs, rhs }
        | Instr::BoolNe { dst, lhs, rhs }
        | Instr::I32Add { dst, lhs, rhs }
        | Instr::I32Sub { dst, lhs, rhs }
        | Instr::I32Mul { dst, lhs, rhs }
        | Instr::I32Div { dst, lhs, rhs }
        | Instr::I32Eq { dst, lhs, rhs }
        | Instr::I32Lt { dst, lhs, rhs }
        | Instr::I32Le { dst, lhs, rhs }
        | Instr::I32Gt { dst, lhs, rhs }
        | Instr::I32Ge { dst, lhs, rhs }
        | Instr::U32Add { dst, lhs, rhs }
        | Instr::U32Sub { dst, lhs, rhs }
        | Instr::U32Mul { dst, lhs, rhs }
        | Instr::U32Div { dst, lhs, rhs }
        | Instr::U32Eq { dst, lhs, rhs }
        | Instr::U32Lt { dst, lhs, rhs }
        | Instr::U32Le { dst, lhs, rhs }
        | Instr::U32Gt { dst, lhs, rhs }
        | Instr::U32Ge { dst, lhs, rhs }
        | Instr::FxAdd { dst, lhs, rhs }
        | Instr::FxSub { dst, lhs, rhs }
        | Instr::FxMul { dst, lhs, rhs }
        | Instr::FxDiv { dst, lhs, rhs }
        | Instr::FxEq { dst, lhs, rhs }
        | Instr::FxLt { dst, lhs, rhs }
        | Instr::FxLe { dst, lhs, rhs }
        | Instr::FxGt { dst, lhs, rhs }
        | Instr::FxGe { dst, lhs, rhs } => {
            check_reg(*dst)?;
            check_reg(*lhs)?;
            check_reg(*rhs)?;
        }
        Instr::Jump { target } => check_jump(*target)?,
        Instr::JumpIfTrue { cond, target } | Instr::JumpIfFalse { cond, target } => {
            check_reg(*cond)?;
            check_jump(*target)?;
        }
        Instr::Call {
            dst,
            function: target,
            arg_base,
            arg_count,
        } => {
            check_reg(*dst)?;
            if target.0 as usize >= program.functions.len() {
                return Err(CoreValidationError::InvalidCall {
                    function,
                    target: *target,
                });
            }
            if (*arg_base).0 as usize + *arg_count as usize > regs as usize {
                return Err(CoreValidationError::InvalidArgWindow {
                    function,
                    base: *arg_base,
                    count: *arg_count,
                });
            }
            let callee = &program.functions[target.0 as usize];
            if *arg_count > callee.regs {
                return Err(CoreValidationError::InvalidArgWindow {
                    function,
                    base: *arg_base,
                    count: *arg_count,
                });
            }
        }
        Instr::Ret { src } => check_reg(*src)?,
    }
    Ok(())
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone)]
struct ExecFrame {
    function: FunctionId,
    regs: Vec<CoreValue>,
    pc: usize,
    return_dst: Option<RegId>,
}

#[derive(Debug, Clone, Copy)]
pub struct CoreExecutor {
    config: CoreConfig,
}

impl CoreExecutor {
    pub const fn new(config: CoreConfig) -> Self {
        Self { config }
    }

    pub const fn config(&self) -> CoreConfig {
        self.config
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn execute(&self, program: &CoreProgram) -> Result<CoreResult, CoreTrap> {
        let result = self.execute_outcome(program);
        match result.status {
            CoreStatus::Returned => Ok(result),
            CoreStatus::Trapped(trap) => Err(trap),
        }
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn execute_outcome(&self, program: &CoreProgram) -> CoreResult {
        if program.entry.0 as usize >= program.functions.len() {
            return CoreResult::trapped(CoreTrap::InvalidFunction, 0);
        }
        let entry = &program.functions[program.entry.0 as usize];
        let mut fuel = FuelMeter::new(self.config.fuel);
        let mut frames = vec![ExecFrame {
            function: program.entry,
            regs: vec![CoreValue::Unit; entry.regs as usize],
            pc: 0,
            return_dst: None,
        }];

        loop {
            let frame_index = frames.len().saturating_sub(1);
            let (function_id, pc) = match frames.last() {
                Some(frame) => (frame.function, frame.pc),
                None => {
                    return CoreResult::returned(
                        CoreValue::Unit,
                        self.config.fuel - fuel.remaining(),
                    )
                }
            };
            let function = &program.functions[function_id.0 as usize];
            let Some(instr) = function.instrs.get(pc) else {
                return CoreResult::trapped(
                    CoreTrap::InvalidPc,
                    self.config.fuel - fuel.remaining(),
                );
            };
            if fuel.consume(1).is_err() {
                return CoreResult::trapped(
                    CoreTrap::FuelExceeded,
                    self.config.fuel - fuel.remaining(),
                );
            }
            match self.step(program, &mut frames, frame_index, instr) {
                Ok(Some(value)) => {
                    return CoreResult::returned(value, self.config.fuel - fuel.remaining())
                }
                Ok(None) => {}
                Err(trap) => return CoreResult::trapped(trap, self.config.fuel - fuel.remaining()),
            }
        }
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    fn step(
        &self,
        program: &CoreProgram,
        frames: &mut Vec<ExecFrame>,
        frame_index: usize,
        instr: &Instr,
    ) -> Result<Option<CoreValue>, CoreTrap> {
        let function_id = frames[frame_index].function;
        let function = &program.functions[function_id.0 as usize];

        let read = |frames: &Vec<ExecFrame>,
                    frame_index: usize,
                    reg: RegId|
         -> Result<CoreValue, CoreTrap> {
            frames[frame_index]
                .regs
                .get(reg.0 as usize)
                .copied()
                .ok_or(CoreTrap::InvalidRegister)
        };
        let write = |frames: &mut Vec<ExecFrame>,
                     frame_index: usize,
                     reg: RegId,
                     value: CoreValue|
         -> Result<(), CoreTrap> {
            match frames[frame_index].regs.get_mut(reg.0 as usize) {
                Some(slot) => {
                    *slot = value;
                    Ok(())
                }
                None => Err(CoreTrap::InvalidRegister),
            }
        };

        let mut advance_pc = true;

        match instr {
            Instr::Nop => {}
            Instr::Trap => return Err(CoreTrap::ExplicitTrap),
            Instr::Assert { cond } => {
                if !as_bool(read(frames, frame_index, *cond)?)? {
                    return Err(CoreTrap::AssertFailed);
                }
            }
            Instr::LoadUnit { dst } => write(frames, frame_index, *dst, CoreValue::Unit)?,
            Instr::LoadQuad { dst, value } => {
                write(frames, frame_index, *dst, CoreValue::Quad(*value))?
            }
            Instr::LoadBool { dst, value } => {
                write(frames, frame_index, *dst, CoreValue::Bool(*value))?
            }
            Instr::LoadI32 { dst, value } => {
                write(frames, frame_index, *dst, CoreValue::I32(*value))?
            }
            Instr::LoadU32 { dst, value } => {
                write(frames, frame_index, *dst, CoreValue::U32(*value))?
            }
            Instr::LoadFx { dst, value } => {
                write(frames, frame_index, *dst, CoreValue::Fx(*value))?
            }
            Instr::QNot { dst, src } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Quad(as_quad(read(frames, frame_index, *src)?)?.inverse()),
            )?,
            Instr::QJoin { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Quad(
                    as_quad(read(frames, frame_index, *lhs)?)?.join(as_quad(read(
                        frames,
                        frame_index,
                        *rhs,
                    )?)?),
                ),
            )?,
            Instr::QMeet { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Quad(
                    as_quad(read(frames, frame_index, *lhs)?)?.meet(as_quad(read(
                        frames,
                        frame_index,
                        *rhs,
                    )?)?),
                ),
            )?,
            Instr::QImpl { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Quad(
                    as_quad(read(frames, frame_index, *lhs)?)?
                        .inverse()
                        .join(as_quad(read(frames, frame_index, *rhs)?)?),
                ),
            )?,
            Instr::QEq { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(
                    as_quad(read(frames, frame_index, *lhs)?)?
                        == as_quad(read(frames, frame_index, *rhs)?)?,
                ),
            )?,
            Instr::QNe { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(
                    as_quad(read(frames, frame_index, *lhs)?)?
                        != as_quad(read(frames, frame_index, *rhs)?)?,
                ),
            )?,
            Instr::QIsKnown { dst, src } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(as_quad(read(frames, frame_index, *src)?)?.is_known()),
            )?,
            Instr::QIsConflict { dst, src } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(as_quad(read(frames, frame_index, *src)?)?.is_conflict()),
            )?,
            Instr::QIsNull { dst, src } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(as_quad(read(frames, frame_index, *src)?)?.is_null()),
            )?,
            Instr::BoolNot { dst, src } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(!as_bool(read(frames, frame_index, *src)?)?),
            )?,
            Instr::BoolAnd { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(
                    as_bool(read(frames, frame_index, *lhs)?)?
                        && as_bool(read(frames, frame_index, *rhs)?)?,
                ),
            )?,
            Instr::BoolOr { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(
                    as_bool(read(frames, frame_index, *lhs)?)?
                        || as_bool(read(frames, frame_index, *rhs)?)?,
                ),
            )?,
            Instr::BoolEq { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(
                    as_bool(read(frames, frame_index, *lhs)?)?
                        == as_bool(read(frames, frame_index, *rhs)?)?,
                ),
            )?,
            Instr::BoolNe { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(
                    as_bool(read(frames, frame_index, *lhs)?)?
                        != as_bool(read(frames, frame_index, *rhs)?)?,
                ),
            )?,
            Instr::I32Add { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::I32(
                    as_i32(read(frames, frame_index, *lhs)?)?
                        .checked_add(as_i32(read(frames, frame_index, *rhs)?)?)
                        .ok_or(CoreTrap::IntegerOverflow)?,
                ),
            )?,
            Instr::I32Sub { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::I32(
                    as_i32(read(frames, frame_index, *lhs)?)?
                        .checked_sub(as_i32(read(frames, frame_index, *rhs)?)?)
                        .ok_or(CoreTrap::IntegerOverflow)?,
                ),
            )?,
            Instr::I32Mul { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::I32(
                    as_i32(read(frames, frame_index, *lhs)?)?
                        .checked_mul(as_i32(read(frames, frame_index, *rhs)?)?)
                        .ok_or(CoreTrap::IntegerOverflow)?,
                ),
            )?,
            Instr::I32Div { dst, lhs, rhs } => {
                let right = as_i32(read(frames, frame_index, *rhs)?)?;
                if right == 0 {
                    return Err(CoreTrap::DivisionByZero);
                }
                let left = as_i32(read(frames, frame_index, *lhs)?)?;
                write(
                    frames,
                    frame_index,
                    *dst,
                    CoreValue::I32(left.checked_div(right).ok_or(CoreTrap::IntegerOverflow)?),
                )?
            }
            Instr::I32Eq { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(
                    as_i32(read(frames, frame_index, *lhs)?)?
                        == as_i32(read(frames, frame_index, *rhs)?)?,
                ),
            )?,
            Instr::I32Lt { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(
                    as_i32(read(frames, frame_index, *lhs)?)?
                        < as_i32(read(frames, frame_index, *rhs)?)?,
                ),
            )?,
            Instr::I32Le { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(
                    as_i32(read(frames, frame_index, *lhs)?)?
                        <= as_i32(read(frames, frame_index, *rhs)?)?,
                ),
            )?,
            Instr::I32Gt { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(
                    as_i32(read(frames, frame_index, *lhs)?)?
                        > as_i32(read(frames, frame_index, *rhs)?)?,
                ),
            )?,
            Instr::I32Ge { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(
                    as_i32(read(frames, frame_index, *lhs)?)?
                        >= as_i32(read(frames, frame_index, *rhs)?)?,
                ),
            )?,
            Instr::U32Add { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::U32(
                    as_u32(read(frames, frame_index, *lhs)?)?
                        .checked_add(as_u32(read(frames, frame_index, *rhs)?)?)
                        .ok_or(CoreTrap::IntegerOverflow)?,
                ),
            )?,
            Instr::U32Sub { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::U32(
                    as_u32(read(frames, frame_index, *lhs)?)?
                        .checked_sub(as_u32(read(frames, frame_index, *rhs)?)?)
                        .ok_or(CoreTrap::IntegerOverflow)?,
                ),
            )?,
            Instr::U32Mul { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::U32(
                    as_u32(read(frames, frame_index, *lhs)?)?
                        .checked_mul(as_u32(read(frames, frame_index, *rhs)?)?)
                        .ok_or(CoreTrap::IntegerOverflow)?,
                ),
            )?,
            Instr::U32Div { dst, lhs, rhs } => {
                let right = as_u32(read(frames, frame_index, *rhs)?)?;
                if right == 0 {
                    return Err(CoreTrap::DivisionByZero);
                }
                let left = as_u32(read(frames, frame_index, *lhs)?)?;
                write(
                    frames,
                    frame_index,
                    *dst,
                    CoreValue::U32(left.checked_div(right).ok_or(CoreTrap::IntegerOverflow)?),
                )?
            }
            Instr::U32Eq { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(
                    as_u32(read(frames, frame_index, *lhs)?)?
                        == as_u32(read(frames, frame_index, *rhs)?)?,
                ),
            )?,
            Instr::U32Lt { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(
                    as_u32(read(frames, frame_index, *lhs)?)?
                        < as_u32(read(frames, frame_index, *rhs)?)?,
                ),
            )?,
            Instr::U32Le { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(
                    as_u32(read(frames, frame_index, *lhs)?)?
                        <= as_u32(read(frames, frame_index, *rhs)?)?,
                ),
            )?,
            Instr::U32Gt { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(
                    as_u32(read(frames, frame_index, *lhs)?)?
                        > as_u32(read(frames, frame_index, *rhs)?)?,
                ),
            )?,
            Instr::U32Ge { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(
                    as_u32(read(frames, frame_index, *lhs)?)?
                        >= as_u32(read(frames, frame_index, *rhs)?)?,
                ),
            )?,
            Instr::FxAdd { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Fx(
                    as_fx(read(frames, frame_index, *lhs)?)?.add_checked(as_fx(read(
                        frames,
                        frame_index,
                        *rhs,
                    )?)?)?,
                ),
            )?,
            Instr::FxSub { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Fx(
                    as_fx(read(frames, frame_index, *lhs)?)?.sub_checked(as_fx(read(
                        frames,
                        frame_index,
                        *rhs,
                    )?)?)?,
                ),
            )?,
            Instr::FxMul { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Fx(
                    as_fx(read(frames, frame_index, *lhs)?)?.mul_checked(as_fx(read(
                        frames,
                        frame_index,
                        *rhs,
                    )?)?)?,
                ),
            )?,
            Instr::FxDiv { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Fx(
                    as_fx(read(frames, frame_index, *lhs)?)?.div_checked(as_fx(read(
                        frames,
                        frame_index,
                        *rhs,
                    )?)?)?,
                ),
            )?,
            Instr::FxEq { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(
                    as_fx(read(frames, frame_index, *lhs)?)?.cmp(as_fx(read(
                        frames,
                        frame_index,
                        *rhs,
                    )?)?)
                        == Ordering::Equal,
                ),
            )?,
            Instr::FxLt { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(
                    as_fx(read(frames, frame_index, *lhs)?)?.cmp(as_fx(read(
                        frames,
                        frame_index,
                        *rhs,
                    )?)?)
                        == Ordering::Less,
                ),
            )?,
            Instr::FxLe { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(matches!(
                    as_fx(read(frames, frame_index, *lhs)?)?.cmp(as_fx(read(
                        frames,
                        frame_index,
                        *rhs
                    )?)?),
                    Ordering::Less | Ordering::Equal
                )),
            )?,
            Instr::FxGt { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(
                    as_fx(read(frames, frame_index, *lhs)?)?.cmp(as_fx(read(
                        frames,
                        frame_index,
                        *rhs,
                    )?)?)
                        == Ordering::Greater,
                ),
            )?,
            Instr::FxGe { dst, lhs, rhs } => write(
                frames,
                frame_index,
                *dst,
                CoreValue::Bool(matches!(
                    as_fx(read(frames, frame_index, *lhs)?)?.cmp(as_fx(read(
                        frames,
                        frame_index,
                        *rhs
                    )?)?),
                    Ordering::Greater | Ordering::Equal
                )),
            )?,
            Instr::Move { dst, src } => {
                write(frames, frame_index, *dst, read(frames, frame_index, *src)?)?
            }
            Instr::Jump { target } => {
                frames[frame_index].pc = *target as usize;
                advance_pc = false;
            }
            Instr::JumpIfTrue { cond, target } => {
                if as_bool(read(frames, frame_index, *cond)?)? {
                    frames[frame_index].pc = *target as usize;
                    advance_pc = false;
                }
            }
            Instr::JumpIfFalse { cond, target } => {
                if !as_bool(read(frames, frame_index, *cond)?)? {
                    frames[frame_index].pc = *target as usize;
                    advance_pc = false;
                }
            }
            Instr::Call {
                dst,
                function: callee_id,
                arg_base,
                arg_count,
            } => {
                if frames.len() >= self.config.max_call_depth as usize {
                    return Err(CoreTrap::CallDepthExceeded);
                }
                let Some(callee) = program.functions.get(callee_id.0 as usize) else {
                    return Err(CoreTrap::InvalidFunction);
                };
                let mut regs = vec![CoreValue::Unit; callee.regs as usize];
                for offset in 0..*arg_count as usize {
                    let value = read(frames, frame_index, RegId(arg_base.0 + offset as u16))?;
                    let Some(slot) = regs.get_mut(offset) else {
                        return Err(CoreTrap::InvalidRegister);
                    };
                    *slot = value;
                }
                frames[frame_index].pc = function
                    .instrs
                    .get(frames[frame_index].pc)
                    .map(|_| frames[frame_index].pc + 1)
                    .unwrap_or(frames[frame_index].pc);
                frames.push(ExecFrame {
                    function: *callee_id,
                    regs,
                    pc: 0,
                    return_dst: Some(*dst),
                });
                advance_pc = false;
            }
            Instr::Ret { src } => {
                let value = read(frames, frame_index, *src)?;
                let return_dst = frames[frame_index].return_dst;
                frames.pop();
                if let Some(parent) = frames.last_mut() {
                    let dst = return_dst.ok_or(CoreTrap::InvalidRegister)?;
                    match parent.regs.get_mut(dst.0 as usize) {
                        Some(slot) => *slot = value,
                        None => return Err(CoreTrap::InvalidRegister),
                    }
                } else {
                    return Ok(Some(value));
                }
                advance_pc = false;
            }
        }

        if advance_pc {
            frames[frame_index].pc += 1;
        }

        Ok(None)
    }
}

fn as_quad(value: CoreValue) -> Result<QuadState, CoreTrap> {
    match value {
        CoreValue::Quad(v) => Ok(v),
        _ => Err(CoreTrap::TypeMismatch),
    }
}

fn as_bool(value: CoreValue) -> Result<bool, CoreTrap> {
    match value {
        CoreValue::Bool(v) => Ok(v),
        _ => Err(CoreTrap::TypeMismatch),
    }
}

fn as_i32(value: CoreValue) -> Result<i32, CoreTrap> {
    match value {
        CoreValue::I32(v) => Ok(v),
        _ => Err(CoreTrap::TypeMismatch),
    }
}

fn as_u32(value: CoreValue) -> Result<u32, CoreTrap> {
    match value {
        CoreValue::U32(v) => Ok(v),
        _ => Err(CoreTrap::TypeMismatch),
    }
}

fn as_fx(value: CoreValue) -> Result<Fx, CoreTrap> {
    match value {
        CoreValue::Fx(v) => Ok(v),
        _ => Err(CoreTrap::TypeMismatch),
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[allow(dead_code)]
pub(crate) struct CoreProgramBuilder {
    functions: Vec<CoreFunction>,
    entry: FunctionId,
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[allow(dead_code)]
impl CoreProgramBuilder {
    pub(crate) fn new(entry: FunctionId) -> Self {
        Self {
            functions: Vec::new(),
            entry,
        }
    }

    pub(crate) fn push_function(
        mut self,
        name_id: SymbolId,
        regs: u16,
        instrs: Vec<Instr>,
    ) -> Self {
        self.functions.push(CoreFunction {
            name_id,
            regs,
            instrs,
        });
        self
    }

    pub(crate) fn build(self) -> Result<CoreProgram, CoreValidationError> {
        let program = CoreProgram {
            functions: self.functions,
            entry: self.entry,
        };
        validate_program(&program, CoreAdmissionProfile::default())?;
        Ok(program)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    fn run(program: &CoreProgram, config: CoreConfig) -> CoreResult {
        CoreExecutor::new(config).execute_outcome(program)
    }

    #[test]
    fn core_value_size_documented() {
        assert!(size_of::<CoreValue>() <= 16);
    }

    #[test]
    fn fx_add_sub_mul_div_checked() {
        let two = Fx::from_raw(2 << 16);
        let one = Fx::from_raw(1 << 16);
        let half = Fx::from_raw(1 << 15);
        assert_eq!(two.add_checked(one).unwrap(), Fx::from_raw(3 << 16));
        assert_eq!(two.sub_checked(one).unwrap(), Fx::from_raw(1 << 16));
        assert_eq!(two.mul_checked(half).unwrap(), Fx::from_raw(1 << 16));
        assert_eq!(two.div_checked(two).unwrap(), Fx::from_raw(1 << 16));
    }

    #[test]
    fn value_equality_for_primitives() {
        assert_eq!(CoreValue::Bool(true), CoreValue::Bool(true));
        assert_eq!(CoreValue::Quad(QuadState::S), CoreValue::Quad(QuadState::S));
        assert_ne!(CoreValue::I32(1), CoreValue::I32(2));
    }

    #[test]
    fn fx_div_by_zero_traps() {
        assert_eq!(
            Fx::from_raw(1 << 16).div_checked(Fx::from_raw(0)),
            Err(CoreTrap::DivisionByZero)
        );
    }

    #[test]
    fn fx_overflow_traps() {
        assert_eq!(
            Fx::from_raw(i32::MAX).add_checked(Fx::from_raw(1)),
            Err(CoreTrap::IntegerOverflow)
        );
    }

    #[test]
    fn opcode_discriminants_are_stable() {
        assert_eq!(CoreOpcode::Nop as u8, 0);
        assert_eq!(CoreOpcode::LoadQuad as u8, 4);
        assert_eq!(CoreOpcode::Call as u8, 54);
        assert_eq!(CoreOpcode::Ret as u8, 55);
    }

    #[test]
    fn opcode_debug_names_are_deterministic() {
        assert_eq!(format!("{:?}", CoreOpcode::QImpl), "QImpl");
    }

    #[test]
    fn opcode_has_no_private_mode_names() {
        let text = format!("{:?}", CoreOpcode::QImpl).to_ascii_lowercase();
        for word in ["tesseract", "andromeda", "hidden", "private"] {
            assert!(!text.contains(word));
        }
    }

    #[test]
    fn instruction_format_supports_all_core_ops() {
        let instrs = vec![
            Instr::Nop,
            Instr::Trap,
            Instr::Assert { cond: RegId(0) },
            Instr::LoadUnit { dst: RegId(0) },
            Instr::LoadQuad {
                dst: RegId(0),
                value: QuadState::N,
            },
            Instr::LoadBool {
                dst: RegId(0),
                value: false,
            },
            Instr::LoadI32 {
                dst: RegId(0),
                value: 0,
            },
            Instr::LoadU32 {
                dst: RegId(0),
                value: 0,
            },
            Instr::LoadFx {
                dst: RegId(0),
                value: Fx::from_raw(0),
            },
            Instr::QNot {
                dst: RegId(0),
                src: RegId(1),
            },
            Instr::QJoin {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::QMeet {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::QImpl {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::QEq {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::QNe {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::QIsKnown {
                dst: RegId(0),
                src: RegId(1),
            },
            Instr::QIsConflict {
                dst: RegId(0),
                src: RegId(1),
            },
            Instr::QIsNull {
                dst: RegId(0),
                src: RegId(1),
            },
            Instr::BoolNot {
                dst: RegId(0),
                src: RegId(1),
            },
            Instr::BoolAnd {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::BoolOr {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::BoolEq {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::BoolNe {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::I32Add {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::I32Sub {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::I32Mul {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::I32Div {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::I32Eq {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::I32Lt {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::I32Le {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::I32Gt {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::I32Ge {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::U32Add {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::U32Sub {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::U32Mul {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::U32Div {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::U32Eq {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::U32Lt {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::U32Le {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::U32Gt {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::U32Ge {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::FxAdd {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::FxSub {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::FxMul {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::FxDiv {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::FxEq {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::FxLt {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::FxLe {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::FxGt {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::FxGe {
                dst: RegId(0),
                lhs: RegId(1),
                rhs: RegId(2),
            },
            Instr::Move {
                dst: RegId(0),
                src: RegId(1),
            },
            Instr::Jump { target: 0 },
            Instr::JumpIfTrue {
                cond: RegId(0),
                target: 0,
            },
            Instr::JumpIfFalse {
                cond: RegId(0),
                target: 0,
            },
            Instr::Call {
                dst: RegId(0),
                function: FunctionId(0),
                arg_base: RegId(1),
                arg_count: 0,
            },
            Instr::Ret { src: RegId(0) },
        ];
        let actual: Vec<_> = instrs
            .into_iter()
            .map(|instr| instr.opcode() as u8)
            .collect();
        let expected: Vec<_> = (CoreOpcode::Nop as u8..=CoreOpcode::Ret as u8).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn instr_size_is_frozen() {
        assert_eq!(size_of::<Instr>(), INSTR_SIZE_BYTES);
    }

    #[test]
    fn frame_new_unit_initialized() {
        let frame = Frame::<2>::new();
        assert_eq!(frame.get(RegId(0)).unwrap(), CoreValue::Unit);
        assert_eq!(frame.pc(), 0);
    }

    #[test]
    fn frame_get_set_register() {
        let mut frame = Frame::<2>::new();
        frame.set(RegId(1), CoreValue::Bool(true)).unwrap();
        assert_eq!(frame.get(RegId(1)).unwrap(), CoreValue::Bool(true));
    }

    #[test]
    fn frame_out_of_bounds_trap() {
        let frame = Frame::<1>::new();
        assert_eq!(frame.get(RegId(2)), Err(CoreTrap::InvalidRegister));
    }

    #[test]
    fn program_entry_validated() {
        let program = CoreProgram {
            functions: vec![CoreFunction {
                name_id: SymbolId(0),
                regs: 1,
                instrs: vec![Instr::Ret { src: RegId(0) }],
            }],
            entry: FunctionId(1),
        };
        assert_eq!(
            validate_program(&program, CoreAdmissionProfile::default()),
            Err(CoreValidationError::InvalidEntry(FunctionId(1)))
        );
    }

    #[test]
    fn function_register_budget_validated() {
        let profile = CoreAdmissionProfile {
            max_registers: 1,
            ..CoreAdmissionProfile::default()
        };
        let program = CoreProgram {
            functions: vec![CoreFunction {
                name_id: SymbolId(0),
                regs: 2,
                instrs: vec![Instr::Ret { src: RegId(0) }],
            }],
            entry: FunctionId(0),
        };
        assert_eq!(
            validate_program(&program, profile),
            Err(CoreValidationError::TooManyRegisters {
                function: FunctionId(0),
                regs: 2,
                max: 1,
            })
        );
    }

    #[test]
    fn empty_program_rejected() {
        let program = CoreProgram {
            functions: vec![],
            entry: FunctionId(0),
        };
        assert_eq!(
            validate_program(&program, CoreAdmissionProfile::default()),
            Err(CoreValidationError::EmptyProgram)
        );
    }

    #[test]
    fn jump_out_of_bounds_rejected() {
        let program = CoreProgram {
            functions: vec![CoreFunction {
                name_id: SymbolId(0),
                regs: 1,
                instrs: vec![Instr::Jump { target: 3 }, Instr::Ret { src: RegId(0) }],
            }],
            entry: FunctionId(0),
        };
        assert_eq!(
            validate_program(&program, CoreAdmissionProfile::default()),
            Err(CoreValidationError::InvalidJump {
                function: FunctionId(0),
                target: 3,
            })
        );
    }

    #[test]
    fn invalid_register_rejected() {
        let program = CoreProgram {
            functions: vec![CoreFunction {
                name_id: SymbolId(0),
                regs: 1,
                instrs: vec![
                    Instr::Move {
                        dst: RegId(0),
                        src: RegId(1),
                    },
                    Instr::Ret { src: RegId(0) },
                ],
            }],
            entry: FunctionId(0),
        };
        assert_eq!(
            validate_program(&program, CoreAdmissionProfile::default()),
            Err(CoreValidationError::InvalidRegister {
                function: FunctionId(0),
                reg: RegId(1),
            })
        );
    }

    #[test]
    fn invalid_call_rejected() {
        let program = CoreProgram {
            functions: vec![CoreFunction {
                name_id: SymbolId(0),
                regs: 1,
                instrs: vec![
                    Instr::Call {
                        dst: RegId(0),
                        function: FunctionId(1),
                        arg_base: RegId(0),
                        arg_count: 0,
                    },
                    Instr::Ret { src: RegId(0) },
                ],
            }],
            entry: FunctionId(0),
        };
        assert_eq!(
            validate_program(&program, CoreAdmissionProfile::default()),
            Err(CoreValidationError::InvalidCall {
                function: FunctionId(0),
                target: FunctionId(1),
            })
        );
    }

    #[test]
    fn builder_creates_simple_program() {
        let program = CoreProgramBuilder::new(FunctionId(0))
            .push_function(SymbolId(0), 1, vec![Instr::Ret { src: RegId(0) }])
            .build()
            .unwrap();
        assert_eq!(program.entry, FunctionId(0));
        assert_eq!(program.functions.len(), 1);
    }

    #[test]
    fn builder_validates_registers() {
        let err = CoreProgramBuilder::new(FunctionId(0))
            .push_function(
                SymbolId(0),
                1,
                vec![
                    Instr::Move {
                        dst: RegId(0),
                        src: RegId(3),
                    },
                    Instr::Ret { src: RegId(0) },
                ],
            )
            .build()
            .unwrap_err();
        assert!(matches!(err, CoreValidationError::InvalidRegister { .. }));
    }

    #[test]
    fn program_returns_unit() {
        let program = CoreProgramBuilder::new(FunctionId(0))
            .push_function(
                SymbolId(0),
                1,
                vec![
                    Instr::LoadUnit { dst: RegId(0) },
                    Instr::Ret { src: RegId(0) },
                ],
            )
            .build()
            .unwrap();
        let result = run(&program, CoreConfig::default());
        assert_eq!(result.status, CoreStatus::Returned);
        assert_eq!(result.return_value, CoreValue::Unit);
    }

    #[test]
    fn root_ret_returns_value_without_stack_trap() {
        let program = CoreProgramBuilder::new(FunctionId(0))
            .push_function(
                SymbolId(0),
                1,
                vec![
                    Instr::LoadI32 {
                        dst: RegId(0),
                        value: 7,
                    },
                    Instr::Ret { src: RegId(0) },
                ],
            )
            .build()
            .unwrap();
        let result = run(&program, CoreConfig::default());
        assert_eq!(result.status, CoreStatus::Returned);
        assert_eq!(result.return_value, CoreValue::I32(7));
    }

    #[test]
    fn quad_join_program() {
        let program = CoreProgramBuilder::new(FunctionId(0))
            .push_function(
                SymbolId(0),
                3,
                vec![
                    Instr::LoadQuad {
                        dst: RegId(0),
                        value: QuadState::T,
                    },
                    Instr::LoadQuad {
                        dst: RegId(1),
                        value: QuadState::F,
                    },
                    Instr::QJoin {
                        dst: RegId(2),
                        lhs: RegId(0),
                        rhs: RegId(1),
                    },
                    Instr::Ret { src: RegId(2) },
                ],
            )
            .build()
            .unwrap();
        let result = run(&program, CoreConfig::default());
        assert_eq!(result.return_value, CoreValue::Quad(QuadState::S));
    }

    #[test]
    fn bool_branch_program() {
        let program = CoreProgramBuilder::new(FunctionId(0))
            .push_function(
                SymbolId(0),
                2,
                vec![
                    Instr::LoadBool {
                        dst: RegId(0),
                        value: false,
                    },
                    Instr::JumpIfFalse {
                        cond: RegId(0),
                        target: 4,
                    },
                    Instr::LoadI32 {
                        dst: RegId(1),
                        value: 1,
                    },
                    Instr::Jump { target: 5 },
                    Instr::LoadI32 {
                        dst: RegId(1),
                        value: 2,
                    },
                    Instr::Ret { src: RegId(1) },
                ],
            )
            .build()
            .unwrap();
        let result = run(&program, CoreConfig::default());
        assert_eq!(result.return_value, CoreValue::I32(2));
    }

    #[test]
    fn fuel_exceeded_program() {
        let program = CoreProgramBuilder::new(FunctionId(0))
            .push_function(
                SymbolId(0),
                1,
                vec![Instr::Jump { target: 0 }, Instr::Ret { src: RegId(0) }],
            )
            .build()
            .unwrap();
        let result = run(
            &program,
            CoreConfig {
                fuel: 3,
                ..CoreConfig::default()
            },
        );
        assert_eq!(result.status, CoreStatus::Trapped(CoreTrap::FuelExceeded));
    }

    #[test]
    fn trap_program() {
        let program = CoreProgramBuilder::new(FunctionId(0))
            .push_function(
                SymbolId(0),
                1,
                vec![Instr::Trap, Instr::Ret { src: RegId(0) }],
            )
            .build()
            .unwrap();
        let result = run(&program, CoreConfig::default());
        assert_eq!(result.status, CoreStatus::Trapped(CoreTrap::ExplicitTrap));
    }

    #[test]
    fn assert_failed_program() {
        let program = CoreProgramBuilder::new(FunctionId(0))
            .push_function(
                SymbolId(0),
                2,
                vec![
                    Instr::LoadBool {
                        dst: RegId(0),
                        value: false,
                    },
                    Instr::Assert { cond: RegId(0) },
                    Instr::LoadI32 {
                        dst: RegId(1),
                        value: 1,
                    },
                    Instr::Ret { src: RegId(1) },
                ],
            )
            .build()
            .unwrap();
        let result = run(&program, CoreConfig::default());
        assert_eq!(result.status, CoreStatus::Trapped(CoreTrap::AssertFailed));
    }

    #[test]
    fn simple_call_returns_value() {
        let program = CoreProgramBuilder::new(FunctionId(0))
            .push_function(
                SymbolId(0),
                2,
                vec![
                    Instr::LoadI32 {
                        dst: RegId(0),
                        value: 41,
                    },
                    Instr::Call {
                        dst: RegId(1),
                        function: FunctionId(1),
                        arg_base: RegId(0),
                        arg_count: 1,
                    },
                    Instr::Ret { src: RegId(1) },
                ],
            )
            .push_function(
                SymbolId(1),
                3,
                vec![
                    Instr::LoadI32 {
                        dst: RegId(1),
                        value: 1,
                    },
                    Instr::I32Add {
                        dst: RegId(2),
                        lhs: RegId(0),
                        rhs: RegId(1),
                    },
                    Instr::Ret { src: RegId(2) },
                ],
            )
            .build()
            .unwrap();
        let result = run(&program, CoreConfig::default());
        assert_eq!(result.return_value, CoreValue::I32(42));
    }

    #[test]
    fn nested_call_returns_value() {
        let program = CoreProgramBuilder::new(FunctionId(0))
            .push_function(
                SymbolId(0),
                2,
                vec![
                    Instr::LoadI32 {
                        dst: RegId(0),
                        value: 40,
                    },
                    Instr::Call {
                        dst: RegId(1),
                        function: FunctionId(1),
                        arg_base: RegId(0),
                        arg_count: 1,
                    },
                    Instr::Ret { src: RegId(1) },
                ],
            )
            .push_function(
                SymbolId(1),
                2,
                vec![
                    Instr::Call {
                        dst: RegId(1),
                        function: FunctionId(2),
                        arg_base: RegId(0),
                        arg_count: 1,
                    },
                    Instr::Ret { src: RegId(1) },
                ],
            )
            .push_function(
                SymbolId(2),
                3,
                vec![
                    Instr::LoadI32 {
                        dst: RegId(1),
                        value: 2,
                    },
                    Instr::I32Add {
                        dst: RegId(2),
                        lhs: RegId(0),
                        rhs: RegId(1),
                    },
                    Instr::Ret { src: RegId(2) },
                ],
            )
            .build()
            .unwrap();
        let result = run(&program, CoreConfig::default());
        assert_eq!(result.return_value, CoreValue::I32(42));
    }

    #[test]
    fn recursive_call_limited_by_depth() {
        let program = CoreProgramBuilder::new(FunctionId(0))
            .push_function(
                SymbolId(0),
                1,
                vec![
                    Instr::Call {
                        dst: RegId(0),
                        function: FunctionId(0),
                        arg_base: RegId(0),
                        arg_count: 0,
                    },
                    Instr::Ret { src: RegId(0) },
                ],
            )
            .build()
            .unwrap();
        let result = run(
            &program,
            CoreConfig {
                max_call_depth: 2,
                ..CoreConfig::default()
            },
        );
        assert_eq!(
            result.status,
            CoreStatus::Trapped(CoreTrap::CallDepthExceeded)
        );
    }
}
