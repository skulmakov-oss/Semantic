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
pub const MAGIC19: [u8; 8] = *b"SEMCOD19";
pub const MAGIC20: [u8; 8] = *b"SEMCOD20";
pub const MAGIC21: [u8; 8] = *b"SEMCOD21";
pub const MAGIC22: [u8; 8] = *b"SEMCOD22";
pub const MAGIC23: [u8; 8] = *b"SEMCOD23";
pub const MAGIC24: [u8; 8] = *b"SEMCOD24";

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
/// #1718: explicit admission authority for `SequenceIndexStatic` ownership
/// path components (both `Borrow` and `Write` events) - see
/// `docs/roadmap/stable_foundation/ssf08_1718_path_family_contract_decision.md`.
/// Deliberately a *separate* bit from `CAP_OWNERSHIP_ADT_BORROW_PATHS` even
/// though both are new in the same header revision: Sequence and ADT are two
/// independently-decided admission domains (the frozen contract gave them
/// different final dispositions - Sequence is fully admitted, ADT is
/// Borrow-only), and folding them into one generic "extended ownership" bit
/// would erase that distinction and make a future, separately-authorized
/// change to just one of them impossible without repurposing this bit.
pub const CAP_OWNERSHIP_SEQUENCE_PATHS: u32 = 1 << 26;
/// #1718: explicit admission authority for `AdtPayload` ownership path
/// components in `Borrow` events only. `Write(AdtPayload)` has no
/// compiler-reachable production path and is unconditionally rejected at
/// decode/verify regardless of header or capability - see this bit's sibling
/// doc comment above and the frozen contract decision. A future, separately
/// authorized promotion of `Write(AdtPayload)` must allocate its own
/// capability, never reinterpret this one.
pub const CAP_OWNERSHIP_ADT_BORROW_PATHS: u32 = 1 << 27;
/// SHF-2A2 (#2015): explicit admission authority for deterministic `Bytes`
/// value operations (`semantic.compiler.bytes/0.1`).
pub const CAP_BYTES_VALUES: u32 = 1 << 28;

pub const SIGNATURE_SECTION_TAG: [u8; 4] = *b"SIG0";

/// SSF-09 D2: tag of the module-level ADT descriptor section. Its presence
/// is derived from the header revision alone
/// (`SEMCODE_ADT_DESCRIPTOR_MIN_REVISION`), never sniffed from content.
pub const ADT_DESCRIPTOR_SECTION_TAG: [u8; 4] = *b"ADT0";

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

/// #1773 (FA-09-005): the first header revision whose per-function envelope
/// carries a canonical callable-signature record (`SIGNATURE_SECTION_TAG`,
/// `parameter_count` + `parameter_family[parameter_count]`). Reuses
/// `HEADER_V18`'s capability set unchanged - like `HEADER_V18` itself
/// (#1732), this closes a missing *version identity* gate (every function's
/// signature is now structurally present and provable), not a missing
/// capability. Every function envelope under this revision carries a SIG0
/// section deterministically (never sniffed - see `decode_semcode_envelope`
/// and its doc comment on why signature presence is derived from the header
/// revision rather than content-sniffed the way `DBG0`/`OWN0` are).
pub const HEADER_V19: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC19,
    epoch: 0,
    rev: 20,
    capabilities: HEADER_V18.capabilities,
};

/// The minimum SemCode header revision whose per-function envelope carries a
/// canonical callable-signature record (#1773 / FA-09-005). Any artifact
/// decoded under a header with `rev < SEMCODE_SIGNATURE_MIN_REVISION` has no
/// signature section for any function, by construction - old artifacts
/// remain structurally decodable, but canonical typed callable execution
/// cannot prove their contracts.
pub const SEMCODE_SIGNATURE_MIN_REVISION: u16 = HEADER_V19.rev;

/// #1726 Checkpoint D2a: a Tuple/Record Borrow event's activation authority
/// (`BorrowActivationResolved::StoreVarSite`, resolved by `sm-ir`'s emitter in
/// Checkpoint D1) becomes structurally representable in OWN0 starting at this
/// revision. Numeric-vs-name discipline (learned from a prior false "V20
/// collision" claim during design, corrected in Checkpoint D1.5): this
/// constant's numeric VALUE is **21**. `HEADER_V19.rev == 20` (that revision
/// belongs to #1773/SIG0). `HEADER_V20.rev == 21` (this one). Never write
/// "numeric revision 20" and "HEADER_V20" as though they were the same thing.
pub const SEMCODE_OWNERSHIP_ANCHOR_MIN_REVISION: u16 = HEADER_V20.rev;

/// OWN0 Borrow-event activation mode tag, revision-gated at
/// `SEMCODE_OWNERSHIP_ANCHOR_MIN_REVISION` (see that constant's doc comment).
/// Below this revision, OWN0's Borrow-event layout is unchanged from every
/// prior revision and carries no activation tag at all. At or above it, every
/// Borrow event (including the ADT/Option/Result producer's, which always
/// encodes `ACTIVATION_MODE_FRAME_ENTRY`) carries exactly one of these two
/// tags. An unrecognized tag byte is a hard structural rejection
/// (`DecodeError::InvalidOwnershipSection`) - never treated as
/// `ACTIVATION_MODE_FRAME_ENTRY` by default, and never a source for guessing.
pub const ACTIVATION_MODE_FRAME_ENTRY: u8 = 0;
pub const ACTIVATION_MODE_STORE_VAR_SITE: u8 = 1;

/// #1891 Checkpoint W2D: OWN0 Write-event execution-mode tag, revision-gated
/// at the same `SEMCODE_OWNERSHIP_ANCHOR_MIN_REVISION` as Borrow's
/// `ACTIVATION_MODE_*` above, occupying the identical wire position (right
/// after the event's `kind` byte, before its path). Below this revision,
/// OWN0's Write-event layout is unchanged from every prior revision and
/// carries no execution-mode tag at all. At or above it, every Write event
/// carries exactly one of these two tags, mirroring `ActivationSiteId`'s
/// pairing discipline for the two valid Write producer instruction kinds
/// (W1.5/W2A: `StoreVar` for producers A/B, `MakeRecord` for producer C).
/// Deliberately a SEPARATE type/value space from `ACTIVATION_MODE_*` even
/// though the numeric tags happen to overlap (0/1) - Borrow's activation
/// authority and Write's execution-site class are different domains and must
/// never be coupled just because they share a wire position and a revision
/// gate. An unrecognized tag byte is a hard structural rejection
/// (`DecodeError::InvalidOwnershipSection`) - never treated as either known
/// mode by default, and never inferred from the path or from opcode bytes.
pub const WRITE_EXECUTION_MODE_STORE_VAR_SITE: u8 = 0;
pub const WRITE_EXECUTION_MODE_MAKE_RECORD_SITE: u8 = 1;

pub const HEADER_V20: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC20,
    epoch: 0,
    rev: 21,
    capabilities: HEADER_V19.capabilities,
};

/// #1718: the first header revision carrying explicit admission authority for
/// `SequenceIndexStatic` (`Borrow`+`Write`) and `AdtPayload` (`Borrow` only)
/// ownership path components - see
/// `docs/roadmap/stable_foundation/ssf08_1718_path_family_contract_decision.md`.
/// Purely additive: `HEADER_V20`'s own capabilities, wire layout, and OWN0
/// execution-site grammar (`SEMCODE_OWNERSHIP_ANCHOR_MIN_REVISION`) are
/// unchanged and fully inherited - this revision answers only "which path
/// components may this header's OWN0 section carry," an orthogonal question
/// to "how are Borrow activation / Write execution sites encoded," which
/// `HEADER_V20` already settled and this revision does not reopen.
/// `Write(AdtPayload)` is deliberately NOT authorized by any capability here
/// or added by any later header without a separate, dedicated decision - see
/// `CAP_OWNERSHIP_ADT_BORROW_PATHS`'s doc comment.
pub const HEADER_V21: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC21,
    epoch: 0,
    rev: 22,
    capabilities: HEADER_V20.capabilities
        | CAP_OWNERSHIP_SEQUENCE_PATHS
        | CAP_OWNERSHIP_ADT_BORROW_PATHS,
};

/// #1718: the minimum SemCode header revision whose capability contract
/// authorizes `SequenceIndexStatic` ownership path components (`Borrow` and
/// `Write` both). Named by numeric revision, not header index, to avoid the
/// exact confusion `SEMCODE_OWNERSHIP_ANCHOR_MIN_REVISION`'s doc comment
/// already warns about: this constant's value is `HEADER_V21.rev == 22`, not
/// `21` (`HEADER_V20`'s revision) and not `20` (`HEADER_V19`'s).
pub const SEMCODE_SEQUENCE_OWNERSHIP_MIN_REVISION: u16 = HEADER_V21.rev;

/// #1718: the minimum SemCode header revision whose capability contract
/// authorizes `AdtPayload` ownership path components in `Borrow` events.
/// Deliberately has no `Write`-side counterpart - `Write(AdtPayload)` is not
/// admitted under any header revision, current or future, without a
/// separate, explicitly authorized contract change (see
/// `CAP_OWNERSHIP_ADT_BORROW_PATHS`). Same numeric value as
/// `SEMCODE_SEQUENCE_OWNERSHIP_MIN_REVISION` (`HEADER_V21.rev == 22`) today,
/// kept as a distinct named constant because the two families' admission
/// authority is independently decided and may diverge in a future revision.
pub const SEMCODE_ADT_BORROW_OWNERSHIP_MIN_REVISION: u16 = HEADER_V21.rev;

/// SSF-09 D2: the ADT descriptor contract header. Under this revision every
/// artifact carries the mandatory `ADT0` section immediately after the magic.
/// Capabilities are inherited unchanged from `HEADER_V21`.
///
/// D2-2 activates it as a whole: the compiler emits it for every artifact,
/// the decoder requires and strictly validates its `ADT0` section, and the
/// verifier admits descriptor-dependent ADT opcodes only against that table.
pub const HEADER_V22: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC22,
    epoch: 0,
    rev: 23,
    capabilities: HEADER_V21.capabilities,
};

/// SSF-09 D2: the minimum header revision whose artifacts carry the mandatory
/// `ADT0` section. Section presence is derived from this revision only.
pub const SEMCODE_ADT_DESCRIPTOR_MIN_REVISION: u16 = HEADER_V22.rev;

/// SHF-3A2 (#2008): the plain-`u32` arithmetic and ordering contract header
/// (`semantic.compiler.u32/0.1`, `docs/spec/compiler_u32_v0.md`). It admits the
/// `u32` opcode family (`CmpU32Lt` .. `ModU32`); nothing else changes, so its
/// capabilities are inherited unchanged from `HEADER_V22`. Plain integer
/// arithmetic is a language execution primitive, not a host capability, so no
/// capability bit is added. The compiler emits it only for programs that use
/// the family; every other artifact keeps the `HEADER_V22` floor.
pub const HEADER_V23: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC23,
    epoch: 0,
    rev: 24,
    capabilities: HEADER_V22.capabilities,
};

/// SHF-3A2 (#2008): the minimum header revision that admits the `u32`
/// arithmetic/ordering opcode family. A `SEMCOD22` (rev 23) artifact carrying
/// any of them is rejected, so the meaning of an existing header never widens.
pub const SEMCODE_U32_ARITH_MIN_REVISION: u16 = HEADER_V23.rev;

/// SHF-2A2 (#2015): the deterministic `Bytes` value contract header
/// (`semantic.compiler.bytes/0.1`, `docs/spec/compiler_bytes_v0.md`). It admits
/// the six core `bytes_*` builtins and callable parameter value family `Bytes`.
/// Capabilities inherit `HEADER_V23`'s capabilities plus `CAP_BYTES_VALUES`.
pub const HEADER_V24: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC24,
    epoch: 0,
    rev: 25,
    capabilities: HEADER_V23.capabilities | CAP_BYTES_VALUES,
};

/// SHF-2A2 (#2015): the minimum header revision that admits deterministic `Bytes`
/// value operations and callable parameters.
pub const SEMCODE_BYTES_MIN_REVISION: u16 = HEADER_V24.rev;

pub fn supported_headers() -> &'static [SemcodeHeaderSpec] {
    &[
        HEADER_V0, HEADER_V1, HEADER_V2, HEADER_V3, HEADER_V4, HEADER_V5, HEADER_V6, HEADER_V7,
        HEADER_V8, HEADER_V9, HEADER_V10, HEADER_V11, HEADER_V12, HEADER_V13, HEADER_V14,
        HEADER_V15, HEADER_V16, HEADER_V17, HEADER_V18, HEADER_V19, HEADER_V20, HEADER_V21,
        HEADER_V22, HEADER_V23, HEADER_V24,
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
    // SHF-3A2 (#2008): plain-u32 family, owner-approved bytes, SEMCOD23+.
    CmpU32Lt = 0x24,
    CmpU32Le = 0x25,
    AddU32 = 0x26,
    SubU32 = 0x27,
    MulU32 = 0x28,
    DivU32 = 0x29,
    ModU32 = 0x2a,
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
            x if x == Self::CmpU32Lt as u8 => Ok(Self::CmpU32Lt),
            x if x == Self::CmpU32Le as u8 => Ok(Self::CmpU32Le),
            x if x == Self::AddU32 as u8 => Ok(Self::AddU32),
            x if x == Self::SubU32 as u8 => Ok(Self::SubU32),
            x if x == Self::MulU32 as u8 => Ok(Self::MulU32),
            x if x == Self::DivU32 as u8 => Ok(Self::DivU32),
            x if x == Self::ModU32 as u8 => Ok(Self::ModU32),
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
    /// Every `Opcode` variant is assigned its minimum SemCode revision
    /// explicitly. Variants established as baseline are explicitly assigned
    /// revision `1` (`SEMCODE0`). Only opcode families with a *provable* later
    /// introduction revision - backed by an actual repository decision record,
    /// not by commit date alone - are explicitly assigned that later revision.
    /// The match is intentionally exhaustive and has no wildcard/default
    /// revision arm, so adding a new `Opcode` variant requires an explicit
    /// revision-policy decision at compile time. This function must not imply
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
            // as the minimum revision. The only family currently assigned a
            // non-baseline minimum revision in this match.
            Self::QTruthAnd | Self::QTruthOr | Self::QTruthNot | Self::QTruthImpl => 19,

            // SHF-3A2 (#2008): plain-u32 arithmetic/ordering, introduced with
            // its own header (SEMCOD23) by explicit owner decision.
            Self::CmpU32Lt
            | Self::CmpU32Le
            | Self::AddU32
            | Self::SubU32
            | Self::MulU32
            | Self::DivU32
            | Self::ModU32 => SEMCODE_U32_ARITH_MIN_REVISION,

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

/// The executable runtime-value family a callable parameter belongs to
/// (#1773 / FA-09-005). This describes the *runtime* shape a canonical
/// callable's argument must have - not the source `Type` AST - so several
/// distinct source types intentionally map to the same family (a `Measured`
/// numeric erases to its base family; `Option`/`Result` both map to `Adt`,
/// matching how they are actually lowered via `IrInstr::AdtTag`). Variants
/// mirror `sm-vm::Value` 1:1; tag `0` is deliberately unused so a
/// zero-initialized or truncated buffer never decodes as a valid family.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallableValueFamily {
    Quad = 1,
    Bool = 2,
    Text = 3,
    Sequence = 4,
    Map = 5,
    Closure = 6,
    I32 = 7,
    U32 = 8,
    Fx = 9,
    F64 = 10,
    Tuple = 11,
    Record = 12,
    Adt = 13,
    Unit = 14,
    Bytes = 15,
}

impl CallableValueFamily {
    pub fn byte(self) -> u8 {
        self as u8
    }

    pub fn from_byte(v: u8) -> Result<Self, SemcodeFormatError> {
        match v {
            x if x == Self::Quad as u8 => Ok(Self::Quad),
            x if x == Self::Bool as u8 => Ok(Self::Bool),
            x if x == Self::Text as u8 => Ok(Self::Text),
            x if x == Self::Sequence as u8 => Ok(Self::Sequence),
            x if x == Self::Map as u8 => Ok(Self::Map),
            x if x == Self::Closure as u8 => Ok(Self::Closure),
            x if x == Self::I32 as u8 => Ok(Self::I32),
            x if x == Self::U32 as u8 => Ok(Self::U32),
            x if x == Self::Fx as u8 => Ok(Self::Fx),
            x if x == Self::F64 as u8 => Ok(Self::F64),
            x if x == Self::Tuple as u8 => Ok(Self::Tuple),
            x if x == Self::Record as u8 => Ok(Self::Record),
            x if x == Self::Adt as u8 => Ok(Self::Adt),
            x if x == Self::Unit as u8 => Ok(Self::Unit),
            x if x == Self::Bytes as u8 => Ok(Self::Bytes),
            _ => Err(SemcodeFormatError::UnknownOpcode(v)),
        }
    }
}

/// The canonical callable-signature record for one function envelope
/// (#1773 / FA-09-005): parameter count and, for each parameter in
/// declaration order, its executable runtime family. `families.len()` is
/// the parameter count - there is no separate count field to disagree with
/// it, so arity and family-count are structurally impossible to desync once
/// a `CallableSignature` exists in memory. On the wire this is the `SIG0`
/// section: tag, `u16` count, then `count` single-byte family tags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallableSignature {
    pub families: Vec<CallableValueFamily>,
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

/// #1736 (FA-05-006): shared bounds-check primitive for every low-level
/// reader below. Uses `checked_add`, never raw `+`, so a maliciously large
/// `start` (e.g. a cursor advanced by an attacker-controlled length field)
/// cannot wrap past `usize::MAX` and produce a false in-bounds result -
/// which is exactly how the pre-fix `*i + width > bytes.len()` comparison
/// could be defeated on 32-bit targets.
fn checked_read_end(
    bytes_len: usize,
    start: usize,
    width: usize,
) -> Result<usize, SemcodeFormatError> {
    let end = start
        .checked_add(width)
        .ok_or(SemcodeFormatError::UnexpectedEof)?;
    if end > bytes_len {
        return Err(SemcodeFormatError::UnexpectedEof);
    }
    Ok(end)
}

pub fn read_u8(bytes: &[u8], i: &mut usize) -> Result<u8, SemcodeFormatError> {
    let end = checked_read_end(bytes.len(), *i, 1)?;
    let v = bytes[*i];
    *i = end;
    Ok(v)
}

pub fn read_u16_le(bytes: &[u8], i: &mut usize) -> Result<u16, SemcodeFormatError> {
    let end = checked_read_end(bytes.len(), *i, 2)?;
    let v = u16::from_le_bytes(bytes[*i..end].try_into().unwrap());
    *i = end;
    Ok(v)
}

pub fn read_u32_le(bytes: &[u8], i: &mut usize) -> Result<u32, SemcodeFormatError> {
    let end = checked_read_end(bytes.len(), *i, 4)?;
    let v = u32::from_le_bytes(bytes[*i..end].try_into().unwrap());
    *i = end;
    Ok(v)
}

pub fn read_i32_le(bytes: &[u8], i: &mut usize) -> Result<i32, SemcodeFormatError> {
    Ok(read_u32_le(bytes, i)? as i32)
}

pub fn read_f64_le(bytes: &[u8], i: &mut usize) -> Result<f64, SemcodeFormatError> {
    let end = checked_read_end(bytes.len(), *i, 8)?;
    let v = f64::from_le_bytes(bytes[*i..end].try_into().unwrap());
    *i = end;
    Ok(v)
}

pub fn read_utf8(bytes: &[u8], i: &mut usize, len: usize) -> Result<String, SemcodeFormatError> {
    let end = checked_read_end(bytes.len(), *i, len)?;
    let s = std::str::from_utf8(&bytes[*i..end])
        .map_err(|_| SemcodeFormatError::InvalidUtf8)?
        .to_string();
    *i = end;
    Ok(s)
}

/// SSF-09 D2: why an ADT descriptor or descriptor table cannot be built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdtDescriptorError {
    EmptyName,
    NameTooLong { name: String },
    DuplicateVariant { descriptor: String, variant: String },
    TooManyVariants { descriptor: String },
    DuplicateDescriptor { name: String },
    TooManyDescriptors,
}

impl core::fmt::Display for AdtDescriptorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AdtDescriptorError::EmptyName => write!(f, "ADT descriptor name is empty"),
            AdtDescriptorError::NameTooLong { name } => write!(
                f,
                "ADT descriptor name '{}' exceeds the u16 length limit",
                name
            ),
            AdtDescriptorError::DuplicateVariant {
                descriptor,
                variant,
            } => write!(
                f,
                "ADT descriptor '{}' repeats variant '{}'",
                descriptor, variant
            ),
            AdtDescriptorError::TooManyVariants { descriptor } => write!(
                f,
                "ADT descriptor '{}' exceeds the u16 variant-count limit",
                descriptor
            ),
            AdtDescriptorError::DuplicateDescriptor { name } => {
                write!(f, "duplicate ADT descriptor identity '{}'", name)
            }
            AdtDescriptorError::TooManyDescriptors => {
                write!(f, "ADT descriptor table exceeds the u16 entry-count limit")
            }
        }
    }
}

impl std::error::Error for AdtDescriptorError {}

fn check_adt_descriptor_name(name: &str) -> Result<(), AdtDescriptorError> {
    if name.is_empty() {
        return Err(AdtDescriptorError::EmptyName);
    }
    if u16::try_from(name.len()).is_err() {
        return Err(AdtDescriptorError::NameTooLong {
            name: name.to_string(),
        });
    }
    Ok(())
}

/// SSF-09 D2: one variant of an [`AdtDescriptor`]. A variant's tag is its
/// position in [`AdtDescriptor::variants`]; the tag is never stored as a
/// second, independent authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdtVariantDescriptor {
    name: String,
    payload_arity: u16,
}

impl AdtVariantDescriptor {
    pub fn new(name: &str, payload_arity: u16) -> Result<Self, AdtDescriptorError> {
        check_adt_descriptor_name(name)?;
        Ok(Self {
            name: name.to_string(),
            payload_arity,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn payload_arity(&self) -> u16 {
        self.payload_arity
    }
}

/// SSF-09 D2: one ADT. Its canonical type name is the ADT's semantic
/// identity; its variants are in source declaration order, so a variant's
/// tag is its index. Payload types are deliberately absent (D3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdtDescriptor {
    name: String,
    variants: Vec<AdtVariantDescriptor>,
}

impl AdtDescriptor {
    pub fn new(
        name: &str,
        variants: Vec<AdtVariantDescriptor>,
    ) -> Result<Self, AdtDescriptorError> {
        check_adt_descriptor_name(name)?;
        if u16::try_from(variants.len()).is_err() {
            return Err(AdtDescriptorError::TooManyVariants {
                descriptor: name.to_string(),
            });
        }
        let mut seen = std::collections::BTreeSet::new();
        for variant in &variants {
            if !seen.insert(variant.name.as_str()) {
                return Err(AdtDescriptorError::DuplicateVariant {
                    descriptor: name.to_string(),
                    variant: variant.name.clone(),
                });
            }
        }
        Ok(Self {
            name: name.to_string(),
            variants,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Variants in declaration order: `variants()[tag]` is the variant with
    /// that tag.
    pub fn variants(&self) -> &[AdtVariantDescriptor] {
        &self.variants
    }
}

/// The frozen built-in ADTs. Their variant order is the tag order the
/// compiler already uses for `Option` and `Result` values.
fn builtin_adt_descriptors() -> [AdtDescriptor; 2] {
    let variant = |name: &str, payload_arity| AdtVariantDescriptor {
        name: name.to_string(),
        payload_arity,
    };
    [
        AdtDescriptor {
            name: "Option".to_string(),
            variants: vec![variant("None", 0), variant("Some", 1)],
        },
        AdtDescriptor {
            name: "Result".to_string(),
            variants: vec![variant("Ok", 1), variant("Err", 1)],
        },
    ]
}

/// SSF-09 D2: the canonical ADT descriptor table of one artifact.
///
/// It always contains the built-in `Option` and `Result` descriptors, so it
/// is never empty, and its descriptors are in strictly ascending order of
/// their name bytes.
///
/// A descriptor's position in [`AdtDescriptorTable::descriptors`] is only a
/// deterministic lookup ordinal within this one table. It is NOT the ADT's
/// identity (the canonical type name is) and it is not stable across
/// artifacts: adding any other ADT can shift it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdtDescriptorTable {
    descriptors: Vec<AdtDescriptor>,
}

impl AdtDescriptorTable {
    /// Builds the canonical table from the built-in descriptors plus `user`.
    /// Two descriptors sharing a name are rejected; a user descriptor named
    /// `Option` or `Result` collides with the built-in.
    pub fn with_builtins(user: Vec<AdtDescriptor>) -> Result<Self, AdtDescriptorError> {
        let mut descriptors: Vec<AdtDescriptor> =
            builtin_adt_descriptors().into_iter().chain(user).collect();
        descriptors.sort_by(|a, b| a.name.as_bytes().cmp(b.name.as_bytes()));
        if let Some(pair) = descriptors.windows(2).find(|w| w[0].name == w[1].name) {
            return Err(AdtDescriptorError::DuplicateDescriptor {
                name: pair[0].name.clone(),
            });
        }
        if u16::try_from(descriptors.len()).is_err() {
            return Err(AdtDescriptorError::TooManyDescriptors);
        }
        Ok(Self { descriptors })
    }

    /// SSF-09 D2-2: the decoder-side constructor. Takes the descriptors in
    /// exactly the order an `ADT0` section encodes them and only validates:
    /// it never adds, drops, sorts or rewrites a descriptor, and never calls
    /// [`AdtDescriptorTable::with_builtins`]. The artifact's table is the
    /// artifact authority, so a missing or non-canonical built-in is a
    /// rejection, not something to repair.
    pub(crate) fn from_encoded_strict(
        descriptors: Vec<AdtDescriptor>,
    ) -> Result<Self, &'static str> {
        if u16::try_from(descriptors.len()).is_err() {
            return Err("ADT0 descriptor count exceeds the u16 limit");
        }
        for pair in descriptors.windows(2) {
            match pair[0].name.as_bytes().cmp(pair[1].name.as_bytes()) {
                core::cmp::Ordering::Less => {}
                core::cmp::Ordering::Equal => return Err("ADT0 repeats a descriptor name"),
                core::cmp::Ordering::Greater => {
                    return Err("ADT0 descriptors are not in strictly ascending name-byte order")
                }
            }
        }
        let table = Self { descriptors };
        for builtin in builtin_adt_descriptors() {
            let option = builtin.name == "Option";
            match table.get(&builtin.name) {
                None if option => return Err("ADT0 is missing the built-in Option descriptor"),
                None => return Err("ADT0 is missing the built-in Result descriptor"),
                Some(found) if *found != builtin && option => {
                    return Err("ADT0 Option descriptor is not the canonical [None/0, Some/1]")
                }
                Some(found) if *found != builtin => {
                    return Err("ADT0 Result descriptor is not the canonical [Ok/1, Err/1]")
                }
                Some(_) => {}
            }
        }
        Ok(table)
    }

    pub fn descriptors(&self) -> &[AdtDescriptor] {
        &self.descriptors
    }

    /// Looks a descriptor up by its canonical type name, the ADT's semantic
    /// identity. Relies on the strict name-byte order both constructors
    /// guarantee.
    pub fn get(&self, name: &str) -> Option<&AdtDescriptor> {
        self.descriptors
            .binary_search_by(|d| d.name.as_bytes().cmp(name.as_bytes()))
            .ok()
            .map(|index| &self.descriptors[index])
    }

    /// Encodes the `ADT0` section, all integers little-endian:
    ///
    /// ```text
    /// section    := "ADT0" descriptor_count:u16 descriptor[descriptor_count]
    /// descriptor := name_len:u16 name:[u8] variant_count:u16 variant[variant_count]
    /// variant    := name_len:u16 name:[u8] payload_arity:u16
    /// ```
    pub fn encode_section(&self) -> Vec<u8> {
        // Every count and name length below was checked to fit `u16` when
        // the table and its descriptors were built.
        let write_name = |out: &mut Vec<u8>, name: &str| {
            write_u16_le(out, name.len() as u16);
            out.extend_from_slice(name.as_bytes());
        };
        let mut out = Vec::new();
        out.extend_from_slice(&ADT_DESCRIPTOR_SECTION_TAG);
        write_u16_le(&mut out, self.descriptors.len() as u16);
        for descriptor in &self.descriptors {
            write_name(&mut out, &descriptor.name);
            write_u16_le(&mut out, descriptor.variants.len() as u16);
            for variant in &descriptor.variants {
                write_name(&mut out, &variant.name);
                write_u16_le(&mut out, variant.payload_arity);
            }
        }
        out
    }
}

// #1736 (FA-05-006): regression coverage for the checked-arithmetic repair.
// Before the fix, `*i + width > bytes.len()` used raw addition, so a cursor
// near `usize::MAX` (reachable from an attacker-controlled cursor/length on
// any target, and trivially reachable from an ordinary u32 length field on a
// 32-bit target) could wrap past zero and pass the bounds check, producing
// an out-of-range slice index panic instead of a decode error.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_u8_rejects_cursor_near_usize_max_without_panicking() {
        let bytes = [1u8, 2, 3];
        let mut i = usize::MAX - 1;
        assert_eq!(
            read_u8(&bytes, &mut i),
            Err(SemcodeFormatError::UnexpectedEof)
        );
    }

    #[test]
    fn read_u16_le_rejects_cursor_near_usize_max_without_panicking() {
        let bytes = [1u8, 2, 3];
        let mut i = usize::MAX - 1;
        assert_eq!(
            read_u16_le(&bytes, &mut i),
            Err(SemcodeFormatError::UnexpectedEof)
        );
    }

    #[test]
    fn read_u32_le_rejects_cursor_near_usize_max_without_panicking() {
        let bytes = [1u8, 2, 3];
        let mut i = usize::MAX - 1;
        assert_eq!(
            read_u32_le(&bytes, &mut i),
            Err(SemcodeFormatError::UnexpectedEof)
        );
    }

    #[test]
    fn read_f64_le_rejects_cursor_near_usize_max_without_panicking() {
        let bytes = [1u8, 2, 3];
        let mut i = usize::MAX - 1;
        assert_eq!(
            read_f64_le(&bytes, &mut i),
            Err(SemcodeFormatError::UnexpectedEof)
        );
    }

    #[test]
    fn read_utf8_rejects_overflowing_cursor_plus_len_without_panicking() {
        let bytes = [1u8, 2, 3];
        let mut i = 1usize;
        assert_eq!(
            read_utf8(&bytes, &mut i, usize::MAX),
            Err(SemcodeFormatError::UnexpectedEof)
        );
    }

    #[test]
    fn ordinary_reads_still_succeed_and_advance_cursor() {
        let bytes: Vec<u8> = vec![
            0x42, 0x01, 0x02, 0x03, 0x04, 1, 2, 3, 4, 5, 6, 7, 8, b'h', b'i',
        ];
        let mut i = 0usize;
        assert_eq!(read_u8(&bytes, &mut i), Ok(0x42));
        assert_eq!(i, 1);
        assert_eq!(read_u32_le(&bytes, &mut i), Ok(0x04030201));
        assert_eq!(i, 5);
        assert_eq!(
            read_f64_le(&bytes, &mut i),
            Ok(f64::from_le_bytes([1, 2, 3, 4, 5, 6, 7, 8]))
        );
        assert_eq!(i, 13);
        assert_eq!(read_utf8(&bytes, &mut i, 2), Ok("hi".to_string()));
        assert_eq!(i, 15);
    }

    #[test]
    fn ordinary_truncation_is_still_rejected() {
        let bytes = [0u8, 1];
        assert_eq!(
            read_u16_le(&bytes, &mut 1),
            Err(SemcodeFormatError::UnexpectedEof)
        );
        assert_eq!(
            read_u32_le(&bytes, &mut 0),
            Err(SemcodeFormatError::UnexpectedEof)
        );
        assert_eq!(
            read_f64_le(&bytes, &mut 0),
            Err(SemcodeFormatError::UnexpectedEof)
        );
        assert_eq!(
            read_utf8(&bytes, &mut 0, 3),
            Err(SemcodeFormatError::UnexpectedEof)
        );
    }
}

// SSF-09 D2-1: ADT descriptor model and ADT0 encoder.
#[cfg(test)]
mod adt_descriptor_tests {
    use super::*;

    fn variant(name: &str, payload_arity: u16) -> AdtVariantDescriptor {
        AdtVariantDescriptor::new(name, payload_arity).expect("valid variant")
    }

    fn adt(name: &str, variants: &[(&str, u16)]) -> AdtDescriptor {
        AdtDescriptor::new(name, variants.iter().map(|(n, a)| variant(n, *a)).collect())
            .expect("valid descriptor")
    }

    fn names(table: &AdtDescriptorTable) -> Vec<&str> {
        table.descriptors().iter().map(|d| d.name()).collect()
    }

    fn shape(descriptor: &AdtDescriptor) -> Vec<(&str, u16)> {
        descriptor
            .variants()
            .iter()
            .map(|v| (v.name(), v.payload_arity()))
            .collect()
    }

    fn find<'a>(table: &'a AdtDescriptorTable, name: &str) -> &'a AdtDescriptor {
        table
            .descriptors()
            .iter()
            .find(|d| d.name() == name)
            .expect("descriptor present")
    }

    #[test]
    fn table_without_user_enums_still_holds_option_and_result() {
        let table = AdtDescriptorTable::with_builtins(vec![]).expect("table");
        assert_eq!(names(&table), ["Option", "Result"]);
    }

    #[test]
    fn builtin_option_and_result_have_frozen_variants_and_arities() {
        let table = AdtDescriptorTable::with_builtins(vec![adt("E", &[("A", 0)])]).expect("table");
        assert_eq!(shape(find(&table, "Option")), [("None", 0), ("Some", 1)]);
        assert_eq!(shape(find(&table, "Result")), [("Ok", 1), ("Err", 1)]);
    }

    #[test]
    fn descriptors_sort_by_raw_utf8_name_bytes_not_by_locale() {
        let table = AdtDescriptorTable::with_builtins(vec![
            adt("b", &[]),
            adt("Z", &[]),
            adt("\u{c9}mile", &[]),
            adt("A", &[]),
            adt("Opt", &[]),
        ])
        .expect("table");
        // 0x5A (Z) < 0x62 (b) < 0xC3 0x89 (E-acute); "Opt" is a prefix of
        // "Option" and sorts first.
        assert_eq!(
            names(&table),
            ["A", "Opt", "Option", "Result", "Z", "b", "\u{c9}mile"]
        );
    }

    #[test]
    fn declaration_order_does_not_affect_the_table() {
        let z = || adt("Z", &[("Z0", 0)]);
        let a = || adt("A", &[("A0", 1)]);
        let za = AdtDescriptorTable::with_builtins(vec![z(), a()]).expect("table");
        let az = AdtDescriptorTable::with_builtins(vec![a(), z()]).expect("table");
        assert_eq!(za, az);
        assert_eq!(names(&za), ["A", "Option", "Result", "Z"]);
        assert_eq!(za.encode_section(), az.encode_section());
    }

    #[test]
    fn descriptor_ordinal_is_a_lookup_position_not_identity() {
        // Adding an unrelated ADT shifts ordinals, but every name still
        // denotes the same descriptor.
        let small = AdtDescriptorTable::with_builtins(vec![adt("Z", &[("Z0", 0)])]).expect("table");
        let large = AdtDescriptorTable::with_builtins(vec![adt("Z", &[("Z0", 0)]), adt("A", &[])])
            .expect("table");
        let position = |t: &AdtDescriptorTable, n: &str| {
            t.descriptors().iter().position(|d| d.name() == n).unwrap()
        };
        assert_ne!(position(&small, "Option"), position(&large, "Option"));
        for name in ["Option", "Result", "Z"] {
            assert_eq!(find(&small, name), find(&large, name));
        }
    }

    #[test]
    fn variants_keep_declaration_order() {
        let table =
            AdtDescriptorTable::with_builtins(vec![adt("E", &[("Zebra", 0), ("Alpha", 0)])])
                .expect("table");
        assert_eq!(shape(find(&table, "E")), [("Zebra", 0), ("Alpha", 0)]);
    }

    #[test]
    fn duplicate_descriptor_identity_is_rejected() {
        assert_eq!(
            AdtDescriptorTable::with_builtins(vec![adt("E", &[]), adt("E", &[])]),
            Err(AdtDescriptorError::DuplicateDescriptor {
                name: "E".to_string()
            })
        );
        for builtin in ["Option", "Result"] {
            assert_eq!(
                AdtDescriptorTable::with_builtins(vec![adt(builtin, &[])]),
                Err(AdtDescriptorError::DuplicateDescriptor {
                    name: builtin.to_string()
                })
            );
        }
    }

    #[test]
    fn duplicate_variant_and_empty_names_are_rejected() {
        assert_eq!(
            AdtDescriptor::new("E", vec![variant("A", 0), variant("A", 1)]),
            Err(AdtDescriptorError::DuplicateVariant {
                descriptor: "E".to_string(),
                variant: "A".to_string()
            })
        );
        assert_eq!(
            AdtDescriptor::new("", vec![]),
            Err(AdtDescriptorError::EmptyName)
        );
        assert_eq!(
            AdtVariantDescriptor::new("", 0),
            Err(AdtDescriptorError::EmptyName)
        );
    }

    #[test]
    fn u16_limits_are_enforced_at_construction() {
        let long = "n".repeat(usize::from(u16::MAX) + 1);
        assert!(matches!(
            AdtVariantDescriptor::new(&long, 0),
            Err(AdtDescriptorError::NameTooLong { .. })
        ));
        assert!(AdtVariantDescriptor::new(&long[1..], 0).is_ok());

        let variants = (0..=usize::from(u16::MAX))
            .map(|i| variant(&format!("V{i}"), 0))
            .collect::<Vec<_>>();
        assert_eq!(
            AdtDescriptor::new("E", variants),
            Err(AdtDescriptorError::TooManyVariants {
                descriptor: "E".to_string()
            })
        );

        // Two built-ins plus 65534 user descriptors exceed u16::MAX.
        let users = (0..usize::from(u16::MAX) - 1)
            .map(|i| adt(&format!("E{i}"), &[]))
            .collect::<Vec<_>>();
        assert_eq!(
            AdtDescriptorTable::with_builtins(users),
            Err(AdtDescriptorError::TooManyDescriptors)
        );
    }

    #[test]
    fn payload_arity_boundary_encodes_as_u16() {
        let table =
            AdtDescriptorTable::with_builtins(vec![adt("W", &[("Max", u16::MAX)])]).expect("table");
        let bytes = table.encode_section();
        let tail = [
            0x01, 0x00, b'W', 0x01, 0x00, 0x03, 0x00, b'M', b'a', b'x', 0xff, 0xff,
        ];
        assert!(bytes.ends_with(&tail), "{bytes:?}");
    }

    #[test]
    fn adt0_section_bytes_are_exact_for_builtins_only() {
        let table = AdtDescriptorTable::with_builtins(vec![]).expect("table");
        #[rustfmt::skip]
        let expected: Vec<u8> = [
            &b"ADT0"[..], &[0x02, 0x00],
            &[0x06, 0x00], b"Option", &[0x02, 0x00],
                &[0x04, 0x00], b"None", &[0x00, 0x00],
                &[0x04, 0x00], b"Some", &[0x01, 0x00],
            &[0x06, 0x00], b"Result", &[0x02, 0x00],
                &[0x02, 0x00], b"Ok", &[0x01, 0x00],
                &[0x03, 0x00], b"Err", &[0x01, 0x00],
        ]
        .concat();
        assert_eq!(table.encode_section(), expected);
    }

    #[test]
    fn adt0_section_bytes_are_exact_for_a_user_enum() {
        let table =
            AdtDescriptorTable::with_builtins(vec![adt("E", &[("A", 0), ("B", 1), ("C", 2)])])
                .expect("table");
        #[rustfmt::skip]
        let expected: Vec<u8> = [
            &b"ADT0"[..], &[0x03, 0x00],
            &[0x01, 0x00], b"E", &[0x03, 0x00],
                &[0x01, 0x00], b"A", &[0x00, 0x00],
                &[0x01, 0x00], b"B", &[0x01, 0x00],
                &[0x01, 0x00], b"C", &[0x02, 0x00],
            &[0x06, 0x00], b"Option", &[0x02, 0x00],
                &[0x04, 0x00], b"None", &[0x00, 0x00],
                &[0x04, 0x00], b"Some", &[0x01, 0x00],
            &[0x06, 0x00], b"Result", &[0x02, 0x00],
                &[0x02, 0x00], b"Ok", &[0x01, 0x00],
                &[0x03, 0x00], b"Err", &[0x01, 0x00],
        ]
        .concat();
        assert_eq!(table.encode_section(), expected);
    }

    #[test]
    fn d2_header_vocabulary_is_defined_and_admitted() {
        assert_eq!(HEADER_V22.magic, *b"SEMCOD22");
        assert_eq!(HEADER_V22.rev, 23);
        assert_eq!(HEADER_V22.epoch, 0);
        assert_eq!(HEADER_V22.capabilities, HEADER_V21.capabilities);
        assert_eq!(SEMCODE_ADT_DESCRIPTOR_MIN_REVISION, 23);
        assert_eq!(ADT_DESCRIPTOR_SECTION_TAG, *b"ADT0");
        // D2-2: SEMCOD22 is supported; admission additionally requires a
        // strictly valid ADT0 section (see semcode_decode).
        assert!(supported_headers().contains(&HEADER_V22));
        assert_eq!(header_spec_from_magic(&MAGIC22), Some(HEADER_V22));
    }

    #[test]
    fn shf3a2_u32_header_vocabulary_is_defined_and_admitted() {
        assert_eq!(HEADER_V23.magic, *b"SEMCOD23");
        assert_eq!(HEADER_V23.rev, 24);
        assert_eq!(HEADER_V23.epoch, HEADER_V22.epoch);
        assert_eq!(HEADER_V23.capabilities, HEADER_V22.capabilities);
        assert_eq!(SEMCODE_U32_ARITH_MIN_REVISION, 24);
        const { assert!(SEMCODE_U32_ARITH_MIN_REVISION > HEADER_V22.rev) };
        assert!(supported_headers().contains(&HEADER_V23));
        assert_eq!(header_spec_from_magic(&MAGIC23), Some(HEADER_V23));
    }

    #[test]
    fn shf2a2_bytes_header_vocabulary_is_defined_and_admitted() {
        assert_eq!(HEADER_V24.magic, *b"SEMCOD24");
        assert_eq!(HEADER_V24.rev, 25);
        assert_eq!(HEADER_V24.epoch, HEADER_V23.epoch);
        assert_eq!(
            HEADER_V24.capabilities,
            HEADER_V23.capabilities | CAP_BYTES_VALUES
        );
        assert_eq!(SEMCODE_BYTES_MIN_REVISION, 25);
        const { assert!(SEMCODE_BYTES_MIN_REVISION > HEADER_V23.rev) };
        assert_eq!(supported_headers().last(), Some(&HEADER_V24));
        assert_eq!(header_spec_from_magic(&MAGIC24), Some(HEADER_V24));
        assert_eq!(CallableValueFamily::Bytes.byte(), 15);
        assert_eq!(
            CallableValueFamily::from_byte(15),
            Ok(CallableValueFamily::Bytes)
        );
    }

    fn encoded(table: &AdtDescriptorTable) -> Vec<AdtDescriptor> {
        table.descriptors().to_vec()
    }

    #[test]
    fn strict_constructor_accepts_canonical_tables_exactly_as_encoded() {
        for user in [vec![], vec![adt("E", &[("A", 0), ("B", 2)])]] {
            let canonical = AdtDescriptorTable::with_builtins(user).expect("table");
            assert_eq!(
                AdtDescriptorTable::from_encoded_strict(encoded(&canonical)),
                Ok(canonical)
            );
        }
    }

    #[test]
    fn strict_constructor_never_repairs_missing_or_non_canonical_builtins() {
        let canonical = encoded(&AdtDescriptorTable::with_builtins(vec![]).expect("table"));
        let without = |name: &str| {
            canonical
                .iter()
                .filter(|d| d.name() != name)
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(
            AdtDescriptorTable::from_encoded_strict(without("Option")),
            Err("ADT0 is missing the built-in Option descriptor")
        );
        assert_eq!(
            AdtDescriptorTable::from_encoded_strict(without("Result")),
            Err("ADT0 is missing the built-in Result descriptor")
        );
        let replace = |name: &str, variants: &[(&str, u16)]| {
            canonical
                .iter()
                .map(|d| {
                    if d.name() == name {
                        adt(name, variants)
                    } else {
                        d.clone()
                    }
                })
                .collect::<Vec<_>>()
        };
        for bad in [
            replace("Option", &[("None", 0), ("Some", 2)]),
            replace("Option", &[("Some", 1), ("None", 0)]),
            replace("Option", &[("Nothing", 0), ("Some", 1)]),
        ] {
            assert_eq!(
                AdtDescriptorTable::from_encoded_strict(bad),
                Err("ADT0 Option descriptor is not the canonical [None/0, Some/1]")
            );
        }
        for bad in [
            replace("Result", &[("Ok", 0), ("Err", 1)]),
            replace("Result", &[("Err", 1), ("Ok", 1)]),
        ] {
            assert_eq!(
                AdtDescriptorTable::from_encoded_strict(bad),
                Err("ADT0 Result descriptor is not the canonical [Ok/1, Err/1]")
            );
        }
    }

    #[test]
    fn strict_constructor_rejects_unsorted_or_duplicate_descriptors_without_sorting() {
        let mut reversed = encoded(&AdtDescriptorTable::with_builtins(vec![]).expect("table"));
        reversed.reverse();
        assert_eq!(
            AdtDescriptorTable::from_encoded_strict(reversed),
            Err("ADT0 descriptors are not in strictly ascending name-byte order")
        );
        let mut duplicated = encoded(&AdtDescriptorTable::with_builtins(vec![]).expect("table"));
        duplicated.insert(0, duplicated[0].clone());
        assert_eq!(
            AdtDescriptorTable::from_encoded_strict(duplicated),
            Err("ADT0 repeats a descriptor name")
        );
    }

    #[test]
    fn get_resolves_descriptors_by_name_not_by_position() {
        let table = AdtDescriptorTable::with_builtins(vec![adt("Z", &[("Z0", 3)]), adt("A", &[])])
            .expect("table");
        assert_eq!(names(&table), ["A", "Option", "Result", "Z"]);
        assert_eq!(table.get("Z").map(|d| d.variants().len()), Some(1));
        assert_eq!(table.get("A").map(|d| d.variants().len()), Some(0));
        assert_eq!(table.get("Option"), Some(&table.descriptors()[1]));
        assert_eq!(table.get("Missing"), None);
    }
}
