pub const MAGIC0: [u8; 8] = *b"SEMCODE0";
pub const MAGIC1: [u8; 8] = *b"SEMCODE1";
pub const MAGIC2: [u8; 8] = *b"SEMCODE2";
pub const MAGIC3: [u8; 8] = *b"SEMCODE3";
pub const MAGIC4: [u8; 8] = *b"SEMCODE4";
pub const MAGIC5: [u8; 8] = *b"SEMCODE5";
pub const MAGIC6: [u8; 8] = *b"SEMCODE6";
pub const MAGIC7: [u8; 8] = *b"SEMCODE7";
pub const MAGIC8: [u8; 8] = *b"SEMCODE8";
pub const MAGIC9: [u8; 8] = *b"SEMCODE9";
pub const MAGIC10: [u8; 8] = *b"SEMCOD10";
pub const MAGIC11: [u8; 8] = *b"SEMCOD11";
pub const MAGIC12: [u8; 8] = *b"SEMCOD12";
pub const MAGIC13: [u8; 8] = *b"SEMCOD13";
pub const MAGIC14: [u8; 8] = *b"SEMCOD14";
pub const MAGIC15: [u8; 8] = *b"SEMCOD15";
pub const MAGIC16: [u8; 8] = *b"SEMCOD16";
pub const MAGIC17: [u8; 8] = *b"SEMCOD17";
pub const MAGIC18: [u8; 8] = *b"SEMCOD18";

pub const CAP_DEBUG_SYMBOLS: u32 = 1 << 0;
pub const CAP_F64_MATH: u32 = 1 << 1;
pub const CAP_GATE_SURFACE: u32 = 1 << 2;
pub const CAP_FX_VALUES: u32 = 1 << 3;
pub const CAP_FX_MATH: u32 = 1 << 4;
pub const CAP_STATE_QUERY: u32 = 1 << 5;
pub const CAP_STATE_UPDATE: u32 = 1 << 6;
pub const CAP_EVENT_POST: u32 = 1 << 7;
pub const CAP_CLOCK_READ: u32 = 1 << 8;
pub const CAP_TEXT_VALUES: u32 = 1 << 9;
pub const CAP_SEQUENCE_VALUES: u32 = 1 << 10;
pub const CAP_CLOSURE_VALUES: u32 = 1 << 11;
pub const CAP_OWNERSHIP_PATHS: u32 = 1 << 12;
pub const CAP_OWNERSHIP_FIELD_PATHS: u32 = 1 << 13;
pub const CAP_SEQUENCE_ITERATION: u32 = 1 << 14;
pub const CAP_MAP_VALUES: u32 = 1 << 15;
pub const CAP_PRNG: u32 = 1 << 16;
pub const CAP_STDOUT: u32 = 1 << 17;
pub const CAP_ARGS_READ: u32 = 1 << 18;
pub const CAP_STDIN_READ_TEXT: u32 = 1 << 19;
pub const CAP_STDOUT_WRITE: u32 = 1 << 20;
pub const CAP_STDERR_WRITE: u32 = 1 << 21;
pub const CAP_PATH_INSPECT: u32 = 1 << 22;
pub const CAP_FS_READ: u32 = 1 << 23;
pub const CAP_FS_WRITE: u32 = 1 << 24;
pub const CAP_TIME_DURATION: u32 = 1 << 25;

pub const OWNERSHIP_SECTION_TAG: [u8; 4] = *b"OWN0";
pub const OWNERSHIP_EVENT_KIND_BORROW: u8 = 0;
pub const OWNERSHIP_EVENT_KIND_WRITE: u8 = 1;
pub const OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX: u8 = 0;
pub const OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL: u8 = 1;
pub const OWNERSHIP_PATH_COMPONENT_ADT_PAYLOAD: u8 = 2;
pub const OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemcodeHeaderSpec {
    pub magic: [u8; 8],
    pub epoch: u16,
    pub rev: u16,
    pub capabilities: u32,
}

pub const HEADER_V0: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC0,
    epoch: 0,
    rev: 1,
    capabilities: CAP_DEBUG_SYMBOLS | CAP_GATE_SURFACE,
};

pub const HEADER_V1: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC1,
    epoch: 0,
    rev: 2,
    capabilities: CAP_DEBUG_SYMBOLS | CAP_F64_MATH | CAP_GATE_SURFACE,
};

pub const HEADER_V2: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC2,
    epoch: 0,
    rev: 3,
    capabilities: CAP_DEBUG_SYMBOLS | CAP_F64_MATH | CAP_GATE_SURFACE | CAP_FX_VALUES,
};

pub const HEADER_V3: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC3,
    epoch: 0,
    rev: 4,
    capabilities: CAP_DEBUG_SYMBOLS | CAP_F64_MATH | CAP_GATE_SURFACE | CAP_FX_VALUES | CAP_FX_MATH,
};

pub const HEADER_V4: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC4,
    epoch: 0,
    rev: 5,
    capabilities: CAP_DEBUG_SYMBOLS
        | CAP_F64_MATH
        | CAP_GATE_SURFACE
        | CAP_FX_VALUES
        | CAP_FX_MATH
        | CAP_STATE_QUERY,
};

pub const HEADER_V5: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC5,
    epoch: 0,
    rev: 6,
    capabilities: CAP_DEBUG_SYMBOLS
        | CAP_F64_MATH
        | CAP_GATE_SURFACE
        | CAP_FX_VALUES
        | CAP_FX_MATH
        | CAP_STATE_QUERY
        | CAP_STATE_UPDATE,
};

pub const HEADER_V6: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC6,
    epoch: 0,
    rev: 7,
    capabilities: CAP_DEBUG_SYMBOLS
        | CAP_F64_MATH
        | CAP_GATE_SURFACE
        | CAP_FX_VALUES
        | CAP_FX_MATH
        | CAP_STATE_QUERY
        | CAP_STATE_UPDATE
        | CAP_EVENT_POST,
};

pub const HEADER_V7: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC7,
    epoch: 0,
    rev: 8,
    capabilities: CAP_DEBUG_SYMBOLS
        | CAP_F64_MATH
        | CAP_GATE_SURFACE
        | CAP_FX_VALUES
        | CAP_FX_MATH
        | CAP_STATE_QUERY
        | CAP_STATE_UPDATE
        | CAP_EVENT_POST
        | CAP_CLOCK_READ,
};

pub const HEADER_V8: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC8,
    epoch: 0,
    rev: 9,
    capabilities: CAP_DEBUG_SYMBOLS
        | CAP_F64_MATH
        | CAP_GATE_SURFACE
        | CAP_FX_VALUES
        | CAP_FX_MATH
        | CAP_STATE_QUERY
        | CAP_STATE_UPDATE
        | CAP_EVENT_POST
        | CAP_CLOCK_READ
        | CAP_TEXT_VALUES,
};

pub const HEADER_V9: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC9,
    epoch: 0,
    rev: 10,
    capabilities: CAP_DEBUG_SYMBOLS
        | CAP_F64_MATH
        | CAP_GATE_SURFACE
        | CAP_FX_VALUES
        | CAP_FX_MATH
        | CAP_STATE_QUERY
        | CAP_STATE_UPDATE
        | CAP_EVENT_POST
        | CAP_CLOCK_READ
        | CAP_TEXT_VALUES
        | CAP_SEQUENCE_VALUES,
};

pub const HEADER_V10: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC10,
    epoch: 0,
    rev: 11,
    capabilities: CAP_DEBUG_SYMBOLS
        | CAP_F64_MATH
        | CAP_GATE_SURFACE
        | CAP_FX_VALUES
        | CAP_FX_MATH
        | CAP_STATE_QUERY
        | CAP_STATE_UPDATE
        | CAP_EVENT_POST
        | CAP_CLOCK_READ
        | CAP_TEXT_VALUES
        | CAP_SEQUENCE_VALUES
        | CAP_CLOSURE_VALUES,
};

pub const HEADER_V11: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC11,
    epoch: 0,
    rev: 12,
    capabilities: CAP_DEBUG_SYMBOLS
        | CAP_F64_MATH
        | CAP_GATE_SURFACE
        | CAP_FX_VALUES
        | CAP_FX_MATH
        | CAP_STATE_QUERY
        | CAP_STATE_UPDATE
        | CAP_EVENT_POST
        | CAP_CLOCK_READ
        | CAP_TEXT_VALUES
        | CAP_SEQUENCE_VALUES
        | CAP_CLOSURE_VALUES
        | CAP_OWNERSHIP_PATHS,
};

pub const HEADER_V12: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC12,
    epoch: 0,
    rev: 13,
    capabilities: CAP_DEBUG_SYMBOLS
        | CAP_F64_MATH
        | CAP_GATE_SURFACE
        | CAP_FX_VALUES
        | CAP_FX_MATH
        | CAP_STATE_QUERY
        | CAP_STATE_UPDATE
        | CAP_EVENT_POST
        | CAP_CLOCK_READ
        | CAP_TEXT_VALUES
        | CAP_SEQUENCE_VALUES
        | CAP_CLOSURE_VALUES
        | CAP_OWNERSHIP_PATHS
        | CAP_OWNERSHIP_FIELD_PATHS,
};

pub const HEADER_V13: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC13,
    epoch: 0,
    rev: 14,
    capabilities: CAP_DEBUG_SYMBOLS
        | CAP_F64_MATH
        | CAP_GATE_SURFACE
        | CAP_FX_VALUES
        | CAP_FX_MATH
        | CAP_STATE_QUERY
        | CAP_STATE_UPDATE
        | CAP_EVENT_POST
        | CAP_CLOCK_READ
        | CAP_TEXT_VALUES
        | CAP_SEQUENCE_VALUES
        | CAP_CLOSURE_VALUES
        | CAP_OWNERSHIP_PATHS
        | CAP_OWNERSHIP_FIELD_PATHS
        | CAP_SEQUENCE_ITERATION,
};

pub const HEADER_V14: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC14,
    epoch: 0,
    rev: 15,
    capabilities: CAP_DEBUG_SYMBOLS
        | CAP_F64_MATH
        | CAP_GATE_SURFACE
        | CAP_FX_VALUES
        | CAP_FX_MATH
        | CAP_STATE_QUERY
        | CAP_STATE_UPDATE
        | CAP_EVENT_POST
        | CAP_CLOCK_READ
        | CAP_TEXT_VALUES
        | CAP_SEQUENCE_VALUES
        | CAP_CLOSURE_VALUES
        | CAP_OWNERSHIP_PATHS
        | CAP_OWNERSHIP_FIELD_PATHS
        | CAP_SEQUENCE_ITERATION
        | CAP_MAP_VALUES,
};

pub const HEADER_V15: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC15,
    epoch: 0,
    rev: 16,
    capabilities: CAP_DEBUG_SYMBOLS
        | CAP_F64_MATH
        | CAP_GATE_SURFACE
        | CAP_FX_VALUES
        | CAP_FX_MATH
        | CAP_STATE_QUERY
        | CAP_STATE_UPDATE
        | CAP_EVENT_POST
        | CAP_CLOCK_READ
        | CAP_TEXT_VALUES
        | CAP_SEQUENCE_VALUES
        | CAP_CLOSURE_VALUES
        | CAP_OWNERSHIP_PATHS
        | CAP_OWNERSHIP_FIELD_PATHS
        | CAP_SEQUENCE_ITERATION
        | CAP_MAP_VALUES
        | CAP_PRNG,
};

pub const HEADER_V16: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC16,
    epoch: 0,
    rev: 17,
    capabilities: CAP_DEBUG_SYMBOLS
        | CAP_F64_MATH
        | CAP_GATE_SURFACE
        | CAP_FX_VALUES
        | CAP_FX_MATH
        | CAP_STATE_QUERY
        | CAP_STATE_UPDATE
        | CAP_EVENT_POST
        | CAP_CLOCK_READ
        | CAP_TEXT_VALUES
        | CAP_SEQUENCE_VALUES
        | CAP_CLOSURE_VALUES
        | CAP_OWNERSHIP_PATHS
        | CAP_OWNERSHIP_FIELD_PATHS
        | CAP_SEQUENCE_ITERATION
        | CAP_MAP_VALUES
        | CAP_PRNG
        | CAP_STDOUT,
};

pub const HEADER_V17: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC17,
    epoch: 0,
    rev: 18,
    capabilities: HEADER_V16.capabilities
        | CAP_ARGS_READ
        | CAP_STDIN_READ_TEXT
        | CAP_STDOUT_WRITE
        | CAP_STDERR_WRITE
        | CAP_PATH_INSPECT
        | CAP_FS_READ
        | CAP_FS_WRITE
        | CAP_TIME_DURATION,
};

/// #1732 (FA-05-002): the first header revision whose contract actually
/// includes the QTruth opcode family (`QTruthAnd`/`QTruthOr`/`QTruthNot`/
/// `QTruthImpl`, `0x17..0x1A`). QTruth needs no new capability bit - it
/// carries forward `HEADER_V17`'s capability set unchanged - because the
/// gap this closes is a missing *version identity* gate, not a missing
/// capability (see `Opcode::minimum_semcode_revision`).
pub const HEADER_V18: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC18,
    epoch: 0,
    rev: 19,
    capabilities: HEADER_V17.capabilities,
};

pub fn supported_headers() -> &'static [SemcodeHeaderSpec] {
    &[
        HEADER_V0, HEADER_V1, HEADER_V2, HEADER_V3, HEADER_V4, HEADER_V5, HEADER_V6, HEADER_V7,
        HEADER_V8, HEADER_V9, HEADER_V10, HEADER_V11, HEADER_V12, HEADER_V13, HEADER_V14,
        HEADER_V15, HEADER_V16, HEADER_V17, HEADER_V18,
    ]
}

pub fn header_spec_from_magic(magic: &[u8; 8]) -> Option<SemcodeHeaderSpec> {
    supported_headers()
        .iter()
        .copied()
        .find(|h| &h.magic == magic)
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Opcode {
    LoadQ = 0x01,
    LoadBool = 0x02,
    LoadI32 = 0x03,
    AddI32 = 0x07,
    SubI32 = 0x08,
    MulI32 = 0x09,
    DivI32 = 0x0a,
    ModI32 = 0x0b,
    ConcatText = 0x0c,
    LoadU32 = 0x06,
    LoadVar = 0x04,
    StoreVar = 0x05,
    QAnd = 0x10,
    QOr = 0x11,
    QNot = 0x12,
    QImpl = 0x13,
    BoolAnd = 0x14,
    BoolOr = 0x15,
    BoolNot = 0x16,
    QTruthAnd = 0x17,
    QTruthOr = 0x18,
    QTruthNot = 0x19,
    QTruthImpl = 0x1a,
    CmpEq = 0x20,
    CmpNe = 0x21,
    CmpI32Lt = 0x22,
    CmpI32Le = 0x23,
    Jmp = 0x30,
    JmpIf = 0x31,
    Call = 0x40,
    Ret = 0x41,
    Assert = 0x42,
    MakeTuple = 0x43,
    TupleGet = 0x44,
    MakeRecord = 0x45,
    RecordGet = 0x46,
    MakeAdt = 0x47,
    AdtTag = 0x48,
    AdtGet = 0x49,
    LoadF64 = 0x50,
    AddF64 = 0x51,
    SubF64 = 0x52,
    MulF64 = 0x53,
    DivF64 = 0x54,
    LoadFx = 0x55,
    AddFx = 0x56,
    SubFx = 0x57,
    MulFx = 0x58,
    DivFx = 0x59,
    LoadText = 0x5a,
    MakeSequence = 0x5b,
    SequenceGet = 0x5c,
    MakeClosure = 0x5d,
    ClosureCall = 0x5e,
    SequenceLen = 0x5f,
    SequenceIsEmpty = 0x67,
    SequenceContains = 0x68,
    SequencePush = 0x69,
    SequencePrepend = 0x6a,
    SequencePop = 0x6b,
    MapEmpty = 0x70,
    MapContains = 0x71,
    MapGet = 0x72,
    MapSet = 0x73,
    RngSeed = 0x74,
    RngNextI32 = 0x75,
    GateRead = 0x60,
    GateWrite = 0x61,
    PulseEmit = 0x62,
    StateQuery = 0x63,
    StateUpdate = 0x64,
    EventPost = 0x65,
    ClockRead = 0x66,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemcodeFormatError {
    UnexpectedEof,
    InvalidUtf8,
    UnknownOpcode(u8),
}

impl core::fmt::Display for SemcodeFormatError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SemcodeFormatError::UnexpectedEof => write!(f, "unexpected EOF"),
            SemcodeFormatError::InvalidUtf8 => write!(f, "invalid utf8"),
            SemcodeFormatError::UnknownOpcode(v) => write!(f, "unknown opcode 0x{:02x}", v),
        }
    }
}

impl std::error::Error for SemcodeFormatError {}

impl Opcode {
    pub fn byte(self) -> u8 {
        self as u8
    }

    pub fn from_byte(v: u8) -> Result<Self, SemcodeFormatError> {
        match v {
            x if x == Self::LoadQ as u8 => Ok(Self::LoadQ),
            x if x == Self::LoadBool as u8 => Ok(Self::LoadBool),
            x if x == Self::LoadI32 as u8 => Ok(Self::LoadI32),
            x if x == Self::AddI32 as u8 => Ok(Self::AddI32),
            x if x == Self::SubI32 as u8 => Ok(Self::SubI32),
            x if x == Self::MulI32 as u8 => Ok(Self::MulI32),
            x if x == Self::DivI32 as u8 => Ok(Self::DivI32),
            x if x == Self::ModI32 as u8 => Ok(Self::ModI32),
            x if x == Self::ConcatText as u8 => Ok(Self::ConcatText),
            x if x == Self::LoadU32 as u8 => Ok(Self::LoadU32),
            x if x == Self::LoadVar as u8 => Ok(Self::LoadVar),
            x if x == Self::StoreVar as u8 => Ok(Self::StoreVar),
            x if x == Self::QAnd as u8 => Ok(Self::QAnd),
            x if x == Self::QOr as u8 => Ok(Self::QOr),
            x if x == Self::QNot as u8 => Ok(Self::QNot),
            x if x == Self::QImpl as u8 => Ok(Self::QImpl),
            x if x == Self::BoolAnd as u8 => Ok(Self::BoolAnd),
            x if x == Self::BoolOr as u8 => Ok(Self::BoolOr),
            x if x == Self::BoolNot as u8 => Ok(Self::BoolNot),
            x if x == Self::QTruthAnd as u8 => Ok(Self::QTruthAnd),
            x if x == Self::QTruthOr as u8 => Ok(Self::QTruthOr),
            x if x == Self::QTruthNot as u8 => Ok(Self::QTruthNot),
            x if x == Self::QTruthImpl as u8 => Ok(Self::QTruthImpl),
            x if x == Self::CmpEq as u8 => Ok(Self::CmpEq),
            x if x == Self::CmpNe as u8 => Ok(Self::CmpNe),
            x if x == Self::CmpI32Lt as u8 => Ok(Self::CmpI32Lt),
            x if x == Self::CmpI32Le as u8 => Ok(Self::CmpI32Le),
            x if x == Self::Jmp as u8 => Ok(Self::Jmp),
            x if x == Self::JmpIf as u8 => Ok(Self::JmpIf),
            x if x == Self::Call as u8 => Ok(Self::Call),
            x if x == Self::Ret as u8 => Ok(Self::Ret),
            x if x == Self::Assert as u8 => Ok(Self::Assert),
            x if x == Self::MakeTuple as u8 => Ok(Self::MakeTuple),
            x if x == Self::TupleGet as u8 => Ok(Self::TupleGet),
            x if x == Self::MakeRecord as u8 => Ok(Self::MakeRecord),
            x if x == Self::RecordGet as u8 => Ok(Self::RecordGet),
            x if x == Self::MakeAdt as u8 => Ok(Self::MakeAdt),
            x if x == Self::AdtTag as u8 => Ok(Self::AdtTag),
            x if x == Self::AdtGet as u8 => Ok(Self::AdtGet),
            x if x == Self::LoadF64 as u8 => Ok(Self::LoadF64),
            x if x == Self::AddF64 as u8 => Ok(Self::AddF64),
            x if x == Self::SubF64 as u8 => Ok(Self::SubF64),
            x if x == Self::MulF64 as u8 => Ok(Self::MulF64),
            x if x == Self::DivF64 as u8 => Ok(Self::DivF64),
            x if x == Self::LoadFx as u8 => Ok(Self::LoadFx),
            x if x == Self::AddFx as u8 => Ok(Self::AddFx),
            x if x == Self::SubFx as u8 => Ok(Self::SubFx),
            x if x == Self::MulFx as u8 => Ok(Self::MulFx),
            x if x == Self::DivFx as u8 => Ok(Self::DivFx),
            x if x == Self::LoadText as u8 => Ok(Self::LoadText),
            x if x == Self::MakeSequence as u8 => Ok(Self::MakeSequence),
            x if x == Self::SequenceGet as u8 => Ok(Self::SequenceGet),
            x if x == Self::MakeClosure as u8 => Ok(Self::MakeClosure),
            x if x == Self::ClosureCall as u8 => Ok(Self::ClosureCall),
            x if x == Self::SequenceLen as u8 => Ok(Self::SequenceLen),
            x if x == Self::SequenceIsEmpty as u8 => Ok(Self::SequenceIsEmpty),
            x if x == Self::SequenceContains as u8 => Ok(Self::SequenceContains),
            x if x == Self::SequencePush as u8 => Ok(Self::SequencePush),
            x if x == Self::SequencePrepend as u8 => Ok(Self::SequencePrepend),
            x if x == Self::SequencePop as u8 => Ok(Self::SequencePop),
            x if x == Self::MapEmpty as u8 => Ok(Self::MapEmpty),
            x if x == Self::MapContains as u8 => Ok(Self::MapContains),
            x if x == Self::MapGet as u8 => Ok(Self::MapGet),
            x if x == Self::MapSet as u8 => Ok(Self::MapSet),
            x if x == Self::RngSeed as u8 => Ok(Self::RngSeed),
            x if x == Self::RngNextI32 as u8 => Ok(Self::RngNextI32),
            x if x == Self::GateRead as u8 => Ok(Self::GateRead),
            x if x == Self::GateWrite as u8 => Ok(Self::GateWrite),
            x if x == Self::PulseEmit as u8 => Ok(Self::PulseEmit),
            x if x == Self::StateQuery as u8 => Ok(Self::StateQuery),
            x if x == Self::StateUpdate as u8 => Ok(Self::StateUpdate),
            x if x == Self::EventPost as u8 => Ok(Self::EventPost),
            x if x == Self::ClockRead as u8 => Ok(Self::ClockRead),
            _ => Err(SemcodeFormatError::UnknownOpcode(v)),
        }
    }

    /// The minimum SemCode header revision (`SemcodeHeaderSpec::rev`) whose
    /// contract this opcode's semantics belong to (see #1732 / FA-05-002).
    ///
    /// This is the single, format-owned authority binding executable opcode
    /// vocabulary to artifact header identity: `sm-verify` uses it to
    /// reject an opcode admitted under a header older than its minimum
    /// revision, and `sm-ir`'s header selection promotes to a header
    /// whose revision covers every opcode a program actually emits.
    ///
    /// Defaults to `1` (`SEMCODE0`, the baseline header) for every opcode.
    /// Only opcode families with a *provable* later introduction revision -
    /// backed by an actual repository decision record, not by commit date
    /// alone - get a non-default entry here; this function must not imply
    /// stronger historical knowledge than the repository has actually
    /// established. Every opcode that is instead gated by a capability bit
    /// (`decode_operands`'s `required_capabilities`) already has its
    /// minimum header enforced through that existing, independent
    /// mechanism - `header.capabilities` is a fixed, monotonically-growing
    /// set per header revision - so this function is only load-bearing for
    /// opcodes that carry no capability requirement at all.
    ///
    /// Currently this covers only the QTruth family
    /// (`QTruthAnd`/`QTruthOr`/`QTruthNot`/`QTruthImpl`), whose minimum
    /// revision is `19` (`SEMCOD18`) per the #1732 repair decision: no
    /// existing header's documented contract ever claimed QTruth (it was
    /// added in #1455 with zero header/capability change across the entire
    /// rollout, #1455/#1457/#1459/#1461/#1463/#1465/#1471), so a new header
    /// revision was introduced specifically to close the identity gap.
    pub fn minimum_semcode_revision(self) -> u16 {
        match self {
            // #1732 (FA-05-002): QTruth Belnap truth-table opcodes -
            // independently provable later introduction (roadmap reservation
            // docs, #1455, zero header/capability change across the entire
            // rollout) with an explicit owner decision establishing SEMCOD18
            // as the minimum revision. The only family with a non-default
            // entry in this match.
            Self::QTruthAnd | Self::QTruthOr | Self::QTruthNot | Self::QTruthImpl => 19,

            // Baseline loads / constants
            Self::LoadQ
            | Self::LoadBool
            | Self::LoadI32
            | Self::LoadU32
            | Self::LoadVar
            | Self::StoreVar => 1,

            // Baseline 32-bit integer arithmetic
            Self::AddI32 | Self::SubI32 | Self::MulI32 | Self::DivI32 | Self::ModI32 => 1,

            // Baseline quad/bool logic - the legacy lattice family QTruth
            // sits right next to in the opcode byte layout, but is itself
            // historically baseline (present since the original enum)
            Self::QAnd
            | Self::QOr
            | Self::QNot
            | Self::QImpl
            | Self::BoolAnd
            | Self::BoolOr
            | Self::BoolNot => 1,

            // Baseline comparisons
            Self::CmpEq | Self::CmpNe | Self::CmpI32Lt | Self::CmpI32Le => 1,

            // Baseline control flow / calls
            Self::Jmp | Self::JmpIf | Self::Call | Self::Ret | Self::Assert => 1,

            // Baseline tuple / record / ADT
            Self::MakeTuple
            | Self::TupleGet
            | Self::MakeRecord
            | Self::RecordGet
            | Self::MakeAdt
            | Self::AdtTag
            | Self::AdtGet => 1,

            // Text (capability-gated: CAP_TEXT_VALUES already transitively
            // enforces the minimum header for these; listed explicitly only
            // for match exhaustiveness)
            Self::ConcatText | Self::LoadText => 1,

            // f64 / fx (capability-gated: CAP_F64_MATH / CAP_FX_VALUES / CAP_FX_MATH)
            Self::LoadF64
            | Self::AddF64
            | Self::SubF64
            | Self::MulF64
            | Self::DivF64
            | Self::LoadFx
            | Self::AddFx
            | Self::SubFx
            | Self::MulFx
            | Self::DivFx => 1,

            // Sequence (capability-gated: CAP_SEQUENCE_VALUES / CAP_SEQUENCE_ITERATION)
            Self::MakeSequence
            | Self::SequenceGet
            | Self::SequenceLen
            | Self::SequenceIsEmpty
            | Self::SequenceContains
            | Self::SequencePush
            | Self::SequencePrepend
            | Self::SequencePop => 1,

            // Closures (capability-gated: CAP_CLOSURE_VALUES)
            Self::MakeClosure | Self::ClosureCall => 1,

            // Map (capability-gated: CAP_MAP_VALUES)
            Self::MapEmpty | Self::MapContains | Self::MapGet | Self::MapSet => 1,

            // PRNG (capability-gated: CAP_PRNG)
            Self::RngSeed | Self::RngNextI32 => 1,

            // Gate host-effect surface (capability-gated: CAP_GATE_SURFACE)
            Self::GateRead | Self::GateWrite | Self::PulseEmit => 1,

            // Host-boundary state/event/clock (capability-gated:
            // CAP_STATE_QUERY / CAP_STATE_UPDATE / CAP_EVENT_POST / CAP_CLOCK_READ)
            Self::StateQuery | Self::StateUpdate | Self::EventPost | Self::ClockRead => 1,
        }
    }
}

pub fn write_u16_le(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
}

pub fn write_u32_le(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

pub fn write_i32_le(out: &mut Vec<u8>, v: i32) {
    out.extend_from_slice(&v.to_le_bytes());
}

pub fn write_f64_le(out: &mut Vec<u8>, v: f64) {
    out.extend_from_slice(&v.to_le_bytes());
}

pub fn read_u8(bytes: &[u8], i: &mut usize) -> Result<u8, SemcodeFormatError> {
    if *i >= bytes.len() {
        return Err(SemcodeFormatError::UnexpectedEof);
    }
    let v = bytes[*i];
    *i += 1;
    Ok(v)
}

pub fn read_u16_le(bytes: &[u8], i: &mut usize) -> Result<u16, SemcodeFormatError> {
    if *i + 2 > bytes.len() {
        return Err(SemcodeFormatError::UnexpectedEof);
    }
    let v = u16::from_le_bytes([bytes[*i], bytes[*i + 1]]);
    *i += 2;
    Ok(v)
}

pub fn read_u32_le(bytes: &[u8], i: &mut usize) -> Result<u32, SemcodeFormatError> {
    if *i + 4 > bytes.len() {
        return Err(SemcodeFormatError::UnexpectedEof);
    }
    let v = u32::from_le_bytes([bytes[*i], bytes[*i + 1], bytes[*i + 2], bytes[*i + 3]]);
    *i += 4;
    Ok(v)
}

pub fn read_i32_le(bytes: &[u8], i: &mut usize) -> Result<i32, SemcodeFormatError> {
    Ok(read_u32_le(bytes, i)? as i32)
}

pub fn read_f64_le(bytes: &[u8], i: &mut usize) -> Result<f64, SemcodeFormatError> {
    if *i + 8 > bytes.len() {
        return Err(SemcodeFormatError::UnexpectedEof);
    }
    let mut raw = [0u8; 8];
    raw.copy_from_slice(&bytes[*i..*i + 8]);
    *i += 8;
    Ok(f64::from_le_bytes(raw))
}

pub fn read_utf8(bytes: &[u8], i: &mut usize, len: usize) -> Result<String, SemcodeFormatError> {
    if *i + len > bytes.len() {
        return Err(SemcodeFormatError::UnexpectedEof);
    }
    let s = std::str::from_utf8(&bytes[*i..*i + len])
        .map_err(|_| SemcodeFormatError::InvalidUtf8)?
        .to_string();
    *i += len;
    Ok(s)
}
