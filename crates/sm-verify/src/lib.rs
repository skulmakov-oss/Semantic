#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use sm_emit::{
    header_spec_from_magic, read_f64_le, read_i32_le, read_u16_le, read_u32_le, read_u8, read_utf8,
    Opcode, SemcodeFormatError, SemcodeHeaderSpec, CAP_CLOCK_READ, CAP_DEBUG_SYMBOLS,
    CAP_EVENT_POST, CAP_F64_MATH, CAP_FX_MATH, CAP_FX_VALUES, CAP_GATE_SURFACE,
    CAP_MAP_VALUES, CAP_OWNERSHIP_FIELD_PATHS, CAP_OWNERSHIP_PATHS, CAP_PRNG,
    CAP_SEQUENCE_ITERATION, CAP_SEQUENCE_VALUES, CAP_STDOUT,
    CAP_STATE_QUERY, CAP_STATE_UPDATE, CAP_TEXT_VALUES, CAP_CLOSURE_VALUES,
    OWNERSHIP_EVENT_KIND_BORROW, OWNERSHIP_EVENT_KIND_WRITE,
    OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL, OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX,
    OWNERSHIP_SECTION_TAG,
};
use sm_runtime_core::RuntimeQuotas;
use std::collections::HashSet;

#[cfg(feature = "std")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationCode {
    BadHeader,
    UnsupportedVersion,
    TruncatedFunction,
    InvalidFunctionName,
    InvalidStringTable,
    InvalidDebugSection,
    InvalidOwnershipSection,
    UnknownOpcode,
    OperandOutOfBounds,
    InvalidJumpTarget,
    InvalidStringReference,
    InvalidRegisterReference,
    UnknownCallTarget,
    ResourceLimitExceeded,
    CapabilityViolation,
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationDiagnostic {
    pub code: VerificationCode,
    pub function: Option<String>,
    pub offset: Option<usize>,
    pub message: String,
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectReport {
    pub diagnostics: Vec<VerificationDiagnostic>,
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedFunction {
    pub name: String,
    pub code_len: usize,
    pub string_count: usize,
    pub debug_symbol_count: usize,
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedProgram {
    pub header: SemcodeHeaderSpec,
    pub functions: Vec<VerifiedFunction>,
}

#[cfg(feature = "std")]
impl core::fmt::Display for RejectReport {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for (idx, diag) in self.diagnostics.iter().enumerate() {
            if idx > 0 {
                writeln!(f)?;
            }
            write!(f, "verify error [{:?}]", diag.code)?;
            if let Some(function) = &diag.function {
                write!(f, " in '{}'", function)?;
            }
            if let Some(offset) = diag.offset {
                write!(f, " @0x{:04x}", offset)?;
            }
            write!(f, ": {}", diag.message)?;
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RejectReport {}

#[cfg(feature = "std")]
pub fn verify_semcode(bytes: &[u8]) -> Result<VerifiedProgram, RejectReport> {
    let mut diagnostics = Vec::new();
    let quotas = RuntimeQuotas::verified_local();

    if bytes.len() < 8 {
        diagnostics.push(diag(
            VerificationCode::BadHeader,
            None,
            None,
            "SemCode file is shorter than the 8-byte header",
        ));
        return Err(RejectReport { diagnostics });
    }

    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[..8]);
    let Some(header) = header_spec_from_magic(&magic) else {
        diagnostics.push(diag(
            VerificationCode::UnsupportedVersion,
            None,
            Some(0),
            format!(
                "unsupported SemCode header '{}'",
                String::from_utf8_lossy(&magic)
            ),
        ));
        return Err(RejectReport { diagnostics });
    };

    let mut cursor = 8usize;
    let mut functions = Vec::new();
    let mut pending_functions = Vec::new();
    while cursor < bytes.len() {
        let function_start = cursor;
        let name_len = match read_u16_le(bytes, &mut cursor) {
            Ok(v) => v as usize,
            Err(_) => {
                diagnostics.push(diag(
                    VerificationCode::TruncatedFunction,
                    None,
                    Some(function_start),
                    "truncated function header while reading name length",
                ));
                break;
            }
        };
        if name_len == 0 {
            diagnostics.push(diag(
                VerificationCode::InvalidFunctionName,
                None,
                Some(function_start),
                "function name must not be empty",
            ));
            break;
        }
        let name = match read_utf8(bytes, &mut cursor, name_len) {
            Ok(v) => v,
            Err(_) => {
                diagnostics.push(diag(
                    VerificationCode::InvalidFunctionName,
                    None,
                    Some(function_start),
                    "function name is truncated or invalid utf-8",
                ));
                break;
            }
        };
        let code_len = match read_u32_le(bytes, &mut cursor) {
            Ok(v) => v as usize,
            Err(_) => {
                diagnostics.push(diag(
                    VerificationCode::TruncatedFunction,
                    Some(name.clone()),
                    Some(function_start),
                    "truncated function header while reading code length",
                ));
                break;
            }
        };
        if cursor + code_len > bytes.len() {
            diagnostics.push(diag(
                VerificationCode::TruncatedFunction,
                Some(name.clone()),
                Some(cursor),
                "function code extends past end of file",
            ));
            break;
        }
        let code = &bytes[cursor..cursor + code_len];
        cursor += code_len;

        match verify_function_code(&name, code, &header, &quotas) {
            Ok(function) => {
                functions.push(function.verified.clone());
                pending_functions.push(function);
            }
            Err(report) => diagnostics.extend(report.diagnostics),
        }
    }

    if pending_functions.len() > quotas.max_frames {
        diagnostics.push(diag(
            VerificationCode::ResourceLimitExceeded,
            None,
            None,
            format!(
                "program defines {} functions, which exceeds the verified-local frame budget of {}",
                pending_functions.len(),
                quotas.max_frames
            ),
        ));
    }

    if header.capabilities & CAP_OWNERSHIP_PATHS != 0
        && !pending_functions
            .iter()
            .any(|function| function.has_ownership_section)
    {
        diagnostics.push(diag(
            VerificationCode::InvalidOwnershipSection,
            None,
            None,
            "header advertises ownership-path metadata but no OWN0 section is present",
        ));
    }

    let known_functions = pending_functions
        .iter()
        .map(|function| function.verified.name.as_str())
        .collect::<HashSet<_>>();
    for function in &pending_functions {
        for (offset, callee) in &function.call_targets {
            if known_functions.contains(callee.as_str()) {
                continue;
            }

            if let Some(required_capabilities) = builtin_call_required_capabilities(callee) {
                if header.capabilities & required_capabilities != required_capabilities {
                    diagnostics.push(diag(
                        VerificationCode::CapabilityViolation,
                        Some(function.verified.name.clone()),
                        Some(*offset),
                        format!(
                            "builtin call target '{}' requires capability bits 0x{required_capabilities:08x}",
                            callee
                        ),
                    ));
                }
                continue;
            }

            diagnostics.push(diag(
                VerificationCode::UnknownCallTarget,
                Some(function.verified.name.clone()),
                Some(*offset),
                format!(
                    "call target '{}' does not resolve to any function in this SemCode program",
                    callee
                ),
            ));
        }
    }

    if diagnostics.is_empty() {
        Ok(VerifiedProgram { header, functions })
    } else {
        Err(RejectReport { diagnostics })
    }
}

#[cfg(feature = "std")]
fn verify_function_code(
    name: &str,
    code: &[u8],
    header: &SemcodeHeaderSpec,
    quotas: &RuntimeQuotas,
) -> Result<PendingVerifiedFunction, RejectReport> {
    let mut cursor = 0usize;
    let string_count = read_u16_le(code, &mut cursor).map_err(|_| {
        reject_one(
            name,
            VerificationCode::InvalidStringTable,
            0,
            "missing string table header",
        )
    })? as usize;
    if string_count > quotas.max_symbol_table {
        return Err(reject_one(
            name,
            VerificationCode::ResourceLimitExceeded,
            0,
            format!(
                "function string table uses {} entries, exceeding the verified-local symbol budget of {}",
                string_count, quotas.max_symbol_table
            ),
        ));
    }

    let mut strings = Vec::with_capacity(string_count);
    for _ in 0..string_count {
        let len = read_u16_le(code, &mut cursor).map_err(|_| {
            reject_one(
                name,
                VerificationCode::InvalidStringTable,
                cursor,
                "truncated string length",
            )
        })? as usize;
        let string = read_utf8(code, &mut cursor, len).map_err(|_| {
            reject_one(
                name,
                VerificationCode::InvalidStringTable,
                cursor,
                "invalid function string entry",
            )
        })?;
        strings.push(string);
    }

    let mut debug_symbol_count = 0usize;
    let mut debug_pcs = Vec::new();
    if cursor + 4 <= code.len() && &code[cursor..cursor + 4] == b"DBG0" {
        cursor += 4;
        debug_symbol_count = read_u16_le(code, &mut cursor).map_err(|_| {
            reject_one(
                name,
                VerificationCode::InvalidDebugSection,
                cursor,
                "truncated debug section header",
            )
        })? as usize;
        if debug_symbol_count > quotas.max_trace_entries {
            return Err(reject_one(
                name,
                VerificationCode::ResourceLimitExceeded,
                cursor,
                format!(
                    "debug section uses {} entries, exceeding the verified-local trace budget of {}",
                    debug_symbol_count, quotas.max_trace_entries
                ),
            ));
        }
        for _ in 0..debug_symbol_count {
            let pc = read_u32_le(code, &mut cursor).map_err(|_| {
                reject_one(
                    name,
                    VerificationCode::InvalidDebugSection,
                    cursor,
                    "truncated debug pc",
                )
            })?;
            read_u32_le(code, &mut cursor).map_err(|_| {
                reject_one(
                    name,
                    VerificationCode::InvalidDebugSection,
                    cursor,
                    "truncated debug line",
                )
            })?;
            read_u16_le(code, &mut cursor).map_err(|_| {
                reject_one(
                    name,
                    VerificationCode::InvalidDebugSection,
                    cursor,
                    "truncated debug column",
                )
            })?;
            debug_pcs.push(pc as usize);
        }
    }

    let has_ownership_section = cursor + 4 <= code.len()
        && &code[cursor..cursor + 4] == OWNERSHIP_SECTION_TAG;
    let ownership_usage = if has_ownership_section {
        verify_ownership_section(name, code, &mut cursor, quotas)?
    } else {
        OwnershipSectionUsage::default()
    };

    let instr_start = cursor;
    let instr_len = code.len().saturating_sub(instr_start);
    let mut instr_starts = Vec::new();
    let mut jump_targets = Vec::new();
    let mut string_refs = Vec::new();
    let mut max_register: Option<usize> = None;
    let mut used_caps = 0u32;
    while cursor < code.len() {
        let offset = cursor - instr_start;
        instr_starts.push(offset);
        let opcode = read_u8(code, &mut cursor).map_err(|_| {
            reject_one(
                name,
                VerificationCode::UnknownOpcode,
                offset,
                "missing opcode byte",
            )
        })?;
        let opcode = Opcode::from_byte(opcode).map_err(|err| match err {
            SemcodeFormatError::UnknownOpcode(_) => reject_one(
                name,
                VerificationCode::UnknownOpcode,
                offset,
                err.to_string(),
            ),
            _ => reject_one(
                name,
                VerificationCode::OperandOutOfBounds,
                offset,
                err.to_string(),
            ),
        })?;
        let refs = decode_operands(name, code, &mut cursor, offset, opcode)?;
        jump_targets.extend(refs.jump_targets);
        string_refs.extend(refs.string_refs);
        used_caps |= refs.required_capabilities;
        max_register = match (max_register, refs.max_register) {
            (Some(lhs), Some(rhs)) => Some(lhs.max(rhs)),
            (None, some) => some,
            (some, None) => some,
        };
    }

    for pc in debug_pcs {
        if pc >= instr_len {
            return Err(reject_one(
                name,
                VerificationCode::InvalidDebugSection,
                pc,
                "debug symbol pc points past the instruction stream",
            ));
        }
    }

    if debug_symbol_count > 0 {
        used_caps |= CAP_DEBUG_SYMBOLS;
    }
    if has_ownership_section {
        used_caps |= CAP_OWNERSHIP_PATHS;
    }
    if ownership_usage.has_record_field_ownership {
        used_caps |= CAP_OWNERSHIP_FIELD_PATHS;
    }

    let missing_caps = used_caps & !header.capabilities;
    if missing_caps != 0 {
        return Err(reject_one(
            name,
            VerificationCode::CapabilityViolation,
            0,
            format!(
                "function requires capability bits 0x{missing_caps:08x}, but header '{}' provides only 0x{:08x}",
                String::from_utf8_lossy(&header.magic),
                header.capabilities
            ),
        ));
    }

    for target in jump_targets {
        if target >= instr_len {
            return Err(reject_one(
                name,
                VerificationCode::InvalidJumpTarget,
                target,
                "jump target points past the instruction stream",
            ));
        }
        if !instr_starts.contains(&target) {
            return Err(reject_one(
                name,
                VerificationCode::InvalidJumpTarget,
                target,
                "jump target does not land on an instruction boundary",
            ));
        }
    }

    let mut call_targets = Vec::new();
    for (offset, sid, usage) in string_refs {
        if sid >= string_count {
            return Err(reject_one(
                name,
                VerificationCode::InvalidStringReference,
                offset,
                format!("{usage} uses missing string id s{sid}"),
            ));
        }
        if usage == "call target" {
            call_targets.push((offset, strings[sid].clone()));
        }
    }

    if let Some(max_register) = max_register {
        let used = max_register + 1;
        if used > quotas.max_registers {
            return Err(reject_one(
                name,
                VerificationCode::InvalidRegisterReference,
                used - 1,
                format!(
                    "function references register r{}, exceeding the verified-local register budget of {}",
                    max_register, quotas.max_registers
                ),
            ));
        }
    }

    Ok(PendingVerifiedFunction {
        verified: VerifiedFunction {
            name: name.to_string(),
            code_len: code.len(),
            string_count,
            debug_symbol_count,
        },
        call_targets,
        has_ownership_section,
    })
}

#[cfg(feature = "std")]
fn decode_operands(
    function: &str,
    code: &[u8],
    cursor: &mut usize,
    offset: usize,
    opcode: Opcode,
) -> Result<OperandRefs, RejectReport> {
    let invalid =
        |msg: &str| reject_one(function, VerificationCode::OperandOutOfBounds, offset, msg);
    let mut refs = OperandRefs::default();
    let mut mark_reg = |reg: u16| {
        let reg = reg as usize;
        refs.max_register = Some(refs.max_register.map_or(reg, |current| current.max(reg)));
    };

    match opcode {
        Opcode::LoadQ => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            read_u8(code, cursor).map_err(|_| invalid("truncated quad literal"))?;
        }
        Opcode::LoadBool => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            read_u8(code, cursor).map_err(|_| invalid("truncated bool literal"))?;
        }
        Opcode::LoadI32 => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            read_i32_le(code, cursor).map_err(|_| invalid("truncated i32 literal"))?;
        }
        Opcode::AddI32 => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            let lhs = read_u16_le(code, cursor).map_err(|_| invalid("truncated lhs register"))?;
            let rhs = read_u16_le(code, cursor).map_err(|_| invalid("truncated rhs register"))?;
            mark_reg(dst);
            mark_reg(lhs);
            mark_reg(rhs);
        }
        Opcode::SubI32 | Opcode::MulI32 | Opcode::DivI32 | Opcode::ModI32 => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            let lhs = read_u16_le(code, cursor).map_err(|_| invalid("truncated lhs register"))?;
            let rhs = read_u16_le(code, cursor).map_err(|_| invalid("truncated rhs register"))?;
            mark_reg(dst);
            mark_reg(lhs);
            mark_reg(rhs);
        }
        Opcode::LoadU32 => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            read_u32_le(code, cursor).map_err(|_| invalid("truncated u32 literal"))?;
        }
        Opcode::LoadF64 => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            refs.required_capabilities |= CAP_F64_MATH;
            read_f64_le(code, cursor).map_err(|_| invalid("truncated f64 literal"))?;
        }
        Opcode::LoadFx => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            refs.required_capabilities |= CAP_FX_VALUES;
            read_i32_le(code, cursor).map_err(|_| invalid("truncated fx literal"))?;
        }
        Opcode::LoadText => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            refs.required_capabilities |= CAP_TEXT_VALUES;
            let sid =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated text literal string id"))?;
            refs.string_refs.push((offset, sid as usize, "text literal"));
        }
        Opcode::ConcatText => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            let lhs = read_u16_le(code, cursor).map_err(|_| invalid("truncated lhs register"))?;
            let rhs = read_u16_le(code, cursor).map_err(|_| invalid("truncated rhs register"))?;
            mark_reg(dst);
            mark_reg(lhs);
            mark_reg(rhs);
            refs.required_capabilities |= CAP_TEXT_VALUES;
        }
        Opcode::MakeSequence => {
            let dst =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated sequence dst register"))?;
            mark_reg(dst);
            refs.required_capabilities |= CAP_SEQUENCE_VALUES;
            let count = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence arity"))?
                as usize;
            for _ in 0..count {
                let src = read_u16_le(code, cursor)
                    .map_err(|_| invalid("truncated sequence item register"))?;
                mark_reg(src);
            }
        }
        Opcode::MakeTuple => {
            let dst =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated tuple dst register"))?;
            mark_reg(dst);
            let count =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated tuple arity"))? as usize;
            if count < 2 {
                return Err(invalid("tuple literal arity must be at least 2"));
            }
            for _ in 0..count {
                let src = read_u16_le(code, cursor)
                    .map_err(|_| invalid("truncated tuple item register"))?;
                mark_reg(src);
            }
        }
        Opcode::MakeRecord => {
            let dst =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated record dst register"))?;
            mark_reg(dst);
            let sid = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated record type string id"))?;
            refs.string_refs
                .push((offset, sid as usize, "record type name"));
            let count = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated record slot count"))?
                as usize;
            if count == 0 {
                return Err(invalid("record literal must encode at least one slot"));
            }
            for _ in 0..count {
                let src = read_u16_le(code, cursor)
                    .map_err(|_| invalid("truncated record slot register"))?;
                mark_reg(src);
            }
        }
        Opcode::MakeAdt => {
            let dst =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated enum dst register"))?;
            mark_reg(dst);
            let sid =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated enum type string id"))?;
            refs.string_refs
                .push((offset, sid as usize, "enum type name"));
            let variant_sid = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated enum variant string id"))?;
            refs.string_refs
                .push((offset, variant_sid as usize, "enum variant name"));
            read_u16_le(code, cursor).map_err(|_| invalid("truncated enum tag"))?;
            let count = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated enum payload count"))?
                as usize;
            for _ in 0..count {
                let src = read_u16_le(code, cursor)
                    .map_err(|_| invalid("truncated enum payload register"))?;
                mark_reg(src);
            }
        }
        Opcode::AdtTag => {
            let dst =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated adt-tag dst register"))?;
            let src =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated adt-tag src register"))?;
            let sid = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated adt-tag type string id"))?;
            mark_reg(dst);
            mark_reg(src);
            refs.string_refs
                .push((offset, sid as usize, "enum type name"));
        }
        Opcode::AdtGet => {
            let dst =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated adt-get dst register"))?;
            let src =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated adt-get src register"))?;
            let sid = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated adt-get type string id"))?;
            read_u16_le(code, cursor).map_err(|_| invalid("truncated adt-get payload index"))?;
            mark_reg(dst);
            mark_reg(src);
            refs.string_refs
                .push((offset, sid as usize, "enum type name"));
        }
        Opcode::RecordGet => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated record-get dst register"))?;
            let src = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated record-get src register"))?;
            let sid = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated record-get type string id"))?;
            read_u16_le(code, cursor).map_err(|_| invalid("truncated record-get slot index"))?;
            mark_reg(dst);
            mark_reg(src);
            refs.string_refs
                .push((offset, sid as usize, "record type name"));
        }
        Opcode::TupleGet => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated tuple-get dst register"))?;
            let src = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated tuple-get src register"))?;
            read_u16_le(code, cursor).map_err(|_| invalid("truncated tuple-get index"))?;
            mark_reg(dst);
            mark_reg(src);
        }
        Opcode::SequenceGet => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence-get dst register"))?;
            let src = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence-get src register"))?;
            let index = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence-get index register"))?;
            mark_reg(dst);
            mark_reg(src);
            mark_reg(index);
            refs.required_capabilities |= CAP_SEQUENCE_VALUES;
        }
        Opcode::SequenceLen => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence-len dst register"))?;
            let src = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence-len src register"))?;
            mark_reg(dst);
            mark_reg(src);
            refs.required_capabilities |= CAP_SEQUENCE_VALUES;
            refs.required_capabilities |= CAP_SEQUENCE_ITERATION;
        }
        Opcode::SequenceIsEmpty => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence-is-empty dst register"))?;
            let src = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence-is-empty src register"))?;
            mark_reg(dst);
            mark_reg(src);
            refs.required_capabilities |= CAP_SEQUENCE_VALUES;
            refs.required_capabilities |= CAP_SEQUENCE_ITERATION;
        }
        Opcode::SequenceContains => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence-contains dst register"))?;
            let seq = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence-contains seq register"))?;
            let val = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence-contains val register"))?;
            mark_reg(dst);
            mark_reg(seq);
            mark_reg(val);
            refs.required_capabilities |= CAP_SEQUENCE_VALUES;
            refs.required_capabilities |= CAP_SEQUENCE_ITERATION;
        }
        Opcode::SequencePush => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence-push dst register"))?;
            let seq = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence-push seq register"))?;
            let val = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence-push val register"))?;
            mark_reg(dst);
            mark_reg(seq);
            mark_reg(val);
            refs.required_capabilities |= CAP_SEQUENCE_VALUES;
            refs.required_capabilities |= CAP_SEQUENCE_ITERATION;
        }
        Opcode::SequencePrepend => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence-prepend dst register"))?;
            let seq = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence-prepend seq register"))?;
            let val = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence-prepend val register"))?;
            mark_reg(dst);
            mark_reg(seq);
            mark_reg(val);
            refs.required_capabilities |= CAP_SEQUENCE_VALUES;
            refs.required_capabilities |= CAP_SEQUENCE_ITERATION;
        }
        Opcode::SequencePop => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence-pop dst register"))?;
            let src = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence-pop src register"))?;
            mark_reg(dst);
            mark_reg(src);
            refs.required_capabilities |= CAP_SEQUENCE_VALUES;
            refs.required_capabilities |= CAP_SEQUENCE_ITERATION;
        }
        Opcode::MapEmpty => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated map-empty dst register"))?;
            mark_reg(dst);
            refs.required_capabilities |= CAP_MAP_VALUES;
        }
        Opcode::MapContains => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated map-contains dst register"))?;
            let map = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated map-contains map register"))?;
            let key = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated map-contains key register"))?;
            mark_reg(dst);
            mark_reg(map);
            mark_reg(key);
            refs.required_capabilities |= CAP_MAP_VALUES;
        }
        Opcode::MapGet => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated map-get dst register"))?;
            let map = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated map-get map register"))?;
            let key = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated map-get key register"))?;
            let default_val = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated map-get default register"))?;
            mark_reg(dst);
            mark_reg(map);
            mark_reg(key);
            mark_reg(default_val);
            refs.required_capabilities |= CAP_MAP_VALUES;
        }
        Opcode::MapSet => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated map-set dst register"))?;
            let map = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated map-set map register"))?;
            let key = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated map-set key register"))?;
            let val = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated map-set val register"))?;
            mark_reg(dst);
            mark_reg(map);
            mark_reg(key);
            mark_reg(val);
            refs.required_capabilities |= CAP_MAP_VALUES;
        }
        Opcode::RngSeed => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated rng-seed dst register"))?;
            let seed = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated rng-seed seed register"))?;
            mark_reg(dst);
            mark_reg(seed);
            refs.required_capabilities |= CAP_PRNG;
        }
        Opcode::RngNextI32 => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated rng-next-i32 dst register"))?;
            let lo = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated rng-next-i32 lo register"))?;
            let hi = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated rng-next-i32 hi register"))?;
            mark_reg(dst);
            mark_reg(lo);
            mark_reg(hi);
            refs.required_capabilities |= CAP_PRNG;
        }
        Opcode::MakeClosure => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated closure dst register"))?;
            mark_reg(dst);
            refs.required_capabilities |= CAP_CLOSURE_VALUES;
            let sid = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated closure function string id"))?;
            refs.string_refs
                .push((offset, sid as usize, "closure function name"));
            let count = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated closure capture arity"))?
                as usize;
            for _ in 0..count {
                let src = read_u16_le(code, cursor)
                    .map_err(|_| invalid("truncated closure capture register"))?;
                mark_reg(src);
            }
        }
        Opcode::ClosureCall => {
            let has_dst =
                read_u8(code, cursor).map_err(|_| invalid("truncated closure-call dst flag"))? != 0;
            if has_dst {
                let dst = read_u16_le(code, cursor)
                    .map_err(|_| invalid("truncated closure-call dst register"))?;
                mark_reg(dst);
            } else {
                let _ = read_u16_le(code, cursor)
                    .map_err(|_| invalid("truncated closure-call dst register"))?;
            }
            let closure = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated closure-call source register"))?;
            let arg = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated closure-call arg register"))?;
            mark_reg(closure);
            mark_reg(arg);
            refs.required_capabilities |= CAP_CLOSURE_VALUES;
        }
        Opcode::LoadVar => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            let sid =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated variable string id"))?;
            refs.string_refs
                .push((offset, sid as usize, "variable reference"));
        }
        Opcode::StoreVar => {
            let sid =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated variable string id"))?;
            refs.string_refs
                .push((offset, sid as usize, "variable reference"));
            let src = read_u16_le(code, cursor).map_err(|_| invalid("truncated src register"))?;
            mark_reg(src);
        }
        Opcode::QNot | Opcode::BoolNot => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            let src = read_u16_le(code, cursor).map_err(|_| invalid("truncated src register"))?;
            mark_reg(dst);
            mark_reg(src);
        }
        Opcode::QAnd
        | Opcode::QOr
        | Opcode::QImpl
        | Opcode::BoolAnd
        | Opcode::BoolOr
        | Opcode::CmpEq
        | Opcode::CmpNe
        | Opcode::CmpI32Lt
        | Opcode::CmpI32Le
        | Opcode::AddF64
        | Opcode::SubF64
        | Opcode::MulF64
        | Opcode::DivF64
        | Opcode::AddFx
        | Opcode::SubFx
        | Opcode::MulFx
        | Opcode::DivFx => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            let lhs = read_u16_le(code, cursor).map_err(|_| invalid("truncated lhs register"))?;
            let rhs = read_u16_le(code, cursor).map_err(|_| invalid("truncated rhs register"))?;
            mark_reg(dst);
            mark_reg(lhs);
            mark_reg(rhs);
            if matches!(
                opcode,
                Opcode::AddF64 | Opcode::SubF64 | Opcode::MulF64 | Opcode::DivF64
            ) {
                refs.required_capabilities |= CAP_F64_MATH;
            }
            if matches!(
                opcode,
                Opcode::AddFx | Opcode::SubFx | Opcode::MulFx | Opcode::DivFx
            ) {
                refs.required_capabilities |= CAP_FX_MATH;
            }
        }
        Opcode::Jmp => {
            let target = read_u32_le(code, cursor).map_err(|_| invalid("truncated jump target"))?;
            refs.jump_targets.push(target as usize);
        }
        Opcode::JmpIf => {
            let cond =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated condition register"))?;
            mark_reg(cond);
            let target = read_u32_le(code, cursor).map_err(|_| invalid("truncated jump target"))?;
            refs.jump_targets.push(target as usize);
        }
        Opcode::Call => {
            read_u8(code, cursor).map_err(|_| invalid("truncated call destination flag"))?;
            let dst =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated call dst register"))?;
            mark_reg(dst);
            let sid =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated callee string id"))?;
            refs.string_refs.push((offset, sid as usize, "call target"));
            let argc = read_u16_le(code, cursor).map_err(|_| invalid("truncated argc"))? as usize;
            for _ in 0..argc {
                let arg = read_u16_le(code, cursor)
                    .map_err(|_| invalid("truncated call arg register"))?;
                mark_reg(arg);
            }
        }
        Opcode::Assert => {
            let cond = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated assert condition register"))?;
            mark_reg(cond);
        }
        Opcode::GateRead => {
            let dst =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated gate dst register"))?;
            mark_reg(dst);
            refs.required_capabilities |= CAP_GATE_SURFACE;
            read_u16_le(code, cursor).map_err(|_| invalid("truncated gate device id"))?;
            read_u16_le(code, cursor).map_err(|_| invalid("truncated gate port"))?;
        }
        Opcode::GateWrite => {
            refs.required_capabilities |= CAP_GATE_SURFACE;
            read_u16_le(code, cursor).map_err(|_| invalid("truncated gate device id"))?;
            read_u16_le(code, cursor).map_err(|_| invalid("truncated gate port"))?;
            let src =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated gate src register"))?;
            mark_reg(src);
        }
        Opcode::PulseEmit => {
            refs.required_capabilities |= CAP_GATE_SURFACE;
            let sid =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated signal string id"))?;
            refs.string_refs
                .push((offset, sid as usize, "pulse signal"));
        }
        Opcode::StateQuery => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated state-query dst register"))?;
            mark_reg(dst);
            refs.required_capabilities |= CAP_STATE_QUERY;
            let sid =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated state query key id"))?;
            refs.string_refs
                .push((offset, sid as usize, "state query key"));
        }
        Opcode::StateUpdate => {
            refs.required_capabilities |= CAP_STATE_UPDATE;
            let sid =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated state update key id"))?;
            refs.string_refs
                .push((offset, sid as usize, "state update key"));
            let src = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated state-update src register"))?;
            mark_reg(src);
        }
        Opcode::EventPost => {
            refs.required_capabilities |= CAP_EVENT_POST;
            let sid =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated event-post signal id"))?;
            refs.string_refs
                .push((offset, sid as usize, "event post signal"));
        }
        Opcode::ClockRead => {
            let dst =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated clock-read dst register"))?;
            mark_reg(dst);
            refs.required_capabilities |= CAP_CLOCK_READ;
        }
        Opcode::Ret => {
            let has_src = read_u8(code, cursor).map_err(|_| invalid("truncated return flag"))?;
            if has_src != 0 {
                let src = read_u16_le(code, cursor)
                    .map_err(|_| invalid("truncated return src register"))?;
                mark_reg(src);
            }
        }
    }

    Ok(refs)
}

#[cfg(feature = "std")]
fn builtin_call_required_capabilities(name: &str) -> Option<u32> {
    match name {
        "sin" | "cos" | "tan" | "sqrt" | "abs" | "pow" => Some(CAP_F64_MATH),
        "to_text" => Some(CAP_TEXT_VALUES),
        "print" => Some(CAP_STDOUT),
        _ => None,
    }
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct OwnershipSectionUsage {
    has_record_field_ownership: bool,
}

#[cfg(feature = "std")]
fn verify_ownership_section(
    function: &str,
    code: &[u8],
    cursor: &mut usize,
    quotas: &RuntimeQuotas,
) -> Result<OwnershipSectionUsage, RejectReport> {
    let section_start = *cursor;
    *cursor += OWNERSHIP_SECTION_TAG.len();
    let mut usage = OwnershipSectionUsage::default();

    let event_count = read_u16_le(code, cursor).map_err(|_| {
        reject_one(
            function,
            VerificationCode::InvalidOwnershipSection,
            section_start,
            "truncated OWN0 section header",
        )
    })? as usize;

    if event_count > quotas.max_symbol_table {
        return Err(reject_one(
            function,
            VerificationCode::ResourceLimitExceeded,
            section_start,
            format!(
                "OWN0 section uses {} events, exceeding the verified-local symbol budget of {}",
                event_count, quotas.max_symbol_table
            ),
        ));
    }

    for event_idx in 0..event_count {
        let event_offset = *cursor;
        let kind = read_u8(code, cursor).map_err(|_| {
            reject_one(
                function,
                VerificationCode::InvalidOwnershipSection,
                event_offset,
                "truncated ownership event kind",
            )
        })?;
        if !matches!(kind, OWNERSHIP_EVENT_KIND_BORROW | OWNERSHIP_EVENT_KIND_WRITE) {
            return Err(reject_one(
                function,
                VerificationCode::InvalidOwnershipSection,
                *cursor - 1,
                format!("unsupported ownership event kind 0x{kind:02x}"),
            ));
        }

        read_u32_le(code, cursor).map_err(|_| {
            reject_one(
                function,
                VerificationCode::InvalidOwnershipSection,
                *cursor,
                "truncated ownership path root",
            )
        })?;
        let component_count = read_u16_le(code, cursor).map_err(|_| {
            reject_one(
                function,
                VerificationCode::InvalidOwnershipSection,
                *cursor,
                "truncated ownership path component count",
            )
        })? as usize;

        if component_count > quotas.max_symbol_table {
            return Err(reject_one(
                function,
                VerificationCode::ResourceLimitExceeded,
                event_offset,
                format!(
                    "ownership path {} uses {} components, exceeding the verified-local symbol budget of {}",
                    event_idx, component_count, quotas.max_symbol_table
                ),
            ));
        }

        let mut event_has_field_component = false;
        for _ in 0..component_count {
            let component_offset = *cursor;
            let component_kind = read_u8(code, cursor).map_err(|_| {
                reject_one(
                    function,
                    VerificationCode::InvalidOwnershipSection,
                    component_offset,
                    "truncated ownership path component kind",
                )
            })?;
            match component_kind {
                OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX => {
                    read_u16_le(code, cursor).map_err(|_| {
                        reject_one(
                            function,
                            VerificationCode::InvalidOwnershipSection,
                            *cursor,
                            "truncated tuple-index ownership path component",
                        )
                    })?;
                }
                OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL => {
                    event_has_field_component = true;
                    read_u32_le(code, cursor).map_err(|_| {
                        reject_one(
                            function,
                            VerificationCode::InvalidOwnershipSection,
                            *cursor,
                            "truncated field-symbol ownership path component",
                        )
                    })?;
                }
                _ => {
                    return Err(reject_one(
                        function,
                        VerificationCode::InvalidOwnershipSection,
                        *cursor - 1,
                        format!(
                            "unsupported ownership path component kind 0x{component_kind:02x}"
                        ),
                    ));
                }
            }
        }

        usage.has_record_field_ownership |= event_has_field_component;
    }

    Ok(usage)
}

#[cfg(feature = "std")]
#[derive(Default)]
struct OperandRefs {
    jump_targets: Vec<usize>,
    string_refs: Vec<(usize, usize, &'static str)>,
    max_register: Option<usize>,
    required_capabilities: u32,
}

#[cfg(feature = "std")]
struct PendingVerifiedFunction {
    verified: VerifiedFunction,
    call_targets: Vec<(usize, String)>,
    has_ownership_section: bool,
}

#[cfg(feature = "std")]
fn reject_one(
    function: &str,
    code: VerificationCode,
    offset: usize,
    message: impl Into<String>,
) -> RejectReport {
    RejectReport {
        diagnostics: vec![diag(
            code,
            Some(function.to_string()),
            Some(offset),
            message,
        )],
    }
}

#[cfg(feature = "std")]
fn diag(
    code: VerificationCode,
    function: Option<String>,
    offset: Option<usize>,
    message: impl Into<String>,
) -> VerificationDiagnostic {
    VerificationDiagnostic {
        code,
        function,
        offset,
        message: message.into(),
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use sm_emit::{
        compile_program_to_semcode, compile_program_to_semcode_with_options_debug, read_u16_le,
        read_u32_le, CompileProfile, OptLevel, MAGIC11, MAGIC12, OWNERSHIP_SECTION_TAG,
    };
    use sm_ir::{emit_ir_to_semcode, IrFunction, IrInstr};

    #[test]
    fn verifier_accepts_valid_semcode() {
        let bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_accepts_fx_semcode() {
        let src = r#"
            fn id(x: fx) -> fx { return x; }
            fn main() {
                let x: fx = 1.25;
                let y: fx = id(-2.0);
                if x == x { return; } else { return; }
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 3);
    }

    #[test]
    fn verifier_accepts_cli_o0_f64_arithmetic_storevar_layout() {
        let src = r#"
            fn main() {
                let y: f64 = 1.0 + 2.0;
                return;
            }
        "#;
        let bytes = compile_program_to_semcode_with_options_debug(
            src,
            CompileProfile::Auto,
            OptLevel::O0,
            false,
        )
        .expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 2);
    }

    #[test]
    fn verifier_accepts_builtin_f64_call_targets() {
        let src = r#"
            fn main() {
                let y: f64 = sqrt(16.0);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode_with_options_debug(
            src,
            CompileProfile::Auto,
            OptLevel::O0,
            false,
        )
        .expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 2);
    }

    #[test]
    fn verifier_accepts_assert_opcode() {
        let src = r#"
            fn main() {
                assert(true);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_accepts_state_query_semcode() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::StateQuery {
                        dst: 0,
                        key: "decision.mode".to_string(),
                    },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 5);
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_accepts_state_update_semcode() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::LoadBool { dst: 0, val: true },
                    IrInstr::StateUpdate {
                        key: "decision.mode".to_string(),
                        src: 0,
                    },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 6);
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_accepts_event_post_semcode() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::EventPost {
                        signal: "alert.raised".to_string(),
                    },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 7);
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_accepts_clock_read_semcode() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![IrInstr::ClockRead { dst: 0 }, IrInstr::Ret { src: None }],
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 8);
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_accepts_text_semcode() {
        let src = r#"
            fn main() {
                let left: text = "alpha";
                let right: text = "alpha";
                assert(left == right);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 9);
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_accepts_u32_numeric_literal_semcode() {
        let src = r#"
            fn main() {
                let left: u32 = 1_000u32;
                let right: u32 = 0x3e8u32;
                assert(left == right);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_accepts_stage1_record_make_record_semcode() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { quality: 0.75, camera: T };
                let _ = ctx;
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_accepts_stage1_record_get_semcode() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { quality: 0.75, camera: T };
                let seen: quad = ctx.camera;
                assert(seen == T);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_accepts_record_pass_return_and_safe_equality_semcode() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn echo(ctx: DecisionContext) -> DecisionContext {
                return ctx;
            }

            fn main() {
                let left: DecisionContext = DecisionContext { quality: 0.75, camera: T };
                let right: DecisionContext = echo(left);
                assert(right == right);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.functions.len(), 2);
    }

    #[test]
    fn verifier_accepts_record_access_policy_scenario() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                badge: quad,
                override_state: quad,
                tamper: quad,
                quality: f64,
            }

            fn allow(ctx: DecisionContext) -> quad {
                if ctx.tamper == T || ctx.tamper == S {
                    return S;
                }
                if ctx.override_state == T {
                    return T;
                }
                if ctx.camera == T && ctx.badge == T {
                    return T;
                }
                return N;
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext {
                    quality: 0.50,
                    tamper: F,
                    override_state: N,
                    badge: T,
                    camera: T,
                };
                let decision: quad = allow(ctx);
                assert(decision == T);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.functions.len(), 2);
    }

    #[test]
    fn verifier_accepts_record_runtime_config_scenario() {
        let src = r#"
            record RuntimeConfig {
                max_steps: u32,
                debug_mode: bool,
                fallback_state: quad,
            }

            fn fallback(cfg: RuntimeConfig) -> quad {
                if cfg.debug_mode == true {
                    return cfg.fallback_state;
                }
                return N;
            }

            fn main() {
                let cfg: RuntimeConfig = RuntimeConfig {
                    fallback_state: S,
                    debug_mode: true,
                    max_steps: 16u32,
                };
                let state: quad = fallback(cfg);
                assert(state == S);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.functions.len(), 2);
    }

    #[test]
    fn verifier_accepts_record_destructuring_bind_semcode() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let DecisionContext { camera: seen_camera, quality: _ } =
                    DecisionContext { quality: 0.75, camera: T };
                assert(seen_camera == T);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_accepts_record_let_else_semcode() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let DecisionContext { camera: T, quality: score } =
                    DecisionContext { quality: 0.75, camera: T } else return;
                assert(score == 0.75);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_accepts_record_copy_with_semcode() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let patched: DecisionContext = ctx with { quality: 1.0 };
                assert(patched.camera == T);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_accepts_record_stage2_ergonomics_scenario() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                override_state: quad,
                quality: f64,
            }

            fn main() {
                let camera: quad = T;
                let override_state: quad = N;
                let quality: f64 = 0.75;
                let ctx: DecisionContext = DecisionContext { camera, override_state, quality };
                let patched: DecisionContext = ctx with { camera };
                let DecisionContext { camera: T, override_state, quality } =
                    patched else return;
                assert(camera == T);
                assert(override_state == N);
                assert(quality == 0.75);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_accepts_ownership_semcode() {
        let bytes = ownership_semcode_bytes();
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 12);
        assert_eq!(verified.functions.len(), 2);
    }

    #[test]
    fn verifier_accepts_record_field_borrow_ownership_semcode() {
        let bytes = record_field_borrow_semcode_bytes();
        assert_eq!(&bytes[..MAGIC12.len()], &MAGIC12);
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 13);
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_accepts_record_field_write_ownership_semcode() {
        let bytes = record_field_write_semcode_bytes();
        assert_eq!(&bytes[..MAGIC12.len()], &MAGIC12);
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 13);
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_accepts_v13_program_without_record_field_ownership_payload() {
        let src = r#"
            fn saw_retry(values: Sequence(i32)) -> bool {
                let found: bool = false;
                for value in values {
                    if value == 2 {
                        found ||= true;
                    }
                }
                return found;
            }

            fn main() {
                let values: Sequence(i32) = [2, 9, 4];
                let found: bool = saw_retry(values);
                assert(found == true);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        assert_eq!(&bytes[..MAGIC12.len()], b"SEMCOD13");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 14);
        assert_eq!(verified.functions.len(), 2);
    }

    #[test]
    fn verifier_rejects_short_header() {
        let report = verify_semcode(b"SEMC").expect_err("must reject");
        assert_eq!(report.diagnostics[0].code, VerificationCode::BadHeader);
    }

    #[test]
    fn verifier_rejects_unknown_opcode() {
        let mut bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");
        let opcode_pos = 8 + 2 + 4 + 4 + 2;
        bytes[opcode_pos] = 0xff;
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(report.diagnostics[0].code, VerificationCode::UnknownOpcode);
    }

    #[test]
    fn verifier_rejects_truncated_function_body() {
        let mut bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");
        bytes.truncate(bytes.len() - 1);
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::TruncatedFunction
        );
    }

    #[test]
    fn verifier_rejects_truncated_string_table() {
        let mut bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");
        let code_len_pos = 8 + 2 + 4;
        bytes[code_len_pos..code_len_pos + 4].copy_from_slice(&1u32.to_le_bytes());
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::InvalidStringTable
        );
    }

    #[test]
    fn verifier_rejects_jump_past_instruction_stream() {
        let mut bytes = compile_program_to_semcode("fn main() { if true { return; } return; }")
            .expect("compile");
        let opcode_pos = find_opcode(&bytes, Opcode::JmpIf.byte()).expect("jmpif");
        let target_pos = opcode_pos + 1 + 2;
        bytes[target_pos..target_pos + 4].copy_from_slice(&999u32.to_le_bytes());
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::InvalidJumpTarget
        );
    }

    #[test]
    fn verifier_rejects_bad_string_reference() {
        let mut bytes =
            compile_program_to_semcode("fn helper() { return; } fn main() { helper(); return; }")
                .expect("compile");
        let opcode_pos = find_opcode(&bytes, Opcode::Call.byte()).expect("call");
        let sid_pos = opcode_pos + 1 + 1 + 2;
        bytes[sid_pos..sid_pos + 2].copy_from_slice(&99u16.to_le_bytes());
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::InvalidStringReference
        );
    }

    #[test]
    fn verifier_rejects_register_past_verified_local_budget() {
        let mut bytes = compile_program_to_semcode("fn main() { let a: bool = true; return; }")
            .expect("compile");
        let opcode_pos = find_opcode(&bytes, Opcode::LoadBool.byte()).expect("load bool");
        let dst_pos = opcode_pos + 1;
        bytes[dst_pos..dst_pos + 2].copy_from_slice(&5000u16.to_le_bytes());
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::InvalidRegisterReference
        );
    }

    #[test]
    fn verifier_rejects_unknown_call_target() {
        let mut bytes =
            compile_program_to_semcode("fn helper() { return; } fn main() { helper(); return; }")
                .expect("compile");
        let helper_pos = bytes
            .windows(b"helper".len())
            .rposition(|window| window == b"helper")
            .expect("helper string");
        bytes[helper_pos..helper_pos + b"helper".len()].copy_from_slice(b"gh0st!");
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::UnknownCallTarget
        );
    }

    #[test]
    fn verifier_rejects_f64_ops_under_v0_capabilities() {
        let src = r#"
            fn main() {
                let x: f64 = 1.0 + 2.0;
                return;
            }
        "#;
        let mut bytes = compile_program_to_semcode(src).expect("compile");
        bytes[7] = b'0';
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::CapabilityViolation
        );
    }

    #[test]
    fn verifier_rejects_missing_ownership_section_under_v11_capabilities() {
        let mut bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");
        bytes[..MAGIC11.len()].copy_from_slice(&MAGIC11);
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::InvalidOwnershipSection
        );
    }

    #[test]
    fn verifier_rejects_invalid_ownership_event_kind() {
        let mut bytes = ownership_semcode_bytes();
        let (_, code_start, code_end) = function_code_span(&bytes, "main");
        let code = &mut bytes[code_start..code_end];
        let section_offset = ownership_section_offset(code);
        let kind_offset = section_offset + 4 + 2;
        code[kind_offset] = 0xff;
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::InvalidOwnershipSection
        );
    }

    #[test]
    fn verifier_rejects_unsupported_ownership_component_kind() {
        let mut bytes = ownership_semcode_bytes();
        let (_, code_start, code_end) = function_code_span(&bytes, "main");
        let code = &mut bytes[code_start..code_end];
        let section_offset = ownership_section_offset(code);
        let component_kind_offset = section_offset + 4 + 2 + 1 + 4 + 2;
        code[component_kind_offset] = 0xff;
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::InvalidOwnershipSection
        );
    }

    #[test]
    fn verifier_rejects_truncated_ownership_path_payload() {
        let mut bytes = ownership_semcode_bytes();
        let (code_len_pos, code_start, code_end) = function_code_span(&bytes, "main");
        let code = &bytes[code_start..code_end];
        let section_offset = ownership_section_offset(code);
        let truncated_code_len = section_offset + 4 + 2 + 1 + 4 + 2 + 1 + 1;
        bytes[code_len_pos..code_len_pos + 4]
            .copy_from_slice(&(truncated_code_len as u32).to_le_bytes());
        bytes.truncate(code_start + truncated_code_len);
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::InvalidOwnershipSection
        );
    }

    #[test]
    fn verifier_rejects_invalid_record_field_component_kind() {
        let mut bytes = record_field_borrow_semcode_bytes();
        let (_, code_start, code_end) = function_code_span(&bytes, "main");
        let code = &mut bytes[code_start..code_end];
        let section_offset = ownership_section_offset(code);
        let component_kind_offset = section_offset + 4 + 2 + 1 + 4 + 2;
        code[component_kind_offset] = 0xff;
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::InvalidOwnershipSection
        );
    }

    #[test]
    fn verifier_rejects_truncated_record_field_payload() {
        let mut bytes = record_field_borrow_semcode_bytes();
        let (code_len_pos, code_start, code_end) = function_code_span(&bytes, "main");
        let code = &bytes[code_start..code_end];
        let section_offset = ownership_section_offset(code);
        let truncated_code_len = section_offset + 4 + 2 + 1 + 4 + 2 + 1 + 3;
        bytes[code_len_pos..code_len_pos + 4]
            .copy_from_slice(&(truncated_code_len as u32).to_le_bytes());
        bytes.truncate(code_start + truncated_code_len);
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::InvalidOwnershipSection
        );
    }

    #[test]
    fn verifier_rejects_record_field_payload_under_v11_capabilities() {
        let mut bytes = record_field_borrow_semcode_bytes();
        bytes[..MAGIC11.len()].copy_from_slice(&MAGIC11);
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::CapabilityViolation
        );
    }

    #[test]
    fn verifier_rejects_ownership_section_under_v10_capabilities() {
        let mut bytes = ownership_semcode_bytes();
        bytes[7] = b'0';
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::CapabilityViolation
        );
    }

    #[test]
    fn verifier_rejects_state_query_under_v3_capabilities() {
        let mut bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::StateQuery {
                        dst: 0,
                        key: "decision.mode".to_string(),
                    },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        bytes[7] = b'3';
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::CapabilityViolation
        );
    }

    #[test]
    fn verifier_rejects_state_update_under_v4_capabilities() {
        let mut bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::LoadBool { dst: 0, val: true },
                    IrInstr::StateUpdate {
                        key: "decision.mode".to_string(),
                        src: 0,
                    },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        bytes[7] = b'4';
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::CapabilityViolation
        );
    }

    #[test]
    fn verifier_rejects_event_post_under_v5_capabilities() {
        let mut bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::EventPost {
                        signal: "alert.raised".to_string(),
                    },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        bytes[7] = b'5';
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::CapabilityViolation
        );
    }

    #[test]
    fn verifier_rejects_clock_read_under_v6_capabilities() {
        let mut bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![IrInstr::ClockRead { dst: 0 }, IrInstr::Ret { src: None }],
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        bytes[7] = b'6';
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::CapabilityViolation
        );
    }

    #[test]
    fn verifier_rejects_text_under_v7_capabilities() {
        let src = r#"
            fn main() {
                let left: text = "alpha";
                let right: text = "alpha";
                assert(left == right);
                return;
            }
        "#;
        let mut bytes = compile_program_to_semcode(src).expect("compile");
        bytes[7] = b'7';
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::CapabilityViolation
        );
    }

    fn ownership_semcode_bytes() -> Vec<u8> {
        let src = r#"
            fn pair() -> (i32, i32) = (1, 2);

            fn main() {
                let pair: (i32, i32) = pair();
                let (ref left, _): (i32, i32) = pair;
                let total: f64 = 0.0;
                total += 1.0;
                return;
            }
        "#;
        compile_program_to_semcode(src).expect("compile")
    }

    fn record_field_borrow_semcode_bytes() -> Vec<u8> {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let DecisionContext { camera: ref seen_camera, quality: _ } = ctx;
                return;
            }
        "#;
        compile_program_to_semcode(src).expect("compile")
    }

    fn record_field_write_semcode_bytes() -> Vec<u8> {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let patched: DecisionContext = ctx with { quality: 1.0 };
                let _ = patched;
                return;
            }
        "#;
        compile_program_to_semcode(src).expect("compile")
    }

    fn function_code_span(bytes: &[u8], target: &str) -> (usize, usize, usize) {
        let mut cursor = 8usize;
        while cursor < bytes.len() {
            let name_len = read_u16_le(bytes, &mut cursor).expect("name length") as usize;
            let name = std::str::from_utf8(&bytes[cursor..cursor + name_len]).expect("utf8 name");
            cursor += name_len;
            let code_len_pos = cursor;
            let code_len = read_u32_le(bytes, &mut cursor).expect("code length") as usize;
            let code_start = cursor;
            let code_end = cursor + code_len;
            if name == target {
                return (code_len_pos, code_start, code_end);
            }
            cursor = code_end;
        }
        panic!("function '{target}' not found");
    }

    fn skip_string_table(code: &[u8]) -> usize {
        let mut cursor = 0usize;
        let count = read_u16_le(code, &mut cursor).expect("string count") as usize;
        for _ in 0..count {
            let len = read_u16_le(code, &mut cursor).expect("string length") as usize;
            cursor += len;
        }
        cursor
    }

    fn ownership_section_offset(code: &[u8]) -> usize {
        let cursor = skip_string_table(code);
        assert!(cursor + OWNERSHIP_SECTION_TAG.len() <= code.len());
        assert_eq!(
            &code[cursor..cursor + OWNERSHIP_SECTION_TAG.len()],
            OWNERSHIP_SECTION_TAG
        );
        cursor
    }

    fn find_opcode(bytes: &[u8], opcode: u8) -> Option<usize> {
        bytes.iter().position(|byte| *byte == opcode)
    }
}
