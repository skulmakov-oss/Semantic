#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use sm_format::semcode_format::{
    read_f64_le, read_i32_le, read_u16_le, read_u32_le, read_u8, Opcode, SemcodeFormatError,
    SemcodeHeaderSpec, CAP_ARGS_READ, CAP_CLOCK_READ, CAP_CLOSURE_VALUES, CAP_DEBUG_SYMBOLS,
    CAP_EVENT_POST, CAP_F64_MATH, CAP_FS_READ, CAP_FS_WRITE, CAP_FX_MATH, CAP_FX_VALUES,
    CAP_GATE_SURFACE, CAP_MAP_VALUES, CAP_OWNERSHIP_FIELD_PATHS, CAP_OWNERSHIP_PATHS,
    CAP_PATH_INSPECT, CAP_PRNG, CAP_SEQUENCE_ITERATION, CAP_SEQUENCE_VALUES, CAP_STATE_QUERY,
    CAP_STATE_UPDATE, CAP_STDERR_WRITE, CAP_STDIN_READ_TEXT, CAP_STDOUT, CAP_STDOUT_WRITE,
    CAP_TEXT_VALUES, CAP_TIME_DURATION,
};
use sm_runtime_core::RuntimeQuotas;
use std::collections::HashSet;

#[cfg(feature = "std")]
pub mod hello_pending_admission;
#[cfg(feature = "std")]
pub mod hello_real_semcode_admission;

#[cfg(feature = "std")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationCode {
    BadHeader,
    UnsupportedVersion,
    TruncatedFunction,
    InvalidFunctionName,
    DuplicateFunction,
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
    /// The byte range immediately after the string table has more than one
    /// valid canonical structural reading: it can be interpreted as an
    /// empty/populated `DBG0` debug section AND, independently, as a
    /// complete, well-formed instruction stream. Verified SemCode must have
    /// exactly one canonical structural interpretation (see #1731); when
    /// both readings are structurally valid, admission fails closed rather
    /// than silently choosing one.
    AmbiguousInstructionFraming,
    /// The decoded opcode's minimum SemCode header revision
    /// (`Opcode::minimum_semcode_revision`) exceeds the artifact's actual
    /// header revision (see #1732 / FA-05-002). This is a version-identity
    /// gap, not a missing-capability gap: the opcode is structurally valid
    /// and every capability it needs (if any) is present, but the artifact
    /// header predates the revision whose contract actually admits this
    /// opcode's semantics.
    OpcodeRequiresNewerHeader,
    /// A control-flow successor reachable from the function entry falls off
    /// the executable instruction stream instead of reaching another
    /// instruction boundary or an admitted terminal instruction (`RET`).
    /// Closed loops remain admissible; this code does not imply termination.
    ReachableFunctionFallthrough,
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

/// Canonical admission token for a verified SemCode artifact.
///
/// Instances of this type represent SemCode bytes that have successfully
/// passed the `verify_semcode_token` admission gate. This forms the first
/// boundary in the canonical verified execution path.
#[cfg(feature = "std")]
#[derive(Debug)]
pub struct VerifiedSemCode<'a> {
    bytes: &'a [u8],
    program: VerifiedProgram,
    decoded: Vec<sm_format::semcode_decode::DecodedFunctionEnvelope<'a>>,
}

/// Error type for canonical entry resolution.
///
/// This error indicates that a `VerifiedSemCode` artifact does not contain
/// the required entry function, preventing it from proceeding to the
/// `VerifiedEntrySemCode` boundary.
#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryResolutionError {
    MissingEntry { entry: String },
}

#[cfg(feature = "std")]
impl core::fmt::Display for EntryResolutionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EntryResolutionError::MissingEntry { entry } => {
                write!(f, "entry function '{}' not found in SemCode", entry)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for EntryResolutionError {}

/// Canonical entry-resolved token for a verified SemCode artifact.
///
/// Instances of this type represent a verified artifact that has also
/// been confirmed to contain a specific entry point function. This forms
/// the second boundary in the canonical verified execution path, directly
/// preceding execution in the `sm-vm` layer.
#[cfg(feature = "std")]
#[derive(Debug, Clone)]
pub struct VerifiedEntrySemCode<'token, 'bytes> {
    artifact: &'token VerifiedSemCode<'bytes>,
    entry: String,
}

#[cfg(feature = "std")]
impl<'token, 'bytes> VerifiedEntrySemCode<'token, 'bytes> {
    pub fn artifact(&self) -> &'token VerifiedSemCode<'bytes> {
        self.artifact
    }

    pub fn entry(&self) -> &str {
        &self.entry
    }

    pub fn bytes(&self) -> &'bytes [u8] {
        self.artifact.bytes()
    }

    pub fn program(&self) -> &VerifiedProgram {
        self.artifact.program()
    }
}

#[cfg(feature = "std")]
impl<'a> VerifiedSemCode<'a> {
    pub fn bytes(&self) -> &'a [u8] {
        self.bytes
    }

    pub fn program(&self) -> &VerifiedProgram {
        &self.program
    }

    pub fn function_names(&self) -> impl Iterator<Item = &str> {
        self.decoded.iter().map(|env| env.name.as_str())
    }

    pub fn has_entry(&self, entry: &str) -> bool {
        self.decoded.iter().any(|env| env.name == entry)
    }

    /// Canonical entry resolution gate.
    ///
    /// Upgrades this admission token to a `VerifiedEntrySemCode` token
    /// if the specified entry function is present in the verified artifact.
    pub fn require_entry<'token>(
        &'token self,
        entry: &str,
    ) -> Result<VerifiedEntrySemCode<'token, 'a>, EntryResolutionError> {
        if self.has_entry(entry) {
            Ok(VerifiedEntrySemCode {
                artifact: self,
                entry: entry.to_string(),
            })
        } else {
            Err(EntryResolutionError::MissingEntry {
                entry: entry.to_string(),
            })
        }
    }

    /// Workspace-internal accessor for VM preparation.
    /// Exposes the decoded internal state safely without creating a stable public API.
    #[cfg(feature = "std")]
    #[doc(hidden)]
    pub fn with_decoded_envelopes<R, F>(&self, f: F) -> R
    where
        F: for<'scope> FnOnce(
            &'scope sm_format::semcode_format::SemcodeHeaderSpec,
            &'scope [sm_format::semcode_decode::DecodedFunctionEnvelope<'a>],
        ) -> R,
    {
        f(&self.program.header, &self.decoded)
    }
}

/// Canonical admission gate for SemCode bytes.
///
/// This is the primary entry point for the canonical verified execution path.
/// It validates raw SemCode bytes against admission policies and returns a
/// `VerifiedSemCode` token if successful.
#[cfg(feature = "std")]
pub fn verify_semcode_token(bytes: &[u8]) -> Result<VerifiedSemCode<'_>, RejectReport> {
    let mut diagnostics = Vec::new();
    let quotas = RuntimeQuotas::verified_local();

    let (header, decoded_functions) =
        match sm_format::semcode_decode::decode_semcode_envelope(bytes) {
            Ok(v) => v,
            Err(err) => {
                let diag = match err {
                    sm_format::semcode_decode::DecodeError::BadHeader => diag(
                        VerificationCode::BadHeader,
                        None,
                        None,
                        "SemCode file is shorter than the 8-byte header",
                    ),
                    sm_format::semcode_decode::DecodeError::UnsupportedVersion {
                        found, ..
                    } => diag(
                        VerificationCode::UnsupportedVersion,
                        None,
                        Some(0),
                        format!("unsupported SemCode header '{}'", found),
                    ),
                    sm_format::semcode_decode::DecodeError::TruncatedFunction { offset, msg } => {
                        diag(VerificationCode::TruncatedFunction, None, Some(offset), msg)
                    }
                    sm_format::semcode_decode::DecodeError::InvalidFunctionName { offset, msg } => {
                        diag(
                            VerificationCode::InvalidFunctionName,
                            None,
                            Some(offset),
                            msg,
                        )
                    }
                    sm_format::semcode_decode::DecodeError::InvalidStringTable { offset, msg } => {
                        diag(
                            VerificationCode::InvalidStringTable,
                            None,
                            Some(offset),
                            msg,
                        )
                    }
                    sm_format::semcode_decode::DecodeError::InvalidDebugSection { offset, msg } => {
                        diag(
                            VerificationCode::InvalidDebugSection,
                            None,
                            Some(offset),
                            msg,
                        )
                    }
                    sm_format::semcode_decode::DecodeError::InvalidOwnershipSection {
                        offset,
                        msg,
                    } => diag(
                        VerificationCode::InvalidOwnershipSection,
                        None,
                        Some(offset),
                        msg,
                    ),
                    sm_format::semcode_decode::DecodeError::ResourceLimit { offset, msg } => diag(
                        VerificationCode::ResourceLimitExceeded,
                        None,
                        Some(offset),
                        msg,
                    ),
                };
                diagnostics.push(diag);
                return Err(RejectReport { diagnostics });
            }
        };

    let mut functions = Vec::new();
    let mut pending_functions = Vec::new();
    let mut seen_names = HashSet::new();

    for env in &decoded_functions {
        let function_start = env.name_offset;
        let name = env.name.clone();

        if !seen_names.insert(name.clone()) {
            diagnostics.push(diag(
                VerificationCode::DuplicateFunction,
                Some(name.clone()),
                Some(function_start),
                format!("duplicate function '{}'", name),
            ));
            break;
        }

        match verify_function_code(env, &header, &quotas) {
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
        for (offset, callee, allows_builtin) in &function.call_targets {
            if known_functions.contains(callee.as_str()) {
                continue;
            }

            if *allows_builtin {
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
        Ok(VerifiedSemCode {
            bytes,
            program: VerifiedProgram { header, functions },
            decoded: decoded_functions,
        })
    } else {
        Err(RejectReport { diagnostics })
    }
}

/// Legacy admission gate (returns verified program model directly).
///
/// Prefer `verify_semcode_token` for canonical execution flows that require
/// the token-based boundary pattern.
#[cfg(feature = "std")]
pub fn verify_semcode(bytes: &[u8]) -> Result<VerifiedProgram, RejectReport> {
    verify_semcode_token(bytes).map(|token| token.program.clone())
}

/// Returns true if `code[start..]` decodes as a complete, well-formed
/// instruction stream: every opcode byte is valid, every operand's byte
/// shape is fully present (including count/flag-controlled variable-length
/// operands), and the walk consumes exactly to `code.len()` with nothing
/// left over.
///
/// This is a STRUCTURAL question only - it must not depend on whether the
/// resulting operand *values* happen to be semantically canonical (e.g. a
/// bool literal byte of 2, or a presence flag of 5). Ambiguous framing is a
/// fact about the bytes' shape: a decoder that doesn't apply the verifier's
/// current canonical-domain policy (an older tool version, a disassembler,
/// a different implementation) could still read this range as instructions.
/// Coupling "was this ambiguous" to "is sm-verify's semantic policy today
/// willing to admit the alternative reading" would make the one-canonical-
/// interpretation invariant depend on semantic policy instead of being a
/// decoder-level fact, and would silently keep the DBG0 reading whenever
/// the competing instruction reading merely contains a non-canonical
/// literal - exactly the kind of hidden-content risk #1731 exists to close.
///
/// Reuses `decode_operands` - the same function, same opcode-shape match,
/// that the normal admission walk uses - with canonical-domain enforcement
/// turned off via its `enforce_canonical_domains` parameter, rather than a
/// second, independently-maintained opcode-shape table. Every byte read and
/// every count/flag-controlled branch is identical to the semantic-
/// admission walk; only the four inline out-of-domain rejections (LOAD_Q,
/// LOAD_BOOL, CALL/CLOSURE_CALL dst-present flag, RET src-present flag) are
/// skipped here.
#[cfg(feature = "std")]
fn instruction_stream_parses_fully(name: &str, code: &[u8], start: usize) -> bool {
    let mut cursor = start;
    while cursor < code.len() {
        let offset = cursor - start;
        let Ok(raw_opcode) = read_u8(code, &mut cursor) else {
            return false;
        };
        let Ok(opcode) = Opcode::from_byte(raw_opcode) else {
            return false;
        };
        if decode_operands(name, code, &mut cursor, offset, opcode, false).is_err() {
            return false;
        }
    }
    cursor == code.len()
}

#[cfg(feature = "std")]
fn verify_function_code(
    env: &sm_format::semcode_decode::DecodedFunctionEnvelope,
    header: &SemcodeHeaderSpec,
    quotas: &RuntimeQuotas,
) -> Result<PendingVerifiedFunction, RejectReport> {
    let name = env.name.as_str();

    // #1731 (FA-05-001): the DBG0 sentinel collides with TupleGet's opcode
    // byte (0x44 = 'D'). If the decoder recognized a DBG0 section, check
    // whether the same bytes, read from string_table_end_offset with no
    // metadata-section recognition at all, would ALSO form a complete,
    // well-formed instruction stream all the way to the end of this
    // function's code. If both readings are valid, the artifact has more
    // than one canonical structural interpretation and admission must fail
    // closed rather than silently keep the DBG0 reading.
    if env.has_debug_section
        && instruction_stream_parses_fully(name, env.code_slice, env.string_table_end_offset)
    {
        return Err(reject_one(
            name,
            VerificationCode::AmbiguousInstructionFraming,
            env.string_table_end_offset,
            "byte range is both a valid DBG0 debug section and a valid instruction stream",
        ));
    }

    let string_count = env.strings.len();
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

    let debug_symbol_count = env.debug_symbols.len();
    if debug_symbol_count > quotas.max_trace_entries {
        return Err(reject_one(
            name,
            VerificationCode::ResourceLimitExceeded,
            0,
            format!(
                "debug section uses {} entries, exceeding the verified-local trace budget of {}",
                debug_symbol_count, quotas.max_trace_entries
            ),
        ));
    }

    let has_ownership_section = env.has_ownership_section;
    let mut has_record_field_ownership = false;
    for p in env.borrowed_paths.iter().chain(env.write_paths.iter()) {
        for c in &p.components {
            match c {
                sm_format::semcode_decode::DecodedAccessPathComponent::FieldSymbol(_) => {
                    has_record_field_ownership = true;
                }
                sm_format::semcode_decode::DecodedAccessPathComponent::AdtPayload { .. } => {
                    // Variant is a global SymbolId; it cannot be bounds-checked
                    // against the local string table. Structural acceptance only.
                }
                sm_format::semcode_decode::DecodedAccessPathComponent::SequenceIndexStatic(_) => {
                    // Static sequence index ownership is structurally accepted.
                }
                _ => {}
            }
        }
    }

    let code = env.code_slice;
    let mut cursor = env.instr_start_offset;
    let instr_start = cursor;
    let instr_len = code.len().saturating_sub(instr_start);
    let mut instr_starts = Vec::new();
    let mut instruction_successors = Vec::new();
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
        let refs = decode_operands(name, code, &mut cursor, offset, opcode, true)?;
        let next_offset = cursor - instr_start;
        let jump_target = refs.jump_targets.first().copied();
        let successors = match (opcode, jump_target) {
            (Opcode::Ret, _) => InstructionSuccessors::None,
            (Opcode::Jmp, Some(target)) => InstructionSuccessors::One(target),
            (Opcode::JmpIf, Some(target)) => InstructionSuccessors::Two(target, next_offset),
            (Opcode::Jmp | Opcode::JmpIf, None) => {
                return Err(reject_one(
                    name,
                    VerificationCode::InvalidJumpTarget,
                    offset,
                    "jump instruction has no decoded target",
                ));
            }
            _ => InstructionSuccessors::One(next_offset),
        };
        instruction_successors.push(successors);
        let min_rev = opcode.minimum_semcode_revision();
        if header.rev < min_rev {
            return Err(reject_one(
                name,
                VerificationCode::OpcodeRequiresNewerHeader,
                offset,
                format!(
                    "opcode {opcode:?} requires SemCode header revision >= {min_rev}, but artifact header '{}' is revision {}",
                    String::from_utf8_lossy(&header.magic),
                    header.rev
                ),
            ));
        }
        jump_targets.extend(refs.jump_targets);
        string_refs.extend(refs.string_refs);
        used_caps |= refs.required_capabilities;
        max_register = match (max_register, refs.max_register) {
            (Some(lhs), Some(rhs)) => Some(lhs.max(rhs)),
            (None, some) => some,
            (some, None) => some,
        };
    }

    for sym in &env.debug_symbols {
        if sym.pc >= instr_len {
            return Err(reject_one(
                name,
                VerificationCode::InvalidDebugSection,
                sym.pc,
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
    if has_record_field_ownership {
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

    verify_reachable_control_flow(name, instr_len, &instr_starts, &instruction_successors)?;

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
        match usage {
            "call target" => call_targets.push((offset, env.strings[sid].clone(), true)),
            "closure function name" => {
                call_targets.push((offset, env.strings[sid].clone(), false));
            }
            _ => {}
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
enum InstructionSuccessors {
    None,
    One(usize),
    Two(usize, usize),
}

#[cfg(feature = "std")]
fn verify_reachable_control_flow(
    function: &str,
    instr_len: usize,
    instr_starts: &[usize],
    instruction_successors: &[InstructionSuccessors],
) -> Result<(), RejectReport> {
    let mut pending = vec![0usize];
    let mut reachable = HashSet::new();

    while let Some(offset) = pending.pop() {
        if !reachable.insert(offset) {
            continue;
        }
        if offset == instr_len {
            return Err(reject_one(
                function,
                VerificationCode::ReachableFunctionFallthrough,
                offset,
                "reachable control flow falls off the end of the instruction stream",
            ));
        }
        let index = instr_starts.binary_search(&offset).map_err(|_| {
            reject_one(
                function,
                VerificationCode::ReachableFunctionFallthrough,
                offset,
                "reachable control-flow successor is not an instruction boundary",
            )
        })?;
        match &instruction_successors[index] {
            InstructionSuccessors::None => {}
            InstructionSuccessors::One(successor) => pending.push(*successor),
            InstructionSuccessors::Two(first, second) => {
                pending.push(*first);
                pending.push(*second);
            }
        }
    }

    Ok(())
}

/// Decodes one instruction's operands, advancing `cursor` past its full byte
/// shape (including count/flag-controlled variable-length fields).
///
/// `enforce_canonical_domains` separates two distinct concerns that this
/// function used to conflate:
///
/// - STRUCTURAL SHAPE: opcode recognition, operand byte widths, and
///   presence/count-controlled byte lengths - always applied, regardless of
///   the flag. This is what determines whether a byte range decodes as a
///   complete instruction stream at all.
/// - SEMANTIC ADMISSION: canonical literal value domains (`LOAD_Q`,
///   `LOAD_BOOL`), canonical presence-flag domains (`CALL`,
///   `CLOSURE_CALL`, `RET`), and canonical arity/cardinality domains
///   (`MAKE_TUPLE` arity `>= 2`, `MAKE_RECORD` slot count `>= 1`) - applied
///   only when `enforce_canonical_domains` is `true`. In every one of these
///   cases the count or flag byte itself is read unconditionally and always
///   determines how many further operand bytes follow (there is no
///   width-affecting difference between a canonical and non-canonical
///   value), so disabling this enforcement never changes byte-shape
///   consumption - only whether an out-of-domain value is rejected.
///
/// Every other rejection in this function's match arms is a truncation /
/// missing-bytes error from a failed `read_*` call - i.e. genuinely
/// structural (unknown opcode is rejected by the caller before this
/// function is even entered; every remaining path here fails only on
/// missing/truncated operand bytes).
///
/// Normal verifier admission (the real instruction-stream walk in
/// `verify_function_code`) must enforce both, so it passes `true`. The
/// `#1731` ambiguity probe (`instruction_stream_parses_fully`) must decide
/// only whether an alternative byte-level *framing* exists at all - a
/// non-canonical operand value doesn't make an otherwise complete
/// instruction-shaped reading any less structurally real - so it passes
/// `false`. This keeps both concerns on one shared opcode-shape match
/// rather than duplicating it.
#[cfg(feature = "std")]
fn decode_operands(
    function: &str,
    code: &[u8],
    cursor: &mut usize,
    offset: usize,
    opcode: Opcode,
    enforce_canonical_domains: bool,
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
            let literal = read_u8(code, cursor).map_err(|_| invalid("truncated quad literal"))?;
            if enforce_canonical_domains && literal > 3 {
                return Err(invalid("non-canonical quad literal: must be 0..=3"));
            }
        }
        Opcode::LoadBool => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            let literal = read_u8(code, cursor).map_err(|_| invalid("truncated bool literal"))?;
            if enforce_canonical_domains && literal > 1 {
                return Err(invalid("non-canonical bool literal: must be 0 or 1"));
            }
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
            let sid = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated text literal string id"))?;
            refs.string_refs
                .push((offset, sid as usize, "text literal"));
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
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence dst register"))?;
            mark_reg(dst);
            refs.required_capabilities |= CAP_SEQUENCE_VALUES;
            let count = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence arity"))? as usize;
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
            if enforce_canonical_domains && count < 2 {
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
            if enforce_canonical_domains && count == 0 {
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
            let dst =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated map-get dst register"))?;
            let map =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated map-get map register"))?;
            let key =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated map-get key register"))?;
            let default_val = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated map-get default register"))?;
            mark_reg(dst);
            mark_reg(map);
            mark_reg(key);
            mark_reg(default_val);
            refs.required_capabilities |= CAP_MAP_VALUES;
        }
        Opcode::MapSet => {
            let dst =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated map-set dst register"))?;
            let map =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated map-set map register"))?;
            let key =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated map-set key register"))?;
            let val =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated map-set val register"))?;
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
            let dst =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated closure dst register"))?;
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
            let has_dst_flag =
                read_u8(code, cursor).map_err(|_| invalid("truncated closure-call dst flag"))?;
            if enforce_canonical_domains && has_dst_flag > 1 {
                return Err(invalid(
                    "non-canonical closure-call dst flag: must be 0 or 1",
                ));
            }
            let has_dst = has_dst_flag != 0;
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
        Opcode::QNot | Opcode::BoolNot | Opcode::QTruthNot => {
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
        | Opcode::QTruthAnd
        | Opcode::QTruthOr
        | Opcode::QTruthImpl
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
            let has_dst_flag =
                read_u8(code, cursor).map_err(|_| invalid("truncated call destination flag"))?;
            if enforce_canonical_domains && has_dst_flag > 1 {
                return Err(invalid(
                    "non-canonical call destination flag: must be 0 or 1",
                ));
            }
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
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated clock-read dst register"))?;
            mark_reg(dst);
            refs.required_capabilities |= CAP_CLOCK_READ;
        }
        Opcode::Ret => {
            let has_src = read_u8(code, cursor).map_err(|_| invalid("truncated return flag"))?;
            if enforce_canonical_domains && has_src > 1 {
                return Err(invalid("non-canonical return flag: must be 0 or 1"));
            }
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
        "args_read" => Some(CAP_ARGS_READ),
        "stdin_read_text" => Some(CAP_STDIN_READ_TEXT),
        "stdout_write" => Some(CAP_STDOUT_WRITE),
        "stderr_write" => Some(CAP_STDERR_WRITE),
        "path_inspect" => Some(CAP_PATH_INSPECT),
        "fs_read_text" => Some(CAP_FS_READ),
        "fs_write_text" => Some(CAP_FS_WRITE),
        "time_duration_ms" => Some(CAP_TIME_DURATION),
        _ => None,
    }
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
    call_targets: Vec<(usize, String, bool)>,
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
    use sm_format::semcode_format::{
        read_u16_le, read_u32_le, MAGIC11, MAGIC12, OWNERSHIP_SECTION_TAG,
    };
    use sm_ir::{
        compile_program_to_semcode, compile_program_to_semcode_with_options_debug,
        emit_ir_to_semcode, CompileProfile, IrFunction, IrInstr, OptLevel,
    };

    fn emit_test_function(instrs: Vec<IrInstr>) -> Vec<u8> {
        emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs,
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit test function")
    }

    #[test]
    fn verifier_rejects_reachable_ordinary_instruction_fallthrough() {
        let bytes = emit_test_function(vec![IrInstr::LoadBool { dst: 0, val: true }]);
        let report =
            verify_semcode(&bytes).expect_err("reachable LOAD_BOOL followed by EOF must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::ReachableFunctionFallthrough
        );
        assert_eq!(report.diagnostics[0].offset, Some(4));
    }

    #[test]
    fn verifier_rejects_empty_instruction_stream() {
        let bytes = emit_test_function(Vec::new());
        let report =
            verify_semcode(&bytes).expect_err("empty executable instruction stream must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::ReachableFunctionFallthrough
        );
        assert_eq!(report.diagnostics[0].offset, Some(0));
    }

    #[test]
    fn verifier_rejects_conditional_branch_with_eof_fallthrough() {
        let bytes = emit_test_function(vec![
            IrInstr::LoadBool { dst: 0, val: true },
            IrInstr::Label {
                name: "branch".to_string(),
            },
            IrInstr::JmpIf {
                cond: 0,
                label: "branch".to_string(),
            },
        ]);
        let report = verify_semcode(&bytes)
            .expect_err("conditional branch with a reachable EOF successor must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::ReachableFunctionFallthrough
        );
    }

    #[test]
    fn verifier_accepts_function_ending_in_ret() {
        let bytes = emit_test_function(vec![IrInstr::Ret { src: None }]);
        verify_semcode(&bytes).expect("RET is a terminal instruction");
    }

    #[test]
    fn verifier_accepts_unconditional_jump_to_valid_return_boundary() {
        let bytes = emit_test_function(vec![
            IrInstr::Jmp {
                label: "return".to_string(),
            },
            IrInstr::LoadBool { dst: 0, val: true },
            IrInstr::Label {
                name: "return".to_string(),
            },
            IrInstr::Ret { src: None },
        ]);
        verify_semcode(&bytes).expect("JMP has only its valid explicit target as successor");
    }

    #[test]
    fn verifier_accepts_structurally_closed_infinite_loop() {
        let bytes = emit_test_function(vec![
            IrInstr::Label {
                name: "loop".to_string(),
            },
            IrInstr::Jmp {
                label: "loop".to_string(),
            },
            IrInstr::LoadBool { dst: 0, val: true },
        ]);
        verify_semcode(&bytes)
            .expect("closed reachable control flow may leave trailing fallthrough unreachable");
    }

    #[test]
    fn verifier_preserves_call_and_closure_call_fallthrough() {
        let direct_call = emit_ir_to_semcode(
            &[
                IrFunction {
                    name: "helper".to_string(),
                    instrs: vec![IrInstr::Ret { src: None }],
                    ownership_events: Vec::new(),
                },
                IrFunction {
                    name: "main".to_string(),
                    instrs: vec![
                        IrInstr::Call {
                            dst: None,
                            name: "helper".to_string(),
                            args: Vec::new(),
                        },
                        IrInstr::Ret { src: None },
                    ],
                    ownership_events: Vec::new(),
                },
            ],
            false,
        )
        .expect("emit direct call");
        verify_semcode(&direct_call).expect("CALL returns to the decoded next instruction");

        let closure_call = emit_ir_to_semcode(
            &[
                IrFunction {
                    name: "helper".to_string(),
                    instrs: vec![IrInstr::Ret { src: None }],
                    ownership_events: Vec::new(),
                },
                IrFunction {
                    name: "main".to_string(),
                    instrs: vec![
                        IrInstr::MakeClosure {
                            dst: 0,
                            name: "helper".to_string(),
                            captures: Vec::new(),
                        },
                        IrInstr::LoadBool { dst: 1, val: true },
                        IrInstr::ClosureCall {
                            dst: None,
                            closure: 0,
                            arg: 1,
                        },
                        IrInstr::Ret { src: None },
                    ],
                    ownership_events: Vec::new(),
                },
            ],
            false,
        )
        .expect("emit closure call");
        verify_semcode(&closure_call)
            .expect("CLOSURE_CALL returns to the decoded next instruction");
    }

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
        let opcode_pos = find_instruction(&bytes, "main", Opcode::JmpIf, 0);
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
        let opcode_pos = find_instruction(&bytes, "main", Opcode::Call, 0);
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
        let opcode_pos = find_instruction(&bytes, "main", Opcode::LoadBool, 0);
        let dst_pos = opcode_pos + 1;
        bytes[dst_pos..dst_pos + 2].copy_from_slice(&5000u16.to_le_bytes());
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::InvalidRegisterReference
        );
    }

    // FA-07-001 (#1741): LOAD_BOOL literal byte must be canonical 0/1. A byte
    // outside that domain must be rejected at admission, not silently
    // normalized to `true` by the VM's `!= 0` check.
    #[test]
    fn verifier_rejects_non_canonical_bool_literal() {
        let mut bytes = compile_program_to_semcode("fn main() { let a: bool = true; return; }")
            .expect("compile");
        let opcode_pos = find_instruction(&bytes, "main", Opcode::LoadBool, 0);
        let literal_pos = opcode_pos + 1 + 2;
        bytes[literal_pos] = 0xff;
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::OperandOutOfBounds
        );
    }

    // FA-07-002 (#1742): LOAD_Q literal byte must be canonical 0..=3. A byte
    // outside that domain must be rejected at admission instead of receiving
    // a Verified token and only failing later in the VM as BadFormat.
    #[test]
    fn verifier_rejects_non_canonical_quad_literal() {
        // This exact fixture interns "a" as a string; the string table's
        // length byte for a 1-character string equals LoadQ's opcode byte
        // (0x01), which used to make a naive whole-buffer byte scan land on
        // the string table instead of the real LOAD_Q instruction (see
        // #1791). find_instruction locates it via the decoded instruction
        // stream instead, so it is unaffected by that coincidence.
        let mut bytes =
            compile_program_to_semcode("fn main() { let a: quad = N; return; }").expect("compile");
        let opcode_pos = find_instruction(&bytes, "main", Opcode::LoadQ, 0);
        let literal_pos = opcode_pos + 1 + 2;
        bytes[literal_pos] = 0xff;
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::OperandOutOfBounds
        );
    }

    // FA-07-003 (#1743): CALL's destination-present flag must be canonical
    // 0/1. The verifier currently reads and discards this byte entirely.
    #[test]
    fn verifier_rejects_non_canonical_call_dst_flag() {
        let mut bytes =
            compile_program_to_semcode("fn helper() { return; } fn main() { helper(); return; }")
                .expect("compile");
        let opcode_pos = find_instruction(&bytes, "main", Opcode::Call, 0);
        bytes[opcode_pos + 1] = 0xff;
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::OperandOutOfBounds
        );
    }

    // FA-07-003 (#1743): CLOSURE_CALL's destination-present flag must be
    // canonical 0/1, not "any nonzero byte is true".
    #[test]
    fn verifier_rejects_non_canonical_closure_call_dst_flag() {
        let mut bytes = emit_ir_to_semcode(
            &[
                IrFunction {
                    name: "helper".to_string(),
                    instrs: vec![IrInstr::Ret { src: None }],
                    ownership_events: Vec::new(),
                },
                IrFunction {
                    name: "main".to_string(),
                    instrs: vec![
                        IrInstr::MakeClosure {
                            dst: 0,
                            name: "helper".to_string(),
                            captures: Vec::new(),
                        },
                        IrInstr::ClosureCall {
                            dst: None,
                            closure: 0,
                            arg: 0,
                        },
                        IrInstr::Ret { src: None },
                    ],
                    ownership_events: Vec::new(),
                },
            ],
            false,
        )
        .expect("emit");
        let opcode_pos = find_instruction(&bytes, "main", Opcode::ClosureCall, 0);
        bytes[opcode_pos + 1] = 0xff;
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::OperandOutOfBounds
        );
    }

    // FA-07-003 (#1743): RET's source-present flag must be canonical 0/1.
    #[test]
    fn verifier_rejects_non_canonical_ret_src_flag() {
        let mut bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");
        let opcode_pos = find_instruction(&bytes, "main", Opcode::Ret, 0);
        bytes[opcode_pos + 1] = 0xff;
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::OperandOutOfBounds
        );
    }

    // FA-05-001 (#1731): TupleGet's opcode byte (0x44 = 'D') can begin the
    // exact same six bytes as the DBG0 debug-section sentinel + an empty
    // debug_symbol_count (`44 42 47 30 00 00` == "DBG0" + 0u16). This exact
    // IR sequence matches the confirmed Phase A fixture: it makes the
    // decoder currently reclassify the producer's real first instruction
    // (TupleGet dst=0x4742) as an empty debug section, hiding a destination
    // register far outside the verified-local register budget from every
    // downstream admission check. Both readings of these six bytes are
    // structurally valid (empty DBG0 metadata, or the start of a real
    // instruction stream), so admission must fail closed rather than
    // silently choosing one.
    #[test]
    fn verifier_rejects_ambiguous_dbg0_tupleget_collision() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::TupleGet {
                        dst: 0x4742,
                        src: 0x0030,
                        index: 0x6600,
                    },
                    IrInstr::Ret { src: None },
                    IrInstr::ClockRead { dst: 0 },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit");

        // B: byte identity - the emitted instruction stream literally spells
        // DBG0 where the real TupleGet begins (string table is empty: a
        // bare 2-byte count=0, so the collision starts at code_slice[2]).
        let (_, functions) =
            sm_format::semcode_decode::decode_semcode_envelope(&bytes).expect("decode");
        let env = &functions[0];
        assert_eq!(&env.code_slice[2..6], b"DBG0");

        // C: decoder reinterpretation - instr_start_offset must currently
        // land past the fake sentinel + fake empty count (2 + 4 + 2 = 8),
        // not at the true instruction start (2, right after the empty
        // string table) - i.e. this is not merely byte-identical, the
        // shared decoder actually consumes it as metadata today.
        assert_eq!(env.instr_start_offset, 8);

        // D+E: admission consequence - despite the hidden TupleGet
        // referencing register 0x4742 (18242), far outside any verified-
        // local register budget, admission must fail closed rather than
        // silently accept it.
        let report = verify_semcode(&bytes).expect_err("must reject ambiguous framing");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::AmbiguousInstructionFraming
        );
    }

    // #1731 regression matrix (2/3): a minimal genuine DBG0 section (debug
    // symbols enabled, one traced instruction) must remain accepted
    // unchanged - it is not byte-identical to any instruction reading, so
    // it is not ambiguous. Built via emit_ir_to_semcode (IR-level, not the
    // source front-end) so this test doesn't depend on the `debug-symbols`
    // cargo feature being enabled for every invocation this repo uses to
    // run sm-verify's tests (e.g. 7hell Hell 4 enables sm-ir/profile-rust
    // only).
    #[test]
    fn verifier_accepts_minimal_genuine_debug_section() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![IrInstr::Ret { src: None }],
                ownership_events: Vec::new(),
            }],
            true,
        )
        .expect("emit");
        let verified = verify_semcode(&bytes).expect("genuine debug section must verify");
        assert_eq!(verified.functions.len(), 1);
        assert!(verified.functions[0].debug_symbol_count > 0);
    }

    // #1731 regression matrix (4): an ordinary, non-empty DBG0 section
    // (several traced instructions) must remain accepted unchanged.
    #[test]
    fn verifier_accepts_ordinary_debug_section_with_multiple_entries() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::LoadI32 { dst: 0, val: 1 },
                    IrInstr::LoadI32 { dst: 1, val: 2 },
                    IrInstr::AddI32 {
                        dst: 2,
                        lhs: 0,
                        rhs: 1,
                    },
                    IrInstr::Ret { src: Some(2) },
                ],
                ownership_events: Vec::new(),
            }],
            true,
        )
        .expect("emit");
        let verified = verify_semcode(&bytes).expect("ordinary debug section must verify");
        assert_eq!(verified.functions.len(), 1);
        assert!(
            verified.functions[0].debug_symbol_count > 1,
            "fixture must exercise a genuinely non-empty debug section"
        );
    }

    // #1731 regression matrix (7/8): a truncated DBG0 tag or count must
    // still hit the existing deterministic malformed-section rejection,
    // unaffected by the new ambiguity check (which only runs once a full,
    // successfully-decoded DBG0 section is already present).
    #[test]
    fn verifier_rejects_truncated_debug_section_tag() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![IrInstr::Ret { src: None }],
                ownership_events: Vec::new(),
            }],
            true,
        )
        .expect("emit");
        // Truncate the artifact to land inside the DBG0 tag/count region,
        // matching this file's existing truncation-test convention.
        let mut truncated = bytes.clone();
        truncated.truncate(truncated.len() - 1);
        let report =
            sm_format::semcode_decode::decode_semcode_envelope(&truncated).expect_err("decode");
        assert!(matches!(
            report,
            sm_format::semcode_decode::DecodeError::InvalidDebugSection { .. }
                | sm_format::semcode_decode::DecodeError::TruncatedFunction { .. }
        ));
    }

    // #1731 review follow-up: the ambiguity probe must be a purely
    // STRUCTURAL question (opcode recognized, every operand's byte shape
    // present, stream consumes exactly to the end) and must NOT be gated on
    // whether the alternative reading's operand *values* are semantically
    // canonical. Before this fix, `instruction_stream_parses_fully` reused
    // `decode_operands` with full canonical-domain enforcement, so an
    // alternative reading that was shape-complete but contained one
    // non-canonical literal (out-of-domain, not out-of-space) was wrongly
    // treated as "not a real competing interpretation" and the DBG0 reading
    // was silently kept.
    //
    // Fixture: reuses the exact #1731 TupleGet collision (dst=0x4742,
    // src=0x0030 spell `DBG0` + an empty debug_symbol_count, exactly like
    // `verifier_rejects_ambiguous_dbg0_tupleget_collision` above), followed
    // by a genuine `LoadBool{dst:0, val:true}` whose literal byte is then
    // hand-patched from canonical `1` to non-canonical `0xff`. This does not
    // change any operand's byte width (the literal is always exactly one
    // byte regardless of its value), so the alternative reading - decoding
    // the whole buffer from the start, ignoring the `DBG0` sniff entirely -
    // remains fully shape-complete: `TupleGet{..}; LoadBool{dst:0,
    // literal:0xff}; Ret{None}`, consuming every byte to the end, with one
    // non-canonical literal.
    //
    // What the `DBG0` reading's own (byte-shifted, effectively garbled)
    // instruction decode would produce is irrelevant here: the ambiguity
    // check runs before that decode is ever attempted, so this test only
    // needs the alternative reading to be genuinely shape-complete.
    #[test]
    fn verifier_rejects_ambiguous_framing_even_when_alternate_reading_has_non_canonical_operand() {
        let mut bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::TupleGet {
                        dst: 0x4742,
                        src: 0x0030,
                        index: 0x6600,
                    },
                    IrInstr::LoadBool { dst: 0, val: true },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit");

        let (_, code_start, _) = function_code_span(&bytes, "main");
        let (_, functions) =
            sm_format::semcode_decode::decode_semcode_envelope(&bytes).expect("decode");
        let env = &functions[0];
        assert!(env.has_debug_section);
        assert_eq!(&env.code_slice[2..6], b"DBG0");

        // TupleGet is 7 bytes (opcode + dst + src + index); LoadBool's
        // literal is the 4th byte of the instruction right after it
        // (opcode + dst(2) + literal).
        let literal_pos = code_start + env.string_table_end_offset + 7 + 1 + 2;
        assert_eq!(
            bytes[literal_pos], 1,
            "must be patching LoadBool's literal byte"
        );
        bytes[literal_pos] = 0xff;

        let report = verify_semcode(&bytes).expect_err("must reject ambiguous framing");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::AmbiguousInstructionFraming
        );
    }

    // #1731 review follow-up (round 2): `MAKE_TUPLE`'s arity-`>=2` check and
    // `MAKE_RECORD`'s slot-count-`>=1` check are cardinality/value-domain
    // constraints, not byte-shape constraints - the count field itself is
    // always read unconditionally and always determines how many further
    // item-register bytes follow, regardless of whether that count value is
    // semantically canonical. They must be gated on
    // `enforce_canonical_domains` exactly like the literal/flag checks
    // above, or the same silent-ambiguity gap reopens for any alternative
    // reading built around a too-small tuple/record.
    //
    // Fixture: the exact #1731 TupleGet collision, followed by a genuine
    // `MakeTuple{dst:0, items:[5]}` - arity 1, non-canonical (`< 2`) but
    // fully shape-complete (the count field legitimately says "one item
    // follows", and one item follows).
    #[test]
    fn verifier_rejects_ambiguous_framing_with_non_canonical_maketuple_arity() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::TupleGet {
                        dst: 0x4742,
                        src: 0x0030,
                        index: 0x6600,
                    },
                    IrInstr::MakeTuple {
                        dst: 0,
                        items: vec![5],
                    },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit");

        let (_, functions) =
            sm_format::semcode_decode::decode_semcode_envelope(&bytes).expect("decode");
        let env = &functions[0];
        assert!(env.has_debug_section);
        assert_eq!(&env.code_slice[2..6], b"DBG0");

        let report = verify_semcode(&bytes).expect_err("must reject ambiguous framing");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::AmbiguousInstructionFraming
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
    fn verifier_rejects_unknown_closure_target() {
        let mut bytes = compile_program_to_semcode(
            "fn main() { let add: Closure(f64 -> f64) = (x => x + 1.0); add(2.0); return; }",
        )
        .expect("compile");
        let (_, functions) =
            sm_format::semcode_decode::decode_semcode_envelope(&bytes).expect("decode");
        let (closure_name_offset, closure_name_len) = functions
            .iter()
            .find(|function| function.name.starts_with("__closure_"))
            .map(|function| (function.name_offset + 2, function.name.len()))
            .expect("closure function");
        let replacement = "x".repeat(closure_name_len);
        bytes[closure_name_offset..closure_name_offset + closure_name_len]
            .copy_from_slice(replacement.as_bytes());

        let report = verify_semcode(&bytes).expect_err("must reject dangling closure target");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::UnknownCallTarget
        );
    }

    #[test]
    fn verifier_rejects_builtin_closure_target() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::LoadF64 { dst: 0, val: 1.0 },
                    IrInstr::MakeClosure {
                        dst: 1,
                        name: "sqrt".to_string(),
                        captures: Vec::new(),
                    },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit");

        let report = verify_semcode(&bytes).expect_err("must reject builtin closure target");
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
    fn verifier_accepts_sequence_index_static_ownership_semcode() {
        let bytes = sequence_index_static_borrow_semcode_bytes();
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_rejects_truncated_sequence_index_static_payload() {
        let mut bytes = sequence_index_static_borrow_semcode_bytes();
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
    fn verifier_accepts_adt_payload_ownership_semcode() {
        let bytes = adt_payload_ownership_semcode_bytes();
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.functions.len(), 2);
    }

    #[test]
    fn verifier_rejects_invalid_adt_payload_component_kind() {
        let mut bytes = adt_payload_ownership_semcode_bytes();
        let (_, code_start, code_end) = function_code_span(&bytes, "read_payload");
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
    fn verifier_rejects_truncated_adt_payload_missing_variant() {
        let mut bytes = adt_payload_ownership_semcode_bytes();
        let (code_len_pos, code_start, code_end) = function_code_span(&bytes, "read_payload");
        let code = &bytes[code_start..code_end];
        let section_offset = ownership_section_offset(code);
        let truncated_code_len = section_offset + 4 + 2 + 1 + 4 + 2 + 1;
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
    fn verifier_rejects_truncated_adt_payload_missing_index() {
        let mut bytes = adt_payload_ownership_semcode_bytes();
        let (code_len_pos, code_start, code_end) = function_code_span(&bytes, "read_payload");
        let code = &bytes[code_start..code_end];
        let section_offset = ownership_section_offset(code);
        let truncated_code_len = section_offset + 4 + 2 + 1 + 4 + 2 + 1 + 4;
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

    fn sequence_index_static_borrow_semcode_bytes() -> Vec<u8> {
        let mut bytes = record_field_borrow_semcode_bytes();
        let (_, code_start, code_end) = function_code_span(&bytes, "main");
        let code = &mut bytes[code_start..code_end];
        let section_offset = ownership_section_offset(code);
        let component_kind_offset = section_offset + 4 + 2 + 1 + 4 + 2;
        code[component_kind_offset] =
            sm_format::semcode_format::OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX;
        bytes
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

    fn adt_payload_ownership_semcode_bytes() -> Vec<u8> {
        let src = r#"
            enum Maybe {
                None,
                Some(f64),
            }

            fn read_payload(value: Maybe) -> f64 {
                let ret: f64 = match value {
                    Maybe::None => { 0.0 }
                    Maybe::Some(ref inner) => {
                        let v: f64 = inner;
                        v
                    }
                };
                return ret;
            }

            fn main() {
                let out: f64 = read_payload(Maybe::Some(2.5));
                assert(out == 2.5);
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

    /// Locates the absolute byte offset of the `occurrence`-th (0-indexed)
    /// instance of `opcode` within `function_name`'s actual decoded
    /// instruction stream.
    ///
    /// Walks instruction-by-instruction from the real `instr_start_offset`,
    /// reusing the same decode logic (`decode_semcode_envelope`,
    /// `decode_operands`) the verifier's own admission path uses, instead of
    /// scanning raw bytes for the first matching value anywhere in the
    /// buffer. This cannot match a header, function-name, string-table,
    /// debug-section, or ownership-section byte, and cannot match an earlier
    /// instruction's operand byte, because those are never mistaken for an
    /// opcode-position read during the walk.
    ///
    /// Panics with a precise message if the function isn't found, if
    /// decoding fails, or if fewer than `occurrence + 1` matches exist -
    /// never silently selects an arbitrary occurrence when more than one
    /// exists.
    fn find_instruction(
        bytes: &[u8],
        function_name: &str,
        opcode: Opcode,
        occurrence: usize,
    ) -> usize {
        let (_, functions) =
            sm_format::semcode_decode::decode_semcode_envelope(bytes).expect("decode semcode");
        let env = functions
            .iter()
            .find(|f| f.name == function_name)
            .unwrap_or_else(|| panic!("function '{function_name}' not found"));

        let code = env.code_slice;
        let mut cursor = env.instr_start_offset;
        let mut matches = Vec::new();
        while cursor < code.len() {
            let instr_offset = cursor;
            let raw = read_u8(code, &mut cursor).expect("read opcode byte");
            let decoded = Opcode::from_byte(raw).expect("valid opcode");
            if decoded == opcode {
                matches.push(instr_offset);
            }
            decode_operands(
                function_name,
                code,
                &mut cursor,
                instr_offset,
                decoded,
                true,
            )
            .expect("decode operands");
        }

        let relative = *matches.get(occurrence).unwrap_or_else(|| {
            panic!(
                "expected occurrence {occurrence} of {opcode:?} in '{function_name}', found \
                 {} match(es) at {matches:?}",
                matches.len()
            )
        });
        env.code_offset + relative
    }

    #[test]
    fn verify_semcode_token_accepts_no_main_helper_semcode() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "helper".to_string(),
                instrs: vec![IrInstr::Ret { src: None }],
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        let token = verify_semcode_token(&bytes).expect("token admission");
        assert!(token.has_entry("helper"));
        assert!(!token.has_entry("main"));
        assert_eq!(token.function_names().collect::<Vec<_>>(), vec!["helper"]);
    }

    #[test]
    fn verify_semcode_token_metadata_matches_verify_semcode() {
        let src = "fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let metadata = verify_semcode(&bytes).expect("verify_semcode");
        let token = verify_semcode_token(&bytes).expect("token admission");
        assert_eq!(token.program(), &metadata);
    }

    #[test]
    fn verify_semcode_token_rejects_malformed_header() {
        let report = verify_semcode_token(b"SEMC").expect_err("must reject");
        assert_eq!(report.diagnostics[0].code, VerificationCode::BadHeader);
    }

    // #1736 (FA-05-006): before the sm-format checked-arithmetic repair, this
    // exact artifact shape (a claimed function code length that overflows
    // cursor arithmetic) could panic inside `decode_semcode_envelope` on a
    // 32-bit target instead of surfacing as a `RejectReport` - i.e. the
    // verifier boundary was not actually panic-safe against malformed input.
    #[test]
    fn verify_semcode_token_rejects_overflowing_code_length_without_panicking() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&sm_format::semcode_format::MAGIC0);
        bytes.extend_from_slice(&4u16.to_le_bytes());
        bytes.extend_from_slice(b"main");
        bytes.extend_from_slice(&0xFFFFFFF0u32.to_le_bytes());
        let report = verify_semcode_token(&bytes).expect_err("must reject, never panic");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::TruncatedFunction
        );
    }

    #[test]
    fn verified_semcode_token_keeps_original_bytes() {
        let src = "fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("token admission");
        assert_eq!(token.bytes(), bytes.as_slice());
    }

    #[test]
    fn verify_semcode_token_rejects_duplicate_function_names() {
        let mut bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");
        let function_block = bytes[8..].to_vec();
        bytes.extend_from_slice(&function_block);
        let report = verify_semcode_token(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::DuplicateFunction
        );
    }

    #[test]
    fn require_entry_main_ok_for_executable_semcode() {
        let src = "fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("token admission");
        let entry_token = token.require_entry("main").expect("require_entry");
        assert_eq!(entry_token.entry(), "main");
    }

    #[test]
    fn require_entry_main_fails_for_helper_only_semcode() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "helper".to_string(),
                instrs: vec![IrInstr::Ret { src: None }],
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        let token = verify_semcode_token(&bytes).expect("token admission");
        let err = token.require_entry("main").expect_err("should fail");
        assert_eq!(
            err,
            EntryResolutionError::MissingEntry {
                entry: "main".to_string()
            }
        );
    }

    #[test]
    fn require_entry_helper_ok_for_helper_only_semcode() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "helper".to_string(),
                instrs: vec![IrInstr::Ret { src: None }],
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        let token = verify_semcode_token(&bytes).expect("token admission");
        let entry_token = token.require_entry("helper").expect("require_entry");
        assert_eq!(entry_token.entry(), "helper");
    }

    #[test]
    fn missing_entry_error_is_not_reject_report() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "helper".to_string(),
                instrs: vec![IrInstr::Ret { src: None }],
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        let token = verify_semcode_token(&bytes).expect("token admission");
        let err = token.require_entry("main").expect_err("should fail");
        let error_msg = err.to_string();
        assert!(error_msg.contains("entry function 'main' not found"));
    }

    #[test]
    fn verified_entry_token_reuses_original_artifact() {
        let src = "fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("token admission");
        let entry_token = token.require_entry("main").expect("require_entry");
        assert_eq!(entry_token.bytes(), token.bytes());
        assert_eq!(entry_token.program(), token.program());
    }

    // #1732 (FA-05-002): QTruth opcodes (0x17..0x1A) were introduced after
    // the SEMCODE0 baseline vocabulary (see docs/roadmap/core_quad/) but
    // carry no capability bit and no minimum-header-revision gate, so a
    // baseline-header artifact containing them was previously admitted.
    // This is the RED->GREEN regression for that gap: same fixture the old
    // `verifier_accepts_unsupported_qtruth_opcodes` test used (a trivial
    // `fn main() { return; }` baseline-header artifact with its opcode
    // stream patched to raw QTruth bytes), but the artifact's header
    // revision (SEMCODE0, rev 1) is below QTruth's minimum (SEMCOD18,
    // rev 19), so admission must now fail closed.
    #[test]
    fn verifier_rejects_qtruth_opcodes_under_baseline_header() {
        let mut bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");
        let code_len_pos = 8 + 2 + 4;
        let opcode_pos = code_len_pos + 4 + 2;

        let mut new_code = Vec::new();
        // QTruthAnd (0x17)
        new_code.push(0x17);
        new_code.extend_from_slice(&0u16.to_le_bytes()); // dst
        new_code.extend_from_slice(&0u16.to_le_bytes()); // lhs
        new_code.extend_from_slice(&0u16.to_le_bytes()); // rhs
                                                         // QTruthOr (0x18)
        new_code.push(0x18);
        new_code.extend_from_slice(&0u16.to_le_bytes()); // dst
        new_code.extend_from_slice(&0u16.to_le_bytes()); // lhs
        new_code.extend_from_slice(&0u16.to_le_bytes()); // rhs
                                                         // QTruthNot (0x19)
        new_code.push(0x19);
        new_code.extend_from_slice(&0u16.to_le_bytes()); // dst
        new_code.extend_from_slice(&0u16.to_le_bytes()); // src
                                                         // QTruthImpl (0x1A)
        new_code.push(0x1A);
        new_code.extend_from_slice(&0u16.to_le_bytes()); // dst
        new_code.extend_from_slice(&0u16.to_le_bytes()); // lhs
        new_code.extend_from_slice(&0u16.to_le_bytes()); // rhs
                                                         // Ret
        new_code.push(Opcode::Ret as u8);
        new_code.push(0);

        bytes.splice(opcode_pos..opcode_pos + 2, new_code.iter().copied());

        let new_code_len = 2 + new_code.len();
        bytes[code_len_pos..code_len_pos + 4].copy_from_slice(&(new_code_len as u32).to_le_bytes());

        assert_eq!(&bytes[0..8], b"SEMCODE0");
        let report = verify_semcode(&bytes).expect_err("must reject QTruth under baseline header");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::OpcodeRequiresNewerHeader
        );
    }

    // #1732 regression matrix (2): the exact same QTruth opcode stream must
    // be accepted once the artifact carries SEMCOD18 (QTruth's actual
    // minimum header revision) - proving the gate is a real revision
    // comparison, not a blanket QTruth rejection.
    #[test]
    fn verifier_accepts_qtruth_opcodes_under_semcod18_header() {
        let mut bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");
        bytes[0..8].copy_from_slice(b"SEMCOD18");
        let code_len_pos = 8 + 2 + 4;
        let opcode_pos = code_len_pos + 4 + 2;

        // SEMCOD18 carries CAP_OWNERSHIP_PATHS (inherited from SEMCOD11+),
        // which requires a present (possibly empty) OWN0 section.
        let mut new_code = Vec::new();
        new_code.extend_from_slice(b"OWN0");
        new_code.extend_from_slice(&0u16.to_le_bytes()); // empty ownership event count
        new_code.push(0x17); // QTruthAnd
        new_code.extend_from_slice(&0u16.to_le_bytes());
        new_code.extend_from_slice(&0u16.to_le_bytes());
        new_code.extend_from_slice(&0u16.to_le_bytes());
        new_code.push(0x18); // QTruthOr
        new_code.extend_from_slice(&0u16.to_le_bytes());
        new_code.extend_from_slice(&0u16.to_le_bytes());
        new_code.extend_from_slice(&0u16.to_le_bytes());
        new_code.push(0x19); // QTruthNot
        new_code.extend_from_slice(&0u16.to_le_bytes());
        new_code.extend_from_slice(&0u16.to_le_bytes());
        new_code.push(0x1A); // QTruthImpl
        new_code.extend_from_slice(&0u16.to_le_bytes());
        new_code.extend_from_slice(&0u16.to_le_bytes());
        new_code.extend_from_slice(&0u16.to_le_bytes());
        new_code.push(Opcode::Ret as u8);
        new_code.push(0);

        bytes.splice(opcode_pos..opcode_pos + 2, new_code.iter().copied());
        let new_code_len = 2 + new_code.len();
        bytes[code_len_pos..code_len_pos + 4].copy_from_slice(&(new_code_len as u32).to_le_bytes());

        let verified = verify_semcode(&bytes).expect("QTruth under SEMCOD18 must verify");
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_rejects_truncated_qtruth_opcodes() {
        let mut bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");
        // Use SEMCOD18 (QTruth's actual minimum header) so this test
        // isolates truncation handling from the #1732 header-revision gate
        // - both are legitimate rejections, but this test is specifically
        // about truncated operand bytes, not about the header revision.
        bytes[0..8].copy_from_slice(b"SEMCOD18");
        let opcode_pos = 8 + 2 + 4 + 4 + 2;

        bytes[opcode_pos] = 0x17; // QTruthAnd (requires 4 operands bytes, only 1 left)

        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::OperandOutOfBounds
        );
    }

    // Structural validity must be established before the header-revision
    // gate applies: a truncated QTruth instruction under a baseline
    // (SEMCODE0) header is not "structurally valid but from an older
    // header" - decode_operands cannot even establish it as a complete
    // instruction - so it must reject as OperandOutOfBounds, not
    // OpcodeRequiresNewerHeader, per docs/spec/verifier.md's diagnostic
    // contract for that code.
    #[test]
    fn verifier_rejects_truncated_qtruth_opcodes_under_baseline_header_as_operand_error() {
        let mut bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");
        assert_eq!(&bytes[0..8], b"SEMCODE0");
        let opcode_pos = 8 + 2 + 4 + 4 + 2;

        bytes[opcode_pos] = 0x17; // QTruthAnd (requires 4 operand bytes, only 1 left)

        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::OperandOutOfBounds
        );
    }

    // #1732 regression matrix (4/5): historically-baseline opcodes -
    // including the legacy lattice `QAnd`/`QOr`/`QNot`/`QImpl` opcodes
    // QTruth sits right next to - have minimum_semcode_revision() == 1, so
    // they must remain fully admissible under SEMCODE0, completely
    // unaffected by the new header-revision gate.
    #[test]
    fn verifier_accepts_baseline_logic_opcodes_under_semcode0_unchanged() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::LoadI32 { dst: 0, val: 1 },
                    IrInstr::LoadI32 { dst: 1, val: 0 },
                    IrInstr::QAnd {
                        dst: 2,
                        lhs: 0,
                        rhs: 1,
                    },
                    IrInstr::QOr {
                        dst: 3,
                        lhs: 0,
                        rhs: 1,
                    },
                    IrInstr::QNot { dst: 4, src: 0 },
                    IrInstr::QImpl {
                        dst: 5,
                        lhs: 0,
                        rhs: 1,
                    },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        assert_eq!(&bytes[0..8], b"SEMCODE0");
        let verified = verify_semcode(&bytes).expect("baseline logic opcodes must verify");
        assert_eq!(verified.header.rev, 1);
    }

    // #1732 regression matrix (6): a program mixing QTruth with an
    // unrelated capability-gated feature (here, f64 math, which alone
    // would only require SEMCODE1) must select the MAXIMUM required
    // header revision (SEMCOD18, rev 19), not whichever feature's
    // if/else-if branch happens to match first.
    #[test]
    fn emitter_selects_maximum_required_revision_for_mixed_program() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::LoadF64 { dst: 0, val: 1.5 },
                    IrInstr::QTruthAnd {
                        dst: 1,
                        lhs: 0,
                        rhs: 0,
                    },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        assert_eq!(&bytes[0..8], b"SEMCOD18");
        let verified = verify_semcode(&bytes).expect("mixed QTruth+f64 program must verify");
        assert_eq!(verified.header.rev, 19);
    }

    // #1732 regression matrix (3): the canonical emitter itself must choose
    // a header whose revision actually covers QTruth, not just structurally
    // valid bytes hand-patched by a test. Uses emit_ir_to_semcode (IR
    // level) so this doesn't depend on the source front-end's own QTruth
    // admission being wired for every feature-gate combination.
    #[test]
    fn emitter_promotes_qtruth_only_program_to_semcod18() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::QTruthAnd {
                        dst: 0,
                        lhs: 0,
                        rhs: 0,
                    },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        assert_eq!(&bytes[0..8], b"SEMCOD18");
        let verified = verify_semcode(&bytes).expect("emitted QTruth program must verify");
        assert_eq!(verified.header.rev, 19);
    }
}
