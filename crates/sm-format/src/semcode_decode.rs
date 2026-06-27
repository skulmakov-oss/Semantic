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
    pub instr_start_offset: usize, // relative to code_slice
    pub code_slice: &'a [u8],      // the full code block for this function
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

        if cursor + code_len > bytes.len() {
            return Err(DecodeError::TruncatedFunction {
                offset: cursor,
                msg: "function code out of bounds",
            });
        }

        let code_offset = cursor;
        let code_slice = &bytes[cursor..cursor + code_len];
        cursor += code_len;

        let (
            strings,
            debug_symbols,
            borrowed_paths,
            write_paths,
            has_ownership_section,
            instr_start_offset,
        ) = parse_string_table_debug_and_ownership(code_offset, code_slice)?;

        functions.push(DecodedFunctionEnvelope {
            name,
            name_offset,
            code_offset,
            code_len,
            strings,
            debug_symbols,
            borrowed_paths,
            write_paths,
            has_ownership_section,
            instr_start_offset,
            code_slice,
        });
    }

    Ok((header, functions))
}

fn parse_string_table_debug_and_ownership(
    base_offset: usize,
    code: &[u8],
) -> Result<
    (
        Vec<String>,
        Vec<DecodedDebugSymbol>,
        Vec<DecodedAccessPath>,
        Vec<DecodedAccessPath>,
        bool,
        usize,
    ),
    DecodeError,
> {
    let mut cursor = 0usize;
    let string_count_offset = base_offset + cursor;
    let count = read_u16_le(code, &mut cursor).map_err(|_| DecodeError::InvalidStringTable {
        offset: string_count_offset,
        msg: "missing string table header",
    })? as usize;

    if count > MAX_STRINGS_PER_FUNCTION {
        return Err(DecodeError::ResourceLimit {
            offset: base_offset + cursor,
            msg: format!(
                "too many strings in function: {} (max {})",
                count, MAX_STRINGS_PER_FUNCTION
            ),
        });
    }

    let mut strings = Vec::with_capacity(count);
    for _ in 0..count {
        let len_offset = base_offset + cursor;
        let len = read_u16_le(code, &mut cursor).map_err(|_| DecodeError::InvalidStringTable {
            offset: len_offset,
            msg: "missing string length",
        })? as usize;

        if len > MAX_STRING_LEN {
            return Err(DecodeError::InvalidStringTable {
                offset: base_offset + cursor,
                msg: "string too long in function string table",
            });
        }

        let str_val =
            read_utf8(code, &mut cursor, len).map_err(|_| DecodeError::InvalidStringTable {
                offset: base_offset + cursor,
                msg: "invalid utf8 in string table",
            })?;
        strings.push(str_val);
    }

    let mut debug_symbols = Vec::new();
    if cursor + 4 <= code.len() && &code[cursor..cursor + 4] == b"DBG0" {
        cursor += 4;
        let dbg_count_offset = base_offset + cursor;
        let count =
            read_u16_le(code, &mut cursor).map_err(|_| DecodeError::InvalidDebugSection {
                offset: dbg_count_offset,
                msg: "missing debug symbol count",
            })? as usize;

        if count > MAX_DEBUG_SYMBOLS_PER_FUNCTION {
            return Err(DecodeError::ResourceLimit {
                offset: base_offset + cursor,
                msg: format!(
                    "too many debug symbols: {} (max {})",
                    count, MAX_DEBUG_SYMBOLS_PER_FUNCTION
                ),
            });
        }

        debug_symbols.reserve(count);
        for _ in 0..count {
            let entry_offset = base_offset + cursor;
            let pc =
                read_u32_le(code, &mut cursor).map_err(|_| DecodeError::InvalidDebugSection {
                    offset: entry_offset,
                    msg: "missing debug pc",
                })? as usize;
            let line =
                read_u32_le(code, &mut cursor).map_err(|_| DecodeError::InvalidDebugSection {
                    offset: base_offset + cursor,
                    msg: "missing debug line",
                })?;
            let col =
                read_u16_le(code, &mut cursor).map_err(|_| DecodeError::InvalidDebugSection {
                    offset: base_offset + cursor,
                    msg: "missing debug col",
                })?;
            debug_symbols.push(DecodedDebugSymbol { pc, line, col });
        }
    }

    let mut borrowed_paths = Vec::new();
    let mut write_paths = Vec::new();
    let mut has_ownership_section = false;
    if cursor + 4 <= code.len() && &code[cursor..cursor + 4] == OWNERSHIP_SECTION_TAG {
        has_ownership_section = true;
        cursor += OWNERSHIP_SECTION_TAG.len();
        let own_count_offset = base_offset + cursor;
        let count =
            read_u16_le(code, &mut cursor).map_err(|_| DecodeError::InvalidOwnershipSection {
                offset: own_count_offset,
                msg: "missing ownership path count",
            })? as usize;

        borrowed_paths.reserve(count);
        write_paths.reserve(count);
        for _ in 0..count {
            let entry_offset = base_offset + cursor;
            let kind =
                read_u8(code, &mut cursor).map_err(|_| DecodeError::InvalidOwnershipSection {
                    offset: entry_offset,
                    msg: "missing ownership event kind",
                })?;
            let root_symbol_id = read_u32_le(code, &mut cursor).map_err(|_| {
                DecodeError::InvalidOwnershipSection {
                    offset: base_offset + cursor,
                    msg: "missing ownership path root",
                }
            })?;
            let component_count = read_u16_le(code, &mut cursor).map_err(|_| {
                DecodeError::InvalidOwnershipSection {
                    offset: base_offset + cursor,
                    msg: "missing ownership path component count",
                }
            })? as usize;

            let mut components = Vec::new();
            for _ in 0..component_count {
                let component_kind = read_u8(code, &mut cursor).map_err(|_| {
                    DecodeError::InvalidOwnershipSection {
                        offset: base_offset + cursor,
                        msg: "missing ownership path component kind",
                    }
                })?;
                match component_kind {
                    OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX => {
                        let index = read_u16_le(code, &mut cursor).map_err(|_| {
                            DecodeError::InvalidOwnershipSection {
                                offset: base_offset + cursor,
                                msg: "missing ownership tuple index",
                            }
                        })?;
                        components.push(DecodedAccessPathComponent::TupleIndex(index));
                    }
                    OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL => {
                        let field = read_u32_le(code, &mut cursor).map_err(|_| {
                            DecodeError::InvalidOwnershipSection {
                                offset: base_offset + cursor,
                                msg: "missing ownership field symbol",
                            }
                        })?;
                        components.push(DecodedAccessPathComponent::FieldSymbol(field));
                    }
                    OWNERSHIP_PATH_COMPONENT_ADT_PAYLOAD => {
                        let variant = read_u32_le(code, &mut cursor).map_err(|_| {
                            DecodeError::InvalidOwnershipSection {
                                offset: base_offset + cursor,
                                msg: "missing ownership adt payload variant",
                            }
                        })?;
                        let index = read_u16_le(code, &mut cursor).map_err(|_| {
                            DecodeError::InvalidOwnershipSection {
                                offset: base_offset + cursor,
                                msg: "missing ownership adt payload index",
                            }
                        })?;
                        components.push(DecodedAccessPathComponent::AdtPayload { variant, index });
                    }
                    OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX => {
                        let index = read_u32_le(code, &mut cursor).map_err(|_| {
                            DecodeError::InvalidOwnershipSection {
                                offset: base_offset + cursor,
                                msg: "missing ownership sequence index",
                            }
                        })?;
                        components.push(DecodedAccessPathComponent::SequenceIndexStatic(index));
                    }
                    _ => {
                        return Err(DecodeError::InvalidOwnershipSection {
                            offset: base_offset + cursor,
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
                        offset: base_offset + cursor,
                        msg: "unsupported ownership event kind",
                    });
                }
            }
        }
    }

    Ok((
        strings,
        debug_symbols,
        borrowed_paths,
        write_paths,
        has_ownership_section,
        cursor,
    ))
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
