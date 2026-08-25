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
    /// The function envelope's `SIG0` callable-signature section is absent,
    /// truncated, or carries an unrecognized parameter-family tag (#1773 /
    /// FA-09-005). Structurally distinct from `UnsupportedVersion`: the
    /// header itself is recognized, but this specific function's signature
    /// record could not be decoded.
    InvalidSignatureSection,
    /// An `Opcode::Call` site's `argc` disagrees with its resolved callee's
    /// canonical parameter count (#1773 / FA-09-005). This is arity
    /// enforcement only - sm-verify cannot prove a CALL argument register's
    /// runtime family statically (registers are untyped storage), so family
    /// mismatches are rejected by the VM before `push_frame` instead.
    CallArgumentCountMismatch,
    /// A register read is reachable from function entry on some execution
    /// path where that register has not been definitely written (#1756 /
    /// FA-07-016). Distinct from `InvalidRegisterReference`: the register
    /// number is in range and within the configured quota - what's missing
    /// is a proof that every incoming path defines it before this read.
    /// Only checked for a function carrying a canonical `SIG0` signature
    /// (`SEMCODE_SIGNATURE_MIN_REVISION`+); a signature-less artifact has no
    /// sound `IN[entry]` to prove against and is unaffected by this code.
    UndefinedRegisterRead,
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

    /// Intentionally public, hidden implementation seam for VM preparation.
    ///
    /// `sm-vm` (a separate workspace crate) calls this to bridge a verified
    /// token into VM-executable state (`prepare_verified_execution` in
    /// `crates/sm-vm/src/semcode_vm.rs`). Rust has no visibility level
    /// narrower than `pub` that still permits a separate crate to call a
    /// method, so this is genuinely part of `sm-verify`'s externally
    /// callable Rust surface -- `#[doc(hidden)]` only suppresses it from
    /// generated documentation, it does not restrict downstream crates from
    /// naming and calling it. It is not recommended for use outside this
    /// workspace's own crates. The closure-based shape only prevents a
    /// caller from retaining the borrowed `&'scope` slice itself past this
    /// call; it is not a containment guarantee -- `DecodedFunctionEnvelope`
    /// derives `Clone`, so a closure can return owned copies of the decoded
    /// data, and `sm_format::semcode_decode::decode_semcode_envelope` is
    /// itself public, so any caller with the same bytes can independently
    /// reconstruct equivalent decoded envelopes without this method at all.
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
/// `VerifiedSemCode` token if successful. Admits against the `VerifiedLocal`
/// quota profile; use `verify_semcode_token_with_quotas` to admit against a
/// different (e.g. `KernelBound`) profile.
#[cfg(feature = "std")]
pub fn verify_semcode_token(bytes: &[u8]) -> Result<VerifiedSemCode<'_>, RejectReport> {
    verify_semcode_token_with_quotas(bytes, RuntimeQuotas::verified_local())
}

/// Admission gate for SemCode bytes against an explicit quota profile.
///
/// Identical to `verify_semcode_token` except the caller supplies the
/// `RuntimeQuotas` admission is checked against, instead of the hardcoded
/// `VerifiedLocal` profile. This lets a context (e.g. `KernelBound`) admit
/// tokens whose register/symbol usage exceeds `VerifiedLocal`'s budget but
/// stays within its own. The returned token's admission proof reflects only
/// the quotas passed here - it says nothing about any other profile.
#[cfg(feature = "std")]
pub fn verify_semcode_token_with_quotas(
    bytes: &[u8],
    quotas: RuntimeQuotas,
) -> Result<VerifiedSemCode<'_>, RejectReport> {
    let mut diagnostics = Vec::new();

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
                    sm_format::semcode_decode::DecodeError::InvalidSignatureSection {
                        offset,
                        msg,
                    } => diag(
                        VerificationCode::InvalidSignatureSection,
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

    // #1754 (FA-07-014): the VM builds ONE program-wide `RuntimeSymbolTable`
    // shared across every function (see `build_vm_program_view_from_decoded`
    // in crates/sm-vm/src/semcode_vm.rs), interning each function's
    // `env.strings` into it and deduplicating by exact string value. A
    // per-function check against `max_symbol_table` cannot catch a program
    // that stays under the budget in every individual function but exceeds
    // it once all functions' distinct strings are unioned program-wide, so
    // the check has to be made at this granularity to match the real
    // runtime resource.
    let unique_runtime_symbols: HashSet<&str> = decoded_functions
        .iter()
        .flat_map(|env| env.strings.iter().map(|s| s.as_str()))
        .collect();
    let unique_runtime_symbol_count = unique_runtime_symbols.len();
    if unique_runtime_symbol_count > quotas.max_symbol_table {
        diagnostics.push(diag(
            VerificationCode::ResourceLimitExceeded,
            None,
            None,
            format!(
                "program-wide runtime symbol table uses {} distinct entries, exceeding the symbol budget of {}",
                unique_runtime_symbol_count, quotas.max_symbol_table
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
    // #1773 (FA-09-005): callee name -> its decoded canonical signature, so
    // the cross-function pass below can enforce arity. `None` for a known
    // callee means the artifact's header predates canonical signatures
    // (`SEMCODE_SIGNATURE_MIN_REVISION`) - nothing to check, matching how
    // pre-#1773 artifacts already behaved.
    let signatures_by_name: std::collections::HashMap<
        &str,
        &sm_format::semcode_format::CallableSignature,
    > = decoded_functions
        .iter()
        .filter_map(|env| env.signature.as_ref().map(|sig| (env.name.as_str(), sig)))
        .collect();
    for function in &pending_functions {
        for (offset, callee, allows_builtin, argc) in &function.call_targets {
            if known_functions.contains(callee.as_str()) {
                // #1773 (FA-09-005): arity enforcement. Family/runtime-type
                // enforcement is the VM's responsibility before `push_frame`
                // (see `validate_call_arguments` in sm-vm) - sm-verify
                // cannot know a CALL argument register's runtime family
                // (registers are untyped storage, see the #1773 architecture
                // checkpoint), so it enforces only what it can prove
                // statically: parameter count.
                if let (Some(argc), Some(signature)) =
                    (argc, signatures_by_name.get(callee.as_str()))
                {
                    if *argc != signature.families.len() {
                        diagnostics.push(diag(
                            VerificationCode::CallArgumentCountMismatch,
                            Some(function.verified.name.clone()),
                            Some(*offset),
                            format!(
                                "call to '{}' passes {} argument(s), but its canonical signature declares {} parameter(s)",
                                callee,
                                argc,
                                signature.families.len()
                            ),
                        ));
                    }
                }
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

    // #1754 (FA-07-014): a per-function string-table check against
    // `max_symbol_table` used to live here, but sm-format already caps each
    // function's local string table at `MAX_STRINGS_PER_FUNCTION` (256, see
    // semcode_decode.rs), which is far below `max_symbol_table` (16_384) -
    // making that check dead code. The real program-wide resource is the VM's
    // `RuntimeSymbolTable` (crates/sm-runtime-core/src/lib.rs), which is
    // built ONCE per program and shared across every function (see
    // `build_vm_program_view_from_decoded` in crates/sm-vm/src/semcode_vm.rs,
    // which interns each function's `env.strings` into a single
    // program-wide, dedup-by-value table). That budget is enforced
    // program-wide in `verify_semcode_token` instead, see the check there.
    let string_count = env.strings.len();

    let debug_symbol_count = env.debug_symbols.len();
    if debug_symbol_count > quotas.max_trace_entries {
        return Err(reject_one(
            name,
            VerificationCode::ResourceLimitExceeded,
            0,
            format!(
                "debug section uses {} entries, exceeding the trace budget of {}",
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
    let mut call_argcs = Vec::new();
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
        call_argcs.extend(refs.call_argcs);
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
        // #1746 (FA-07-006): range alone does not prove `sym.pc` denotes an
        // actual instruction boundary - it could land inside a decoded
        // operand's bytes. Reuse the instruction-start set this same
        // verification walk already built, exactly like the jump-target
        // check below does for `InvalidJumpTarget`. `instr_starts` is
        // pushed in strictly increasing order (each iteration's `offset` is
        // read from `cursor` before `cursor` only ever advances further),
        // so `binary_search` is valid and avoids an O(symbols x
        // instructions) linear scan - the format permits up to 8,192 debug
        // symbols per function, so a debug-heavy artifact could otherwise
        // force a disproportionate amount of verifier CPU per function
        // (review finding on this PR).
        if instr_starts.binary_search(&sym.pc).is_err() {
            return Err(reject_one(
                name,
                VerificationCode::InvalidDebugSection,
                sym.pc,
                "debug symbol pc does not land on an instruction boundary",
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

    let reachable_offsets =
        verify_reachable_control_flow(name, instr_len, &instr_starts, &instruction_successors)?;

    // #1773 (FA-09-005): argc, keyed by the Call instruction's own offset -
    // only "call target" entries (real `Opcode::Call` sites) ever have an
    // entry here; "closure function name" entries come from `MakeClosure`
    // (a static reference check, not a call site with a known argc) and are
    // deliberately left with `argc: None` below.
    let call_argcs: std::collections::HashMap<usize, usize> = call_argcs.into_iter().collect();

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
            "call target" => call_targets.push((
                offset,
                env.strings[sid].clone(),
                true,
                call_argcs.get(&offset).copied(),
            )),
            "closure function name" => {
                call_targets.push((offset, env.strings[sid].clone(), false, None));
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
                    "function references register r{}, exceeding the register budget of {}",
                    max_register, quotas.max_registers
                ),
            ));
        }
    }

    // #1756 (FA-07-016): forward MUST definite-assignment proof - every
    // reachable register read must be definitely defined on every incoming
    // execution path, not merely in range. Runs only when this function has
    // a canonical `SIG0` signature (`env.signature.is_some()`): a pre-#1773
    // artifact's header predates canonical per-function arity metadata, so
    // `IN[entry]` cannot be soundly determined for it - caller-inferred or
    // convention-inferred entry definitions are exactly the unsound
    // heuristic #1756's own research explicitly rejected (see the issue's
    // Phase B checkpoint). This preserves pre-#1756 admission behavior for
    // signature-less artifacts exactly, mirroring how #1773's own
    // `validate_call_arguments`/arity check already treats `signature: None`
    // as "nothing new to enforce" - not a new decision, the same one
    // continued.
    if let Some(signature) = env.signature.as_ref() {
        let entry_param_count = signature.families.len();
        // #1756: the domain must be bounded by the already-established
        // register quota before any allocation sized by it - an
        // attacker-controlled SIG0 parameter count (bounded only by the
        // decoder's own MAX_SIGNATURE_PARAMETERS_PER_FUNCTION, independent
        // of this verification call's actual `quotas.max_registers`) must
        // not be allowed to size a per-node bitset on its own.
        if entry_param_count > quotas.max_registers {
            return Err(reject_one(
                name,
                VerificationCode::InvalidRegisterReference,
                0,
                format!(
                    "canonical signature declares {} parameter register(s), exceeding the register budget of {}",
                    entry_param_count, quotas.max_registers
                ),
            ));
        }
        prove_definite_register_assignment(
            name,
            code,
            instr_start,
            &instr_starts,
            &instruction_successors,
            &reachable_offsets,
            entry_param_count,
        )?;
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

/// Returns the set of reachable instruction offsets on success. #1756
/// (FA-07-016) reuses this exact return value as the authoritative
/// reachable-node set for its definite-assignment pass, rather than
/// re-deriving reachability with a second traversal.
#[cfg(feature = "std")]
fn verify_reachable_control_flow(
    function: &str,
    instr_len: usize,
    instr_starts: &[usize],
    instruction_successors: &[InstructionSuccessors],
) -> Result<HashSet<usize>, RejectReport> {
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

    Ok(reachable)
}

/// #1756 (FA-07-016): a compact fixed-size bitset over register numbers
/// `0..domain_size`. `domain_size` is always bounded by the register-range
/// quota already enforced before this pass runs (see the caller), so this
/// never allocates from an attacker-controlled unbounded register id.
///
/// #1756 Codex review round 8: this now represents the MAY-MISSING dataflow
/// lattice element (a register set under UNION meet), not the original
/// MUST-DEFINED one (intersection meet) - see `compute_missing_sets`'s doc
/// comment for the full duality derivation. `RegSet` itself is an
/// unopinionated bitset either way; only the meet operation used on it (now
/// plain per-bit `insert`, driving the worklist in `compute_missing_sets`,
/// rather than a whole-set `intersect_with`) and its initialization
/// (BOTTOM=empty for non-entry nodes, the dual of the old TOP=full rule)
/// changed.
#[cfg(feature = "std")]
#[derive(Clone)]
struct RegSet {
    words: Vec<u64>,
}

#[cfg(feature = "std")]
impl RegSet {
    fn word_count(domain_size: usize) -> usize {
        domain_size.saturating_add(63) / 64
    }

    /// Every register in the domain is set. Used only to compute `entry`'s
    /// fixed MISSING set via complement (`full() `then removing
    /// `ENTRY_DEFS` bits) - no node is ever initialized to this directly
    /// under the round-8 MAY-missing formulation (see `compute_missing_
    /// sets`'s doc comment for why BOTTOM/empty, not this, is every
    /// non-entry node's correct starting point).
    fn full(domain_size: usize) -> Self {
        let mut words = vec![u64::MAX; Self::word_count(domain_size)];
        let rem = domain_size % 64;
        if rem != 0 {
            if let Some(last) = words.last_mut() {
                *last &= (1u64 << rem) - 1;
            }
        }
        RegSet { words }
    }

    fn empty(domain_size: usize) -> Self {
        RegSet {
            words: vec![0u64; Self::word_count(domain_size)],
        }
    }

    fn contains(&self, reg: usize) -> bool {
        let word = reg / 64;
        let bit = reg % 64;
        self.words.get(word).is_some_and(|w| (w >> bit) & 1 != 0)
    }

    fn insert(&mut self, reg: usize) {
        let word = reg / 64;
        let bit = reg % 64;
        if let Some(w) = self.words.get_mut(word) {
            *w |= 1u64 << bit;
        }
    }

    fn remove(&mut self, reg: usize) {
        let word = reg / 64;
        let bit = reg % 64;
        if let Some(w) = self.words.get_mut(word) {
            *w &= !(1u64 << bit);
        }
    }
}

/// #1756 (FA-07-016): forward MUST dataflow proving every reachable register
/// read is definitely defined on every incoming execution path (meet =
/// intersection). Reuses the verifier's own successor relation
/// (`instruction_successors`) to build predecessors, and its own computed
/// reachable-offset set (`reachable_offsets`, from
/// `verify_reachable_control_flow`) to decide which nodes this proof
/// actually applies to - no second CFG/branch-target engine, no
/// re-derivation of reachability.
///
/// `entry_param_count` registers `0..entry_param_count` are `IN[entry]`
/// (`ENTRY_DEFS`, from the caller's canonical `SIG0` signature) - the only
/// registers defined at function entry, regardless of numeric range or
/// register-storage capacity. Every other node initializes to TOP
/// (`RegSet::full`), per the non-negotiable loop-correctness rule.
///
/// Unreachable nodes are never allocated per-node dataflow state at all (see
/// the Codex-review note below): an unreachable predecessor's contribution
/// to any meet would always be TOP (the identity element for intersection),
/// so an edge from an unreachable node is simply never recorded rather than
/// being represented and then ignored. This preserves the verifier's
/// existing, separate policy on structurally valid-but-unreachable code
/// untouched - this pass only judges reachable reads.
///
/// Codex review round 1 on this PR (#1840) found two independent
/// amplification sources in an earlier revision, fixed together here:
///
/// 1. Allocating one `RegSet` per node for every structurally decoded
///    instruction, reachable or not, let a structurally valid artifact (an
///    entry `RET` followed by megabytes of never-executed trailing
///    instructions, still legal per the verifier's reachable-control-flow
///    policy) force memory proportional to total instruction count, with no
///    existing quota bounding that count. Fixed: dataflow state below is
///    allocated and indexed only for reachable nodes (by position within
///    `reachable_indices`, not by raw instruction index) - the same
///    reachable set the existing verifier already walks for its own
///    reachability BFS, not a new, larger one.
/// 2. Sizing the register domain from the raw numeric span
///    (`0..=max_register_id`) rather than the registers actually in use
///    meant a single reference to a high register number (still legal and
///    in-budget) forced a full-width bitset even when only one or two
///    registers mattered - a program referencing only `{r0, r4095}` needs a
///    domain of size 2, not 4096. Fixed: the register universe `U` below is
///    built densely, from the registers `ENTRY_DEFS` and reachable
///    instructions actually reference, each remapped to a compact index;
///    diagnostics still report the original raw register identity.
///
/// Both fixes only change how densely this pass's own bounded state is
/// packed - `U`'s size can never exceed the pre-existing register-budget
/// quota (every register `U` collects was already proven in-range by that
/// check before this function runs), so neither fix relaxes or replaces
/// that quota; it remains the only source of truth for the upper bound.
///
/// Computes the two quantities that bound `prove_definite_register_
/// assignment`'s own memory use: the reachable node index set (ascending,
/// position 0 = function entry), the reachable nodes' own reads/writes (also
/// position-indexed), and the dense register universe `U` (ascending,
/// deduplicated). Factored out from that function so tests can assert
/// directly on the returned lengths - the exact per-node and per-register-
/// domain multipliers - rather than relying only on timing to demonstrate
/// the Codex-review fixes above.
///
/// `reads_writes_of` is called exactly once per REACHABLE raw instruction
/// index, never for an unreachable one (Codex review round 3 on this PR,
/// #1840: the prior revision built dense `instr_reads`/`instr_writes: Vec<
/// Vec<u16>>` for every structurally decoded instruction in the caller,
/// before reachability was even known - a `Vec<u16>` costs 24 bytes even
/// empty, so millions of unreachable instructions cost hundreds of MB for
/// these two arrays alone. The real caller's closure re-decodes just that
/// one reachable instruction's operands on demand via `decode_operands`;
/// tests can supply a trivial closure over a small, hand-built map instead).
///
/// Codex review round 4 on this PR: even after round 3 removed the
/// unreachable-instruction cost, storing each reachable node's reads/writes
/// as its OWN `Vec<u16>` (one 24-byte header plus a separate heap
/// allocation, per node, per array) still meant a large fully REACHABLE
/// straight-line stream (e.g. millions of `LOAD_BOOL r0` in a row, no
/// branching, no unreachable padding at all) cost dozens of bytes of
/// heap-container overhead per instruction. Fixed: reads and writes are now
/// returned CSR-style - one flat `Vec<u16>` holding every reachable
/// instruction's registers back to back, plus an offset table
/// (`reads_offsets`/`writes_offsets`, `u32` per node, `reachable_count + 1`
/// entries) marking each node's slice boundaries - a single allocation per
/// array instead of one per node. Use `csr_slice` to read a given
/// position's slice back out.
///
/// Returns `(reachable_indices, reads_flat, reads_offsets, writes_flat,
/// writes_offsets, universe)`.
#[cfg(feature = "std")]
type DataflowDomainAccounting = (Vec<usize>, Vec<u16>, Vec<u32>, Vec<u16>, Vec<u32>, Vec<u16>);

#[cfg(feature = "std")]
fn dataflow_domain_accounting(
    instr_starts: &[usize],
    reachable_offsets: &HashSet<usize>,
    entry_param_count: usize,
    mut reads_writes_of: impl FnMut(usize) -> (Vec<u16>, Vec<u16>),
) -> DataflowDomainAccounting {
    let mut reachable_indices: Vec<usize> = reachable_offsets
        .iter()
        .filter_map(|offset| instr_starts.binary_search(offset).ok())
        .collect();
    reachable_indices.sort_unstable();

    let mut reads_flat: Vec<u16> = Vec::new();
    let mut reads_offsets: Vec<u32> = Vec::with_capacity(reachable_indices.len() + 1);
    let mut writes_flat: Vec<u16> = Vec::new();
    let mut writes_offsets: Vec<u32> = Vec::with_capacity(reachable_indices.len() + 1);
    let mut universe: Vec<u16> = (0..entry_param_count).map(|r| r as u16).collect();
    for &idx in &reachable_indices {
        reads_offsets.push(reads_flat.len() as u32);
        writes_offsets.push(writes_flat.len() as u32);
        let (reads, writes) = reads_writes_of(idx);
        universe.extend_from_slice(&reads);
        universe.extend_from_slice(&writes);
        reads_flat.extend_from_slice(&reads);
        writes_flat.extend_from_slice(&writes);
    }
    reads_offsets.push(reads_flat.len() as u32);
    writes_offsets.push(writes_flat.len() as u32);
    universe.sort_unstable();
    universe.dedup();

    (
        reachable_indices,
        reads_flat,
        reads_offsets,
        writes_flat,
        writes_offsets,
        universe,
    )
}

/// Reads position `pos`'s slice back out of a CSR-encoded (flat, offsets)
/// pair - built by `dataflow_domain_accounting` for register lists, and (as
/// of #1756 Codex review round 7) by the SCC-membership table in
/// `prove_definite_register_assignment` for reachable positions - generic
/// over the element type since both callers share the identical
/// flat-Vec-plus-offset-table shape.
#[cfg(feature = "std")]
fn csr_slice<'a, T>(flat: &'a [T], offsets: &[u32], pos: usize) -> &'a [T] {
    &flat[offsets[pos] as usize..offsets[pos + 1] as usize]
}

/// Invokes `visit` once per REACHABLE successor position of raw instruction
/// `idx`, resolved from the verifier's own `instruction_successors` (no
/// second CFG) via `instr_starts`/`reachable_indices` binary search. A
/// target that isn't a real instruction boundary can only belong to an
/// unreachable fallthrough off the end of the stream - every jump target
/// was already fully validated above regardless of reachability, so this is
/// never a real, reachable edge silently dropped.
#[cfg(feature = "std")]
fn for_each_reachable_successor(
    idx: usize,
    instruction_successors: &[InstructionSuccessors],
    instr_starts: &[usize],
    reachable_indices: &[usize],
    mut visit: impl FnMut(usize),
) {
    let mut link = |target: usize| {
        if let Ok(target_idx) = instr_starts.binary_search(&target) {
            if let Ok(target_pos) = reachable_indices.binary_search(&target_idx) {
                visit(target_pos);
            }
        }
    };
    match &instruction_successors[idx] {
        InstructionSuccessors::None => {}
        InstructionSuccessors::One(target) => link(*target),
        InstructionSuccessors::Two(first, second) => {
            link(*first);
            link(*second);
        }
    }
}

/// Delivers `bit` to `target`'s MISSING set, in place - `target`'s `RegSet`
/// is mutated directly (`insert`), never cloned or replaced, which is what
/// eliminates the round-8 "duplicate live RegSet states" amplification (see
/// `prove_definite_register_assignment`'s doc comment): there is exactly
/// one `RegSet` per reachable position, period, so no number of branches
/// independently computing the same result can ever duplicate it. Returns
/// `true` iff this was new information (the bit wasn't already present) -
/// the caller only enqueues `(target, bit)` for further propagation when
/// this is `true`, which is what guarantees each (position, bit) pair is
/// ever enqueued at most once (MISSING only ever grows, so once a bit is
/// present it can never need re-delivery). Position 0 (entry) is never
/// delivered to - its MISSING set is fixed, see the sibling doc comment for
/// the soundness argument.
#[cfg(feature = "std")]
fn deliver(missing: &mut [RegSet], target: usize, bit: usize) -> bool {
    if target != 0 && !missing[target].contains(bit) {
        missing[target].insert(bit);
        true
    } else {
        false
    }
}

/// Core bit-level MAY-MISSING worklist - factored out from `prove_definite_
/// register_assignment` so tests can assert directly on its `event_count`
/// output (Codex review round 8's explicit request for allocation/
/// relaxation counters, not timing alone) using small, hand-constructed
/// `successors_of`/`writes_contains` closures rather than real SemCode
/// bytes.
///
/// Returns `(missing, event_count)`: the converged MISSING set for every
/// reachable position (position 0 = entry, whose value is `entry_missing`
/// unchanged), and the total number of (position, bit) events the worklist
/// processed after seeding.
///
/// **Work bound.** Every `(position, bit)` pair is enqueued at MOST once,
/// ever, across the whole computation - `deliver` only returns `true` (the
/// caller's only enqueue trigger) the one time a bit transitions from
/// absent to present in some position's `MISSING` set, and `MISSING` only
/// ever grows (union, never shrinks) - so the total number of *distinct*
/// (position, bit) pairs that can ever be enqueued is bounded by
/// `reachable_count * domain_size`, REGARDLESS of the reachable subgraph's
/// structure: cyclic, irreducible, arbitrary fan-in through a shared loop
/// header, all of it is irrelevant, because a bit that is already present
/// at a node can never need re-delivery to it. Each dequeued event costs
/// O(1) amortized: a single `writes_contains` check (a linear scan over
/// that position's own reads/writes list, which is small - at most a
/// handful of registers per instruction, by construction, not tied to
/// `domain_size`) plus O(out_degree) <= O(2) successor deliveries. This
/// gives a provable `O(reachable_count * domain_size)` TOTAL bit-level
/// work bound, independent of ordering - #1756 Codex review round 8's
/// second finding (`local_reverse_postorder_ranks`, round 7's fix, does not
/// bound convergence inside one large strongly-connected component against
/// an adversarial backedge-fan-in shape) is closed by this property: there
/// is no "large SCC" concept left to reprocess, because nothing at the bit
/// level is ever reprocessed. `successors_of`/`writes_contains` need no
/// predecessor structure, no reverse-postorder ranking, and no strongly-
/// connected-component decomposition - `tarjan_scc`, `TarjanFrame`,
/// `nth_reachable_successor`, `SccInfo`, `scc_membership_csr`, and
/// `local_reverse_postorder_ranks` (rounds 6-7's SCC/ordering machinery)
/// are removed entirely as of this round: they existed only to bound
/// whole-`RegSet` reprocessing, which this formulation does not do in the
/// first place.
#[cfg(feature = "std")]
fn compute_missing_sets(
    reachable_count: usize,
    domain_size: usize,
    entry_missing: RegSet,
    writes_contains: impl Fn(usize, usize) -> bool,
    mut successors_of: impl FnMut(usize) -> [Option<usize>; 2],
) -> (Vec<RegSet>, usize) {
    let mut missing: Vec<RegSet> = (0..reachable_count)
        .map(|pos| {
            if pos == 0 {
                entry_missing.clone()
            } else {
                RegSet::empty(domain_size)
            }
        })
        .collect();
    if reachable_count == 0 {
        return (missing, 0);
    }

    let mut queue: std::collections::VecDeque<(usize, usize)> = std::collections::VecDeque::new();

    // Seed from entry's own MISSING_OUT (post-write) - entry's own MISSING
    // is fixed and never re-delivered to (see `deliver`), so this is the
    // one place entry's contribution enters the worklist.
    let entry_successors = successors_of(0);
    for bit in 0..domain_size {
        if missing[0].contains(bit) && !writes_contains(0, bit) {
            for succ in entry_successors.into_iter().flatten() {
                if deliver(&mut missing, succ, bit) {
                    queue.push_back((succ, bit));
                }
            }
        }
    }

    let mut event_count: usize = 0;
    while let Some((pos, bit)) = queue.pop_front() {
        event_count += 1;
        if writes_contains(pos, bit) {
            continue; // killed here - defined by this instruction's own write
        }
        for succ in successors_of(pos).into_iter().flatten() {
            if deliver(&mut missing, succ, bit) {
                queue.push_back((succ, bit));
            }
        }
    }

    (missing, event_count)
}

/// #1756 (FA-07-016) Codex review round 8: the dual, MAY-MISSING
/// formulation of definite-register-assignment, replacing the round 1-7
/// MUST/DEFINED/intersection formulation entirely.
///
/// **Derivation** (De Morgan's law over the equations this replaces):
///
///   DEFINED[entry] = ENTRY_DEFS;  DEFINED[n] = intersect(OUT[p] for p in preds(n))
///   OUT[n] = DEFINED[n] union WRITES[n]
///
/// Let MISSING[n] = U minus DEFINED[n]. Then:
///
///   MISSING[entry] = U minus ENTRY_DEFS   (fixed - see below)
///   MISSING[n] = U minus intersect(OUT[p]) = union(U minus OUT[p]) = union(MISSING_OUT[p])
///   MISSING_OUT[n] = U minus OUT[n] = U minus (DEFINED[n] union WRITES[n])
///                  = (U minus DEFINED[n]) intersect (U minus WRITES[n])
///                  = MISSING[n] minus WRITES[n]
///
/// A register `r` is read-safe at `n` iff `r` is not in MISSING[n] - the
/// identical predicate as before (`r` in DEFINED[n]), just phrased via the
/// complement; `decode_operands`'s per-opcode read/write classification
/// (including MAP_GET's conservative `default_val` read, CALL/CLOSURE_CALL's
/// has-dst-flag asymmetry, and same-instruction read-before-write - reads
/// are validated against `MISSING[n]`, the PRE-write state, exactly as the
/// original formulation validated against `DEFINED[n]` pre-write) is
/// entirely unchanged; only the dataflow equations computing `MISSING`
/// itself are new.
///
/// **Initialization**: non-entry nodes start at BOTTOM = empty (the dual of
/// the non-negotiable TOP=U rule the original formulation required - union's
/// identity element is the empty set, as intersection's was U). This is
/// load-bearing for the identical reason `c1756_case12` proves for the
/// original rule: under the wrong rule (non-entry nodes starting at U, the
/// naive-looking mirror of the WRONG empty-init the original formulation
/// explicitly rejected), a loop's first, still-unconverged pass through its
/// own back edge would union with U (already everything, unable to add
/// anything) and get permanently stuck reporting every register missing,
/// even ones a real forward path (unrelated to the loop) already proves
/// defined - the dual of case 12's own wrongly-empty-forever failure mode,
/// under the DEFINED<->MISSING, intersect<->union, TOP<->BOTTOM
/// substitution throughout. `c1756_case12_accepts_loop_read_requires_top_
/// initialization` (kept, unmodified, as a regression) still exercises this
/// exact property under the new formulation.
///
/// Entry (position 0) is never re-unioned by any predecessor, including a
/// back edge that targets instruction 0 directly (structurally legal - a
/// jump to offset 0 is a valid instruction boundary): by the dual of the
/// round-6 argument, every reachable MISSING_OUT is provably a SUBSET of
/// MISSING[entry] (writes only ever REMOVE bits from MISSING, monotonically,
/// so by induction no reachable node's missing set can ever grow past what
/// entry itself starts with) - unioning entry's fixed MISSING with any such
/// subset changes nothing. `compute_missing_sets` encodes this as an
/// explicit `target != 0` guard in `deliver`, rather than relying on the
/// argument alone.
///
/// **Work bound**: see `compute_missing_sets`'s doc comment - this
/// formulation's bit-level worklist is what makes reverse-postorder ranking
/// and strongly-connected-component decomposition (rounds 5-7's successive
/// fixes) unnecessary. Both existed solely to bound how many times a WHOLE
/// `RegSet` could be re-narrowed and re-compared (an O(|U|/64)-per-event
/// cost with no bound on event count without careful ordering); at the bit
/// level, each (position, register) event is inherently processed at most
/// once, so no adversarial ordering or fan-in can force "reprocessing" -
/// there is nothing to reprocess. This also directly closes Codex review
/// round 8's first finding (duplicate live `RegSet` states across
/// branches): every reachable position owns exactly one `RegSet`, allocated
/// once and mutated in place via `insert` - there is no clone-on-write, no
/// structural-sharing bookkeeping, and therefore no way for independently-
/// computed-but-equal branch states to duplicate memory, regardless of how
/// many branches compute the identical result.
#[cfg(feature = "std")]
fn prove_definite_register_assignment(
    function: &str,
    code: &[u8],
    instr_start: usize,
    instr_starts: &[usize],
    instruction_successors: &[InstructionSuccessors],
    reachable_offsets: &HashSet<usize>,
    entry_param_count: usize,
) -> Result<(), RejectReport> {
    if instr_starts.is_empty() {
        return Ok(());
    }

    // Codex review round 3 on this PR (#1840): re-decodes exactly one
    // reachable instruction's operands on demand, via the same
    // `decode_operands` the main structural walk already used - never a
    // second decoder, and never materialized for an unreachable
    // instruction. Every byte here already passed full structural
    // validation earlier in `verify_function_code` (opcode recognition,
    // operand shape, canonical domains), so a decode failure here would be
    // an internal invariant violation, not an attacker-reachable outcome.
    let reads_writes_of = |idx: usize| -> (Vec<u16>, Vec<u16>) {
        let offset = instr_starts[idx];
        let mut cursor = instr_start + offset;
        let opcode_byte =
            read_u8(code, &mut cursor).expect("previously-decoded instruction must re-decode");
        let opcode =
            Opcode::from_byte(opcode_byte).expect("previously-decoded instruction must re-decode");
        let refs = decode_operands(function, code, &mut cursor, offset, opcode, true)
            .expect("previously-decoded instruction must re-decode");
        (refs.reads, refs.writes)
    };
    let (reachable_indices, reads_flat, reads_offsets, writes_flat, writes_offsets, universe) =
        dataflow_domain_accounting(
            instr_starts,
            reachable_offsets,
            entry_param_count,
            reads_writes_of,
        );
    let reachable_count = reachable_indices.len();
    debug_assert_eq!(reachable_indices.first(), Some(&0));
    let domain_size = universe.len();
    let dense = |raw: u16| -> usize {
        universe
            .binary_search(&raw)
            .expect("register must be in U by construction")
    };

    let mut entry_missing = RegSet::full(domain_size);
    for r in 0..entry_param_count {
        entry_missing.remove(dense(r as u16));
    }

    let writes_contains = |pos: usize, bit: usize| -> bool {
        csr_slice(&writes_flat, &writes_offsets, pos)
            .iter()
            .any(|&w| dense(w) == bit)
    };
    let successors_of = |pos: usize| -> [Option<usize>; 2] {
        let mut out = [None, None];
        let mut i = 0usize;
        for_each_reachable_successor(
            reachable_indices[pos],
            instruction_successors,
            instr_starts,
            &reachable_indices,
            |succ_pos| {
                if i < 2 {
                    out[i] = Some(succ_pos);
                    i += 1;
                }
            },
        );
        out
    };

    let (missing, _event_count) = compute_missing_sets(
        reachable_count,
        domain_size,
        entry_missing,
        writes_contains,
        successors_of,
    );

    // Validate reads against the converged fixed point only - never during
    // an unstable iteration, so the reported diagnostic always describes the
    // actual least/greatest fixed point, not transient worklist state.
    // Iterating reachable positions in ascending (offset) order, then each
    // instruction's own reads in their fixed decode order, makes the first
    // reported diagnostic deterministic across repeated runs on identical
    // bytes, independent of any `HashSet`/`HashMap` iteration order.
    for pos in 0..reachable_count {
        let idx = reachable_indices[pos];
        for &r in csr_slice(&reads_flat, &reads_offsets, pos) {
            if missing[pos].contains(dense(r)) {
                return Err(reject_one(
                    function,
                    VerificationCode::UndefinedRegisterRead,
                    instr_starts[idx],
                    format!(
                        "register r{r} is read but not definitely defined on every execution path reaching this instruction"
                    ),
                ));
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
            refs.writes.push(dst);
            let literal = read_u8(code, cursor).map_err(|_| invalid("truncated quad literal"))?;
            if enforce_canonical_domains && literal > 3 {
                return Err(invalid("non-canonical quad literal: must be 0..=3"));
            }
        }
        Opcode::LoadBool => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            refs.writes.push(dst);
            let literal = read_u8(code, cursor).map_err(|_| invalid("truncated bool literal"))?;
            if enforce_canonical_domains && literal > 1 {
                return Err(invalid("non-canonical bool literal: must be 0 or 1"));
            }
        }
        Opcode::LoadI32 => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            refs.writes.push(dst);
            read_i32_le(code, cursor).map_err(|_| invalid("truncated i32 literal"))?;
        }
        Opcode::AddI32 => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            let lhs = read_u16_le(code, cursor).map_err(|_| invalid("truncated lhs register"))?;
            let rhs = read_u16_le(code, cursor).map_err(|_| invalid("truncated rhs register"))?;
            mark_reg(dst);
            mark_reg(lhs);
            mark_reg(rhs);
            refs.reads.push(lhs);
            refs.reads.push(rhs);
            refs.writes.push(dst);
        }
        Opcode::SubI32 | Opcode::MulI32 | Opcode::DivI32 | Opcode::ModI32 => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            let lhs = read_u16_le(code, cursor).map_err(|_| invalid("truncated lhs register"))?;
            let rhs = read_u16_le(code, cursor).map_err(|_| invalid("truncated rhs register"))?;
            mark_reg(dst);
            mark_reg(lhs);
            mark_reg(rhs);
            refs.reads.push(lhs);
            refs.reads.push(rhs);
            refs.writes.push(dst);
        }
        Opcode::LoadU32 => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            refs.writes.push(dst);
            read_u32_le(code, cursor).map_err(|_| invalid("truncated u32 literal"))?;
        }
        Opcode::LoadF64 => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            refs.writes.push(dst);
            refs.required_capabilities |= CAP_F64_MATH;
            read_f64_le(code, cursor).map_err(|_| invalid("truncated f64 literal"))?;
        }
        Opcode::LoadFx => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            refs.writes.push(dst);
            refs.required_capabilities |= CAP_FX_VALUES;
            read_i32_le(code, cursor).map_err(|_| invalid("truncated fx literal"))?;
        }
        Opcode::LoadText => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            refs.writes.push(dst);
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
            refs.reads.push(lhs);
            refs.reads.push(rhs);
            refs.writes.push(dst);
            refs.required_capabilities |= CAP_TEXT_VALUES;
        }
        Opcode::MakeSequence => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence dst register"))?;
            mark_reg(dst);
            refs.writes.push(dst);
            refs.required_capabilities |= CAP_SEQUENCE_VALUES;
            let count = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence arity"))? as usize;
            for _ in 0..count {
                let src = read_u16_le(code, cursor)
                    .map_err(|_| invalid("truncated sequence item register"))?;
                mark_reg(src);
                refs.reads.push(src);
            }
        }
        Opcode::MakeTuple => {
            let dst =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated tuple dst register"))?;
            mark_reg(dst);
            refs.writes.push(dst);
            let count =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated tuple arity"))? as usize;
            if enforce_canonical_domains && count < 2 {
                return Err(invalid("tuple literal arity must be at least 2"));
            }
            for _ in 0..count {
                let src = read_u16_le(code, cursor)
                    .map_err(|_| invalid("truncated tuple item register"))?;
                mark_reg(src);
                refs.reads.push(src);
            }
        }
        Opcode::MakeRecord => {
            let dst =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated record dst register"))?;
            mark_reg(dst);
            refs.writes.push(dst);
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
                refs.reads.push(src);
            }
        }
        Opcode::MakeAdt => {
            let dst =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated enum dst register"))?;
            mark_reg(dst);
            refs.writes.push(dst);
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
                refs.reads.push(src);
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
            refs.reads.push(src);
            refs.writes.push(dst);
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
            refs.reads.push(src);
            refs.writes.push(dst);
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
            refs.reads.push(src);
            refs.writes.push(dst);
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
            refs.reads.push(src);
            refs.writes.push(dst);
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
            refs.reads.push(src);
            refs.reads.push(index);
            refs.writes.push(dst);
            refs.required_capabilities |= CAP_SEQUENCE_VALUES;
        }
        Opcode::SequenceLen => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence-len dst register"))?;
            let src = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated sequence-len src register"))?;
            mark_reg(dst);
            mark_reg(src);
            refs.reads.push(src);
            refs.writes.push(dst);
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
            refs.reads.push(src);
            refs.writes.push(dst);
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
            refs.reads.push(seq);
            refs.reads.push(val);
            refs.writes.push(dst);
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
            refs.reads.push(seq);
            refs.reads.push(val);
            refs.writes.push(dst);
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
            refs.reads.push(seq);
            refs.reads.push(val);
            refs.writes.push(dst);
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
            refs.reads.push(src);
            refs.writes.push(dst);
            refs.required_capabilities |= CAP_SEQUENCE_VALUES;
            refs.required_capabilities |= CAP_SEQUENCE_ITERATION;
        }
        Opcode::MapEmpty => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated map-empty dst register"))?;
            mark_reg(dst);
            refs.writes.push(dst);
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
            refs.reads.push(map);
            refs.reads.push(key);
            refs.writes.push(dst);
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
            refs.reads.push(map);
            refs.reads.push(key);
            // #1756 (FA-07-016): MAP_GET reads `default_val` lazily at
            // runtime, only on a key miss (see #1771). The verifier cannot
            // soundly prove key presence, so this pass conservatively
            // requires `default_val` definitely defined unconditionally -
            // never a value-sensitive "only on the miss path" proof.
            refs.reads.push(default_val);
            refs.writes.push(dst);
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
            refs.reads.push(map);
            refs.reads.push(key);
            refs.reads.push(val);
            refs.writes.push(dst);
            refs.required_capabilities |= CAP_MAP_VALUES;
        }
        Opcode::RngSeed => {
            let dst = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated rng-seed dst register"))?;
            let seed = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated rng-seed seed register"))?;
            mark_reg(dst);
            mark_reg(seed);
            refs.reads.push(seed);
            refs.writes.push(dst);
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
            refs.reads.push(lo);
            refs.reads.push(hi);
            refs.writes.push(dst);
            refs.required_capabilities |= CAP_PRNG;
        }
        Opcode::MakeClosure => {
            let dst =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated closure dst register"))?;
            mark_reg(dst);
            refs.writes.push(dst);
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
                refs.reads.push(src);
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
                refs.writes.push(dst);
            } else {
                // #1756 (FA-07-016): matches the pre-existing `mark_reg`
                // asymmetry below - the encoded dummy dst register bytes are
                // still consumed from the stream, but this opcode form
                // defines no register at all, so the dummy dst is neither a
                // read nor a write.
                let _ = read_u16_le(code, cursor)
                    .map_err(|_| invalid("truncated closure-call dst register"))?;
            }
            let closure = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated closure-call source register"))?;
            let arg = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated closure-call arg register"))?;
            mark_reg(closure);
            mark_reg(arg);
            refs.reads.push(closure);
            refs.reads.push(arg);
            refs.required_capabilities |= CAP_CLOSURE_VALUES;
        }
        Opcode::LoadVar => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            refs.writes.push(dst);
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
            // #1756 (FA-07-016): reads the source register into the named
            // local-variable slot. Register dataflow only - the variable
            // store itself is outside this pass's domain (see LOAD_VAR,
            // which writes only its destination register, not a register
            // form of the variable it names).
            refs.reads.push(src);
        }
        Opcode::QNot | Opcode::BoolNot | Opcode::QTruthNot => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            let src = read_u16_le(code, cursor).map_err(|_| invalid("truncated src register"))?;
            mark_reg(dst);
            mark_reg(src);
            refs.reads.push(src);
            refs.writes.push(dst);
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
            refs.reads.push(lhs);
            refs.reads.push(rhs);
            refs.writes.push(dst);
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
            refs.reads.push(cond);
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
            let has_dst = has_dst_flag != 0;
            if has_dst {
                let dst = read_u16_le(code, cursor)
                    .map_err(|_| invalid("truncated call dst register"))?;
                mark_reg(dst);
                refs.writes.push(dst);
            } else {
                // #1756 (FA-07-016): same asymmetry as `ClosureCall` above -
                // the dummy dst register bytes are consumed but define
                // nothing; this form's `CALL` defines no register.
                let _ = read_u16_le(code, cursor)
                    .map_err(|_| invalid("truncated call dst register"))?;
            }
            let sid =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated callee string id"))?;
            refs.string_refs.push((offset, sid as usize, "call target"));
            let argc = read_u16_le(code, cursor).map_err(|_| invalid("truncated argc"))? as usize;
            // #1773 (FA-09-005): recorded alongside the "call target" string
            // ref above, keyed by the same `offset`, so the cross-function
            // pass can enforce arity against the callee's canonical
            // signature without re-decoding operands.
            refs.call_argcs.push((offset, argc));
            for _ in 0..argc {
                let arg = read_u16_le(code, cursor)
                    .map_err(|_| invalid("truncated call arg register"))?;
                mark_reg(arg);
                refs.reads.push(arg);
            }
        }
        Opcode::Assert => {
            let cond = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated assert condition register"))?;
            mark_reg(cond);
            refs.reads.push(cond);
        }
        Opcode::GateRead => {
            let dst =
                read_u16_le(code, cursor).map_err(|_| invalid("truncated gate dst register"))?;
            mark_reg(dst);
            refs.writes.push(dst);
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
            refs.reads.push(src);
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
            refs.writes.push(dst);
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
            refs.reads.push(src);
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
            refs.writes.push(dst);
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
                refs.reads.push(src);
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
    // #1773 (FA-09-005): (offset, argc) for each `Opcode::Call` site, keyed
    // by the same offset as its "call target" string_refs entry.
    call_argcs: Vec<(usize, usize)>,
    // #1756 (FA-07-016): every register this instruction reads/writes, in
    // the order fixed by `decode_operands`'s exhaustive `Opcode` match below
    // (the same match that already classifies every operand for the
    // register-range quota). `reads` and `writes` are independent - a
    // register can appear in both (e.g. an opcode reading and writing the
    // same numeric register would list it in both), never merged, so the
    // definite-assignment pass can validate reads against `IN[n]` before
    // folding writes into `OUT[n]`.
    reads: Vec<u16>,
    writes: Vec<u16>,
}

#[cfg(feature = "std")]
struct PendingVerifiedFunction {
    verified: VerifiedFunction,
    // (offset, callee name, allows_builtin, argc for real Call sites)
    call_targets: Vec<(usize, String, bool, Option<usize>)>,
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
        read_u16_le, read_u32_le, CallableValueFamily, MAGIC0, MAGIC10, MAGIC11, MAGIC18, MAGIC19,
        MAGIC3, MAGIC4, MAGIC5, MAGIC6, MAGIC7, OWNERSHIP_SECTION_TAG, SIGNATURE_SECTION_TAG,
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
                params: Vec::new(),
            }],
            false,
        )
        .expect("emit test function")
    }

    // #1756 (FA-07-016): like `emit_test_function`, but with a caller-chosen
    // canonical SIG0 parameter list, for regressions that need a non-empty
    // `ENTRY_DEFS`.
    fn emit_test_function_with_params(
        params: Vec<CallableValueFamily>,
        instrs: Vec<IrInstr>,
    ) -> Vec<u8> {
        emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs,
                ownership_events: Vec::new(),
                params,
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
                    params: Vec::new(),
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
                    params: Vec::new(),
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
                    params: Vec::new(),
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
                    params: Vec::new(),
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
        // #1773 (FA-09-005): SEMCOD19/rev20 is now the floor for every
        // compiled artifact - see the analogous sm-ir comment.
        assert_eq!(verified.header.rev, 20);
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
        assert_eq!(verified.header.rev, 20);
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
        assert_eq!(verified.header.rev, 20);
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
                params: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 20);
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
                params: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 20);
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
                params: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 20);
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_accepts_clock_read_semcode() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![IrInstr::ClockRead { dst: 0 }, IrInstr::Ret { src: None }],
                ownership_events: Vec::new(),
                params: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 20);
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
        assert_eq!(verified.header.rev, 20);
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
        assert_eq!(verified.header.rev, 20);
        assert_eq!(verified.functions.len(), 2);
    }

    #[test]
    fn verifier_accepts_record_field_borrow_ownership_semcode() {
        let bytes = record_field_borrow_semcode_bytes();
        assert_eq!(&bytes[..MAGIC19.len()], &MAGIC19);
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 20);
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_accepts_record_field_write_ownership_semcode() {
        let bytes = record_field_write_semcode_bytes();
        assert_eq!(&bytes[..MAGIC19.len()], &MAGIC19);
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 20);
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
        assert_eq!(&bytes[..MAGIC19.len()], b"SEMCOD19");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 20);
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
        // #1773 (FA-09-005): the first opcode byte's absolute offset now
        // depends on whether a SIG0 section (and, at this revision, OWN0)
        // precede the instruction stream - located via the real decode
        // rather than a hand-counted literal, which silently drifted out of
        // the instruction stream once SIG0 became mandatory.
        let (_, code_start, _) = function_code_span(&bytes, "main");
        let (_, functions) =
            sm_format::semcode_decode::decode_semcode_envelope(&bytes).expect("decode");
        let main = functions
            .iter()
            .find(|f| f.name == "main")
            .expect("main function");
        let opcode_pos = code_start + main.instr_start_offset;
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

    // #1757 (FA-07-017): a register above VerifiedLocal's 4096 budget but
    // within KernelBound's 8192 budget must be rejected under the default
    // (VerifiedLocal) admission gate.
    #[test]
    fn verify_semcode_token_default_still_rejects_register_past_verified_local_budget() {
        let mut bytes = compile_program_to_semcode("fn main() { let a: bool = true; return; }")
            .expect("compile");
        let opcode_pos = find_instruction(&bytes, "main", Opcode::LoadBool, 0);
        let dst_pos = opcode_pos + 1;
        bytes[dst_pos..dst_pos + 2].copy_from_slice(&5000u16.to_le_bytes());
        let report = verify_semcode_token(&bytes).expect_err("must reject under default quotas");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::InvalidRegisterReference
        );
    }

    // The same register must be ACCEPTED when the caller explicitly admits
    // against KernelBound quotas via the new quota-aware API.
    #[test]
    fn verify_semcode_token_with_quotas_accepts_register_within_kernel_bound_budget() {
        let mut bytes = compile_program_to_semcode("fn main() { let a: bool = true; return; }")
            .expect("compile");
        let load_opcode_pos = find_instruction(&bytes, "main", Opcode::LoadBool, 0);
        let load_dst_pos = load_opcode_pos + 1;
        bytes[load_dst_pos..load_dst_pos + 2].copy_from_slice(&5000u16.to_le_bytes());
        // #1756 (FA-07-016): the compiled body also has `StoreVar "a" src=r0`
        // reading whatever register `LoadBool` originally wrote. Patching
        // only `LoadBool`'s dst would leave that `StoreVar` reading a
        // register nothing ever defines - repoint it at the same patched
        // register so this fixture keeps proving "r5000 is within budget",
        // not "an undefined read happens to still be in range".
        let store_opcode_pos = find_instruction(&bytes, "main", Opcode::StoreVar, 0);
        let store_src_pos = store_opcode_pos + 1 + 2;
        bytes[store_src_pos..store_src_pos + 2].copy_from_slice(&5000u16.to_le_bytes());
        verify_semcode_token_with_quotas(&bytes, RuntimeQuotas::kernel_bound())
            .expect("r5000 must be within KernelBound's register budget");
    }

    // KernelBound is not infinite: a register above its own 8192 budget must
    // still be rejected even when explicitly admitting against KernelBound.
    #[test]
    fn verify_semcode_token_with_quotas_rejects_register_past_kernel_bound_budget() {
        let mut bytes = compile_program_to_semcode("fn main() { let a: bool = true; return; }")
            .expect("compile");
        let opcode_pos = find_instruction(&bytes, "main", Opcode::LoadBool, 0);
        let dst_pos = opcode_pos + 1;
        bytes[dst_pos..dst_pos + 2].copy_from_slice(&8200u16.to_le_bytes());
        let report = verify_semcode_token_with_quotas(&bytes, RuntimeQuotas::kernel_bound())
            .expect_err("must reject past KernelBound's own register budget");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::InvalidRegisterReference
        );
    }

    // No regression to the common path: an ordinary small artifact still
    // admits cleanly under the unchanged default quota profile.
    #[test]
    fn verify_semcode_token_default_still_accepts_ordinary_artifact() {
        let bytes = compile_program_to_semcode("fn main() { let a: bool = true; return; }")
            .expect("compile");
        verify_semcode_token(&bytes).expect("ordinary artifact must still admit");
    }

    // #1757 review follow-up: since quotas are now caller-supplied, a
    // rejection diagnostic must not hardcode "verified-local" - that text
    // would misidentify the active policy for any other profile (e.g. a
    // KernelBound rejection reporting its 8192-register budget as if it
    // were verified-local's, whose real budget is 4096).
    #[test]
    fn verify_semcode_token_with_quotas_rejection_message_does_not_hardcode_verified_local() {
        let mut bytes = compile_program_to_semcode("fn main() { let a: bool = true; return; }")
            .expect("compile");
        let opcode_pos = find_instruction(&bytes, "main", Opcode::LoadBool, 0);
        let dst_pos = opcode_pos + 1;
        bytes[dst_pos..dst_pos + 2].copy_from_slice(&8200u16.to_le_bytes());
        let report = verify_semcode_token_with_quotas(&bytes, RuntimeQuotas::kernel_bound())
            .expect_err("must reject past KernelBound's own register budget");
        assert!(
            !report.diagnostics[0].message.contains("verified-local"),
            "diagnostic must not claim a KernelBound rejection is a verified-local budget: {}",
            report.diagnostics[0].message
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

    // FA-07-013 (#1753): a no-destination CALL (has_dst=0) must not count its
    // ignored dst placeholder as a live register reference. The emitter
    // writes a dummy dst field even when has_dst=0; that dummy value must
    // never reach mark_reg, so an oversized placeholder must not spuriously
    // blow the register budget.
    #[test]
    fn verifier_accepts_call_no_dst_with_out_of_budget_placeholder() {
        let mut bytes =
            compile_program_to_semcode("fn helper() { return; } fn main() { helper(); return; }")
                .expect("compile");
        let opcode_pos = find_instruction(&bytes, "main", Opcode::Call, 0);
        assert_eq!(
            bytes[opcode_pos + 1],
            0u8,
            "expected has_dst=0 for a no-assignment call"
        );
        let dst_pos = opcode_pos + 2;
        bytes[dst_pos..dst_pos + 2].copy_from_slice(&5000u16.to_le_bytes());
        verify_semcode(&bytes).expect("no-dst call must ignore its dummy dst placeholder");
    }

    // FA-07-013 (#1753) control: a CALL that genuinely has a destination
    // (has_dst=1) must still have that real dst register validated against
    // the register budget - the fix must not blanket-skip mark_reg.
    #[test]
    fn verifier_rejects_call_real_dst_past_register_budget() {
        let mut bytes = compile_program_to_semcode(
            "fn helper() -> bool { return true; } fn main() { let a: bool = helper(); return; }",
        )
        .expect("compile");
        let opcode_pos = find_instruction(&bytes, "main", Opcode::Call, 0);
        assert_eq!(
            bytes[opcode_pos + 1],
            1u8,
            "expected has_dst=1 for an assigned call"
        );
        let dst_pos = opcode_pos + 2;
        bytes[dst_pos..dst_pos + 2].copy_from_slice(&5000u16.to_le_bytes());
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::InvalidRegisterReference
        );
    }

    // FA-07-013 (#1753) control: CLOSURE_CALL already ignores its dummy dst
    // placeholder when has_dst=0 - this locks in that reference behavior
    // that Opcode::Call's decode_operands branch was made to mirror.
    #[test]
    fn verifier_accepts_closure_call_no_dst_with_out_of_budget_placeholder() {
        let mut bytes = emit_ir_to_semcode(
            &[
                IrFunction {
                    name: "helper".to_string(),
                    instrs: vec![IrInstr::Ret { src: None }],
                    ownership_events: Vec::new(),
                    params: Vec::new(),
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
                    params: Vec::new(),
                },
            ],
            false,
        )
        .expect("emit");
        let opcode_pos = find_instruction(&bytes, "main", Opcode::ClosureCall, 0);
        assert_eq!(bytes[opcode_pos + 1], 0u8, "expected has_dst=0");
        let dst_pos = opcode_pos + 2;
        bytes[dst_pos..dst_pos + 2].copy_from_slice(&5000u16.to_le_bytes());
        verify_semcode(&bytes).expect("no-dst closure-call must ignore its dummy dst placeholder");
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
                    params: Vec::new(),
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
                    params: Vec::new(),
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
        let emitted = emit_ir_to_semcode(
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
                params: Vec::new(),
            }],
            false,
        )
        .expect("emit");

        // #1773 (FA-09-005): `emit_ir_to_semcode` now unconditionally
        // targets SEMCOD19+ (mandatory OWN0 + SIG0 sections), which places
        // real, correctly-tagged sections between the string table and the
        // instruction stream - closing off exactly this collision site for
        // any artifact the current emitter can produce (the DBG0 sniff runs
        // once, immediately after the string table, and now finds a real
        // "OWN0" tag there instead of the TupleGet's colliding bytes). The
        // #1731 vulnerability this test proves closed remains real for
        // pre-#1732 header revisions (V0-V10, no mandatory OWN0), so this
        // rebuilds the bare `[string table][instruction stream]` envelope
        // shape those headers produce - string table and instruction-stream
        // bytes taken verbatim from the real emission above, only the
        // now-mandatory OWN0/SIG0 sections excised - to keep proving the
        // fix holds for every artifact shape the verifier must still admit.
        let (_, emitted_functions) =
            sm_format::semcode_decode::decode_semcode_envelope(&emitted).expect("decode");
        let emitted_env = &emitted_functions[0];
        let string_table = &emitted_env.code_slice[..2];
        let instr_stream = &emitted_env.code_slice[emitted_env.instr_start_offset..];
        let mut code = Vec::new();
        code.extend_from_slice(string_table);
        code.extend_from_slice(instr_stream);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MAGIC0);
        bytes.extend_from_slice(&4u16.to_le_bytes());
        bytes.extend_from_slice(b"main");
        bytes.extend_from_slice(&(code.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&code);

        // B: byte identity - the reconstructed instruction stream literally
        // spells DBG0 where the real TupleGet begins (string table is
        // empty: a bare 2-byte count=0, so the collision starts at
        // code_slice[2]).
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
                params: Vec::new(),
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
                params: Vec::new(),
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

    /// #1746 (FA-07-006) test helper: overwrites the `pc` field of the
    /// `symbol_index`-th `DBG0` debug-symbol entry in `bytes` in place, via
    /// the decoded format offsets (`code_offset`, `string_table_end_offset`)
    /// rather than a raw-byte search. Entry layout after the `DBG0` tag (4
    /// bytes) and symbol count (2 bytes) is `pc: u32 LE, line: u32,
    /// col: u16` (10 bytes per entry), matching
    /// `semcode_decode::parse_string_table_debug_and_ownership`.
    fn overwrite_debug_symbol_pc(bytes: &mut [u8], symbol_index: usize, new_pc: u32) {
        let entry_offset = {
            let (_, functions) = sm_format::semcode_decode::decode_semcode_envelope(bytes)
                .expect("decode for mutation");
            let f = &functions[0];
            f.code_offset + f.string_table_end_offset + 4 + 2 + symbol_index * 10
        };
        bytes[entry_offset..entry_offset + 4].copy_from_slice(&new_pc.to_le_bytes());
    }

    // #1746 (FA-07-006) regression matrix (1/7 and 2/7): a debug-symbol pc
    // that is numerically in range but lands inside a decoded instruction's
    // operand bytes - not on an instruction start - must reject. `LOAD_BOOL
    // dst, val` is a real 4-byte instruction (opcode + 2-byte dst register +
    // 1-byte bool literal) at instruction offset 0; offsets 1 and 2 are two
    // distinct interior bytes of its `dst` operand, proving this is a
    // boundary-membership check (`instr_starts.contains`), not a single
    // hardcoded interior offset.
    #[test]
    fn verifier_rejects_debug_pc_pointing_into_operand_middle_byte() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::LoadBool { dst: 0, val: true },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: Vec::new(),
                params: Vec::new(),
            }],
            true,
        )
        .expect("emit");
        let mut mutated = bytes.clone();
        overwrite_debug_symbol_pc(&mut mutated, 0, 1);
        let report = verify_semcode(&mutated)
            .expect_err("debug pc pointing into the first interior operand byte must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::InvalidDebugSection
        );
        assert_eq!(report.diagnostics[0].offset, Some(1));
    }

    #[test]
    fn verifier_rejects_debug_pc_pointing_into_operand_final_byte() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::LoadBool { dst: 0, val: true },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: Vec::new(),
                params: Vec::new(),
            }],
            true,
        )
        .expect("emit");
        let mut mutated = bytes.clone();
        overwrite_debug_symbol_pc(&mut mutated, 0, 2);
        let report = verify_semcode(&mutated)
            .expect_err("debug pc pointing into a second interior operand byte must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::InvalidDebugSection
        );
        assert_eq!(report.diagnostics[0].offset, Some(2));
    }

    // #1746 regression matrix (3/7 and 4/7): pc == 0 at the first
    // instruction, and pc pointing at a later real instruction start, must
    // both remain accepted - unmutated, genuinely compiler-emitted debug
    // symbols for a two-instruction function land exactly on the two real
    // instruction starts (0 and 4).
    #[test]
    fn verifier_accepts_debug_pc_at_first_and_later_instruction_starts() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::LoadBool { dst: 0, val: true },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: Vec::new(),
                params: Vec::new(),
            }],
            true,
        )
        .expect("emit");
        let (_, functions) =
            sm_format::semcode_decode::decode_semcode_envelope(&bytes).expect("decode");
        let pcs: Vec<usize> = functions[0].debug_symbols.iter().map(|s| s.pc).collect();
        assert_eq!(
            pcs,
            vec![0, 4],
            "fixture must exercise pc=0 (first instruction) and a later instruction start"
        );
        let verified =
            verify_semcode(&bytes).expect("debug pcs on real instruction starts must verify");
        assert_eq!(verified.functions[0].debug_symbol_count, 2);
    }

    // #1746 regression matrix (5/7): out-of-range debug pc must still
    // reject with the existing diagnostic and message, unchanged by the new
    // boundary check (range is checked first).
    #[test]
    fn verifier_rejects_out_of_range_debug_pc() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::LoadBool { dst: 0, val: true },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: Vec::new(),
                params: Vec::new(),
            }],
            true,
        )
        .expect("emit");
        let mut mutated = bytes.clone();
        overwrite_debug_symbol_pc(&mut mutated, 0, 100);
        let report = verify_semcode(&mutated).expect_err("out-of-range debug pc must still reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::InvalidDebugSection
        );
        assert_eq!(report.diagnostics[0].offset, Some(100));
        assert_eq!(
            report.diagnostics[0].message,
            "debug symbol pc points past the instruction stream"
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
                params: Vec::new(),
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
        let emitted = emit_ir_to_semcode(
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
                params: Vec::new(),
            }],
            false,
        )
        .expect("emit");

        // #1773 (FA-09-005): rebuild the bare `[string table][instruction
        // stream]` envelope shape (no mandatory OWN0/SIG0) so the DBG0 sniff
        // lands on the TupleGet collision again - see the identical comment
        // on `verifier_rejects_ambiguous_dbg0_tupleget_collision` above.
        let (_, emitted_functions) =
            sm_format::semcode_decode::decode_semcode_envelope(&emitted).expect("decode");
        let emitted_env = &emitted_functions[0];
        let mut code = Vec::new();
        code.extend_from_slice(&emitted_env.code_slice[..2]);
        code.extend_from_slice(&emitted_env.code_slice[emitted_env.instr_start_offset..]);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MAGIC0);
        bytes.extend_from_slice(&4u16.to_le_bytes());
        bytes.extend_from_slice(b"main");
        bytes.extend_from_slice(&(code.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&code);

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
        let emitted = emit_ir_to_semcode(
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
                params: Vec::new(),
            }],
            false,
        )
        .expect("emit");

        // #1773 (FA-09-005): rebuild the bare `[string table][instruction
        // stream]` envelope shape - see the identical comment on
        // `verifier_rejects_ambiguous_dbg0_tupleget_collision` above.
        let (_, emitted_functions) =
            sm_format::semcode_decode::decode_semcode_envelope(&emitted).expect("decode");
        let emitted_env = &emitted_functions[0];
        let mut code = Vec::new();
        code.extend_from_slice(&emitted_env.code_slice[..2]);
        code.extend_from_slice(&emitted_env.code_slice[emitted_env.instr_start_offset..]);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MAGIC0);
        bytes.extend_from_slice(&4u16.to_le_bytes());
        bytes.extend_from_slice(b"main");
        bytes.extend_from_slice(&(code.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&code);

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

    // #1773 (FA-09-005) permanent regressions: sm-verify's arity
    // enforcement. Built at the IR level (not the source front-end) because
    // the front-end always emits a call site whose argc matches the
    // callee's real declared arity - a genuine mismatch can only arise from
    // a malformed/adversarial artifact, exactly what the verifier's
    // admission gate must catch.

    #[test]
    fn verifier_rejects_call_argument_count_mismatch() {
        let bytes = emit_ir_to_semcode(
            &[
                IrFunction {
                    name: "callee".to_string(),
                    instrs: vec![IrInstr::Ret { src: None }],
                    ownership_events: Vec::new(),
                    params: vec![CallableValueFamily::I32, CallableValueFamily::I32],
                },
                IrFunction {
                    name: "main".to_string(),
                    instrs: vec![
                        IrInstr::LoadI32 { dst: 0, val: 1 },
                        IrInstr::Call {
                            dst: None,
                            name: "callee".to_string(),
                            args: vec![0],
                        },
                        IrInstr::Ret { src: None },
                    ],
                    ownership_events: Vec::new(),
                    params: Vec::new(),
                },
            ],
            false,
        )
        .expect("emit");

        let report = verify_semcode(&bytes).expect_err("must reject argc/arity mismatch");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::CallArgumentCountMismatch
        );
    }

    #[test]
    fn verifier_accepts_call_matching_argument_count() {
        let bytes = emit_ir_to_semcode(
            &[
                IrFunction {
                    name: "callee".to_string(),
                    instrs: vec![IrInstr::Ret { src: None }],
                    ownership_events: Vec::new(),
                    params: vec![CallableValueFamily::I32, CallableValueFamily::I32],
                },
                IrFunction {
                    name: "main".to_string(),
                    instrs: vec![
                        IrInstr::LoadI32 { dst: 0, val: 1 },
                        IrInstr::LoadI32 { dst: 1, val: 2 },
                        IrInstr::Call {
                            dst: None,
                            name: "callee".to_string(),
                            args: vec![0, 1],
                        },
                        IrInstr::Ret { src: None },
                    ],
                    ownership_events: Vec::new(),
                    params: Vec::new(),
                },
            ],
            false,
        )
        .expect("emit");

        let verified = verify_semcode(&bytes).expect("matching argc must verify");
        assert_eq!(verified.functions.len(), 2);
    }

    // --- #1756 (FA-07-016, umbrella #1617) regression matrix -------------
    //
    // Forward MUST dataflow proving every reachable register read is
    // definitely defined on every incoming execution path. `ENTRY_DEFS` is
    // exactly `{r0..signature.families.len()-1}` from the function's
    // canonical SIG0 signature (#1773); every other register starts
    // undefined until an instruction actually writes it. Cases 1-26 below
    // match the campaign's required regression matrix one-to-one.

    #[test]
    fn c1756_case1_rejects_zero_arg_entry_undefined_read() {
        let bytes = emit_test_function(vec![IrInstr::Ret { src: Some(0) }]);
        let report =
            verify_semcode(&bytes).expect_err("r0 undefined at a zero-arg entry must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::UndefinedRegisterRead
        );
    }

    #[test]
    fn c1756_case2_accepts_one_parameter_entry_read() {
        let bytes = emit_test_function_with_params(
            vec![CallableValueFamily::I32],
            vec![IrInstr::Ret { src: Some(0) }],
        );
        verify_semcode(&bytes).expect("r0 is entry-defined by SIG0's one parameter");
    }

    #[test]
    fn c1756_case3_accepts_multiple_parameter_entry_reads() {
        let bytes = emit_test_function_with_params(
            vec![
                CallableValueFamily::I32,
                CallableValueFamily::I32,
                CallableValueFamily::I32,
            ],
            vec![
                IrInstr::AddI32 {
                    dst: 3,
                    lhs: 0,
                    rhs: 1,
                },
                IrInstr::AddI32 {
                    dst: 3,
                    lhs: 3,
                    rhs: 2,
                },
                IrInstr::Ret { src: Some(3) },
            ],
        );
        verify_semcode(&bytes).expect("r0..r2 are all entry-defined by SIG0's three parameters");
    }

    #[test]
    fn c1756_case4_rejects_register_just_past_parameter_prefix() {
        let bytes = emit_test_function_with_params(
            vec![CallableValueFamily::I32],
            vec![IrInstr::Ret { src: Some(1) }],
        );
        let report = verify_semcode(&bytes)
            .expect_err("r1 is not entry-defined by a one-parameter signature");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::UndefinedRegisterRead
        );
    }

    #[test]
    fn c1756_case5_rejects_read_before_write_same_register() {
        let bytes = emit_test_function(vec![
            IrInstr::AddI32 {
                dst: 6,
                lhs: 5,
                rhs: 5,
            },
            IrInstr::LoadI32 { dst: 5, val: 1 },
            IrInstr::Ret { src: Some(6) },
        ]);
        let report = verify_semcode(&bytes).expect_err("r5 is read before any write reaches it");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::UndefinedRegisterRead
        );
    }

    #[test]
    fn c1756_case6_accepts_write_before_read_same_register() {
        let bytes = emit_test_function(vec![
            IrInstr::LoadI32 { dst: 5, val: 1 },
            IrInstr::AddI32 {
                dst: 6,
                lhs: 5,
                rhs: 5,
            },
            IrInstr::Ret { src: Some(6) },
        ]);
        verify_semcode(&bytes).expect("r5 is written before it is read");
    }

    #[test]
    fn c1756_case7_accepts_read_when_both_branches_define() {
        let bytes = emit_test_function(vec![
            IrInstr::LoadBool { dst: 0, val: true },
            IrInstr::JmpIf {
                cond: 0,
                label: "then".to_string(),
            },
            IrInstr::LoadI32 { dst: 5, val: 1 }, // else path
            IrInstr::Jmp {
                label: "join".to_string(),
            },
            IrInstr::Label {
                name: "then".to_string(),
            },
            IrInstr::LoadI32 { dst: 5, val: 2 }, // then path
            IrInstr::Label {
                name: "join".to_string(),
            },
            IrInstr::Ret { src: Some(5) },
        ]);
        verify_semcode(&bytes).expect("both branches define r5 before the join reads it");
    }

    #[test]
    fn c1756_case8_rejects_read_when_only_one_branch_defines() {
        let bytes = emit_test_function(vec![
            IrInstr::LoadBool { dst: 0, val: true },
            IrInstr::JmpIf {
                cond: 0,
                label: "then".to_string(),
            },
            IrInstr::Jmp {
                label: "join".to_string(),
            }, // else path: no write
            IrInstr::Label {
                name: "then".to_string(),
            },
            IrInstr::LoadI32 { dst: 5, val: 2 }, // only the then path writes r5
            IrInstr::Label {
                name: "join".to_string(),
            },
            IrInstr::Ret { src: Some(5) },
        ]);
        let report = verify_semcode(&bytes).expect_err("only the 'then' branch defines r5");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::UndefinedRegisterRead
        );
    }

    #[test]
    fn c1756_case9_accepts_definition_before_branch_survives_join() {
        let bytes = emit_test_function(vec![
            IrInstr::LoadI32 { dst: 5, val: 1 },
            IrInstr::LoadBool { dst: 0, val: true },
            IrInstr::JmpIf {
                cond: 0,
                label: "join".to_string(),
            },
            IrInstr::Label {
                name: "join".to_string(),
            },
            IrInstr::Ret { src: Some(5) },
        ]);
        verify_semcode(&bytes).expect("r5 defined before the branch remains defined on both paths");
    }

    #[test]
    fn c1756_case10_accepts_dominating_definition_before_loop() {
        let bytes = emit_test_function(vec![
            IrInstr::LoadI32 { dst: 5, val: 1 },
            IrInstr::Label {
                name: "loop_head".to_string(),
            },
            IrInstr::AddI32 {
                dst: 6,
                lhs: 5,
                rhs: 5,
            },
            IrInstr::Jmp {
                label: "loop_head".to_string(),
            },
        ]);
        verify_semcode(&bytes)
            .expect("r5 defined once before the loop remains valid on every iteration");
    }

    #[test]
    fn c1756_case11_rejects_first_iteration_reading_loop_carried_undefined_register() {
        let bytes = emit_test_function(vec![
            IrInstr::Label {
                name: "loop_head".to_string(),
            },
            IrInstr::AddI32 {
                dst: 6,
                lhs: 5,
                rhs: 5,
            }, // reads r5, which no path yet defines on entry to loop_head
            IrInstr::LoadI32 { dst: 5, val: 1 }, // defines r5 only for the NEXT iteration
            IrInstr::Jmp {
                label: "loop_head".to_string(),
            },
        ]);
        let report = verify_semcode(&bytes)
            .expect_err("the first iteration reads r5 before any path has defined it");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::UndefinedRegisterRead
        );
    }

    /// Case 12 proves the non-negotiable TOP-initialization rule is load-
    /// bearing, not merely a stated preference. `loop_head` (the `AddI32`)
    /// has two predecessors: the entry `LoadI32` and the back-edge `JmpIf`.
    /// `r5` is written exactly once, before the loop, and never rewritten
    /// inside it - nothing inside the loop body ever re-adds `r5` to a set
    /// that excludes it. Under the correct rule (non-entry nodes start at
    /// `TOP = U`), `IN[loop_head]` in the very first fixed-point round is
    /// `OUT[entry] ∩ TOP = OUT[entry]`, which already (correctly) contains
    /// `r5`. Under the incorrect rule (non-entry nodes start at `∅`),
    /// `OUT[back-edge]` starts at `∅`, so `IN[loop_head] = OUT[entry] ∩ ∅ =
    /// ∅` in round one - and because MUST-intersection only ever shrinks a
    /// set and nothing in this loop ever re-adds `r5`, that wrongly-empty
    /// result can never self-correct in any later round: it is the loop's
    /// actual (wrong) converged fixed point, not a transient one. This is
    /// exactly the "correct fixed point" vs. "wrong fixed point" failure
    /// the rule exists to prevent - not just a slower convergence.
    #[test]
    fn c1756_case12_accepts_loop_read_requires_top_initialization() {
        let bytes = emit_test_function(vec![
            IrInstr::LoadI32 { dst: 5, val: 1 }, // entry: the only write to r5 anywhere
            IrInstr::Label {
                name: "loop_head".to_string(),
            },
            IrInstr::AddI32 {
                dst: 6,
                lhs: 5,
                rhs: 5,
            }, // loop_head: reads r5 every iteration
            IrInstr::JmpIf {
                cond: 6,
                label: "loop_head".to_string(),
            }, // conditional back-edge
            IrInstr::Ret { src: Some(6) }, // exit path
        ]);
        verify_semcode(&bytes).expect(
            "r5, defined once before the loop and never invalidated inside it, must remain \
             provably defined at loop_head - this is exactly the case that would incorrectly \
             reject under a non-entry-nodes-start-at-empty-set initialization strategy",
        );
    }

    #[test]
    fn c1756_case13_rejects_call_argument_undefined() {
        let bytes = emit_ir_to_semcode(
            &[
                IrFunction {
                    name: "callee".to_string(),
                    instrs: vec![IrInstr::Ret { src: None }],
                    ownership_events: Vec::new(),
                    params: vec![CallableValueFamily::I32],
                },
                IrFunction {
                    name: "main".to_string(),
                    instrs: vec![
                        IrInstr::Call {
                            dst: None,
                            name: "callee".to_string(),
                            args: vec![7],
                        },
                        IrInstr::Ret { src: None },
                    ],
                    ownership_events: Vec::new(),
                    params: Vec::new(),
                },
            ],
            false,
        )
        .expect("emit");
        let report = verify_semcode(&bytes).expect_err("call argument r7 is undefined");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::UndefinedRegisterRead
        );
    }

    #[test]
    fn c1756_case14_accepts_call_argument_defined() {
        let bytes = emit_ir_to_semcode(
            &[
                IrFunction {
                    name: "callee".to_string(),
                    instrs: vec![IrInstr::Ret { src: None }],
                    ownership_events: Vec::new(),
                    params: vec![CallableValueFamily::I32],
                },
                IrFunction {
                    name: "main".to_string(),
                    instrs: vec![
                        IrInstr::LoadI32 { dst: 7, val: 1 },
                        IrInstr::Call {
                            dst: None,
                            name: "callee".to_string(),
                            args: vec![7],
                        },
                        IrInstr::Ret { src: None },
                    ],
                    ownership_events: Vec::new(),
                    params: Vec::new(),
                },
            ],
            false,
        )
        .expect("emit");
        verify_semcode(&bytes).expect("call argument r7 is defined");
    }

    #[test]
    fn c1756_case15_accepts_call_destination_defined_after_call() {
        let bytes = emit_ir_to_semcode(
            &[
                IrFunction {
                    name: "callee".to_string(),
                    instrs: vec![
                        IrInstr::LoadI32 { dst: 0, val: 1 },
                        IrInstr::Ret { src: Some(0) },
                    ],
                    ownership_events: Vec::new(),
                    params: Vec::new(),
                },
                IrFunction {
                    name: "main".to_string(),
                    instrs: vec![
                        IrInstr::Call {
                            dst: Some(0),
                            name: "callee".to_string(),
                            args: vec![],
                        },
                        IrInstr::Ret { src: Some(0) },
                    ],
                    ownership_events: Vec::new(),
                    params: Vec::new(),
                },
            ],
            false,
        )
        .expect("emit");
        verify_semcode(&bytes).expect("a CALL with a destination defines it on fallthrough");
    }

    #[test]
    fn c1756_case16_rejects_reading_call_without_destination_placeholder() {
        let bytes = emit_ir_to_semcode(
            &[
                IrFunction {
                    name: "callee".to_string(),
                    instrs: vec![IrInstr::Ret { src: None }],
                    ownership_events: Vec::new(),
                    params: Vec::new(),
                },
                IrFunction {
                    name: "main".to_string(),
                    instrs: vec![
                        IrInstr::Call {
                            dst: None,
                            name: "callee".to_string(),
                            args: vec![],
                        },
                        // r0 was never defined - the no-dst CALL's encoded
                        // dummy destination must not have defined it.
                        IrInstr::Ret { src: Some(0) },
                    ],
                    ownership_events: Vec::new(),
                    params: Vec::new(),
                },
            ],
            false,
        )
        .expect("emit");
        let report = verify_semcode(&bytes)
            .expect_err("no-dst CALL must not define its encoded placeholder register");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::UndefinedRegisterRead
        );
    }

    #[test]
    fn c1756_case17_rejects_undefined_return_source() {
        let bytes = emit_test_function_with_params(
            vec![CallableValueFamily::I32, CallableValueFamily::I32],
            vec![IrInstr::Ret { src: Some(5) }],
        );
        let report = verify_semcode(&bytes)
            .expect_err("return source r5 is unrelated to either declared parameter");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::UndefinedRegisterRead
        );
    }

    /// Case 18 is distinct from case 15: it specifically proves that
    /// `Value::Unit` is an ordinary defined value for this pass's purposes,
    /// not an "undefined register" sentinel - the callee explicitly returns
    /// `Unit` (`Ret { src: None }`), and reading the caller's destination
    /// register afterward must still be accepted purely because the
    /// register was *written*, independent of what runtime value it holds.
    #[test]
    fn c1756_case18_accepts_reading_register_defined_by_unit_returning_call() {
        let bytes = emit_ir_to_semcode(
            &[
                IrFunction {
                    name: "returns_unit".to_string(),
                    instrs: vec![IrInstr::Ret { src: None }],
                    ownership_events: Vec::new(),
                    params: Vec::new(),
                },
                IrFunction {
                    name: "main".to_string(),
                    instrs: vec![
                        IrInstr::Call {
                            dst: Some(0),
                            name: "returns_unit".to_string(),
                            args: vec![],
                        },
                        IrInstr::Ret { src: Some(0) },
                    ],
                    ownership_events: Vec::new(),
                    params: Vec::new(),
                },
            ],
            false,
        )
        .expect("emit");
        verify_semcode(&bytes)
            .expect("r0 is defined by the call regardless of its runtime value being Unit");
    }

    #[test]
    fn c1756_case19_accepts_unit_parameter_entry_read() {
        let bytes = emit_test_function_with_params(
            vec![CallableValueFamily::Unit],
            vec![IrInstr::Ret { src: Some(0) }],
        );
        verify_semcode(&bytes)
            .expect("r0 is entry-defined even though its declared family is Unit");
    }

    #[test]
    fn c1756_case20_rejects_map_get_default_undefined() {
        let bytes = emit_test_function(vec![
            IrInstr::MapEmpty { dst: 0 },
            IrInstr::LoadI32 { dst: 1, val: 1 },
            IrInstr::MapGet {
                dst: 2,
                map: 0,
                key: 1,
                default_val: 9, // never written anywhere
            },
            IrInstr::Ret { src: Some(2) },
        ]);
        let report = verify_semcode(&bytes).expect_err(
            "MAP_GET's default register is a conservative, unconditional read (#1756) - key \
             presence is never statically proven, matching the runtime laziness rule (#1771) \
             being distinct from this static proof",
        );
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::UndefinedRegisterRead
        );
    }

    #[test]
    fn c1756_case21_accepts_map_get_default_defined() {
        let bytes = emit_test_function(vec![
            IrInstr::MapEmpty { dst: 0 },
            IrInstr::LoadI32 { dst: 1, val: 1 },
            IrInstr::LoadI32 { dst: 9, val: 0 },
            IrInstr::MapGet {
                dst: 2,
                map: 0,
                key: 1,
                default_val: 9,
            },
            IrInstr::Ret { src: Some(2) },
        ]);
        verify_semcode(&bytes).expect("MAP_GET's default register is defined");
    }

    #[test]
    fn c1756_case22_rejects_map_get_map_source_undefined() {
        let bytes = emit_test_function(vec![
            IrInstr::LoadI32 { dst: 1, val: 1 },
            IrInstr::LoadI32 { dst: 9, val: 0 },
            IrInstr::MapGet {
                dst: 2,
                map: 0, // never written
                key: 1,
                default_val: 9,
            },
            IrInstr::Ret { src: Some(2) },
        ]);
        let report = verify_semcode(&bytes).expect_err("MAP_GET's map register is undefined");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::UndefinedRegisterRead
        );
    }

    #[test]
    fn c1756_case23_rejects_conditional_branch_condition_undefined() {
        let bytes = emit_test_function(vec![
            IrInstr::JmpIf {
                cond: 0, // never written
                label: "target".to_string(),
            },
            IrInstr::Label {
                name: "target".to_string(),
            },
            IrInstr::Ret { src: None },
        ]);
        let report = verify_semcode(&bytes).expect_err("branch condition r0 is undefined");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::UndefinedRegisterRead
        );
    }

    #[test]
    fn c1756_case24_ignores_undefined_read_in_unreachable_code() {
        let bytes = emit_test_function(vec![
            IrInstr::Ret { src: None },
            // Unreachable (RET has no successor). r99 is undefined, but
            // this pass only judges reachable reads, matching the
            // verifier's existing, separate policy on structurally valid
            // but unreachable code (see `verifier_accepts_structurally_
            // closed_infinite_loop`'s trailing-code precedent).
            IrInstr::AddI32 {
                dst: 1,
                lhs: 99,
                rhs: 99,
            },
        ]);
        verify_semcode(&bytes)
            .expect("an undefined read in unreachable code must not be judged by this pass");
    }

    /// Codex review round 1 on PR #1840: the original implementation
    /// allocated one `domain_size`-sized `RegSet` per DECODED instruction,
    /// reachable or not - so a structurally valid artifact consisting of an
    /// entry `RET` followed by many unreachable instructions referencing a
    /// high register number could force memory proportional to
    /// `total_instruction_count * domain_size`, with no existing quota
    /// bounding total instruction count. Fixed by allocating dataflow state
    /// only for reachable nodes. This regression exercises exactly that
    /// shape (reachable `RET` immediately, then many unreachable
    /// instructions touching a near-budget register) and proves it still
    /// verifies correctly - a real functional check on the fixed code path,
    /// not just a memory-bound argument.
    #[test]
    fn c1756_rejects_unreachable_bloat_stays_cheap_and_correct() {
        let mut instrs = vec![IrInstr::Ret { src: None }];
        for _ in 0..500 {
            instrs.push(IrInstr::AddI32 {
                dst: 4000,
                lhs: 4000,
                rhs: 4000,
            });
        }
        let bytes = emit_test_function(instrs);
        verify_semcode(&bytes).expect(
            "hundreds of unreachable instructions referencing a high register must not be \
             judged by this pass, and must not meaningfully slow or bloat verification",
        );
    }

    /// Codex review round 3 on PR #1840: even after round 1's reachable-only
    /// `RegSet` allocation fix, `verify_function_code`'s main decode loop
    /// still built a dense `Vec<Vec<u16>>` of reads/writes for every
    /// structurally decoded instruction, reachable or not - an empty
    /// `Vec<u16>` costs 24 bytes even with no heap allocation, so a
    /// signature-bearing function padded with a large number of cheap
    /// unreachable instructions (their own example: an entry `RET` followed
    /// by megabytes of two-byte `RET`s) still cost hundreds of MB for these
    /// two arrays alone, despite only one reachable node. Fixed by no longer
    /// building them at all in the main decode loop; `prove_definite_
    /// register_assignment` re-decodes only the instructions `reachable_
    /// offsets` actually names. 50,000 unreachable `RET`s (100 KB of dead
    /// code) is enough to prove this stays fast and correct without
    /// literally allocating gigabytes in a unit test.
    #[test]
    fn c1756_rejects_large_unreachable_ret_padding_without_dense_metadata() {
        let mut instrs = vec![IrInstr::Ret { src: None }];
        for _ in 0..50_000 {
            instrs.push(IrInstr::Ret { src: None });
        }
        let bytes = emit_test_function(instrs);
        verify_semcode(&bytes).expect(
            "tens of thousands of unreachable two-byte RETs must not require per-instruction \
             reads/writes metadata to be materialized for every one of them",
        );
    }

    /// Codex review round 4 on PR #1840: even after round 3 stopped
    /// materializing metadata for unreachable instructions, a large fully
    /// REACHABLE straight-line stream (their example: millions of
    /// `LOAD_BOOL r0` in a row, no branching, no unreachable padding at
    /// all) still cost dozens of bytes of `Vec<Vec<u16>>`/`Vec<Vec<usize>>`
    /// heap-container overhead per instruction, from the reads/writes
    /// arrays AND the predecessor/successor lists. Fixed by CSR-flattening
    /// reads/writes (`dataflow_domain_accounting`) and eliminating the
    /// predecessor structure entirely in favor of edge relaxation over the
    /// verifier's own `instruction_successors` (`for_each_reachable_
    /// successor`). 300,000 reachable `LOAD_BOOL r0` instructions - every
    /// one of them a repeated write to the same already-defined register,
    /// so `Rc`-sharing (round 2) also keeps every `RegSet` after the first
    /// shared - is enough to prove this stays fast and correct without
    /// literally constructing a multi-million-instruction artifact in a
    /// unit test.
    #[test]
    fn c1756_accepts_large_fully_reachable_linear_stream() {
        let mut instrs: Vec<IrInstr> = Vec::new();
        for _ in 0..300_000 {
            instrs.push(IrInstr::LoadBool { dst: 0, val: true });
        }
        instrs.push(IrInstr::Ret { src: Some(0) });
        let bytes = emit_test_function(instrs);
        verify_semcode(&bytes).expect(
            "a long, fully reachable, non-branching stream that repeatedly writes the same \
             already-defined register must verify correctly and stay fast",
        );
    }

    /// Codex review round 5 on PR #1840: reproduces the exact adversarial
    /// shape described - a wide dispatch where every arm defines all but
    /// one (a DIFFERENT one per arm) of a shared register domain, joining
    /// into a single node, followed by a long fallthrough tail. Under a
    /// plain FIFO edge-relaxation worklist, each of the `ARMS` arms
    /// narrows the join's `IN` by exactly one more bit (regardless of
    /// processing order, since it's a strict, growing set difference), and
    /// every such narrowing that changes the join's `OUT` re-propagates
    /// through the *entire* `TAIL_LEN`-node tail again -
    /// `O(ARMS * TAIL_LEN)` relaxations. Fixed by processing positions in
    /// `reverse_postorder_ranks` order (see that function's doc comment):
    /// every arm is ordered ahead of the join it feeds, so all `ARMS`
    /// contributions have already narrowed `in_sets[join]` in place by the
    /// time the join is first popped and actually processed - the tail is
    /// walked once, not once per arm.
    ///
    /// Correctness is exercised too, not just performance: since arm `i`
    /// defines every register in `0..ARMS` except register `i`, the
    /// intersection over all arms is the EMPTY set within that domain - no
    /// register in `0..ARMS` survives the join, so the read at the far end
    /// of the tail (`r0`) is correctly rejected as undefined, regardless of
    /// how long the intervening tail is.
    #[test]
    fn c1756_rejects_wide_dispatch_join_feeding_long_tail_without_per_arm_blowup() {
        const ARMS: u16 = 128;
        const TAIL_LEN: usize = 20_000;
        const COND_REG: u16 = ARMS; // outside the 0..ARMS join domain

        let mut instrs: Vec<IrInstr> = vec![IrInstr::LoadBool {
            dst: COND_REG,
            val: true,
        }];
        for arm in 0..ARMS - 1 {
            instrs.push(IrInstr::JmpIf {
                cond: COND_REG,
                label: format!("arm{arm}"),
            });
        }
        // Fallthrough arm (ARMS - 1): reached only once every JmpIf above
        // has fallen through, so its body must be the next physical
        // instruction after the dispatch chain - defines every register
        // except r(ARMS - 1).
        for r in 0..ARMS {
            if r != ARMS - 1 {
                instrs.push(IrInstr::LoadI32 {
                    dst: r,
                    val: r as i32,
                });
            }
        }
        instrs.push(IrInstr::Jmp {
            label: "join".to_string(),
        });
        // The remaining ARMS - 1 arms, each reached by its own JmpIf target
        // above - defines every register except its own index.
        for arm in 0..ARMS - 1 {
            instrs.push(IrInstr::Label {
                name: format!("arm{arm}"),
            });
            for r in 0..ARMS {
                if r != arm {
                    instrs.push(IrInstr::LoadI32 {
                        dst: r,
                        val: r as i32,
                    });
                }
            }
            instrs.push(IrInstr::Jmp {
                label: "join".to_string(),
            });
        }
        instrs.push(IrInstr::Label {
            name: "join".to_string(),
        });
        for _ in 0..TAIL_LEN {
            // Writes a register outside the 0..ARMS join domain - carries
            // the join's (already-converged) state forward without adding
            // new information, the same "cheap tail" shape as the other
            // large stress regressions above.
            instrs.push(IrInstr::LoadI32 {
                dst: COND_REG,
                val: 0,
            });
        }
        instrs.push(IrInstr::Ret { src: Some(0) }); // r0: undefined by arm 0, so missing at the join

        let bytes = emit_test_function(instrs);
        let report = verify_semcode(&bytes).expect_err(
            "r0 is missing from arm 0's definitions, so no register in 0..ARMS survives the \
             128-arm join, and the tail read of r0 must be rejected",
        );
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::UndefinedRegisterRead
        );
    }

    /// Codex review round 6 on PR #1840, second finding (a sibling to the
    /// frame-size one above): a loop header `H` fed by `ARMS` sibling
    /// nodes `S_0..S_{k-1}` via back edges, each defining every register
    /// in `0..ARMS` except a different one, `H` itself then feeding a
    /// long exit tail - `dispatch` reaches every `S_i` (each writes,
    /// jumps to `H`), and `H`'s own `JmpIf` back to `dispatch` closes the
    /// cycle (`dispatch` is `H`'s DFS ancestor - `H` is discovered only
    /// as some `S_i`'s descendant - so this is a genuine back edge; `H`
    /// and every `S_i` that participates in the cycle land in ONE SCC,
    /// mutually reachable through it). `H` has NO predecessor besides the
    /// arms' back edges, so it starts at TOP (the standard non-entry
    /// initialization) and needs a genuine, necessary contribution from
    /// ALL `ARMS` arms - not something a single predecessor already
    /// determines - before its fixed point is settled; `dispatch` itself
    /// starts empty (straight from entry, no SIG0 params), so each arm's
    /// own exclusion write is what determines its output, not something
    /// inherited from a prior pass.
    ///
    /// Correctness: since `S_i` defines every register in `0..ARMS`
    /// except `i`, the intersection over all `ARMS` arms at `H` is the
    /// EMPTY set within that domain (the same argument as the round-5
    /// test) - the read at the far end of the tail (`r0`) is correctly
    /// rejected as undefined, regardless of the intervening loop or tail
    /// length. This is the primary purpose of this regression: proving
    /// the SCC-based worklist converges a genuine, multi-source cycle to
    /// the mathematically correct fixed point. It is not, on its own,
    /// strong TIMING evidence against round 5's pure-RPO ordering: in
    /// this specific instruction layout, `dispatch` (and therefore every
    /// arm) is discovered by DFS before `H` even exists, so plain RPO
    /// rank already happens to order every arm ahead of `H` here too -
    /// the round-6 reply documents this honestly, together with the
    /// general, well-established argument for why SCC-restricted
    /// processing bounds total work for the broader class of CFGs RPO
    /// alone cannot.
    #[test]
    fn c1756_rejects_loop_header_fed_by_many_backedge_arms_without_per_arm_blowup() {
        const ARMS: u16 = 128;
        const TAIL_LEN: usize = 20_000;
        const COND_REG: u16 = ARMS;

        // `dispatch`'s own IN comes straight from entry (empty - no SIG0
        // params), so it - and every arm reached only through it - starts
        // genuinely empty, not full: each arm's "define everything except
        // my own index" write is what actually determines its output,
        // not something inherited from a prior loop pass. `header` has NO
        // direct edge from entry or dispatch - its only predecessors are
        // the arms' back edges - so it starts at TOP (the standard
        // non-entry initialization) and is narrowed ONE MEANINGFUL STEP
        // PER ARM as their outputs arrive, needing all `ARMS` of them
        // before its fixed point is fully determined - a genuine,
        // necessary multi-round convergence, not a value some single
        // predecessor already fixes on its own.
        let mut instrs: Vec<IrInstr> = vec![IrInstr::LoadBool {
            dst: COND_REG,
            val: true,
        }];
        instrs.push(IrInstr::Label {
            name: "dispatch".to_string(),
        });
        for arm in 0..ARMS - 1 {
            instrs.push(IrInstr::JmpIf {
                cond: COND_REG,
                label: format!("arm{arm}"),
            });
        }
        // Fallthrough arm (ARMS - 1): defines every register except
        // itself, then jumps to the header - closing the loop, since
        // header's own back edge (below) returns here.
        for r in 0..ARMS {
            if r != ARMS - 1 {
                instrs.push(IrInstr::LoadI32 {
                    dst: r,
                    val: r as i32,
                });
            }
        }
        instrs.push(IrInstr::Jmp {
            label: "header".to_string(),
        });
        // The remaining ARMS - 1 arms, each reached by its own JmpIf target
        // above - defines every register except its own index, then jumps
        // to the header too.
        for arm in 0..ARMS - 1 {
            instrs.push(IrInstr::Label {
                name: format!("arm{arm}"),
            });
            for r in 0..ARMS {
                if r != arm {
                    instrs.push(IrInstr::LoadI32 {
                        dst: r,
                        val: r as i32,
                    });
                }
            }
            instrs.push(IrInstr::Jmp {
                label: "header".to_string(),
            });
        }
        instrs.push(IrInstr::Label {
            name: "header".to_string(),
        });
        // The back edge: `dispatch` is `header`'s DFS ancestor here (every
        // arm was discovered as dispatch's descendant, and each leads to
        // header), so this is a genuine back edge, not a forward join -
        // `header` and every arm form one strongly-connected component.
        instrs.push(IrInstr::JmpIf {
            cond: COND_REG,
            label: "dispatch".to_string(),
        });
        // Exit tail: header's fallthrough - reached only once the loop
        // actually exits, with header's fully-converged fixed point.
        for _ in 0..TAIL_LEN {
            instrs.push(IrInstr::LoadI32 {
                dst: COND_REG,
                val: 0,
            });
        }
        instrs.push(IrInstr::Ret { src: Some(0) }); // r0: undefined by arm 0

        let bytes = emit_test_function(instrs);
        let report = verify_semcode(&bytes).expect_err(
            "r0 is missing from arm 0's definitions, so no register in 0..ARMS survives the \
             header's intersection over all back-edge arms, and the tail read of r0 must be \
             rejected",
        );
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::UndefinedRegisterRead
        );
    }

    /// Codex review round 7 on PR #1840: reproduces the "bound convergence
    /// work inside each SCC" shape - one big cycle `c_0 -> c_1 -> ... ->
    /// c_{CHAIN_LEN-1} -> back to c_0`, laid out in REVERSE physical order
    /// (`c_{CHAIN_LEN-1}`'s bytes come first, `c_0`'s come last - the exact
    /// technique `c1756_backward_propagating_chain_still_rejects_and_
    /// converges_promptly` already uses for the whole-graph case), where
    /// the first `ARMS` cycle nodes (`c_0..c_{ARMS-1}`) each also have
    /// their own external arm `arm_i` (reached from a shared dispatch off
    /// entry) contributing "every register except r_i" within the `0..
    /// ARMS` domain - `arm_i`'s own write cost is bounded by `ARMS`, not
    /// `CHAIN_LEN`, so the ring can be made long independently of how many
    /// distinct exclusions feed it. `c_i`'s true fixed point (for `i <
    /// ARMS`) is the intersection of `arm_0..arm_i`'s contributions -
    /// genuinely cumulative along the ring, not something any single arm
    /// already determines. A plain FIFO seeded in ascending-position (=
    /// descending logical-index, per the reversed layout) order only
    /// advances newly-arriving information one hop per pass against that
    /// mismatch - round 5's exact ordering hazard, reproduced entirely
    /// INSIDE one SCC where round 6's cross-SCC isolation cannot help.
    /// Fixed by `local_reverse_postorder_ranks`: the same RPO-ordering
    /// technique round 5 applied to the whole graph, now applied to each
    /// SCC's own internal convergence.
    ///
    /// Correctness: the converged fixed point at the far end of the ring
    /// is the intersection of every arm's contribution - the empty set
    /// within `0..ARMS`, since `arm_i` excludes `r_i` for every `i` - so
    /// the tail read of `r0` is correctly rejected regardless of ring
    /// length or physical layout.
    #[test]
    fn c1756_rejects_reverse_physical_cycle_with_many_external_arms_without_per_arm_blowup() {
        const ARMS: u16 = 64;
        const CHAIN_LEN: u32 = 40_000;
        const COND_REG: u16 = ARMS;
        const TAIL_LEN: usize = 20_000;

        let mut instrs: Vec<IrInstr> = vec![IrInstr::LoadBool {
            dst: COND_REG,
            val: true,
        }];
        instrs.push(IrInstr::Label {
            name: "dispatch".to_string(),
        });
        for arm in 0..ARMS - 1 {
            instrs.push(IrInstr::JmpIf {
                cond: COND_REG,
                label: format!("arm{arm}"),
            });
        }
        // Fallthrough arm (ARMS - 1): defines every register in 0..ARMS
        // except itself, then jumps into the ring at c_{ARMS - 1}.
        for r in 0..ARMS {
            if r != ARMS - 1 {
                instrs.push(IrInstr::LoadI32 {
                    dst: r,
                    val: r as i32,
                });
            }
        }
        instrs.push(IrInstr::Jmp {
            label: format!("c{}", ARMS - 1),
        });
        // The remaining arms, each reached by its own JmpIf target above -
        // defines every register in 0..ARMS except its own index, then
        // jumps into the ring at the matching c_i.
        for arm in 0..ARMS - 1 {
            instrs.push(IrInstr::Label {
                name: format!("arm{arm}"),
            });
            for r in 0..ARMS {
                if r != arm {
                    instrs.push(IrInstr::LoadI32 {
                        dst: r,
                        val: r as i32,
                    });
                }
            }
            instrs.push(IrInstr::Jmp {
                label: format!("c{arm}"),
            });
        }
        // c_{CHAIN_LEN - 1}: the ring's last logical node - back edge to
        // c_0 (closing the loop) or fallthrough to the exit tail. Emitted
        // FIRST physically (lowest position), even though it's LAST in
        // logical/control-flow order.
        instrs.push(IrInstr::Label {
            name: format!("c{}", CHAIN_LEN - 1),
        });
        instrs.push(IrInstr::JmpIf {
            cond: COND_REG,
            label: "c0".to_string(),
        });
        for _ in 0..TAIL_LEN {
            instrs.push(IrInstr::LoadI32 {
                dst: COND_REG,
                val: 0,
            });
        }
        instrs.push(IrInstr::Ret { src: Some(0) }); // r0: undefined by arm 0
                                                    // c_{CHAIN_LEN - 2} down to c_0: each jumps to its logical
                                                    // successor - emitted in DESCENDING index order, so ascending
                                                    // physical position is the exact reverse of logical/control-flow
                                                    // order (c_{CHAIN_LEN - 1} lowest position .. c_0 highest).
                                                    // Positions ARMS..CHAIN_LEN-1 have no external arm at all - pure
                                                    // pass-through, carrying the already-converging state forward
                                                    // without adding new information - so the ring's own length can
                                                    // grow far beyond ARMS at negligible construction cost.
        for i in (0..CHAIN_LEN - 1).rev() {
            instrs.push(IrInstr::Label {
                name: format!("c{i}"),
            });
            instrs.push(IrInstr::Jmp {
                label: format!("c{}", i + 1),
            });
        }

        let bytes = emit_test_function(instrs);
        let report = verify_semcode(&bytes).expect_err(
            "each arm_i excludes r_i, so the intersection over all ARMS arms at the far end \
             of the ring is empty within 0..ARMS, and the tail read of r0 must be rejected",
        );
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::UndefinedRegisterRead
        );
    }

    /// Codex review round 8 on PR #1840, first finding ("duplicate live
    /// RegSet states across branches"): many branch arms each performing
    /// the IDENTICAL state-changing write would, under every pre-round-8
    /// scheme (`Rc`-sharing included - `Rc`-sharing only ever avoided
    /// cloning along a *shared* predecessor path, it never interned
    /// independently-computed-but-equal values from *different* branches),
    /// each independently allocate an equal, duplicate `RegSet`. Under the
    /// round-8 MAY-missing formulation this class of duplication is
    /// structurally impossible: `compute_missing_sets` returns exactly one
    /// `RegSet` per reachable position - `missing.len() == reachable_count`
    /// always, by construction of the `Vec` it returns - mutated in place
    /// via `deliver`, with no clone-on-write path for any number of
    /// branches to trigger. This test uses `compute_missing_sets` directly
    /// (bypassing full SemCode construction, per Codex's own request for
    /// accounting evidence over timing) at Codex's stated scale
    /// (`|U|` = 4096) with 2,000 arms, all writing the identical register -
    /// direct accounting evidence that `event_count` scales with `arms +
    /// domain_size`, never with `arms * domain_size` or any product
    /// involving how many arms compute the same result.
    #[test]
    fn c1756_many_identical_branch_arms_allocate_exactly_one_regset_per_position() {
        const DOMAIN_SIZE: usize = 4096;
        const ARMS: usize = 2_000;
        // Positions: 0 = entry; 1..=ARMS = a dispatch chain (position i's
        // successors are [arm i's body, dispatch chain continues]); ARMS+1
        // ..=2*ARMS = arm bodies (every one writes bit 0, and ONLY bit 0);
        // 2*ARMS+1 = join (reached by every arm).
        let dispatch_start = 1usize;
        let arm_start = ARMS + 1;
        let join = 2 * ARMS + 1;
        let reachable_count = join + 1;

        let successors_of = |pos: usize| -> [Option<usize>; 2] {
            if pos == 0 {
                [Some(dispatch_start), None]
            } else if (dispatch_start..arm_start).contains(&pos) {
                let i = pos - dispatch_start;
                let next_dispatch = if i + 1 < ARMS {
                    Some(dispatch_start + i + 1)
                } else {
                    None
                };
                [Some(arm_start + i), next_dispatch]
            } else if (arm_start..join).contains(&pos) {
                [Some(join), None]
            } else {
                [None, None]
            }
        };
        // Every arm body writes bit 0 - the identical write, in every arm.
        let writes_contains = |pos: usize, bit: usize| (arm_start..join).contains(&pos) && bit == 0;

        let entry_missing = RegSet::full(DOMAIN_SIZE); // no SIG0 params: nothing defined at entry
        let (missing, event_count) = compute_missing_sets(
            reachable_count,
            DOMAIN_SIZE,
            entry_missing,
            writes_contains,
            successors_of,
        );
        assert_eq!(
            missing.len(),
            reachable_count,
            "exactly one RegSet per reachable position, always - the structural property that \
             makes duplicate live states impossible regardless of arm count"
        );
        assert!(
            !missing[join].contains(0),
            "bit 0 is written by every one of the 2,000 arms, so it must be defined (not \
             missing) at the join"
        );
        assert!(
            missing[join].contains(1),
            "bit 1 is written by no arm, so it must still be missing at the join"
        );
        assert!(
            event_count <= reachable_count * DOMAIN_SIZE,
            "event_count ({event_count}) must be bounded by reachable_count * domain_size \
             ({}), never multiplied further by how many arms compute the identical result",
            reachable_count * DOMAIN_SIZE
        );
    }

    /// Codex review round 8 on PR #1840, second finding ("local RPO still
    /// reprocesses large SCC via backedges"): reproduces the same
    /// adversarial shape as `c1756_rejects_reverse_physical_cycle_with_
    /// many_external_arms_without_per_arm_blowup` - a ring laid out in
    /// reverse propagation order, fed at many distinct points by external
    /// arms each contributing a different missing register - directly via
    /// `compute_missing_sets`, asserting `event_count` explicitly (Codex's
    /// own request for a counter, not timing alone). Under the round-7
    /// local-RPO worklist this shape could still force repeated whole-
    /// component reprocessing (each backedge source re-queueing the header
    /// before the others had all contributed); under the round-8 bit-level
    /// worklist there is no "large SCC" concept left to reprocess, so the
    /// event count stays bounded by `reachable_count * domain_size`
    /// regardless of how many external arms feed the cycle or how it is
    /// physically laid out.
    #[test]
    fn c1756_many_external_arms_feeding_one_cycle_via_backedges_bounds_event_count() {
        const ARMS: usize = 200;
        const RING_LEN: usize = 4_000;
        const DOMAIN_SIZE: usize = ARMS;
        // Positions: 0 = entry; 1..=ARMS = a dispatch chain (no writes -
        // dispatch[i]'s successors are [arm i's body, dispatch[i+1]]);
        // ARMS+1..=2*ARMS = arm bodies (arm i writes every register except
        // i, then enters the ring at c_0); 2*ARMS+1..2*ARMS+RING_LEN = the
        // ring c_0..c_{RING_LEN-1}, laid out so position 2*ARMS+1+k
        // corresponds to LOGICAL ring index (RING_LEN-1-k) - reverse
        // propagation order, matching the physically-reversed byte-level
        // test - closing the loop from c_{RING_LEN-1} back to c_0.
        let dispatch_start = 1usize;
        let arm_start = ARMS + 1;
        let ring_start = 2 * ARMS + 1;
        let ring_pos = |logical_i: usize| -> usize { ring_start + (RING_LEN - 1 - logical_i) };
        let reachable_count = ring_start + RING_LEN;

        let successors_of = |pos: usize| -> [Option<usize>; 2] {
            if pos == 0 {
                [Some(dispatch_start), None]
            } else if (dispatch_start..arm_start).contains(&pos) {
                let i = pos - dispatch_start;
                let next_dispatch = if i + 1 < ARMS {
                    Some(dispatch_start + i + 1)
                } else {
                    None
                };
                [Some(arm_start + i), next_dispatch]
            } else if (arm_start..ring_start).contains(&pos) {
                [Some(ring_pos(0)), None]
            } else {
                // A ring position at physical `pos` corresponds to logical
                // index `logical_i` - find it by inverting `ring_pos`.
                let logical_i = RING_LEN - 1 - (pos - ring_start);
                if logical_i + 1 < RING_LEN {
                    [Some(ring_pos(logical_i + 1)), None]
                } else {
                    [Some(ring_pos(0)), None] // c_{RING_LEN-1}: back edge to c_0
                }
            }
        };
        // Arm i writes every register in 0..ARMS except i - reached by
        // every arm body position specifically (not the dispatch chain or
        // the ring, neither of which write anything).
        let writes_contains = |pos: usize, bit: usize| {
            (arm_start..ring_start).contains(&pos) && bit != (pos - arm_start)
        };

        let entry_missing = RegSet::full(DOMAIN_SIZE);
        let (missing, event_count) = compute_missing_sets(
            reachable_count,
            DOMAIN_SIZE,
            entry_missing,
            writes_contains,
            successors_of,
        );
        // Every arm excludes a different register (arm i excludes bit i),
        // so the intersection over all ARMS arms - reflected at every ring
        // position once converged - is empty.
        for bit in 0..ARMS {
            assert!(
                missing[ring_pos(0)].contains(bit),
                "bit {bit} is excluded by arm {bit} specifically, so no register in 0..ARMS \
                 survives the ring's converged fixed point"
            );
        }
        assert!(
            event_count <= reachable_count * DOMAIN_SIZE,
            "event_count ({event_count}) must be bounded by reachable_count * domain_size \
             ({}), regardless of how many external arms feed the cycle via backedges",
            reachable_count * DOMAIN_SIZE
        );
    }

    /// Codex review round 1 on PR #1840, second fix: the register domain
    /// must be the actual registers in use (`U`), densely packed, not the
    /// raw `0..=max_register_id` numeric span. This function references
    /// only `r0` and `r4095` - two registers, nowhere near each other
    /// numerically - so `U` must have exactly 2 entries, not 4096, while
    /// still proving both cases correctly: `r0` (written, then read) is
    /// accepted; `r4095` (read without being written) is rejected. A bug in
    /// the dense-index remapping (an off-by-one, an unsorted `universe`, a
    /// stale raw-index lookup) would most likely surface as exactly this
    /// kind of correctness failure on sparse, wide-spanning register use.
    #[test]
    fn c1756_dense_domain_handles_sparse_wide_spanning_registers() {
        let accepted = emit_test_function(vec![
            IrInstr::LoadI32 { dst: 0, val: 1 },
            IrInstr::Ret { src: Some(0) },
        ]);
        verify_semcode(&accepted).expect("r0 is written before it is read");

        let rejected = emit_test_function(vec![IrInstr::Ret { src: Some(4095) }]);
        let report = verify_semcode(&rejected)
            .expect_err("r4095 is read but never written or entry-defined");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::UndefinedRegisterRead
        );
    }

    /// Codex review round 2 on PR #1840, finding 1: making the padding
    /// reachable still amplified memory under round 1's fix, because
    /// `domain_size` scales with *actual* register usage, and a program can
    /// genuinely, reachably use every register in the budget. This
    /// reproduces exactly that shape - define every register `r0..r4095`
    /// once, then continue through a long fallthrough chain of writes to
    /// registers already defined (the realistic form of "cheap writes" that
    /// doesn't grow `U` further) - and proves it still verifies correctly.
    /// Fixed by `Rc`-sharing (see `apply_writes`): every node in the long
    /// tail writes only already-defined bits, so it shares its
    /// predecessor's `Rc<RegSet>` instead of allocating its own.
    #[test]
    fn c1756_reachable_full_domain_then_long_redefine_chain_stays_correct() {
        let mut instrs = Vec::new();
        for r in 0..4096u16 {
            instrs.push(IrInstr::LoadI32 { dst: r, val: 0 });
        }
        for _ in 0..4000 {
            // Re-writes r0, already defined above - grows no new
            // information, the exact "cheap write" tail the finding
            // describes.
            instrs.push(IrInstr::LoadI32 { dst: 0, val: 1 });
        }
        instrs.push(IrInstr::Ret { src: Some(4095) });
        let bytes = emit_test_function(instrs);
        verify_semcode(&bytes).expect(
            "every register 0..4095 is genuinely defined before the final read, regardless of \
             how long the redefine tail is",
        );
    }

    /// Codex review round 2 on PR #1840, finding 2: the round-1 fixed-point
    /// loop rescanned reachable positions in ascending (offset) order every
    /// round, so information propagating *against* that order (as it does
    /// here: entry jumps to the LAST instruction, and each subsequent
    /// instruction jumps one step backward toward an early, undefined
    /// `RET`) only advanced one node per round - `O(reachable_count)`
    /// rounds of `O(reachable_count)` work each. Fixed by a genuine
    /// predecessor/successor worklist: each node's `IN` is computed
    /// directly from its actual predecessor(s)' `OUT`, propagated forward
    /// along real edges regardless of node-index order, so this chain
    /// converges in a single pass over its edges, not a quadratic scan.
    #[test]
    fn c1756_backward_propagating_chain_still_rejects_and_converges_promptly() {
        const CHAIN_LEN: usize = 3000;
        let mut instrs = vec![IrInstr::Jmp {
            label: format!("l{CHAIN_LEN}"),
        }];
        // "l1" resolves to the very next real instruction, this RET.
        instrs.push(IrInstr::Label {
            name: "l1".to_string(),
        });
        instrs.push(IrInstr::Ret { src: Some(0) }); // never defined
        for i in 2..=CHAIN_LEN {
            instrs.push(IrInstr::Label {
                name: format!("l{i}"),
            });
            instrs.push(IrInstr::Jmp {
                label: format!("l{}", i - 1),
            });
        }
        // Physical/offset order: entry, l1(RET), l2(Jmp l1), l3(Jmp l2), ...,
        // l{CHAIN_LEN}(Jmp l{CHAIN_LEN-1}). Execution/propagation order is
        // the reverse: entry -> l{CHAIN_LEN} -> l{CHAIN_LEN-1} -> ... -> l2
        // -> l1(RET) - directly opposite the ascending-offset scan order
        // the pre-fix round-robin loop used.
        let bytes = emit_test_function(instrs);
        let report = verify_semcode(&bytes).expect_err(
            "r0 is read by the RET at the far end of the backward chain and is never defined",
        );
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::UndefinedRegisterRead
        );
    }

    /// Direct accounting evidence for the Codex-review round-1, round-3, and
    /// round-4 fixes, per the review's own request not to rely only on
    /// timing. `dataflow_domain_accounting` takes plain, hand-constructible
    /// inputs - this asserts its outputs' exact lengths (the quantities
    /// that bound `prove_definite_register_assignment`'s memory) directly,
    /// without needing to run a full decode walk. The `reads_writes_of`
    /// closure is backed by a small map covering ONLY the reachable indices
    /// actually queried - proving directly that unreachable indices are
    /// never even looked up, let alone materialized as a dense per-
    /// instruction entry (round 3's fix). The offsets arrays' lengths
    /// (`reachable_count + 1`, a `u32` each - not one `Vec<u16>` per node)
    /// are round 4's fix: reads/writes are now one flat allocation each,
    /// not `reachable_count` separate heap-container allocations.
    #[test]
    fn c1756_accounting_bounds_reachable_nodes_and_dense_universe_exactly() {
        // 501 structurally decoded nodes (an entry RET plus 500 unreachable
        // instructions), but only node 0 is reachable, and nothing is ever
        // read or written - the exact "RET followed by unreachable bloat"
        // shape Codex's finding described.
        let instr_starts: Vec<usize> = (0..501).collect();
        let mut reachable_offsets = HashSet::new();
        reachable_offsets.insert(0);
        let reads_writes: std::collections::HashMap<usize, (Vec<u16>, Vec<u16>)> =
            std::collections::HashMap::from([(0, (Vec::new(), Vec::new()))]);
        let (reachable_indices, reads_flat, reads_offsets, writes_flat, writes_offsets, universe) =
            dataflow_domain_accounting(&instr_starts, &reachable_offsets, 0, |idx| {
                reads_writes
                    .get(&idx)
                    .cloned()
                    .expect("must only be queried for the one reachable index")
            });
        assert_eq!(
            reachable_indices.len(),
            1,
            "500 unreachable nodes must not be allocated dataflow state"
        );
        assert_eq!(reads_flat.len(), 0, "nothing is ever read");
        assert_eq!(writes_flat.len(), 0, "nothing is ever written");
        assert_eq!(
            reads_offsets.len(),
            2,
            "one flat offset table sized reachable_count+1, not one Vec per node"
        );
        assert_eq!(writes_offsets.len(), 2);
        assert_eq!(
            universe.len(),
            0,
            "no register is ever read, written, or entry-defined"
        );

        // Two reachable nodes, referencing only r0 and r4095 - numerically
        // 4095 apart, but only 2 distinct registers actually in use.
        let instr_starts: Vec<usize> = vec![0, 1];
        let mut reachable_offsets = HashSet::new();
        reachable_offsets.insert(0);
        reachable_offsets.insert(1);
        let reads_writes: std::collections::HashMap<usize, (Vec<u16>, Vec<u16>)> =
            std::collections::HashMap::from([
                (0, (Vec::new(), vec![0])),
                (1, (vec![4095], Vec::new())),
            ]);
        let (reachable_indices, reads_flat, reads_offsets, writes_flat, writes_offsets, universe) =
            dataflow_domain_accounting(&instr_starts, &reachable_offsets, 0, |idx| {
                reads_writes.get(&idx).cloned().expect("both are reachable")
            });
        assert_eq!(reachable_indices.len(), 2);
        assert_eq!(reads_flat, vec![4095], "only node 1's one read");
        assert_eq!(writes_flat, vec![0], "only node 0's one write");
        assert_eq!(reads_offsets, vec![0, 0, 1]);
        assert_eq!(writes_offsets, vec![0, 1, 1]);
        assert_eq!(csr_slice(&reads_flat, &reads_offsets, 0), &[] as &[u16]);
        assert_eq!(csr_slice(&reads_flat, &reads_offsets, 1), &[4095]);
        assert_eq!(csr_slice(&writes_flat, &writes_offsets, 0), &[0]);
        assert_eq!(csr_slice(&writes_flat, &writes_offsets, 1), &[] as &[u16]);
        assert_eq!(
            universe.len(),
            2,
            "the dense register domain must track the 2 registers actually referenced \
             ({{r0, r4095}}), not the raw 4096-wide numeric span between them: {universe:?}"
        );
    }

    #[test]
    fn c1756_case25_reports_deterministic_first_diagnostic_among_multiple_undefined_reads() {
        let bytes = emit_test_function(vec![
            IrInstr::AddI32 {
                dst: 2,
                lhs: 10,
                rhs: 11,
            }, // r10, r11 both undefined
            IrInstr::AddI32 {
                dst: 3,
                lhs: 20,
                rhs: 21,
            }, // r20, r21 also undefined
            IrInstr::Ret { src: Some(3) },
        ]);
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::UndefinedRegisterRead
        );
        assert_eq!(report.diagnostics[0].offset, Some(0));
        assert!(
            report.diagnostics[0].message.contains("r10"),
            "the first reported undefined register must be the lowest-offset, first-decoded \
             operand (r10, not r11/r20/r21): {}",
            report.diagnostics[0].message
        );
    }

    #[test]
    fn c1756_case26_repeated_verification_yields_identical_diagnostic() {
        let bytes = emit_test_function(vec![
            IrInstr::AddI32 {
                dst: 2,
                lhs: 10,
                rhs: 11,
            },
            IrInstr::Ret { src: Some(2) },
        ]);
        let first = verify_semcode(&bytes).expect_err("must reject");
        for _ in 0..9 {
            let again = verify_semcode(&bytes).expect_err("must reject");
            assert_eq!(
                again, first,
                "repeated verification of identical bytes must produce an identical diagnostic"
            );
        }
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
                params: Vec::new(),
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
        let bytes = compile_program_to_semcode(src).expect("compile");
        let bytes = downgrade_header_stripping_signature(&bytes, MAGIC0);
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::CapabilityViolation
        );
    }

    #[test]
    fn verifier_rejects_missing_ownership_section_under_v11_capabilities() {
        // #1773 (FA-09-005): hand-built directly under MAGIC11 with no OWN0
        // section at all - `compile_program_to_semcode`'s output now always
        // includes a real (if empty) OWN0 section (mandatory once its own
        // chosen header reaches V11's floor), so stripping only the SIG0
        // span (as `downgrade_header_stripping_signature` does for the
        // capability-downgrade tests above) would leave OWN0 genuinely
        // present here - the opposite of what this test needs to prove.
        let new_code = [Opcode::Ret as u8, 0];
        let bytes = minimal_semcode_bytes_with_header(MAGIC11, "main", &new_code);
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
        let bytes = record_field_borrow_semcode_bytes();
        let bytes = downgrade_header_stripping_signature(&bytes, MAGIC11);
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::CapabilityViolation
        );
    }

    #[test]
    fn verifier_rejects_ownership_section_under_v10_capabilities() {
        let bytes = ownership_semcode_bytes();
        let bytes = downgrade_header_stripping_signature(&bytes, MAGIC10);
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::CapabilityViolation
        );
    }

    #[test]
    fn verifier_rejects_state_query_under_v3_capabilities() {
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
                params: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        let bytes = downgrade_header_stripping_signature(&bytes, MAGIC3);
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::CapabilityViolation
        );
    }

    #[test]
    fn verifier_rejects_state_update_under_v4_capabilities() {
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
                params: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        let bytes = downgrade_header_stripping_signature(&bytes, MAGIC4);
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::CapabilityViolation
        );
    }

    #[test]
    fn verifier_rejects_event_post_under_v5_capabilities() {
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
                params: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        let bytes = downgrade_header_stripping_signature(&bytes, MAGIC5);
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::CapabilityViolation
        );
    }

    #[test]
    fn verifier_rejects_clock_read_under_v6_capabilities() {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "main".to_string(),
                instrs: vec![IrInstr::ClockRead { dst: 0 }, IrInstr::Ret { src: None }],
                ownership_events: Vec::new(),
                params: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        let bytes = downgrade_header_stripping_signature(&bytes, MAGIC6);
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
        let bytes = compile_program_to_semcode(src).expect("compile");
        let bytes = downgrade_header_stripping_signature(&bytes, MAGIC7);
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

    /// #1773 (FA-09-005): rebuilds `bytes` under `target_magic`, stripping
    /// each function's `SIG0` section along the way. Every artifact produced
    /// by `compile_program_to_semcode`/`emit_ir_to_semcode` now
    /// unconditionally carries a `SIG0` section (the new floor is
    /// `SEMCODE_SIGNATURE_MIN_REVISION`), so simply overwriting the header
    /// magic bytes to simulate an older revision - the technique several
    /// pre-#1773 tests in this module used - leaves a structurally
    /// inconsistent artifact: a header claiming a revision that structurally
    /// cannot carry `SIG0`, over a body that still does. This reconstructs
    /// what the body would genuinely have looked like had it been compiled
    /// under an older header, matching real pre-#1773 artifact shape.
    fn downgrade_header_stripping_signature(bytes: &[u8], target_magic: [u8; 8]) -> Vec<u8> {
        let (_, functions) = sm_format::semcode_decode::decode_semcode_envelope(bytes)
            .expect("decode current bytes");
        let mut out = Vec::new();
        out.extend_from_slice(&target_magic);
        for f in &functions {
            let sig0_len = f
                .signature
                .as_ref()
                .map(|sig| SIGNATURE_SECTION_TAG.len() + 2 + sig.families.len())
                .unwrap_or(0);
            let sig0_start = f.instr_start_offset - sig0_len;
            let mut code = Vec::new();
            code.extend_from_slice(&f.code_slice[..sig0_start]);
            code.extend_from_slice(&f.code_slice[f.instr_start_offset..]);
            let name_bytes = f.name.as_bytes();
            out.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
            out.extend_from_slice(name_bytes);
            out.extend_from_slice(&(code.len() as u32).to_le_bytes());
            out.extend_from_slice(&code);
        }
        out
    }

    /// #1773 (FA-09-005): a minimal, fully hand-built artifact - explicit
    /// header magic, empty string table, caller-supplied raw instruction
    /// bytes - with no OWN0/SIG0 section at all. Several pre-#1773 tests
    /// patched `compile_program_to_semcode`'s output at a hand-counted byte
    /// offset assuming the instruction stream began immediately after a
    /// bare empty string table; that assumption broke once every compiled
    /// artifact unconditionally gained OWN0+SIG0 sections. This sidesteps
    /// the compiler entirely for tests that need full, explicit control
    /// over byte layout under an arbitrary (possibly pre-#1732) header.
    fn minimal_semcode_bytes_with_header(magic: [u8; 8], name: &str, instrs: &[u8]) -> Vec<u8> {
        let mut code = Vec::new();
        code.extend_from_slice(&0u16.to_le_bytes()); // empty string table
        code.extend_from_slice(instrs);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&magic);
        bytes.extend_from_slice(&(name.len() as u16).to_le_bytes());
        bytes.extend_from_slice(name.as_bytes());
        bytes.extend_from_slice(&(code.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&code);
        bytes
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
                params: Vec::new(),
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
                params: Vec::new(),
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
                params: Vec::new(),
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
                params: Vec::new(),
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
        // #1773 (FA-09-005): hand-built directly under MAGIC0 rather than
        // patching `compile_program_to_semcode`'s output - every compiled
        // artifact now unconditionally carries OWN0+SIG0 sections, so a
        // hand-counted byte offset assuming the instruction stream starts
        // right after a bare string table no longer lands on real opcode
        // bytes. See `minimal_semcode_bytes_with_header`'s doc comment.
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

        let bytes = minimal_semcode_bytes_with_header(MAGIC0, "main", &new_code);

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
        // #1773 (FA-09-005): hand-built directly under MAGIC18 - see the
        // comment on `verifier_rejects_qtruth_opcodes_under_baseline_header`.

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

        let bytes = minimal_semcode_bytes_with_header(MAGIC18, "main", &new_code);

        let verified = verify_semcode(&bytes).expect("QTruth under SEMCOD18 must verify");
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_rejects_truncated_qtruth_opcodes() {
        // #1773 (FA-09-005): hand-built directly under MAGIC18 - see the
        // comment on `verifier_rejects_qtruth_opcodes_under_baseline_header`.
        // Use SEMCOD18 (QTruth's actual minimum header) so this test
        // isolates truncation handling from the #1732 header-revision gate
        // - both are legitimate rejections, but this test is specifically
        // about truncated operand bytes, not about the header revision.
        // SEMCOD18 carries CAP_OWNERSHIP_PATHS, which requires a present
        // (possibly empty) OWN0 section.
        let mut new_code = Vec::new();
        new_code.extend_from_slice(b"OWN0");
        new_code.extend_from_slice(&0u16.to_le_bytes());
        new_code.push(0x17); // QTruthAnd (requires 4 operand bytes, zero left)
        let bytes = minimal_semcode_bytes_with_header(MAGIC18, "main", &new_code);

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
        // #1773 (FA-09-005): hand-built directly under MAGIC0 - see the
        // comment on `verifier_rejects_qtruth_opcodes_under_baseline_header`.
        let new_code = [0x17u8]; // QTruthAnd (requires 4 operand bytes, zero left)
        let bytes = minimal_semcode_bytes_with_header(MAGIC0, "main", &new_code);
        assert_eq!(&bytes[0..8], b"SEMCODE0");

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
                params: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        // #1773 (FA-09-005): SEMCOD19/rev20 is now the floor for every
        // emitted artifact regardless of which opcodes it uses (was
        // SEMCODE0/rev1, this program's own promotion floor) - see the
        // analogous sm-ir comment.
        assert_eq!(&bytes[0..8], b"SEMCOD19");
        let verified = verify_semcode(&bytes).expect("baseline logic opcodes must verify");
        assert_eq!(verified.header.rev, 20);
    }

    // #1732 regression matrix (6): a program mixing QTruth with an
    // unrelated capability-gated feature (here, f64 math, which alone
    // would only require SEMCODE1) must select the MAXIMUM required
    // header revision - originally SEMCOD18/rev19, now SEMCOD19/rev20
    // since #1773 (FA-09-005) raised the floor for every emitted artifact
    // regardless of opcodes used - not whichever feature's if/else-if
    // branch happens to match first.
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
                params: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        assert_eq!(&bytes[0..8], b"SEMCOD19");
        let verified = verify_semcode(&bytes).expect("mixed QTruth+f64 program must verify");
        assert_eq!(verified.header.rev, 20);
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
                    // #1756 (FA-07-016): r0 must be definitely defined before
                    // `QTruthAnd` reads it as both `lhs` and `rhs` - this
                    // fixture only exercises header-revision promotion for
                    // the opcode, not runtime QTruth semantics, so a plain
                    // `LoadI32` satisfies definedness without needing this
                    // module to import `QuadVal` for a real `LoadQ`.
                    IrInstr::LoadI32 { dst: 0, val: 0 },
                    IrInstr::QTruthAnd {
                        dst: 0,
                        lhs: 0,
                        rhs: 0,
                    },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: Vec::new(),
                params: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        // #1773 (FA-09-005): SEMCOD19/rev20 is now the floor (was SEMCOD18/
        // rev19, QTruth's own promotion floor) - see the comment on
        // `emitter_selects_maximum_required_revision_for_mixed_program`.
        assert_eq!(&bytes[0..8], b"SEMCOD19");
        let verified = verify_semcode(&bytes).expect("emitted QTruth program must verify");
        assert_eq!(verified.header.rev, 20);
    }

    // #1751 (FA-07-011): `pending_functions.len()` is a static count of
    // function definitions in the whole program; `quotas.max_frames` is a
    // dynamic runtime call-stack-depth budget enforced independently (and
    // correctly) at execution time in sm-vm's `push_frame`. Comparing the
    // two conflated "how many functions exist" with "how deep can the call
    // stack get". sm-format already owns the real static function-count
    // bound (`MAX_FUNCTIONS = 1024`, enforced at decode time), so the
    // verifier must not duplicate or misuse `max_frames` for this purpose.
    #[test]
    fn verify_semcode_token_accepts_many_function_definitions_regardless_of_frame_quota() {
        // Control: previously accepted (256 total functions, at the old
        // static bound) and must remain accepted.
        let bytes_256 = compile_many_functions(255);
        verify_semcode_token(&bytes_256).expect("256 functions must be accepted");

        // #1751 repro: previously rejected with ResourceLimitExceeded citing
        // "verified-local frame budget of 256" purely because the program
        // defines more functions than the runtime frame quota - a static
        // function count has nothing to do with live call-stack depth, so
        // this must now be accepted.
        let bytes_258 = compile_many_functions(257);
        verify_semcode_token(&bytes_258)
            .expect("function count must not be checked against the runtime frame quota");
    }

    fn compile_many_functions(extra_fn_count: usize) -> Vec<u8> {
        let mut src = String::new();
        for i in 0..extra_fn_count {
            src.push_str(&format!("fn fn_{i}() {{ return; }}\n"));
        }
        src.push_str("fn main() { return; }");
        compile_program_to_semcode(&src).expect("compile")
    }

    // #1754 (FA-07-014) test matrix: the verifier must sum DISTINCT
    // runtime symbols (function-local string-table entries, deduplicated
    // by exact value) across the WHOLE program against max_symbol_table
    // (16_384), matching the VM's single shared `RuntimeSymbolTable`
    // (crates/sm-runtime-core/src/lib.rs), not a per-function check.

    fn function_with_strings(name: &str, strings: &[String]) -> IrFunction {
        let mut instrs: Vec<IrInstr> = strings
            .iter()
            .map(|s| IrInstr::LoadText {
                dst: 0,
                val: s.clone(),
            })
            .collect();
        instrs.push(IrInstr::Ret { src: None });
        IrFunction {
            name: name.to_string(),
            instrs,
            ownership_events: Vec::new(),
            params: Vec::new(),
        }
    }

    /// Union of every function's local strings, deduplicated by value -
    /// the same quantity `RuntimeSymbolTable` (via
    /// `build_vm_program_view_from_decoded`) ends up holding.
    fn distinct_runtime_symbol_count(bytes: &[u8]) -> usize {
        let (_, decoded) =
            sm_format::semcode_decode::decode_semcode_envelope(bytes).expect("decode");
        decoded
            .iter()
            .flat_map(|env| env.strings.iter().map(|s| s.as_str()))
            .collect::<HashSet<_>>()
            .len()
    }

    #[test]
    fn verifier_accepts_heavy_duplication_under_program_wide_symbol_quota() {
        // 165 functions x 100 strings, but every function reuses the exact
        // same 100 string values: raw per-function-summed entries
        // (16_500) exceed max_symbol_table, while the DISTINCT program-wide
        // count (100) stays far under it. A naive "sum all local string
        // table lengths" implementation would wrongly reject this; the
        // correct dedup-by-value implementation must accept it.
        let shared_strings: Vec<String> = (0..100).map(|j| format!("dup_s{j}")).collect();
        let funcs: Vec<IrFunction> = (0..165)
            .map(|i| function_with_strings(&format!("f{i}"), &shared_strings))
            .collect();
        let bytes = emit_ir_to_semcode(&funcs, false).expect("emit");

        let raw_sum: usize = funcs.len() * shared_strings.len();
        assert!(
            raw_sum > 16_384,
            "raw sum must exceed the quota to be a meaningful control"
        );
        let distinct = distinct_runtime_symbol_count(&bytes);
        assert_eq!(distinct, 100);
        assert!(distinct <= 16_384);

        verify_semcode_token(&bytes)
            .expect("duplicate-heavy program under the distinct-symbol quota must be accepted");
    }

    #[test]
    fn verifier_accepts_program_wide_symbols_at_exact_quota_boundary() {
        // 64 functions x 256 strings, all globally distinct, each function
        // exactly at sm-format's MAX_STRINGS_PER_FUNCTION cap: 64 * 256 =
        // 16_384, exactly max_symbol_table. This hits the boundary exactly.
        let functions_count = 64usize;
        let strings_per_function = 256usize;
        assert_eq!(functions_count * strings_per_function, 16_384);
        assert_eq!(
            strings_per_function,
            sm_format::semcode_decode::MAX_STRINGS_PER_FUNCTION
        );

        let funcs: Vec<IrFunction> = (0..functions_count)
            .map(|i| {
                let strings: Vec<String> = (0..strings_per_function)
                    .map(|j| format!("b{i}_{j}"))
                    .collect();
                function_with_strings(&format!("f{i}"), &strings)
            })
            .collect();
        let bytes = emit_ir_to_semcode(&funcs, false).expect("emit");

        let distinct = distinct_runtime_symbol_count(&bytes);
        assert_eq!(distinct, 16_384);

        verify_semcode_token(&bytes)
            .expect("program exactly at the symbol quota boundary must be accepted");
    }

    #[test]
    fn verifier_rejects_program_wide_symbols_over_quota() {
        // Permanent regression test for the #1754 reproduction: 66
        // functions x 250 globally-distinct strings each (66 * 250 =
        // 16_500 > 16_384), with every individual function's local string
        // table staying at 250 <= MAX_STRINGS_PER_FUNCTION (256) - proving
        // this is caught only by GLOBAL, program-wide accounting, not by
        // any per-function limit.
        let functions_count = 66usize;
        let strings_per_function = 250usize;
        assert!(strings_per_function <= sm_format::semcode_decode::MAX_STRINGS_PER_FUNCTION);
        assert!(functions_count <= 256); // stay under the unrelated #1751 max_frames check

        let funcs: Vec<IrFunction> = (0..functions_count)
            .map(|i| {
                let strings: Vec<String> = (0..strings_per_function)
                    .map(|j| format!("f{i}_s{j}"))
                    .collect();
                function_with_strings(&format!("f{i}"), &strings)
            })
            .collect();
        let bytes = emit_ir_to_semcode(&funcs, false).expect("emit");

        let distinct = distinct_runtime_symbol_count(&bytes);
        assert_eq!(distinct, functions_count * strings_per_function);
        assert!(distinct > 16_384);

        let report = verify_semcode_token(&bytes)
            .expect_err("program-wide distinct runtime symbol count over quota must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::ResourceLimitExceeded
        );
        assert!(
            report.diagnostics[0]
                .message
                .contains("program-wide runtime symbol table"),
            "diagnostic message must name the program-wide runtime symbol table: {}",
            report.diagnostics[0].message
        );
    }
}
