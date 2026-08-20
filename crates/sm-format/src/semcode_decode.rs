use crate::local_format::*;

pub const MAX_FUNCTIONS: usize = 1024;
pub const MAX_STRING_LEN: usize = 1024;
pub const MAX_STRINGS_PER_FUNCTION: usize = 256;
pub const MAX_DEBUG_SYMBOLS_PER_FUNCTION: usize = 8192;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    BadHeader,
    UnsupportedVersion { found: String, supported: String },
    TruncatedFunction { offset: usize, msg: &'static str },
    InvalidFunctionName { offset: usize, msg: &'static str },
    InvalidStringTable { offset: usize, msg: &'static str },
    InvalidDebugSection { offset: usize, msg: &'static str },
    InvalidOwnershipSection { offset: usize, msg: &'static str },
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedAccessPath {
    pub root_symbol_id: u32,
    pub components: Vec<DecodedAccessPathComponent>,
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

        let parsed = parse_string_table_debug_and_ownership(code_offset, code_slice)?;

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
                }),
                OWNERSHIP_EVENT_KIND_WRITE => write_paths.push(DecodedAccessPath {
                    root_symbol_id,
                    components,
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

    Ok(StringTableDebugOwnershipParse {
        strings,
        debug_symbols,
        borrowed_paths,
        write_paths,
        has_ownership_section,
        has_debug_section,
        string_table_end_offset,
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
}
