use crate::local_format::*;

pub const MAX_FUNCTIONS: usize = 1024;
pub const MAX_STRING_LEN: usize = 1024;
pub const MAX_STRINGS_PER_FUNCTION: usize = 256;
pub const MAX_DEBUG_SYMBOLS_PER_FUNCTION: usize = 8192;
pub const MAX_SIGNATURE_PARAMETERS_PER_FUNCTION: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    BadHeader,
    UnsupportedVersion { found: String, supported: String },
    TruncatedFunction { offset: usize, msg: &'static str },
    InvalidFunctionName { offset: usize, msg: &'static str },
    InvalidStringTable { offset: usize, msg: &'static str },
    InvalidDebugSection { offset: usize, msg: &'static str },
    InvalidOwnershipSection { offset: usize, msg: &'static str },
    InvalidSignatureSection { offset: usize, msg: &'static str },
    ResourceLimit { offset: usize, msg: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedDebugSymbol {
    pub pc: usize,
    pub line: u32,
    pub col: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodedAccessPathComponent {
    TupleIndex(u16),
    FieldSymbol(u32),
    AdtPayload { variant: u32, index: u16 },
    SequenceIndexStatic(u32),
}

/// #1726 Checkpoint D2a: a Borrow event's resolved activation authority, as
/// carried on the wire from `SEMCODE_OWNERSHIP_ANCHOR_MIN_REVISION` onward.
/// `StoreVarSite`'s `u32` is an `ExecutableAnchor` value (a byte offset
/// relative to the function's own `instr_start`, the same domain `sm-vm`'s
/// `Frame.pc` already uses) - `sm-format` decodes it as an opaque `u32` and
/// makes no claim about what it points at; that cross-check is a separate,
/// later checkpoint (verifier admission, D2b). Always `None` for Write events
/// and for any Borrow event decoded under a pre-anchor revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodedBorrowActivation {
    FrameEntry,
    StoreVarSite(u32),
}

/// #1891 Checkpoint W2D: a Write event's resolved execution-site class, as
/// carried on the wire from `SEMCODE_OWNERSHIP_ANCHOR_MIN_REVISION` onward.
/// Both variants' `u32` is an `ExecutableAnchor` value (a byte offset
/// relative to the function's own `instr_start`) - `sm-format` decodes it as
/// an opaque `u32` and makes no claim about what it points at; that
/// cross-check is a separate, later checkpoint (verifier admission, W2E).
/// Deliberately a distinct type from `DecodedBorrowActivation`, never
/// conflated with it even though both are revision-gated at the same
/// floor and occupy the same wire position within their own event kind -
/// `None` for Borrow events, and for any Write event decoded under a
/// pre-anchor revision (an explicit "legacy/no execution-mode information"
/// state, never a manufactured anchor).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodedWriteExecution {
    StoreVarSite(u32),
    MakeRecordSite(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedAccessPath {
    pub root_symbol_id: u32,
    pub components: Vec<DecodedAccessPathComponent>,
    pub activation: Option<DecodedBorrowActivation>,
    pub write_execution: Option<DecodedWriteExecution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedFunctionEnvelope<'a> {
    pub name: String,
    pub name_offset: usize,
    pub code_offset: usize,
    pub code_len: usize,
    pub strings: Vec<String>,
    pub debug_symbols: Vec<DecodedDebugSymbol>,
    pub borrowed_paths: Vec<DecodedAccessPath>,
    pub write_paths: Vec<DecodedAccessPath>,
    pub has_ownership_section: bool,
    // Whether the `DBG0` sentinel was recognized for this function. This is
    // a purely structural fact (no opcode/operand knowledge), kept so
    // callers can tell "no debug section" apart from "an empty debug
    // section" - both currently produce an empty `debug_symbols`, but only
    // the latter means the DBG0-sniff branch was taken and
    // `instr_start_offset` was advanced past it. See #1731: the DBG0
    // sentinel collides with `TupleGet`'s opcode byte (0x44 = 'D'), so a
    // producer-emitted instruction stream can coincidentally spell the same
    // six bytes as an empty DBG0 section. This flag lets sm-verify check,
    // at its own admission boundary, whether the bytes between the string
    // table and `instr_start_offset` also form a structurally valid
    // instruction stream under the alternative (no-DBG0) reading, and
    // reject as ambiguous if so - without sm-format needing any opcode/
    // operand-shape knowledge itself.
    pub has_debug_section: bool,
    // Cursor position (relative to code_slice) immediately after the
    // string table, before any DBG0/OWN0 sentinel is sniffed. Always
    // well-defined; equals `instr_start_offset` when neither section is
    // present.
    pub string_table_end_offset: usize,
    // #1773 (FA-09-005): the canonical callable-signature record for this
    // function, if present. Unlike `has_debug_section`/`has_ownership_section`
    // (which are content-sniffed - the DBG0/OWN0 tags are looked for
    // regardless of header revision, see #1731's TupleGet/DBG0 byte
    // collision), signature presence is derived *deterministically* from
    // `header.rev >= SEMCODE_SIGNATURE_MIN_REVISION` and never sniffed: a
    // pre-#1773 header structurally cannot carry a SIG0 section, so there is
    // nothing to (mis)interpret in its instruction stream as one. `None`
    // means "this artifact's header predates canonical callable signatures";
    // it is never used to mean "signature section present but empty" - a
    // present, zero-parameter signature decodes as `Some(CallableSignature
    // { families: vec![] })`.
    pub signature: Option<CallableSignature>,
    pub instr_start_offset: usize, // relative to code_slice
    pub code_slice: &'a [u8],      // the full code block for this function
}

struct StringTableDebugOwnershipParse {
    strings: Vec<String>,
    debug_symbols: Vec<DecodedDebugSymbol>,
    borrowed_paths: Vec<DecodedAccessPath>,
    write_paths: Vec<DecodedAccessPath>,
    has_ownership_section: bool,
    has_debug_section: bool,
    string_table_end_offset: usize,
    signature: Option<CallableSignature>,
    instr_start_offset: usize,
}

pub fn decode_semcode_envelope<'a>(
    bytes: &'a [u8],
) -> Result<(SemcodeHeaderSpec, Vec<DecodedFunctionEnvelope<'a>>), DecodeError> {
    if bytes.len() < 8 {
        return Err(DecodeError::BadHeader);
    }
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let Some(header) = header_spec_from_magic(&magic) else {
        let found = String::from_utf8_lossy(&magic).to_string();
        let supported = supported_headers()
            .iter()
            .map(|h| String::from_utf8_lossy(&h.magic).to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(DecodeError::UnsupportedVersion { found, supported });
    };

    let mut cursor = 8usize;
    let mut functions = Vec::new();

    while cursor < bytes.len() {
        if functions.len() >= MAX_FUNCTIONS {
            return Err(DecodeError::ResourceLimit {
                offset: cursor,
                msg: format!("too many functions (>{})", MAX_FUNCTIONS),
            });
        }

        let name_offset = cursor;
        let name_len =
            read_u16_le(bytes, &mut cursor).map_err(|_| DecodeError::TruncatedFunction {
                offset: name_offset,
                msg: "missing function name length",
            })? as usize;

        if name_len == 0 {
            return Err(DecodeError::InvalidFunctionName {
                offset: cursor,
                msg: "empty function name",
            });
        }
        if name_len > MAX_STRING_LEN {
            return Err(DecodeError::InvalidFunctionName {
                offset: cursor,
                msg: "function name too long",
            });
        }

        let name = read_utf8(bytes, &mut cursor, name_len).map_err(|_| {
            DecodeError::InvalidFunctionName {
                offset: cursor,
                msg: "invalid utf8 in function name",
            }
        })?;

        let code_len_offset = cursor;
        let code_len =
            read_u32_le(bytes, &mut cursor).map_err(|_| DecodeError::TruncatedFunction {
                offset: code_len_offset,
                msg: "missing function code length",
            })? as usize;

        let code_end =
            checked_end(cursor, code_len, bytes.len()).ok_or(DecodeError::TruncatedFunction {
                offset: cursor,
                msg: "function code out of bounds",
            })?;

        let code_offset = cursor;
        let code_slice = &bytes[cursor..code_end];
        cursor = code_end;

        let parsed = parse_string_table_debug_and_ownership(code_offset, code_slice, header.rev)?;

        functions.push(DecodedFunctionEnvelope {
            name,
            name_offset,
            code_offset,
            code_len,
            strings: parsed.strings,
            debug_symbols: parsed.debug_symbols,
            borrowed_paths: parsed.borrowed_paths,
            write_paths: parsed.write_paths,
            has_ownership_section: parsed.has_ownership_section,
            has_debug_section: parsed.has_debug_section,
            string_table_end_offset: parsed.string_table_end_offset,
            signature: parsed.signature,
            instr_start_offset: parsed.instr_start_offset,
            code_slice,
        });
    }

    Ok((header, functions))
}

/// #1736 (FA-05-006): `diag_offset(base_offset, cursor)` for a diagnostic error-offset
/// field only - every call site here is already inside an error-construction
/// closure triggered by an already-failed `read_*` call, so this value never
/// gates an accept/reject decision. `saturating_add` (rather than raw `+`)
/// avoids ever panicking while building an error message; unlike the real
/// bounds-check arithmetic in `checked_read_end` and
/// `decode_semcode_envelope` - where wrapping/saturating is banned because
/// it could hide an out-of-bounds read behind a false-accept - saturating
/// here only affects a cosmetic offset field after rejection has already
/// been decided by the failed `read_*` call.
fn diag_offset(base_offset: usize, cursor: usize) -> usize {
    base_offset.saturating_add(cursor)
}

/// #1736 (FA-05-006): the one shared, width-independent bounds-check
/// primitive for every real accept/reject decision in this file - the
/// function `code_len` check and the `DBG0`/`OWN0` section tag-sniffs below.
/// Mirrors `local_format.rs`'s `checked_read_end`, but returns `Option`
/// rather than a format-specific `Result` since each call site here maps
/// the `None` case to its own distinct `DecodeError` variant (or, for the
/// tag-sniffs, treats it as "optional section absent" - see the doc comment
/// on the `DBG0` check below for why that is not a rejection). Always uses
/// `checked_add`, never raw `+`, so a `cursor`/`len` combination that would
/// overflow `usize` (reachable from `code_len`, a fully attacker-controlled
/// `u32` field, on a 32-bit target) is treated exactly like an ordinary
/// out-of-bounds length, never a wrapped false-accept.
fn checked_end(cursor: usize, len: usize, total: usize) -> Option<usize> {
    cursor.checked_add(len).filter(|&end| end <= total)
}

fn parse_string_table_debug_and_ownership(
    base_offset: usize,
    code: &[u8],
    header_rev: u16,
) -> Result<StringTableDebugOwnershipParse, DecodeError> {
    let mut cursor = 0usize;
    let string_count_offset = diag_offset(base_offset, cursor);
    let count = read_u16_le(code, &mut cursor).map_err(|_| DecodeError::InvalidStringTable {
        offset: string_count_offset,
        msg: "missing string table header",
    })? as usize;

    if count > MAX_STRINGS_PER_FUNCTION {
        return Err(DecodeError::ResourceLimit {
            offset: diag_offset(base_offset, cursor),
            msg: format!(
                "too many strings in function: {} (max {})",
                count, MAX_STRINGS_PER_FUNCTION
            ),
        });
    }

    let mut strings = Vec::with_capacity(count);
    for _ in 0..count {
        let len_offset = diag_offset(base_offset, cursor);
        let len = read_u16_le(code, &mut cursor).map_err(|_| DecodeError::InvalidStringTable {
            offset: len_offset,
            msg: "missing string length",
        })? as usize;

        if len > MAX_STRING_LEN {
            return Err(DecodeError::InvalidStringTable {
                offset: diag_offset(base_offset, cursor),
                msg: "string too long in function string table",
            });
        }

        let str_val =
            read_utf8(code, &mut cursor, len).map_err(|_| DecodeError::InvalidStringTable {
                offset: diag_offset(base_offset, cursor),
                msg: "invalid utf8 in string table",
            })?;
        strings.push(str_val);
    }

    let string_table_end_offset = cursor;
    let mut has_debug_section = false;
    let mut debug_symbols = Vec::new();
    // #1736: `checked_end` here only guards the lookahead slice read
    // (`&code[cursor..end]`) against an out-of-bounds panic - it does NOT
    // gate accept/reject. A `None`/non-matching result means "no DBG0 tag
    // here", which is the ordinary, successful case for a function with no
    // debug section: the block below is skipped and decoding proceeds to
    // `Ok(...)`. Genuine corruption of a section that IS present (a
    // truncated count or entry) is still caught deterministically by the
    // `read_*` calls inside the block, same as everywhere else in this file.
    let dbg_tag_end = checked_end(cursor, 4, code.len());
    let is_dbg0 = dbg_tag_end.is_some_and(|end| &code[cursor..end] == b"DBG0");
    if is_dbg0 {
        has_debug_section = true;
        cursor = dbg_tag_end.expect("is_dbg0 implies dbg_tag_end is Some");
        let dbg_count_offset = diag_offset(base_offset, cursor);
        let count =
            read_u16_le(code, &mut cursor).map_err(|_| DecodeError::InvalidDebugSection {
                offset: dbg_count_offset,
                msg: "missing debug symbol count",
            })? as usize;

        if count > MAX_DEBUG_SYMBOLS_PER_FUNCTION {
            return Err(DecodeError::ResourceLimit {
                offset: diag_offset(base_offset, cursor),
                msg: format!(
                    "too many debug symbols: {} (max {})",
                    count, MAX_DEBUG_SYMBOLS_PER_FUNCTION
                ),
            });
        }

        debug_symbols.reserve(count);
        for _ in 0..count {
            let entry_offset = diag_offset(base_offset, cursor);
            let pc =
                read_u32_le(code, &mut cursor).map_err(|_| DecodeError::InvalidDebugSection {
                    offset: entry_offset,
                    msg: "missing debug pc",
                })? as usize;
            let line =
                read_u32_le(code, &mut cursor).map_err(|_| DecodeError::InvalidDebugSection {
                    offset: diag_offset(base_offset, cursor),
                    msg: "missing debug line",
                })?;
            let col =
                read_u16_le(code, &mut cursor).map_err(|_| DecodeError::InvalidDebugSection {
                    offset: diag_offset(base_offset, cursor),
                    msg: "missing debug col",
                })?;
            debug_symbols.push(DecodedDebugSymbol { pc, line, col });
        }
    }

    let mut borrowed_paths = Vec::new();
    let mut write_paths = Vec::new();
    let mut has_ownership_section = false;
    // #1736: same "lookahead probe, not a rejection" semantics as the DBG0
    // check above - see its doc comment.
    let own_tag_end = checked_end(cursor, OWNERSHIP_SECTION_TAG.len(), code.len());
    let is_own0 = own_tag_end.is_some_and(|end| code[cursor..end] == OWNERSHIP_SECTION_TAG);
    if is_own0 {
        has_ownership_section = true;
        cursor = own_tag_end.expect("is_own0 implies own_tag_end is Some");
        let own_count_offset = diag_offset(base_offset, cursor);
        let count =
            read_u16_le(code, &mut cursor).map_err(|_| DecodeError::InvalidOwnershipSection {
                offset: own_count_offset,
                msg: "missing ownership path count",
            })? as usize;

        borrowed_paths.reserve(count);
        write_paths.reserve(count);
        for _ in 0..count {
            let entry_offset = diag_offset(base_offset, cursor);
            let kind =
                read_u8(code, &mut cursor).map_err(|_| DecodeError::InvalidOwnershipSection {
                    offset: entry_offset,
                    msg: "missing ownership event kind",
                })?;
            // #1726 Checkpoint D2a / #1891 Checkpoint W2D: the mode tag
            // exists ONLY at/above the anchor revision, and its meaning
            // depends on the event's own kind - Borrow's activation mode and
            // Write's execution mode occupy the identical wire position
            // (right after `kind`) but are decoded into two entirely
            // separate types, never coupled just because their numeric tags
            // happen to overlap (0/1 in both). The header revision is the
            // sole grammar authority (no sniffing, no try-then-fallback). An
            // event kind that is neither Borrow nor Write reads no mode byte
            // here at any revision; the `unsupported ownership event kind`
            // rejection below still catches it.
            let mut activation = None;
            let mut write_execution = None;
            if header_rev >= SEMCODE_OWNERSHIP_ANCHOR_MIN_REVISION {
                match kind {
                    OWNERSHIP_EVENT_KIND_BORROW => {
                        let mode = read_u8(code, &mut cursor).map_err(|_| {
                            DecodeError::InvalidOwnershipSection {
                                offset: diag_offset(base_offset, cursor),
                                msg: "missing borrow activation mode",
                            }
                        })?;
                        activation = Some(match mode {
                            ACTIVATION_MODE_FRAME_ENTRY => DecodedBorrowActivation::FrameEntry,
                            ACTIVATION_MODE_STORE_VAR_SITE => {
                                let anchor = read_u32_le(code, &mut cursor).map_err(|_| {
                                    DecodeError::InvalidOwnershipSection {
                                        offset: diag_offset(base_offset, cursor),
                                        msg: "missing borrow executable anchor",
                                    }
                                })?;
                                DecodedBorrowActivation::StoreVarSite(anchor)
                            }
                            _ => {
                                return Err(DecodeError::InvalidOwnershipSection {
                                    offset: diag_offset(base_offset, cursor),
                                    msg: "unrecognized borrow activation mode",
                                });
                            }
                        });
                    }
                    OWNERSHIP_EVENT_KIND_WRITE => {
                        let mode = read_u8(code, &mut cursor).map_err(|_| {
                            DecodeError::InvalidOwnershipSection {
                                offset: diag_offset(base_offset, cursor),
                                msg: "missing write execution mode",
                            }
                        })?;
                        write_execution = Some(match mode {
                            WRITE_EXECUTION_MODE_STORE_VAR_SITE => {
                                let anchor = read_u32_le(code, &mut cursor).map_err(|_| {
                                    DecodeError::InvalidOwnershipSection {
                                        offset: diag_offset(base_offset, cursor),
                                        msg: "missing write executable anchor",
                                    }
                                })?;
                                DecodedWriteExecution::StoreVarSite(anchor)
                            }
                            WRITE_EXECUTION_MODE_MAKE_RECORD_SITE => {
                                let anchor = read_u32_le(code, &mut cursor).map_err(|_| {
                                    DecodeError::InvalidOwnershipSection {
                                        offset: diag_offset(base_offset, cursor),
                                        msg: "missing write executable anchor",
                                    }
                                })?;
                                DecodedWriteExecution::MakeRecordSite(anchor)
                            }
                            _ => {
                                return Err(DecodeError::InvalidOwnershipSection {
                                    offset: diag_offset(base_offset, cursor),
                                    msg: "unrecognized write execution mode",
                                });
                            }
                        });
                    }
                    _ => {}
                }
            }
            let root_symbol_id = read_u32_le(code, &mut cursor).map_err(|_| {
                DecodeError::InvalidOwnershipSection {
                    offset: diag_offset(base_offset, cursor),
                    msg: "missing ownership path root",
                }
            })?;
            let component_count = read_u16_le(code, &mut cursor).map_err(|_| {
                DecodeError::InvalidOwnershipSection {
                    offset: diag_offset(base_offset, cursor),
                    msg: "missing ownership path component count",
                }
            })? as usize;

            let mut components = Vec::new();
            for _ in 0..component_count {
                let component_kind = read_u8(code, &mut cursor).map_err(|_| {
                    DecodeError::InvalidOwnershipSection {
                        offset: diag_offset(base_offset, cursor),
                        msg: "missing ownership path component kind",
                    }
                })?;
                match component_kind {
                    OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX => {
                        let index = read_u16_le(code, &mut cursor).map_err(|_| {
                            DecodeError::InvalidOwnershipSection {
                                offset: diag_offset(base_offset, cursor),
                                msg: "missing ownership tuple index",
                            }
                        })?;
                        components.push(DecodedAccessPathComponent::TupleIndex(index));
                    }
                    OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL => {
                        let field = read_u32_le(code, &mut cursor).map_err(|_| {
                            DecodeError::InvalidOwnershipSection {
                                offset: diag_offset(base_offset, cursor),
                                msg: "missing ownership field symbol",
                            }
                        })?;
                        components.push(DecodedAccessPathComponent::FieldSymbol(field));
                    }
                    OWNERSHIP_PATH_COMPONENT_ADT_PAYLOAD => {
                        let variant = read_u32_le(code, &mut cursor).map_err(|_| {
                            DecodeError::InvalidOwnershipSection {
                                offset: diag_offset(base_offset, cursor),
                                msg: "missing ownership adt payload variant",
                            }
                        })?;
                        let index = read_u16_le(code, &mut cursor).map_err(|_| {
                            DecodeError::InvalidOwnershipSection {
                                offset: diag_offset(base_offset, cursor),
                                msg: "missing ownership adt payload index",
                            }
                        })?;
                        components.push(DecodedAccessPathComponent::AdtPayload { variant, index });
                    }
                    OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX => {
                        let index = read_u32_le(code, &mut cursor).map_err(|_| {
                            DecodeError::InvalidOwnershipSection {
                                offset: diag_offset(base_offset, cursor),
                                msg: "missing ownership sequence index",
                            }
                        })?;
                        components.push(DecodedAccessPathComponent::SequenceIndexStatic(index));
                    }
                    _ => {
                        return Err(DecodeError::InvalidOwnershipSection {
                            offset: diag_offset(base_offset, cursor),
                            msg: "unsupported ownership path component kind",
                        });
                    }
                }
            }
            match kind {
                OWNERSHIP_EVENT_KIND_BORROW => borrowed_paths.push(DecodedAccessPath {
                    root_symbol_id,
                    components,
                    activation,
                    write_execution: None,
                }),
                OWNERSHIP_EVENT_KIND_WRITE => write_paths.push(DecodedAccessPath {
                    root_symbol_id,
                    components,
                    activation: None,
                    write_execution,
                }),
                _ => {
                    return Err(DecodeError::InvalidOwnershipSection {
                        offset: diag_offset(base_offset, cursor),
                        msg: "unsupported ownership event kind",
                    });
                }
            }
        }
    }

    // #1773 (FA-09-005): deliberately NOT a content-sniff like DBG0/OWN0
    // above. Signature-record presence is a structural fact of the header
    // revision alone, never of instruction-stream content, so a pre-#1773
    // header's instruction stream is never even inspected for a coincidental
    // "SIG0" byte match - closing off the collision class #1731 documented
    // for DBG0 (TupleGet's opcode byte spells 'D') before it could recur
    // here. `header_rev >= SEMCODE_SIGNATURE_MIN_REVISION` is exactly the
    // condition `sm-ir`'s emitter uses to decide whether to write this
    // section (see `emit_semcode_function`), so decode and emit agree by
    // construction.
    let signature = if header_rev >= SEMCODE_SIGNATURE_MIN_REVISION {
        // #1773 review follow-up: every header at or above
        // SEMCODE_SIGNATURE_MIN_REVISION carries CAP_OWNERSHIP_PATHS
        // (HEADER_V19 inherits it unchanged from V18/V11, and capabilities
        // only ever grow across revisions in this format - never shrink -
        // so this holds for any future revision too), meaning OWN0 is
        // just as mandatory per function as SIG0 is. Before this check,
        // a function could omit or strip its own OWN0 section entirely
        // and still decode a SIG0 placed right after the string table -
        // sm-verify's program-wide ownership check only proves *some*
        // function in the artifact has OWN0 (`.any(...)`), not that
        // *this* one does, so a multi-function artifact could smuggle one
        // OWN0-less function past admission and have the VM execute it
        // with no borrow/write path enforcement at all. Rejecting here,
        // before SIG0 is even inspected, closes that per-function gap
        // structurally rather than relying on a coarser whole-program
        // policy check.
        if !has_ownership_section {
            return Err(DecodeError::InvalidOwnershipSection {
                offset: diag_offset(base_offset, cursor),
                msg: "header requires per-function ownership-path metadata but no OWN0 section is present",
            });
        }
        let tag_offset = diag_offset(base_offset, cursor);
        let tag_end = checked_end(cursor, SIGNATURE_SECTION_TAG.len(), code.len()).ok_or(
            DecodeError::InvalidSignatureSection {
                offset: tag_offset,
                msg: "missing SIG0 section tag",
            },
        )?;
        if code[cursor..tag_end] != SIGNATURE_SECTION_TAG {
            return Err(DecodeError::InvalidSignatureSection {
                offset: tag_offset,
                msg: "missing SIG0 section tag",
            });
        }
        cursor = tag_end;

        let count_offset = diag_offset(base_offset, cursor);
        let count =
            read_u16_le(code, &mut cursor).map_err(|_| DecodeError::InvalidSignatureSection {
                offset: count_offset,
                msg: "missing parameter count",
            })? as usize;

        if count > MAX_SIGNATURE_PARAMETERS_PER_FUNCTION {
            return Err(DecodeError::ResourceLimit {
                offset: diag_offset(base_offset, cursor),
                msg: format!(
                    "too many callable-signature parameters: {} (max {})",
                    count, MAX_SIGNATURE_PARAMETERS_PER_FUNCTION
                ),
            });
        }

        let mut families = Vec::with_capacity(count);
        for _ in 0..count {
            let family_offset = diag_offset(base_offset, cursor);
            let raw =
                read_u8(code, &mut cursor).map_err(|_| DecodeError::InvalidSignatureSection {
                    offset: family_offset,
                    msg: "missing parameter family tag",
                })?;
            let family = CallableValueFamily::from_byte(raw).map_err(|_| {
                DecodeError::InvalidSignatureSection {
                    offset: family_offset,
                    msg: "unknown parameter family tag",
                }
            })?;
            families.push(family);
        }

        Some(CallableSignature { families })
    } else {
        None
    };

    Ok(StringTableDebugOwnershipParse {
        strings,
        debug_symbols,
        borrowed_paths,
        write_paths,
        has_ownership_section,
        has_debug_section,
        string_table_end_offset,
        signature,
        instr_start_offset: cursor,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sequence_ownership_semcode_bytes(index: u32) -> Vec<u8> {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_BORROW);
        code.extend_from_slice(&0u32.to_le_bytes());
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX);
        code.extend_from_slice(&index.to_le_bytes());

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MAGIC11);
        bytes.extend_from_slice(&4u16.to_le_bytes());
        bytes.extend_from_slice(b"main");
        bytes.extend_from_slice(&(code.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&code);
        bytes
    }

    // #1736 (FA-05-006): before the fix, `cursor + code_len` (a fully
    // attacker-controlled u32 field) used raw addition, so a huge claimed
    // code length could wrap `cursor` past zero on a 32-bit target and
    // build a slice whose end sits before its start - an out-of-range slice
    // index panic, not a decode error. This is the exact artifact shape
    // that reproduced `slice index starts at 18 but ends at 2` under
    // `cargo test --target i686-pc-windows-msvc --release`.
    // #1751 (FA-07-011): sm-format is sole owner of the static function-count
    // bound (`MAX_FUNCTIONS`). This confirms the decoder still enforces it
    // directly (independent of, and unaffected by, sm-verify no longer
    // conflating function-definition count with the runtime frame quota).
    fn minimal_function_bytes(name: &str) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(name.len() as u16).to_le_bytes());
        bytes.extend_from_slice(name.as_bytes());
        bytes.extend_from_slice(&2u32.to_le_bytes()); // code_len: 2 bytes
        bytes.extend_from_slice(&0u16.to_le_bytes()); // empty string table
        bytes
    }

    #[test]
    fn decode_accepts_exactly_max_functions() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MAGIC0);
        for i in 0..MAX_FUNCTIONS {
            bytes.extend_from_slice(&minimal_function_bytes(&format!("f{i}")));
        }
        let (_, functions) = decode_semcode_envelope(&bytes).expect("must accept MAX_FUNCTIONS");
        assert_eq!(functions.len(), MAX_FUNCTIONS);
    }

    #[test]
    fn decode_rejects_one_more_than_max_functions() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MAGIC0);
        for i in 0..(MAX_FUNCTIONS + 1) {
            bytes.extend_from_slice(&minimal_function_bytes(&format!("f{i}")));
        }
        let err = decode_semcode_envelope(&bytes).expect_err("must reject over MAX_FUNCTIONS");
        assert!(matches!(err, DecodeError::ResourceLimit { .. }));
    }

    #[test]
    fn decode_rejects_code_len_that_would_overflow_cursor_arithmetic() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MAGIC0);
        bytes.extend_from_slice(&4u16.to_le_bytes());
        bytes.extend_from_slice(b"main");
        bytes.extend_from_slice(&0xFFFFFFF0u32.to_le_bytes());
        let err = decode_semcode_envelope(&bytes).expect_err("must reject, never panic");
        assert!(matches!(err, DecodeError::TruncatedFunction { .. }));
    }

    #[test]
    fn decode_rejects_debug_section_with_truncated_count_deterministically() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes()); // empty string table
        code.extend_from_slice(b"DBG0");
        code.push(0x01); // truncated debug-symbol count (needs 2 bytes)

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MAGIC0);
        bytes.extend_from_slice(&4u16.to_le_bytes());
        bytes.extend_from_slice(b"main");
        bytes.extend_from_slice(&(code.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&code);

        let err = decode_semcode_envelope(&bytes).expect_err("must reject, never panic");
        assert!(matches!(err, DecodeError::InvalidDebugSection { .. }));
    }

    #[test]
    fn decode_rejects_ownership_section_with_truncated_count_deterministically() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes()); // empty string table
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.push(0x01); // truncated ownership-path count (needs 2 bytes)

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MAGIC0);
        bytes.extend_from_slice(&4u16.to_le_bytes());
        bytes.extend_from_slice(b"main");
        bytes.extend_from_slice(&(code.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&code);

        let err = decode_semcode_envelope(&bytes).expect_err("must reject, never panic");
        assert!(matches!(err, DecodeError::InvalidOwnershipSection { .. }));
    }

    #[test]
    fn decode_rejects_string_table_entry_length_over_the_max() {
        let mut code = Vec::new();
        code.extend_from_slice(&1u16.to_le_bytes()); // one string table entry
        code.extend_from_slice(&(MAX_STRING_LEN as u16 + 1).to_le_bytes()); // too long

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MAGIC0);
        bytes.extend_from_slice(&4u16.to_le_bytes());
        bytes.extend_from_slice(b"main");
        bytes.extend_from_slice(&(code.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&code);

        let err = decode_semcode_envelope(&bytes).expect_err("must reject, never panic");
        assert!(matches!(err, DecodeError::InvalidStringTable { .. }));
    }

    // #1736 (FA-05-006): width-independent regression coverage for
    // `checked_end`, the one shared primitive backing the `code_len` check
    // and the `DBG0`/`OWN0` tag-sniffs. Forcing `cursor` to `usize::MAX - 1`
    // directly exercises the overflow path on every target width, including
    // ordinary 64-bit CI - unlike `decode_rejects_code_len_that_would_overflow_cursor_arithmetic`
    // above, whose `code_len` is bounded to `u32` and so can only overflow a
    // small starting `cursor` on an actual 32-bit target. This closes the
    // gap without depending on a manual `--target i686-pc-windows-msvc` run
    // (still genuinely verified once; see the PR/issue evidence) ever being
    // repeated to catch a regression.
    #[test]
    fn checked_end_rejects_cursor_near_usize_max_on_any_target_width() {
        assert_eq!(checked_end(usize::MAX - 1, 4, 100), None);
        assert_eq!(checked_end(usize::MAX - 1, usize::MAX, 100), None);
    }

    #[test]
    fn checked_end_still_accepts_ordinary_in_bounds_input() {
        assert_eq!(checked_end(4, 4, 100), Some(8));
        assert_eq!(checked_end(96, 4, 100), Some(100));
    }

    #[test]
    fn checked_end_still_rejects_ordinary_truncation() {
        assert_eq!(checked_end(97, 4, 100), None);
    }

    #[test]
    fn decode_sequence_index_static_ownership_component() {
        let bytes = sequence_ownership_semcode_bytes(7);
        let (_, functions) = decode_semcode_envelope(&bytes).expect("decode");
        let env = &functions[0];
        assert_eq!(env.borrowed_paths.len(), 1);
        assert_eq!(
            env.borrowed_paths[0].components,
            vec![DecodedAccessPathComponent::SequenceIndexStatic(7)]
        );
    }

    // #1773 (FA-09-005) format tests: the SIG0 section.

    fn function_bytes_with_header(magic: [u8; 8], name: &str, code: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&magic);
        bytes.extend_from_slice(&(name.len() as u16).to_le_bytes());
        bytes.extend_from_slice(name.as_bytes());
        bytes.extend_from_slice(&(code.len() as u32).to_le_bytes());
        bytes.extend_from_slice(code);
        bytes
    }

    #[test]
    fn decode_rev20_header_roundtrips_multi_family_signature() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes()); // empty string table
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&0u16.to_le_bytes()); // empty ownership path count
        code.extend_from_slice(&SIGNATURE_SECTION_TAG);
        code.extend_from_slice(&3u16.to_le_bytes());
        code.push(CallableValueFamily::I32.byte());
        code.push(CallableValueFamily::Bool.byte());
        code.push(CallableValueFamily::Adt.byte());

        let bytes = function_bytes_with_header(MAGIC19, "main", &code);
        let (header, functions) = decode_semcode_envelope(&bytes).expect("decode");
        assert_eq!(header.rev, SEMCODE_SIGNATURE_MIN_REVISION);
        let sig = functions[0].signature.as_ref().expect("signature present");
        assert_eq!(
            sig.families,
            vec![
                CallableValueFamily::I32,
                CallableValueFamily::Bool,
                CallableValueFamily::Adt,
            ]
        );
    }

    #[test]
    fn decode_rev20_header_accepts_zero_parameter_signature() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&SIGNATURE_SECTION_TAG);
        code.extend_from_slice(&0u16.to_le_bytes());

        let bytes = function_bytes_with_header(MAGIC19, "main", &code);
        let (_, functions) = decode_semcode_envelope(&bytes).expect("decode");
        let sig = functions[0].signature.as_ref().expect("signature present");
        assert!(sig.families.is_empty());
    }

    #[test]
    fn decode_pre_rev20_header_never_has_a_signature() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());

        let bytes = function_bytes_with_header(MAGIC18, "main", &code);
        let (_, functions) = decode_semcode_envelope(&bytes).expect("decode");
        assert_eq!(functions[0].signature, None);
    }

    #[test]
    fn decode_rev20_header_rejects_missing_signature_section() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&0u16.to_le_bytes());
        // No SIG0 tag follows, even though the header requires one.

        let bytes = function_bytes_with_header(MAGIC19, "main", &code);
        let err = decode_semcode_envelope(&bytes).expect_err("must reject, never fall back");
        assert!(matches!(err, DecodeError::InvalidSignatureSection { .. }));
    }

    // #1773 review follow-up: a rev20+ function that strips its own OWN0
    // section and places SIG0 immediately after the string table must be
    // rejected, not silently decoded as "no ownership metadata for this
    // function". Every header at SEMCODE_SIGNATURE_MIN_REVISION or above
    // carries CAP_OWNERSHIP_PATHS, so OWN0 is exactly as mandatory per
    // function as SIG0 is - without this check, sm-verify's program-wide
    // `.any(has_ownership_section)` policy check could be satisfied by a
    // sibling function in the same multi-function artifact while this one
    // silently loses all borrow/write path enforcement.
    #[test]
    fn decode_rev20_header_rejects_function_with_sig0_but_no_own0() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes()); // empty string table
                                                     // OWN0 deliberately omitted - SIG0 placed directly after it.
        code.extend_from_slice(&SIGNATURE_SECTION_TAG);
        code.extend_from_slice(&0u16.to_le_bytes());

        let bytes = function_bytes_with_header(MAGIC19, "main", &code);
        let err = decode_semcode_envelope(&bytes)
            .expect_err("a rev20+ function missing OWN0 must be rejected before SIG0 is trusted");
        assert!(matches!(err, DecodeError::InvalidOwnershipSection { .. }));
    }

    /// The same exploit, but with a second, well-formed function in the
    /// same artifact - proving the rejection is genuinely per-function, not
    /// something a sibling function's real OWN0 section could paper over
    /// via sm-verify's whole-program `.any(...)` check.
    #[test]
    fn decode_rev20_header_rejects_own0_less_function_even_with_a_well_formed_sibling() {
        let mut good_code = Vec::new();
        good_code.extend_from_slice(&0u16.to_le_bytes());
        good_code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        good_code.extend_from_slice(&0u16.to_le_bytes());
        good_code.extend_from_slice(&SIGNATURE_SECTION_TAG);
        good_code.extend_from_slice(&0u16.to_le_bytes());

        let mut bad_code = Vec::new();
        bad_code.extend_from_slice(&0u16.to_le_bytes());
        bad_code.extend_from_slice(&SIGNATURE_SECTION_TAG);
        bad_code.extend_from_slice(&0u16.to_le_bytes());

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MAGIC19);
        for (name, code) in [("good", &good_code), ("bad", &bad_code)] {
            bytes.extend_from_slice(&(name.len() as u16).to_le_bytes());
            bytes.extend_from_slice(name.as_bytes());
            bytes.extend_from_slice(&(code.len() as u32).to_le_bytes());
            bytes.extend_from_slice(code);
        }

        let err = decode_semcode_envelope(&bytes)
            .expect_err("a well-formed sibling function must not excuse this one's missing OWN0");
        assert!(matches!(err, DecodeError::InvalidOwnershipSection { .. }));
    }

    #[test]
    fn decode_rev20_header_rejects_truncated_signature_count() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&SIGNATURE_SECTION_TAG);
        code.push(0x01); // truncated count (needs 2 bytes)

        let bytes = function_bytes_with_header(MAGIC19, "main", &code);
        let err = decode_semcode_envelope(&bytes).expect_err("must reject, never panic");
        assert!(matches!(err, DecodeError::InvalidSignatureSection { .. }));
    }

    #[test]
    fn decode_rev20_header_rejects_invalid_family_tag() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&SIGNATURE_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(0); // family tag 0 is deliberately never valid
        code.push(0xff); // pad so this isn't also a truncation

        let bytes = function_bytes_with_header(MAGIC19, "main", &code);
        let err = decode_semcode_envelope(&bytes).expect_err("must reject, never panic");
        assert!(matches!(err, DecodeError::InvalidSignatureSection { .. }));
    }

    #[test]
    fn decode_rev20_header_rejects_arity_family_count_desync_by_construction() {
        // A SIG0 count of 2 but only one family byte on the wire: the decoder
        // must reject rather than ever produce a `CallableSignature` whose
        // `families.len()` disagrees with the declared count.
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&SIGNATURE_SECTION_TAG);
        code.extend_from_slice(&2u16.to_le_bytes());
        code.push(CallableValueFamily::I32.byte());

        let bytes = function_bytes_with_header(MAGIC19, "main", &code);
        let err = decode_semcode_envelope(&bytes).expect_err("must reject, never panic");
        assert!(matches!(err, DecodeError::InvalidSignatureSection { .. }));
    }

    #[test]
    fn decode_rejects_truncated_sequence_index_static_payload() {
        let mut bytes = sequence_ownership_semcode_bytes(7);
        let code_len_pos = 8 + 2 + 4;
        let code_len = u32::from_le_bytes(
            bytes[code_len_pos..code_len_pos + 4]
                .try_into()
                .expect("code len"),
        );
        bytes[code_len_pos..code_len_pos + 4].copy_from_slice(&(code_len - 1).to_le_bytes());
        bytes.pop();
        let err = decode_semcode_envelope(&bytes).expect_err("must reject");
        assert!(matches!(err, DecodeError::InvalidOwnershipSection { .. }));
    }

    // #1726 Checkpoint D2a: rev21 (HEADER_V20/MAGIC20) OWN0 Borrow grammar,
    // structural round-trip and fail-closed corruption coverage. Numeric
    // discipline throughout: HEADER_V19.rev == 20 (SIG0/#1773's floor,
    // legacy Borrow grammar); HEADER_V20.rev == 21 (this section's floor,
    // new Borrow grammar). A minimal empty SIG0 section is included in every
    // rev21/rev20 fixture below because both header revisions are
    // `>= SEMCODE_SIGNATURE_MIN_REVISION` and therefore structurally require
    // one, unrelated to anything OWN0-specific being tested here.

    fn empty_sig0() -> Vec<u8> {
        let mut sig0 = Vec::new();
        sig0.extend_from_slice(&SIGNATURE_SECTION_TAG);
        sig0.extend_from_slice(&0u16.to_le_bytes());
        sig0
    }

    #[test]
    fn decode_rev21_header_borrow_frame_entry_round_trips() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes()); // empty string table
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_BORROW);
        code.push(ACTIVATION_MODE_FRAME_ENTRY);
        code.extend_from_slice(&7u32.to_le_bytes()); // root_symbol_id
        code.extend_from_slice(&0u16.to_le_bytes()); // component_count
        code.extend_from_slice(&empty_sig0());

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let (header, functions) = decode_semcode_envelope(&bytes).expect("decode");
        assert_eq!(header.rev, 21);
        assert_eq!(functions[0].borrowed_paths.len(), 1);
        assert_eq!(functions[0].borrowed_paths[0].root_symbol_id, 7);
        assert_eq!(
            functions[0].borrowed_paths[0].activation,
            Some(DecodedBorrowActivation::FrameEntry)
        );
    }

    #[test]
    fn decode_rev21_header_borrow_store_var_site_round_trips() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_BORROW);
        code.push(ACTIVATION_MODE_STORE_VAR_SITE);
        code.extend_from_slice(&123u32.to_le_bytes()); // executable anchor
        code.extend_from_slice(&9u32.to_le_bytes()); // root_symbol_id
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&empty_sig0());

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let (header, functions) = decode_semcode_envelope(&bytes).expect("decode");
        assert_eq!(header.rev, 21);
        assert_eq!(functions[0].borrowed_paths[0].root_symbol_id, 9);
        assert_eq!(
            functions[0].borrowed_paths[0].activation,
            Some(DecodedBorrowActivation::StoreVarSite(123))
        );
    }

    #[test]
    fn decode_rev20_header_write_still_uses_legacy_grammar() {
        // Numeric-explicit: HEADER_V19.rev == 20, the SIG0 floor -- NOT the
        // ownership-anchor grammar, which starts at HEADER_V20.rev == 21.
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_WRITE);
        // No execution-mode byte: legacy layout, root_symbol_id immediately
        // follows kind, byte-for-byte identical to every pre-W2D revision.
        code.extend_from_slice(&42u32.to_le_bytes());
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&empty_sig0());

        let bytes = function_bytes_with_header(MAGIC19, "main", &code);
        let (header, functions) = decode_semcode_envelope(&bytes).expect("decode");
        assert_eq!(header.rev, 20);
        assert_eq!(functions[0].write_paths.len(), 1);
        assert_eq!(functions[0].write_paths[0].root_symbol_id, 42);
        assert_eq!(functions[0].write_paths[0].write_execution, None);
    }

    // #1891 Checkpoint W2D, item 13.B: a rev21 StoreVarSite Write round-trips
    // its exact execution mode and anchor.
    #[test]
    fn decode_rev21_header_write_event_store_var_site_round_trips() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_WRITE);
        code.push(WRITE_EXECUTION_MODE_STORE_VAR_SITE);
        code.extend_from_slice(&99u32.to_le_bytes());
        code.extend_from_slice(&42u32.to_le_bytes());
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&empty_sig0());

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let (_, functions) = decode_semcode_envelope(&bytes).expect("decode");
        assert_eq!(functions[0].write_paths.len(), 1);
        assert_eq!(functions[0].write_paths[0].root_symbol_id, 42);
        assert_eq!(
            functions[0].write_paths[0].write_execution,
            Some(DecodedWriteExecution::StoreVarSite(99))
        );
    }

    // Item 13.C: a rev21 MakeRecordSite Write round-trips its exact execution
    // mode and anchor - the same wire position, a different tag.
    #[test]
    fn decode_rev21_header_write_event_make_record_site_round_trips() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_WRITE);
        code.push(WRITE_EXECUTION_MODE_MAKE_RECORD_SITE);
        code.extend_from_slice(&58u32.to_le_bytes());
        code.extend_from_slice(&7u32.to_le_bytes());
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&empty_sig0());

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let (_, functions) = decode_semcode_envelope(&bytes).expect("decode");
        assert_eq!(functions[0].write_paths.len(), 1);
        assert_eq!(functions[0].write_paths[0].root_symbol_id, 7);
        assert_eq!(
            functions[0].write_paths[0].write_execution,
            Some(DecodedWriteExecution::MakeRecordSite(58))
        );
    }

    // Item 13.D: two Write records sharing a MakeRecord site carry the same
    // exact mode and anchor, with distinct paths - N events, one anchor,
    // never deduplicated into one path record (item 7).
    #[test]
    fn decode_rev21_header_multi_field_record_update_writes_share_one_anchor() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&2u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_WRITE);
        code.push(WRITE_EXECUTION_MODE_MAKE_RECORD_SITE);
        code.extend_from_slice(&58u32.to_le_bytes());
        code.extend_from_slice(&7u32.to_le_bytes());
        code.extend_from_slice(&0u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_WRITE);
        code.push(WRITE_EXECUTION_MODE_MAKE_RECORD_SITE);
        code.extend_from_slice(&58u32.to_le_bytes());
        code.extend_from_slice(&8u32.to_le_bytes());
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&empty_sig0());

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let (_, functions) = decode_semcode_envelope(&bytes).expect("decode");
        assert_eq!(functions[0].write_paths.len(), 2);
        assert_eq!(
            functions[0].write_paths[0].write_execution,
            functions[0].write_paths[1].write_execution
        );
        assert_ne!(
            functions[0].write_paths[0].root_symbol_id,
            functions[0].write_paths[1].root_symbol_id
        );
    }

    // Item 13.E: repeated same-root assignments carry distinct anchors, never
    // collapsed into one.
    #[test]
    fn decode_rev21_header_repeated_assignment_writes_have_distinct_anchors() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&2u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_WRITE);
        code.push(WRITE_EXECUTION_MODE_STORE_VAR_SITE);
        code.extend_from_slice(&10u32.to_le_bytes());
        code.extend_from_slice(&1u32.to_le_bytes());
        code.extend_from_slice(&0u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_WRITE);
        code.push(WRITE_EXECUTION_MODE_STORE_VAR_SITE);
        code.extend_from_slice(&20u32.to_le_bytes());
        code.extend_from_slice(&1u32.to_le_bytes());
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&empty_sig0());

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let (_, functions) = decode_semcode_envelope(&bytes).expect("decode");
        assert_eq!(functions[0].write_paths.len(), 2);
        assert_eq!(
            functions[0].write_paths[0].write_execution,
            Some(DecodedWriteExecution::StoreVarSite(10))
        );
        assert_eq!(
            functions[0].write_paths[1].write_execution,
            Some(DecodedWriteExecution::StoreVarSite(20))
        );
    }

    // Item 13.F: a mixed rev21 artifact proves both grammars coexist
    // correctly in one OWN0 section.
    #[test]
    fn decode_rev21_header_mixed_borrow_and_write_events_coexist() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&2u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_BORROW);
        code.push(ACTIVATION_MODE_STORE_VAR_SITE);
        code.extend_from_slice(&55u32.to_le_bytes());
        code.extend_from_slice(&1u32.to_le_bytes());
        code.extend_from_slice(&0u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_WRITE);
        code.push(WRITE_EXECUTION_MODE_MAKE_RECORD_SITE);
        code.extend_from_slice(&58u32.to_le_bytes());
        code.extend_from_slice(&2u32.to_le_bytes());
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&empty_sig0());

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let (_, functions) = decode_semcode_envelope(&bytes).expect("decode");
        assert_eq!(functions[0].borrowed_paths.len(), 1);
        assert_eq!(functions[0].write_paths.len(), 1);
        assert_eq!(
            functions[0].borrowed_paths[0].activation,
            Some(DecodedBorrowActivation::StoreVarSite(55))
        );
        assert_eq!(
            functions[0].write_paths[0].write_execution,
            Some(DecodedWriteExecution::MakeRecordSite(58))
        );
    }

    // Item 13.G: a FrameEntry Borrow stays anchorless while a Write in the
    // same section remains anchored - the two domains never leak into each
    // other's grammar.
    #[test]
    fn decode_rev21_header_frame_entry_borrow_and_anchored_write_coexist() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&2u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_BORROW);
        code.push(ACTIVATION_MODE_FRAME_ENTRY);
        code.extend_from_slice(&1u32.to_le_bytes());
        code.extend_from_slice(&0u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_WRITE);
        code.push(WRITE_EXECUTION_MODE_STORE_VAR_SITE);
        code.extend_from_slice(&10u32.to_le_bytes());
        code.extend_from_slice(&2u32.to_le_bytes());
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&empty_sig0());

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let (_, functions) = decode_semcode_envelope(&bytes).expect("decode");
        assert_eq!(
            functions[0].borrowed_paths[0].activation,
            Some(DecodedBorrowActivation::FrameEntry)
        );
        assert_eq!(
            functions[0].write_paths[0].write_execution,
            Some(DecodedWriteExecution::StoreVarSite(10))
        );
    }

    #[test]
    fn decode_rev21_header_mixed_frame_entry_and_store_var_site() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&2u16.to_le_bytes());
        // ADT/Option/Result-shaped: FrameEntry.
        code.push(OWNERSHIP_EVENT_KIND_BORROW);
        code.push(ACTIVATION_MODE_FRAME_ENTRY);
        code.extend_from_slice(&1u32.to_le_bytes());
        code.extend_from_slice(&0u16.to_le_bytes());
        // Tuple/Record-shaped: StoreVarSite.
        code.push(OWNERSHIP_EVENT_KIND_BORROW);
        code.push(ACTIVATION_MODE_STORE_VAR_SITE);
        code.extend_from_slice(&55u32.to_le_bytes());
        code.extend_from_slice(&2u32.to_le_bytes());
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&empty_sig0());

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let (_, functions) = decode_semcode_envelope(&bytes).expect("decode");
        assert_eq!(functions[0].borrowed_paths.len(), 2);
        assert_eq!(
            functions[0].borrowed_paths[0].activation,
            Some(DecodedBorrowActivation::FrameEntry)
        );
        assert_eq!(functions[0].borrowed_paths[0].root_symbol_id, 1);
        assert_eq!(
            functions[0].borrowed_paths[1].activation,
            Some(DecodedBorrowActivation::StoreVarSite(55))
        );
        assert_eq!(functions[0].borrowed_paths[1].root_symbol_id, 2);
    }

    #[test]
    fn decode_rev20_header_borrow_still_uses_legacy_grammar() {
        // Numeric-explicit: HEADER_V19.rev == 20, the SIG0 floor -- NOT the
        // ownership-anchor grammar, which starts at HEADER_V20.rev == 21.
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_BORROW);
        // No activation byte: legacy layout, root_symbol_id immediately
        // follows kind, byte-for-byte identical to every pre-D2a revision.
        code.extend_from_slice(&3u32.to_le_bytes());
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&empty_sig0());

        let bytes = function_bytes_with_header(MAGIC19, "main", &code);
        let (header, functions) = decode_semcode_envelope(&bytes).expect("decode");
        assert_eq!(header.rev, 20);
        assert_eq!(functions[0].borrowed_paths[0].root_symbol_id, 3);
        assert_eq!(functions[0].borrowed_paths[0].activation, None);
    }

    #[test]
    fn decode_rejects_unrecognized_borrow_activation_mode() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_BORROW);
        code.push(2); // neither ACTIVATION_MODE_FRAME_ENTRY nor _STORE_VAR_SITE
        code.extend_from_slice(&1u32.to_le_bytes());
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&empty_sig0());

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let err = decode_semcode_envelope(&bytes).expect_err("must reject, never guess");
        assert!(matches!(err, DecodeError::InvalidOwnershipSection { .. }));
    }

    #[test]
    fn decode_rejects_truncated_borrow_activation_mode() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_BORROW);
        // Truncated immediately after `kind` -- no activation mode byte at all.

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let err = decode_semcode_envelope(&bytes).expect_err("must reject, never panic");
        assert!(matches!(err, DecodeError::InvalidOwnershipSection { .. }));
    }

    #[test]
    fn decode_rejects_truncated_store_var_site_anchor() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_BORROW);
        code.push(ACTIVATION_MODE_STORE_VAR_SITE);
        code.extend_from_slice(&[0x01, 0x02]); // anchor needs 4 bytes, only 2 present

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let err = decode_semcode_envelope(&bytes).expect_err("must reject, never panic");
        assert!(matches!(err, DecodeError::InvalidOwnershipSection { .. }));
    }

    // #1891 Checkpoint W2D, item 12: exhaustive malformed rev21 Write cases,
    // mirroring the Borrow malformed-input tests above one-for-one.

    #[test]
    fn decode_rejects_unrecognized_write_execution_mode() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_WRITE);
        code.push(2); // neither WRITE_EXECUTION_MODE_STORE_VAR_SITE nor _MAKE_RECORD_SITE
        code.extend_from_slice(&1u32.to_le_bytes());
        code.extend_from_slice(&1u32.to_le_bytes());
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&empty_sig0());

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let err = decode_semcode_envelope(&bytes).expect_err("must reject, never guess");
        assert!(matches!(err, DecodeError::InvalidOwnershipSection { .. }));
    }

    #[test]
    fn decode_rejects_truncated_write_execution_mode() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_WRITE);
        // Truncated immediately after `kind` -- no execution mode byte at all.

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let err = decode_semcode_envelope(&bytes).expect_err("must reject, never panic");
        assert!(matches!(err, DecodeError::InvalidOwnershipSection { .. }));
    }

    #[test]
    fn decode_rejects_truncated_write_execution_anchor() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_WRITE);
        code.push(WRITE_EXECUTION_MODE_STORE_VAR_SITE);
        code.extend_from_slice(&[0x01, 0x02]); // anchor needs 4 bytes, only 2 present

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let err = decode_semcode_envelope(&bytes).expect_err("must reject, never panic");
        assert!(matches!(err, DecodeError::InvalidOwnershipSection { .. }));
    }

    #[test]
    fn decode_rejects_write_truncated_after_anchor_before_root() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_WRITE);
        code.push(WRITE_EXECUTION_MODE_MAKE_RECORD_SITE);
        code.extend_from_slice(&58u32.to_le_bytes());
        // Truncated immediately after the anchor -- no root_symbol_id at all.

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let err = decode_semcode_envelope(&bytes).expect_err("must reject, never panic");
        assert!(matches!(err, DecodeError::InvalidOwnershipSection { .. }));
    }

    #[test]
    fn decode_rejects_write_truncated_root() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_WRITE);
        code.push(WRITE_EXECUTION_MODE_STORE_VAR_SITE);
        code.extend_from_slice(&99u32.to_le_bytes());
        code.extend_from_slice(&[0x07, 0x00]); // root needs 4 bytes, only 2 present

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let err = decode_semcode_envelope(&bytes).expect_err("must reject, never panic");
        assert!(matches!(err, DecodeError::InvalidOwnershipSection { .. }));
    }

    #[test]
    fn decode_rejects_write_truncated_component_count() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_WRITE);
        code.push(WRITE_EXECUTION_MODE_STORE_VAR_SITE);
        code.extend_from_slice(&99u32.to_le_bytes());
        code.extend_from_slice(&7u32.to_le_bytes());
        code.push(0x01); // component count needs 2 bytes, only 1 present

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let err = decode_semcode_envelope(&bytes).expect_err("must reject, never panic");
        assert!(matches!(err, DecodeError::InvalidOwnershipSection { .. }));
    }

    #[test]
    fn decode_rejects_write_malformed_component_payload() {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_WRITE);
        code.push(WRITE_EXECUTION_MODE_MAKE_RECORD_SITE);
        code.extend_from_slice(&58u32.to_le_bytes());
        code.extend_from_slice(&7u32.to_le_bytes());
        code.extend_from_slice(&1u16.to_le_bytes()); // claims one component
        code.push(0xFF); // not a recognized component kind

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let err = decode_semcode_envelope(&bytes).expect_err("must reject, never guess");
        assert!(matches!(err, DecodeError::InvalidOwnershipSection { .. }));
    }

    #[test]
    fn decode_rejects_legacy_write_bytes_under_rev21_header_deterministically() {
        // A real pre-W2D producer emits root_symbol_id immediately after
        // `kind`, with no execution-mode byte. Decoded under a rev21 header,
        // the decoder unconditionally expects a mode byte first: this must
        // reject deterministically (root_symbol_id's own low byte is 5,
        // neither a valid StoreVarSite(0) nor MakeRecordSite(1) tag), never
        // silently reinterpret the legacy bytes as if they were rev21-shaped.
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_WRITE);
        code.extend_from_slice(&5u32.to_le_bytes()); // legacy root_symbol_id
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&empty_sig0());

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let err = decode_semcode_envelope(&bytes)
            .expect_err("legacy bytes must not be heuristically reinterpreted as rev21");
        assert!(matches!(err, DecodeError::InvalidOwnershipSection { .. }));
    }

    #[test]
    fn decode_rejects_rev21_shaped_write_bytes_under_rev20_header() {
        // The reverse direction: bytes shaped for the new grammar, decoded
        // under HEADER_V19 (rev 20, the SIG0 floor -- NOT the ownership-anchor
        // floor). rev20 always uses the legacy reader, which has no concept
        // of an execution-mode byte at all, so it misreads the mode/anchor
        // bytes as the start of root_symbol_id/component_count. The header
        // revision is the only grammar authority in either direction -- no
        // fallback or retry is attempted -- so this must not decode into
        // anything resembling the intended rev21 event; it must fail
        // structurally somewhere in this same section.
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_WRITE);
        code.push(WRITE_EXECUTION_MODE_MAKE_RECORD_SITE);
        code.extend_from_slice(&0xAAAA_AAAAu32.to_le_bytes()); // intended anchor
        code.extend_from_slice(&0xBBBB_BBBBu32.to_le_bytes()); // intended root_symbol_id
        code.extend_from_slice(&0u16.to_le_bytes()); // intended component_count
        code.extend_from_slice(&empty_sig0());

        let bytes = function_bytes_with_header(MAGIC19, "main", &code);
        assert!(
            decode_semcode_envelope(&bytes).is_err(),
            "rev21-shaped bytes read under the legacy rev20 grammar must not succeed"
        );
    }

    #[test]
    fn decode_rejects_legacy_borrow_bytes_under_rev21_header_deterministically() {
        // A real pre-D2a producer emits root_symbol_id immediately after
        // `kind`, with no activation byte. Decoded under a rev21 header, the
        // decoder unconditionally expects an activation-mode byte first: this
        // must reject deterministically (root_symbol_id's own low byte is 5,
        // neither a valid FrameEntry(0) nor StoreVarSite(1) tag), never
        // silently reinterpret the legacy bytes as if they were rev21-shaped.
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_BORROW);
        code.extend_from_slice(&5u32.to_le_bytes()); // legacy root_symbol_id
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&empty_sig0());

        let bytes = function_bytes_with_header(MAGIC20, "main", &code);
        let err = decode_semcode_envelope(&bytes)
            .expect_err("legacy bytes must not be heuristically reinterpreted as rev21");
        assert!(matches!(err, DecodeError::InvalidOwnershipSection { .. }));
    }

    #[test]
    fn decode_rejects_rev21_shaped_borrow_bytes_under_rev20_header() {
        // The reverse direction: bytes shaped for the new grammar, decoded
        // under HEADER_V19 (rev 20, the SIG0 floor -- NOT the ownership-anchor
        // floor). rev20 always uses the legacy reader, which has no concept
        // of an activation-mode byte at all, so it misreads the mode/anchor
        // bytes as the start of root_symbol_id/component_count. The header
        // revision is the only grammar authority in either direction -- no
        // fallback or retry is attempted -- so this must not decode into
        // anything resembling the intended rev21 event; it must fail
        // structurally somewhere in this same section.
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes());
        code.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        code.extend_from_slice(&1u16.to_le_bytes());
        code.push(OWNERSHIP_EVENT_KIND_BORROW);
        code.push(ACTIVATION_MODE_STORE_VAR_SITE);
        code.extend_from_slice(&0xAAAA_AAAAu32.to_le_bytes()); // intended anchor
        code.extend_from_slice(&0xBBBB_BBBBu32.to_le_bytes()); // intended root_symbol_id
        code.extend_from_slice(&0u16.to_le_bytes()); // intended component_count
        code.extend_from_slice(&empty_sig0());

        let bytes = function_bytes_with_header(MAGIC19, "main", &code);
        assert!(
            decode_semcode_envelope(&bytes).is_err(),
            "rev21-shaped bytes read under the legacy rev20 grammar must not succeed"
        );
    }
}
