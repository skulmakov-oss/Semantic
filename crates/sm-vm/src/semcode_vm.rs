use crate::semcode_format::{
    header_spec_from_magic, read_f64_le, read_i32_le, read_u16_le, read_u32_le, read_u8, read_utf8,
    supported_headers, Opcode, SemcodeFormatError, SemcodeHeaderSpec,
    OWNERSHIP_EVENT_KIND_BORROW, OWNERSHIP_EVENT_KIND_WRITE,
    OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL, OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX,
    OWNERSHIP_SECTION_TAG,
};
use crate::QuadVal;
use prom_abi::{AbiError, AbiValue, HostCallId, PrometheusHostAbi};
use prom_cap::{CapabilityChecker, CapabilityDenied, UiCapabilityChecker, UiCapabilityDenied};
use prom_ui::UiOperationId;
use sm_runtime_core::{
    AccessPath, AdtCarrier, ExecutionConfig, ExecutionContext, QuotaExceeded, QuotaKind,
    RecordCarrier, RuntimeQuotas, RuntimeSymbolTable, RuntimeTrap, SymbolId,
};
use sm_verify::verify_semcode;
use sm_verify::RejectReport;
use std::collections::{HashMap, HashSet};

const MAX_FUNCTIONS: usize = 4096;
const MAX_STRINGS_PER_FUNCTION: usize = 4096;
const MAX_STRING_LEN: usize = 8192;
const MAX_DEBUG_SYMBOLS_PER_FUNCTION: usize = 8192;

/// Scalar key type for Map values.
///
/// Only scalar types that support deterministic equality are admitted as map keys.
/// This mirrors the type-checker's key guard (i32|u32|bool|text|quad) and avoids
/// making `Value` universally hashable or ordered.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MapKey {
    I32(i32),
    U32(u32),
    Bool(bool),
    Text(String),
    Quad(u8),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClosureValue {
    pub function_name: String,
    pub captures: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Quad(QuadVal),
    Bool(bool),
    Text(String),
    Sequence(Vec<Value>),
    Map(Vec<(MapKey, Value)>),
    Closure(ClosureValue),
    I32(i32),
    F64(f64),
    U32(u32),
    Fx(i32),
    Tuple(Vec<Value>),
    Record(RecordCarrier<Value>),
    Adt(AdtCarrier<Value>),
    Unit,
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub pc: usize,
    pub regs: Vec<Value>,
    pub locals: HashMap<SymbolId, Value>,
    pub borrowed_paths: Vec<AccessPath>,
    next_write_path: usize,
    pub func: String,
    pub return_dst: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct FunctionBytecode {
    pub name: String,
    pub strings: Vec<String>,
    pub symbol_ids: Vec<SymbolId>,
    pub debug_symbols: Vec<DebugSymbol>,
    pub borrowed_paths: Vec<AccessPath>,
    write_paths: Vec<AccessPath>,
    pub code: Vec<u8>,
    pub instr_start: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugSymbol {
    pub pc: usize,
    pub line: u32,
    pub col: u16,
}

#[derive(Debug, Clone)]
pub struct VM {
    pub functions: HashMap<String, FunctionBytecode>,
    pub callstack: Vec<Frame>,
    pub config: ExecutionConfig,
    pub effect_calls: usize,
    pub symbols: RuntimeSymbolTable,
    /// PRNG state for random_seed / random_next_i32 (xorshift64; 0 = unseeded).
    pub prng_state: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    BadHeader,
    UnsupportedBytecodeVersion { found: String, supported: String },
    BadFormat(String),
    UnknownFunction(String),
    InvalidJumpAddress { func: String, addr: usize },
    TypeMismatchRuntime(String),
    StackUnderflow,
    StackOverflow,
    QuotaExceeded(QuotaExceeded),
    VerifierRejected(RejectReport),
    UnknownVariable(String),
    InvalidStringId(u16),
    HostAbi(AbiError),
    CapabilityDenied(CapabilityDenied),
    /// A UI operation was attempted but the required UI capability was not
    /// admitted in the manifest. Wired at M7 Wave 1.
    UiCapabilityDenied(UiCapabilityDenied),
    Trap(RuntimeTrap),
}

impl core::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RuntimeError::BadHeader => write!(f, "bad SemCode header"),
            RuntimeError::UnsupportedBytecodeVersion { found, supported } => write!(
                f,
                "unsupported SemCode version '{}'; supported versions: {}. hint: recompile source with current semantic",
                found, supported
            ),
            RuntimeError::BadFormat(m) => write!(f, "bad SemCode format: {}", m),
            RuntimeError::UnknownFunction(n) => write!(f, "unknown function '{}'", n),
            RuntimeError::InvalidJumpAddress { func, addr } => {
                write!(f, "invalid jump address {} in '{}'", addr, func)
            }
            RuntimeError::TypeMismatchRuntime(m) => write!(f, "runtime type mismatch: {}", m),
            RuntimeError::StackUnderflow => write!(f, "stack underflow"),
            RuntimeError::StackOverflow => write!(f, "stack overflow"),
            RuntimeError::QuotaExceeded(exceeded) => write!(
                f,
                "quota exceeded: {:?} limit={} used={}",
                exceeded.kind, exceeded.limit, exceeded.used
            ),
            RuntimeError::VerifierRejected(report) => write!(f, "{report}"),
            RuntimeError::UnknownVariable(n) => write!(f, "unknown variable '{}'", n),
            RuntimeError::InvalidStringId(id) => write!(f, "invalid string id {}", id),
            RuntimeError::HostAbi(err) => write!(f, "{err}"),
            RuntimeError::CapabilityDenied(err) => write!(f, "{err}"),
            RuntimeError::UiCapabilityDenied(err) => write!(f, "{err}"),
            RuntimeError::Trap(RuntimeTrap::AssertionFailed) => write!(f, "assertion failed"),
            RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict) => {
                write!(f, "write path overlaps active borrow")
            }
            RuntimeError::Trap(trap) => write!(f, "runtime trap: {:?}", trap),
        }
    }
}

impl std::error::Error for RuntimeError {}

pub fn run_semcode(bytes: &[u8]) -> Result<(), RuntimeError> {
    run_semcode_with_config(
        bytes,
        ExecutionConfig::for_context(ExecutionContext::VerifiedLocal),
    )
}

pub fn run_verified_semcode(bytes: &[u8]) -> Result<(), RuntimeError> {
    run_verified_semcode_with_config(
        bytes,
        ExecutionConfig::for_context(ExecutionContext::VerifiedLocal),
    )
}

pub fn run_semcode_with_entry(bytes: &[u8], entry: &str) -> Result<(), RuntimeError> {
    run_semcode_with_entry_and_config(
        bytes,
        entry,
        ExecutionConfig::for_context(ExecutionContext::VerifiedLocal),
    )
}

pub fn run_semcode_with_config(bytes: &[u8], config: ExecutionConfig) -> Result<(), RuntimeError> {
    run_semcode_with_entry_and_config(bytes, "main", config)
}

pub fn run_verified_semcode_with_config(
    bytes: &[u8],
    config: ExecutionConfig,
) -> Result<(), RuntimeError> {
    run_verified_semcode_with_entry_and_config(bytes, "main", config)
}

pub fn run_verified_semcode_with_entry(bytes: &[u8], entry: &str) -> Result<(), RuntimeError> {
    run_verified_semcode_with_entry_and_config(
        bytes,
        entry,
        ExecutionConfig::for_context(ExecutionContext::VerifiedLocal),
    )
}

pub fn run_verified_semcode_with_entry_and_config(
    bytes: &[u8],
    entry: &str,
    config: ExecutionConfig,
) -> Result<(), RuntimeError> {
    verify_semcode(bytes).map_err(RuntimeError::VerifierRejected)?;
    run_semcode_with_entry_and_config(bytes, entry, config)
}

pub fn run_verified_semcode_with_host_and_capabilities<
    H: PrometheusHostAbi,
    C: CapabilityChecker,
>(
    bytes: &[u8],
    host: &mut H,
    capabilities: &C,
) -> Result<(), RuntimeError> {
    run_verified_semcode_with_host_and_capabilities_and_config(
        bytes,
        "main",
        host,
        capabilities,
        ExecutionConfig::for_context(ExecutionContext::KernelBound),
    )
}

pub fn run_verified_semcode_with_host_and_capabilities_and_config<
    H: PrometheusHostAbi,
    C: CapabilityChecker,
>(
    bytes: &[u8],
    entry: &str,
    host: &mut H,
    capabilities: &C,
    config: ExecutionConfig,
) -> Result<(), RuntimeError> {
    verify_semcode(bytes).map_err(RuntimeError::VerifierRejected)?;
    let (_, symbols, functions) = parse_semcode(bytes)?;
    let mut vm = VM {
        functions,
        callstack: Vec::new(),
        config,
        effect_calls: 0,
        symbols,
        prng_state: 0,
    };
    push_frame(&mut vm, entry, Vec::new(), None)?;
    let mut bridge = PrometheusVmHost { host, capabilities };
    exec_loop(&mut vm, &mut bridge)
}

/// Run verified SemCode with both a standard capability checker and a UI
/// capability checker.
///
/// UI operations attempted without the corresponding `UiCapabilityKind` in
/// `ui_capabilities` will return `RuntimeError::UiCapabilityDenied`. This is
/// the Wave 1 denial path for the M7 UI application boundary track.
pub fn run_verified_semcode_with_ui_capabilities<
    H: PrometheusHostAbi,
    C: CapabilityChecker,
    U: UiCapabilityChecker,
>(
    bytes: &[u8],
    host: &mut H,
    capabilities: &C,
    ui_capabilities: &U,
) -> Result<(), RuntimeError> {
    verify_semcode(bytes).map_err(RuntimeError::VerifierRejected)?;
    let (_, symbols, functions) = parse_semcode(bytes)?;
    let mut vm = VM {
        functions,
        callstack: Vec::new(),
        config: ExecutionConfig::for_context(ExecutionContext::KernelBound),
        effect_calls: 0,
        symbols,
        prng_state: 0,
    };
    push_frame(&mut vm, "main", Vec::new(), None)?;
    let mut bridge = PrometheusUiVmHost { host, capabilities, ui_capabilities };
    exec_loop(&mut vm, &mut bridge)
}

pub fn run_semcode_with_entry_and_config(
    bytes: &[u8],
    entry: &str,
    config: ExecutionConfig,
) -> Result<(), RuntimeError> {
    let (_, symbols, functions) = parse_semcode(bytes)?;
    let mut vm = VM {
        functions,
        callstack: Vec::new(),
        config,
        effect_calls: 0,
        symbols,
        prng_state: 0,
    };
    push_frame(&mut vm, entry, Vec::new(), None)?;
    let mut host = LegacyVmHost;
    exec_loop(&mut vm, &mut host)
}

pub fn disasm_semcode(bytes: &[u8]) -> Result<String, RuntimeError> {
    let (spec, _, functions) = parse_semcode(bytes)?;
    let mut out = String::new();
    let header = if bytes.len() >= 8 { &bytes[0..8] } else { &[] };
    out.push_str(&format!(
        "{} epoch={}.{} caps=0x{:08x}\n",
        String::from_utf8_lossy(header),
        spec.epoch,
        spec.rev,
        spec.capabilities
    ));
    let mut ordered = functions.values().collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.name.cmp(&right.name));
    for f in ordered {
        out.push_str(&format!(
            "fn {}: code={} bytes, strings={}, debug={}\n",
            f.name,
            f.code.len(),
            f.strings.len(),
            f.debug_symbols.len()
        ));
        let mut pc = 0usize;
        while pc < f.code.len().saturating_sub(f.instr_start) {
            let (line, next) = disasm_one(f, pc)?;
            out.push_str(&format!("  {:04x}: {}\n", pc, line));
            pc = next;
        }
    }
    Ok(out)
}

fn parse_semcode(
    bytes: &[u8],
) -> Result<
    (
        SemcodeHeaderSpec,
        RuntimeSymbolTable,
        HashMap<String, FunctionBytecode>,
    ),
    RuntimeError,
> {
    if bytes.len() < 8 {
        return Err(RuntimeError::BadHeader);
    }
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let Some(header) = header_spec_from_magic(&magic) else {
        if &magic[0..7] == b"SEMCODE" {
            return Err(RuntimeError::UnsupportedBytecodeVersion {
                found: String::from_utf8_lossy(&magic).to_string(),
                supported: supported_headers()
                    .iter()
                    .map(|h| String::from_utf8_lossy(&h.magic).to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            });
        }
        return Err(RuntimeError::BadHeader);
    };
    let mut i = 8usize;
    let mut out = HashMap::new();
    let mut runtime_symbols = RuntimeSymbolTable::new();
    while i < bytes.len() {
        if out.len() >= MAX_FUNCTIONS {
            return Err(RuntimeError::BadFormat(format!(
                "too many functions (>{})",
                MAX_FUNCTIONS
            )));
        }
        let name_len = read_u16_le(bytes, &mut i).map_err(map_format_err)? as usize;
        if name_len == 0 {
            return Err(RuntimeError::BadFormat("empty function name".to_string()));
        }
        if name_len > MAX_STRING_LEN {
            return Err(RuntimeError::BadFormat(format!(
                "function name too long: {}",
                name_len
            )));
        }
        let name = read_utf8(bytes, &mut i, name_len).map_err(map_format_err)?;
        let code_len = read_u32_le(bytes, &mut i).map_err(map_format_err)? as usize;
        if i + code_len > bytes.len() {
            return Err(RuntimeError::BadFormat(
                "function code out of bounds".to_string(),
            ));
        }
        let code = bytes[i..i + code_len].to_vec();
        i += code_len;

        let (strings, debug_symbols, borrowed_paths, write_paths, instr_start) =
            parse_string_table_debug_and_ownership(&code)?;
        let symbol_ids = strings
            .iter()
            .map(|name| runtime_symbols.intern(name))
            .collect::<Vec<_>>();
        let remap_paths = |paths: Vec<AccessPath>| {
            paths
                .into_iter()
                .map(|path| {
                    let local_root = path.root.raw() as usize;
                    let root = symbol_ids.get(local_root).copied().unwrap_or(path.root);
                    Ok(AccessPath {
                        root,
                        components: path.components,
                    })
                })
                .collect::<Result<Vec<_>, RuntimeError>>()
        };
        let borrowed_paths = remap_paths(borrowed_paths)?;
        let write_paths = remap_paths(write_paths)?;
        let f = FunctionBytecode {
            name: name.clone(),
            strings,
            symbol_ids,
            debug_symbols,
            borrowed_paths,
            write_paths,
            code,
            instr_start,
        };
        validate_function_bytecode(&f)?;
        if out.insert(name.clone(), f).is_some() {
            return Err(RuntimeError::BadFormat(format!(
                "duplicate function '{}'",
                name
            )));
        }
    }
    Ok((header, runtime_symbols, out))
}

fn parse_string_table_debug_and_ownership(
    code: &[u8],
) -> Result<
    (
        Vec<String>,
        Vec<DebugSymbol>,
        Vec<AccessPath>,
        Vec<AccessPath>,
        usize,
    ),
    RuntimeError,
> {
    let mut i = 0usize;
    let count = read_u16_le(code, &mut i).map_err(map_format_err)? as usize;
    if count > MAX_STRINGS_PER_FUNCTION {
        return Err(RuntimeError::BadFormat(format!(
            "too many strings in function: {} (max {})",
            count, MAX_STRINGS_PER_FUNCTION
        )));
    }
    let mut strings = Vec::with_capacity(count);
    for _ in 0..count {
        let len = read_u16_le(code, &mut i).map_err(map_format_err)? as usize;
        if len > MAX_STRING_LEN {
            return Err(RuntimeError::BadFormat(format!(
                "string too long in function string table: {}",
                len
            )));
        }
        strings.push(read_utf8(code, &mut i, len).map_err(map_format_err)?);
    }
    let mut debug_symbols = Vec::new();
    if i + 4 <= code.len() && &code[i..i + 4] == b"DBG0" {
        i += 4;
        let count = read_u16_le(code, &mut i).map_err(map_format_err)? as usize;
        if count > MAX_DEBUG_SYMBOLS_PER_FUNCTION {
            return Err(RuntimeError::BadFormat(format!(
                "too many debug symbols in function: {} (max {})",
                count, MAX_DEBUG_SYMBOLS_PER_FUNCTION
            )));
        }
        debug_symbols.reserve(count);
        for _ in 0..count {
            let pc = read_u32_le(code, &mut i).map_err(map_format_err)? as usize;
            let line = read_u32_le(code, &mut i).map_err(map_format_err)?;
            let col = read_u16_le(code, &mut i).map_err(map_format_err)?;
            debug_symbols.push(DebugSymbol { pc, line, col });
        }
    }
    let mut borrowed_paths = Vec::new();
    let mut write_paths = Vec::new();
    if i + 4 <= code.len() && &code[i..i + 4] == OWNERSHIP_SECTION_TAG {
        i += OWNERSHIP_SECTION_TAG.len();
        let count = read_u16_le(code, &mut i).map_err(map_format_err)? as usize;
        borrowed_paths.reserve(count);
        write_paths.reserve(count);
        for _ in 0..count {
            let kind = read_u8(code, &mut i).map_err(map_format_err)?;
            let root = SymbolId(read_u32_le(code, &mut i).map_err(map_format_err)?);
            let component_count = read_u16_le(code, &mut i).map_err(map_format_err)? as usize;
            let mut path = AccessPath::new(root);
            for _ in 0..component_count {
                let component_kind = read_u8(code, &mut i).map_err(map_format_err)?;
                match component_kind {
                    OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX => {
                        let index = read_u16_le(code, &mut i).map_err(map_format_err)?;
                        path = path.tuple_index(index);
                    }
                    OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL => {
                        let field = SymbolId(read_u32_le(code, &mut i).map_err(map_format_err)?);
                        path = path.field(field);
                    }
                    _ => {
                        return Err(RuntimeError::BadFormat(format!(
                            "unsupported ownership path component kind 0x{component_kind:02x}"
                        )));
                    }
                }
            }
            match kind {
                OWNERSHIP_EVENT_KIND_BORROW => borrowed_paths.push(path),
                OWNERSHIP_EVENT_KIND_WRITE => write_paths.push(path),
                _ => {
                    return Err(RuntimeError::BadFormat(format!(
                        "unsupported ownership event kind 0x{kind:02x}"
                    )))
                }
            }
        }
    }
    Ok((strings, debug_symbols, borrowed_paths, write_paths, i))
}

fn map_format_err(err: SemcodeFormatError) -> RuntimeError {
    match err {
        SemcodeFormatError::UnexpectedEof => RuntimeError::BadFormat("unexpected EOF".to_string()),
        SemcodeFormatError::InvalidUtf8 => RuntimeError::BadFormat("invalid utf8".to_string()),
        SemcodeFormatError::UnknownOpcode(op) => {
            RuntimeError::BadFormat(format!("unknown opcode 0x{:02x}", op))
        }
    }
}

fn validate_function_bytecode(f: &FunctionBytecode) -> Result<(), RuntimeError> {
    if f.instr_start > f.code.len() {
        return Err(RuntimeError::BadFormat(format!(
            "invalid instr_start in '{}'",
            f.name
        )));
    }
    let mut cur = f.instr_start;
    let mut starts: HashSet<usize> = HashSet::new();
    let mut jumps: Vec<usize> = Vec::new();
    while cur < f.code.len() {
        starts.insert(cur - f.instr_start);
        let opcode = Opcode::from_byte(read_u8(&f.code, &mut cur).map_err(map_format_err)?)
            .map_err(map_format_err)?;
        match opcode {
            Opcode::LoadQ => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u8(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::LoadBool => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u8(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::LoadI32 => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_i32_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::AddI32 => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::SubI32 | Opcode::MulI32 | Opcode::DivI32 | Opcode::ModI32 => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::LoadU32 => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u32_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::LoadF64 => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_f64_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::LoadFx => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_i32_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::AddFx | Opcode::SubFx | Opcode::MulFx | Opcode::DivFx => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::MakeTuple => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                if count < 2 {
                    return Err(RuntimeError::BadFormat(format!(
                        "tuple literal arity must be at least 2 in '{}'",
                        f.name
                    )));
                }
                for _ in 0..count {
                    let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                }
            }
            Opcode::MakeSequence => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                for _ in 0..count {
                    let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                }
            }
            Opcode::MakeClosure => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                for _ in 0..count {
                    let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                }
            }
            Opcode::MakeRecord => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                if count == 0 {
                    return Err(RuntimeError::BadFormat(format!(
                        "record literal slot count must be at least 1 in '{}'",
                        f.name
                    )));
                }
                for _ in 0..count {
                    let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                }
            }
            Opcode::MakeAdt => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                for _ in 0..count {
                    let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                }
            }
            Opcode::AdtTag => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::AdtGet => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::RecordGet => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::TupleGet => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::SequenceGet => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::SequenceLen => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::SequenceIsEmpty => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::SequenceContains | Opcode::SequencePush | Opcode::SequencePrepend => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::SequencePop => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::MapEmpty => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::MapContains => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::MapGet => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::MapSet => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::RngSeed => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::RngNextI32 => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::ClosureCall => {
                let _ = read_u8(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::LoadVar => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::StoreVar => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::QNot | Opcode::BoolNot => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::LoadText => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::ConcatText => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
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
            | Opcode::DivF64 => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::Jmp => {
                let addr = read_u32_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                jumps.push(addr);
            }
            Opcode::JmpIf => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let addr = read_u32_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                jumps.push(addr);
            }
            Opcode::Call => {
                let _ = read_u8(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let argc = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                for _ in 0..argc {
                    let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                }
            }
            Opcode::Assert => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::GateRead => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::GateWrite => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::PulseEmit => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::StateQuery => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::StateUpdate => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::EventPost => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::ClockRead => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::Ret => {
                let has_src = read_u8(&f.code, &mut cur).map_err(map_format_err)? != 0;
                if has_src {
                    let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                }
            }
        }
    }
    if cur != f.code.len() {
        return Err(RuntimeError::BadFormat(format!(
            "trailing bytes in '{}'",
            f.name
        )));
    }
    let instr_len = f.code.len().saturating_sub(f.instr_start);
    for ds in &f.debug_symbols {
        if ds.pc >= instr_len {
            return Err(RuntimeError::BadFormat(format!(
                "debug pc out of range in '{}': {}",
                f.name, ds.pc
            )));
        }
    }
    for addr in jumps {
        if addr >= instr_len {
            return Err(RuntimeError::BadFormat(format!(
                "jump out of range in '{}': {}",
                f.name, addr
            )));
        }
        if !starts.contains(&addr) {
            return Err(RuntimeError::BadFormat(format!(
                "jump to non-instruction boundary in '{}': {}",
                f.name, addr
            )));
        }
    }
    Ok(())
}

trait VmHostBridge {
    fn gate_read(&mut self, device_id: u16, port: u16) -> Result<Value, RuntimeError>;
    fn gate_write(&mut self, device_id: u16, port: u16, value: Value) -> Result<(), RuntimeError>;
    fn pulse_emit(&mut self, signal: &str) -> Result<(), RuntimeError>;
    fn state_query(&mut self, key: &str) -> Result<Value, RuntimeError>;
    fn state_update(&mut self, key: &str, value: Value) -> Result<(), RuntimeError>;
    fn event_post(&mut self, signal: &str) -> Result<(), RuntimeError>;
    fn clock_read(&mut self) -> Result<Value, RuntimeError>;

    /// UI boundary operations — Wave 1 denial path.
    ///
    /// Default implementations return not-admitted. Overridden in
    /// `PrometheusVmHost` when a `UiCapabilityChecker` is provided.
    fn ui_window_create(&mut self) -> Result<(), RuntimeError> {
        Err(RuntimeError::BadFormat(
            "UI operations are not admitted in the current execution context; \
             M7 UI boundary requires an explicit UiCapabilityChecker"
                .to_string(),
        ))
    }
    fn ui_window_run(&mut self) -> Result<(), RuntimeError> {
        Err(RuntimeError::BadFormat(
            "UI operations are not admitted in the current execution context; \
             M7 UI boundary requires an explicit UiCapabilityChecker"
                .to_string(),
        ))
    }
    fn ui_event_poll(&mut self) -> Result<(), RuntimeError> {
        Err(RuntimeError::BadFormat(
            "UI operations are not admitted in the current execution context; \
             M7 UI boundary requires an explicit UiCapabilityChecker"
                .to_string(),
        ))
    }
    fn ui_frame_submit(&mut self) -> Result<(), RuntimeError> {
        Err(RuntimeError::BadFormat(
            "UI operations are not admitted in the current execution context; \
             M7 UI boundary requires an explicit UiCapabilityChecker"
                .to_string(),
        ))
    }
}

struct LegacyVmHost;

impl VmHostBridge for LegacyVmHost {
    fn gate_read(&mut self, device_id: u16, port: u16) -> Result<Value, RuntimeError> {
        Ok(Value::I32(((device_id as i32) << 16) | (port as i32)))
    }

    fn gate_write(
        &mut self,
        _device_id: u16,
        _port: u16,
        _value: Value,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn pulse_emit(&mut self, _signal: &str) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn state_query(&mut self, key: &str) -> Result<Value, RuntimeError> {
        Ok(Value::I32(stable_state_query_fallback(key)))
    }

    fn state_update(&mut self, _key: &str, _value: Value) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn event_post(&mut self, _signal: &str) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn clock_read(&mut self) -> Result<Value, RuntimeError> {
        Ok(Value::U32(0))
    }
}

struct PrometheusVmHost<'a, H: PrometheusHostAbi, C: CapabilityChecker> {
    host: &'a mut H,
    capabilities: &'a C,
}

impl<'a, H: PrometheusHostAbi, C: CapabilityChecker> VmHostBridge for PrometheusVmHost<'a, H, C> {
    fn gate_read(&mut self, device_id: u16, port: u16) -> Result<Value, RuntimeError> {
        self.capabilities
            .require_call(HostCallId::GateRead)
            .map_err(RuntimeError::CapabilityDenied)?;
        self.host
            .gate_read(device_id, port)
            .map(value_from_abi)
            .map_err(RuntimeError::HostAbi)
    }

    fn gate_write(&mut self, device_id: u16, port: u16, value: Value) -> Result<(), RuntimeError> {
        self.capabilities
            .require_call(HostCallId::GateWrite)
            .map_err(RuntimeError::CapabilityDenied)?;
        self.host
            .gate_write(device_id, port, value_to_abi(value)?)
            .map_err(RuntimeError::HostAbi)
    }

    fn pulse_emit(&mut self, signal: &str) -> Result<(), RuntimeError> {
        self.capabilities
            .require_call(HostCallId::PulseEmit)
            .map_err(RuntimeError::CapabilityDenied)?;
        self.host.pulse_emit(signal).map_err(RuntimeError::HostAbi)
    }

    fn state_query(&mut self, key: &str) -> Result<Value, RuntimeError> {
        self.capabilities
            .require_call(HostCallId::StateQuery)
            .map_err(RuntimeError::CapabilityDenied)?;
        self.host
            .state_query(key)
            .map(value_from_abi)
            .map_err(RuntimeError::HostAbi)
    }

    fn state_update(&mut self, key: &str, value: Value) -> Result<(), RuntimeError> {
        self.capabilities
            .require_call(HostCallId::StateUpdate)
            .map_err(RuntimeError::CapabilityDenied)?;
        self.host
            .state_update(key, value_to_abi(value)?)
            .map_err(RuntimeError::HostAbi)
    }

    fn event_post(&mut self, signal: &str) -> Result<(), RuntimeError> {
        self.capabilities
            .require_call(HostCallId::EventPost)
            .map_err(RuntimeError::CapabilityDenied)?;
        self.host
            .event_post(signal)
            .map_err(RuntimeError::HostAbi)
    }

    fn clock_read(&mut self) -> Result<Value, RuntimeError> {
        self.capabilities
            .require_call(HostCallId::ClockRead)
            .map_err(RuntimeError::CapabilityDenied)?;
        self.host
            .clock_read()
            .map(Value::U32)
            .map_err(RuntimeError::HostAbi)
    }
}

/// VM host bridge that also carries a `UiCapabilityChecker`.
///
/// Created when the caller provides an explicit UI capability manifest.
/// UI operations check the `UiCapabilityChecker` and return
/// `RuntimeError::UiCapabilityDenied` when the capability is absent.
struct PrometheusUiVmHost<'a, H: PrometheusHostAbi, C: CapabilityChecker, U: UiCapabilityChecker> {
    host: &'a mut H,
    capabilities: &'a C,
    ui_capabilities: &'a U,
}

impl<'a, H: PrometheusHostAbi, C: CapabilityChecker, U: UiCapabilityChecker> VmHostBridge
    for PrometheusUiVmHost<'a, H, C, U>
{
    fn gate_read(&mut self, device_id: u16, port: u16) -> Result<Value, RuntimeError> {
        self.capabilities
            .require_call(HostCallId::GateRead)
            .map_err(RuntimeError::CapabilityDenied)?;
        self.host
            .gate_read(device_id, port)
            .map(value_from_abi)
            .map_err(RuntimeError::HostAbi)
    }

    fn gate_write(&mut self, device_id: u16, port: u16, value: Value) -> Result<(), RuntimeError> {
        self.capabilities
            .require_call(HostCallId::GateWrite)
            .map_err(RuntimeError::CapabilityDenied)?;
        self.host
            .gate_write(device_id, port, value_to_abi(value)?)
            .map_err(RuntimeError::HostAbi)
    }

    fn pulse_emit(&mut self, signal: &str) -> Result<(), RuntimeError> {
        self.capabilities
            .require_call(HostCallId::PulseEmit)
            .map_err(RuntimeError::CapabilityDenied)?;
        self.host.pulse_emit(signal).map_err(RuntimeError::HostAbi)
    }

    fn state_query(&mut self, key: &str) -> Result<Value, RuntimeError> {
        self.capabilities
            .require_call(HostCallId::StateQuery)
            .map_err(RuntimeError::CapabilityDenied)?;
        self.host
            .state_query(key)
            .map(value_from_abi)
            .map_err(RuntimeError::HostAbi)
    }

    fn state_update(&mut self, key: &str, value: Value) -> Result<(), RuntimeError> {
        self.capabilities
            .require_call(HostCallId::StateUpdate)
            .map_err(RuntimeError::CapabilityDenied)?;
        self.host
            .state_update(key, value_to_abi(value)?)
            .map_err(RuntimeError::HostAbi)
    }

    fn event_post(&mut self, signal: &str) -> Result<(), RuntimeError> {
        self.capabilities
            .require_call(HostCallId::EventPost)
            .map_err(RuntimeError::CapabilityDenied)?;
        self.host
            .event_post(signal)
            .map_err(RuntimeError::HostAbi)
    }

    fn clock_read(&mut self) -> Result<Value, RuntimeError> {
        self.capabilities
            .require_call(HostCallId::ClockRead)
            .map_err(RuntimeError::CapabilityDenied)?;
        self.host
            .clock_read()
            .map(Value::U32)
            .map_err(RuntimeError::HostAbi)
    }

    fn ui_window_create(&mut self) -> Result<(), RuntimeError> {
        self.ui_capabilities
            .require_ui_op(UiOperationId::WindowCreate)
            .map_err(RuntimeError::UiCapabilityDenied)
        // Actual window creation is deferred to Wave 2 (desktop lifecycle).
    }

    fn ui_window_run(&mut self) -> Result<(), RuntimeError> {
        self.ui_capabilities
            .require_ui_op(UiOperationId::WindowRun)
            .map_err(RuntimeError::UiCapabilityDenied)
        // Actual event/frame loop is deferred to Wave 2.
    }

    fn ui_event_poll(&mut self) -> Result<(), RuntimeError> {
        self.ui_capabilities
            .require_ui_op(UiOperationId::EventPoll)
            .map_err(RuntimeError::UiCapabilityDenied)
        // Actual event polling is deferred to Wave 2.
    }

    fn ui_frame_submit(&mut self) -> Result<(), RuntimeError> {
        self.ui_capabilities
            .require_ui_op(UiOperationId::FrameSubmit)
            .map_err(RuntimeError::UiCapabilityDenied)
        // Actual frame submission is deferred to Wave 3 (drawing surface).
    }
}

fn exec_loop<H: VmHostBridge>(vm: &mut VM, host: &mut H) -> Result<(), RuntimeError> {
    loop {
        let Some(frame_idx) = vm.callstack.len().checked_sub(1) else {
            return Ok(());
        };
        let func_name = vm.callstack[frame_idx].func.clone();
        let f = vm
            .functions
            .get(&func_name)
            .cloned()
            .ok_or_else(|| RuntimeError::UnknownFunction(func_name.clone()))?;
        let pc = vm.callstack[frame_idx].pc;
        let instr_rel_len = f.code.len().saturating_sub(f.instr_start);
        if pc >= instr_rel_len {
            return Err(RuntimeError::BadFormat(format!(
                "pc out of range in '{}': {}",
                func_name, pc
            )));
        }
        let mut cur = f.instr_start + pc;
        let opcode = Opcode::from_byte(read_u8(&f.code, &mut cur).map_err(map_format_err)?)
            .map_err(map_format_err)?;
        let next_pc: usize;

        match opcode {
            Opcode::LoadQ => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let q = match read_u8(&f.code, &mut cur).map_err(map_format_err)? {
                    0 => QuadVal::N,
                    1 => QuadVal::F,
                    2 => QuadVal::T,
                    3 => QuadVal::S,
                    v => {
                        return Err(RuntimeError::BadFormat(format!(
                            "invalid quad literal {}",
                            v
                        )))
                    }
                };
                set_reg(vm, frame_idx, dst, Value::Quad(q))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::LoadBool => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let b = read_u8(&f.code, &mut cur).map_err(map_format_err)? != 0;
                set_reg(vm, frame_idx, dst, Value::Bool(b))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::LoadI32 => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let v = read_i32_le(&f.code, &mut cur).map_err(map_format_err)?;
                set_reg(vm, frame_idx, dst, Value::I32(v))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::AddI32 => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let rhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let l = as_i32(get_reg(vm, frame_idx, lhs)?)?;
                let r = as_i32(get_reg(vm, frame_idx, rhs)?)?;
                let out = l.wrapping_add(r);
                set_reg(vm, frame_idx, dst, Value::I32(out))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::SubI32 => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let rhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let l = as_i32(get_reg(vm, frame_idx, lhs)?)?;
                let r = as_i32(get_reg(vm, frame_idx, rhs)?)?;
                let out = l.wrapping_sub(r);
                set_reg(vm, frame_idx, dst, Value::I32(out))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::MulI32 => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let rhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let l = as_i32(get_reg(vm, frame_idx, lhs)?)?;
                let r = as_i32(get_reg(vm, frame_idx, rhs)?)?;
                let out = l.wrapping_mul(r);
                set_reg(vm, frame_idx, dst, Value::I32(out))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::DivI32 => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let rhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let l = as_i32(get_reg(vm, frame_idx, lhs)?)?;
                let r = as_i32(get_reg(vm, frame_idx, rhs)?)?;
                let out = i32_div_raw(l, r)?;
                set_reg(vm, frame_idx, dst, Value::I32(out))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::ModI32 => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let rhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let l = as_i32(get_reg(vm, frame_idx, lhs)?)?;
                let r = as_i32(get_reg(vm, frame_idx, rhs)?)?;
                let out = i32_mod_raw(l, r)?;
                set_reg(vm, frame_idx, dst, Value::I32(out))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::LoadU32 => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let v = read_u32_le(&f.code, &mut cur).map_err(map_format_err)?;
                set_reg(vm, frame_idx, dst, Value::U32(v))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::LoadF64 => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let v = read_f64_le(&f.code, &mut cur).map_err(map_format_err)?;
                set_reg(vm, frame_idx, dst, Value::F64(v))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::LoadFx => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let v = read_i32_le(&f.code, &mut cur).map_err(map_format_err)?;
                set_reg(vm, frame_idx, dst, Value::Fx(v))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::LoadText => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let value = decode_text_literal(lookup_str(&f, sid)?);
                set_reg(vm, frame_idx, dst, Value::Text(value))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::ConcatText => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let rhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let mut out = as_text(get_reg(vm, frame_idx, lhs)?)?;
                out.push_str(&as_text(get_reg(vm, frame_idx, rhs)?)?);
                set_reg(vm, frame_idx, dst, Value::Text(out))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::MakeSequence => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                let mut items = Vec::with_capacity(count);
                for _ in 0..count {
                    let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                    items.push(get_reg(vm, frame_idx, src)?);
                }
                set_reg(vm, frame_idx, dst, Value::Sequence(items))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::MakeClosure => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                let function_name = lookup_str(&f, sid)?.to_string();
                let mut captures = Vec::with_capacity(count);
                for _ in 0..count {
                    let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                    captures.push(get_reg(vm, frame_idx, src)?);
                }
                set_reg(
                    vm,
                    frame_idx,
                    dst,
                    Value::Closure(ClosureValue {
                        function_name,
                        captures,
                    }),
                )?;
                next_pc = cur - f.instr_start;
            }
            Opcode::MakeTuple => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                let mut items = Vec::with_capacity(count);
                for _ in 0..count {
                    let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                    items.push(get_reg(vm, frame_idx, src)?);
                }
                set_reg(vm, frame_idx, dst, Value::Tuple(items))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::MakeRecord => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                let type_name = lookup_str(&f, sid)?.to_string();
                let mut slots = Vec::with_capacity(count);
                for _ in 0..count {
                    let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                    slots.push(get_reg(vm, frame_idx, src)?);
                }
                set_reg(
                    vm,
                    frame_idx,
                    dst,
                    Value::Record(RecordCarrier { type_name, slots }),
                )?;
                next_pc = cur - f.instr_start;
            }
            Opcode::MakeAdt => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let variant_sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let tag = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                let type_name = lookup_str(&f, sid)?.to_string();
                let variant_name = lookup_str(&f, variant_sid)?.to_string();
                let mut payload = Vec::with_capacity(count);
                for _ in 0..count {
                    let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                    payload.push(get_reg(vm, frame_idx, src)?);
                }
                set_reg(
                    vm,
                    frame_idx,
                    dst,
                    Value::Adt(AdtCarrier {
                        type_name,
                        variant_name,
                        tag,
                        payload,
                    }),
                )?;
                next_pc = cur - f.instr_start;
            }
            Opcode::AdtTag => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let expected_name = lookup_str(&f, sid)?.to_string();
                let adt = get_reg(vm, frame_idx, src)?;
                let Value::Adt(adt) = adt else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "ADT_TAG source must be enum".to_string(),
                    ));
                };
                if adt.type_name != expected_name {
                    return Err(RuntimeError::TypeMismatchRuntime(format!(
                        "ADT_TAG expected enum '{}', got '{}'",
                        expected_name, adt.type_name
                    )));
                }
                set_reg(vm, frame_idx, dst, Value::I32(i32::from(adt.tag)))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::AdtGet => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let index = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                let expected_name = lookup_str(&f, sid)?.to_string();
                let adt = get_reg(vm, frame_idx, src)?;
                let Value::Adt(adt) = adt else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "ADT_GET source must be enum".to_string(),
                    ));
                };
                if adt.type_name != expected_name {
                    return Err(RuntimeError::TypeMismatchRuntime(format!(
                        "ADT_GET expected enum '{}', got '{}'",
                        expected_name, adt.type_name
                    )));
                }
                let item = adt.payload.get(index).cloned().ok_or_else(|| {
                    RuntimeError::BadFormat(format!("adt-get index out of bounds: {}", index))
                })?;
                set_reg(vm, frame_idx, dst, item)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::RecordGet => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let index = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                let expected_name = lookup_str(&f, sid)?.to_string();
                let record = get_reg(vm, frame_idx, src)?;
                let Value::Record(record) = record else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "RECORD_GET source must be record".to_string(),
                    ));
                };
                if record.type_name != expected_name {
                    return Err(RuntimeError::TypeMismatchRuntime(format!(
                        "RECORD_GET expected record '{}', got '{}'",
                        expected_name, record.type_name
                    )));
                }
                let item = record.slots.get(index).cloned().ok_or_else(|| {
                    RuntimeError::BadFormat(format!("record-get index out of bounds: {}", index))
                })?;
                set_reg(vm, frame_idx, dst, item)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::TupleGet => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let index = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                let tuple = get_reg(vm, frame_idx, src)?;
                let Value::Tuple(items) = tuple else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "TUPLE_GET source must be tuple".to_string(),
                    ));
                };
                let item = items.get(index).cloned().ok_or_else(|| {
                    RuntimeError::BadFormat(format!("tuple-get index out of bounds: {}", index))
                })?;
                set_reg(vm, frame_idx, dst, item)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::SequenceGet => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let index_reg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sequence = get_reg(vm, frame_idx, src)?;
                let Value::Sequence(items) = sequence else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "SEQUENCE_GET source must be sequence".to_string(),
                    ));
                };
                let index = as_i32(get_reg(vm, frame_idx, index_reg)?)?;
                if index < 0 {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "SEQUENCE_GET index must be non-negative".to_string(),
                    ));
                }
                let item = items.get(index as usize).cloned().ok_or_else(|| {
                    RuntimeError::TypeMismatchRuntime(format!(
                        "SEQUENCE_GET index out of bounds: {}",
                        index
                    ))
                })?;
                set_reg(vm, frame_idx, dst, item)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::SequenceLen => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sequence = get_reg(vm, frame_idx, src)?;
                let Value::Sequence(items) = sequence else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "SEQUENCE_LEN source must be sequence".to_string(),
                    ));
                };
                let len = i32::try_from(items.len()).map_err(|_| {
                    RuntimeError::BadFormat("SEQUENCE_LEN exceeds i32 range".to_string())
                })?;
                set_reg(vm, frame_idx, dst, Value::I32(len))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::SequenceIsEmpty => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sequence = get_reg(vm, frame_idx, src)?;
                let Value::Sequence(items) = sequence else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "SEQUENCE_IS_EMPTY source must be sequence".to_string(),
                    ));
                };
                set_reg(vm, frame_idx, dst, Value::Bool(items.is_empty()))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::SequenceContains => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let seq_reg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let val_reg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sequence = get_reg(vm, frame_idx, seq_reg)?;
                let Value::Sequence(items) = sequence else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "SEQUENCE_CONTAINS first argument must be sequence".to_string(),
                    ));
                };
                let search = get_reg(vm, frame_idx, val_reg)?;
                set_reg(vm, frame_idx, dst, Value::Bool(items.contains(&search)))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::SequencePush => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let seq_reg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let val_reg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sequence = get_reg(vm, frame_idx, seq_reg)?;
                let Value::Sequence(items) = sequence else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "SEQUENCE_PUSH first argument must be sequence".to_string(),
                    ));
                };
                let new_val = get_reg(vm, frame_idx, val_reg)?;
                let mut new_items = items;
                new_items.push(new_val);
                set_reg(vm, frame_idx, dst, Value::Sequence(new_items))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::SequencePrepend => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let seq_reg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let val_reg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sequence = get_reg(vm, frame_idx, seq_reg)?;
                let Value::Sequence(items) = sequence else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "SEQUENCE_PREPEND first argument must be sequence".to_string(),
                    ));
                };
                let new_val = get_reg(vm, frame_idx, val_reg)?;
                let mut new_items = Vec::with_capacity(items.len() + 1);
                new_items.push(new_val);
                new_items.extend(items);
                set_reg(vm, frame_idx, dst, Value::Sequence(new_items))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::SequencePop => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sequence = get_reg(vm, frame_idx, src)?;
                let Value::Sequence(items) = sequence else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "SEQUENCE_POP source must be sequence".to_string(),
                    ));
                };
                if items.is_empty() {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "SEQUENCE_POP source must be non-empty".to_string(),
                    ));
                }
                let mut new_items = items.clone();
                new_items.pop();
                set_reg(vm, frame_idx, dst, Value::Sequence(new_items))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::MapEmpty => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                set_reg(vm, frame_idx, dst, Value::Map(Vec::new()))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::MapContains => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let map_reg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let key_reg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let map_val = get_reg(vm, frame_idx, map_reg)?;
                let Value::Map(pairs) = map_val else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "MAP_CONTAINS first argument must be a map".to_string(),
                    ));
                };
                let key = map_key_from_value(get_reg(vm, frame_idx, key_reg)?)?;
                let found = pairs.iter().any(|(k, _)| k == &key);
                set_reg(vm, frame_idx, dst, Value::Bool(found))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::MapGet => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let map_reg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let key_reg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let default_reg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let map_val = get_reg(vm, frame_idx, map_reg)?;
                let Value::Map(pairs) = map_val else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "MAP_GET first argument must be a map".to_string(),
                    ));
                };
                let key = map_key_from_value(get_reg(vm, frame_idx, key_reg)?)?;
                let result = pairs
                    .iter()
                    .find(|(k, _)| k == &key)
                    .map(|(_, v)| v.clone())
                    .unwrap_or_else(|| get_reg(vm, frame_idx, default_reg).unwrap_or(Value::Unit));
                set_reg(vm, frame_idx, dst, result)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::MapSet => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let map_reg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let key_reg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let val_reg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let map_val = get_reg(vm, frame_idx, map_reg)?;
                let Value::Map(pairs) = map_val else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "MAP_SET first argument must be a map".to_string(),
                    ));
                };
                let key = map_key_from_value(get_reg(vm, frame_idx, key_reg)?)?;
                let val = get_reg(vm, frame_idx, val_reg)?;
                let mut new_pairs: Vec<(MapKey, Value)> =
                    pairs.iter().filter(|(k, _)| k != &key).cloned().collect();
                new_pairs.push((key, val));
                set_reg(vm, frame_idx, dst, Value::Map(new_pairs))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::RngSeed => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let seed_reg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let seed_val = get_reg(vm, frame_idx, seed_reg)?;
                let Value::I32(seed) = seed_val else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "RNG_SEED seed argument must be i32".to_string(),
                    ));
                };
                // Map i32 seed to a non-zero u64.  seed==0 becomes 1 to avoid
                // the xorshift64 zero fixed-point.
                let raw = seed as u64;
                vm.prng_state = if raw == 0 { 1 } else { raw };
                set_reg(vm, frame_idx, dst, Value::Unit)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::RngNextI32 => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lo_reg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let hi_reg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lo_val = get_reg(vm, frame_idx, lo_reg)?;
                let hi_val = get_reg(vm, frame_idx, hi_reg)?;
                let (Value::I32(lo), Value::I32(hi)) = (lo_val, hi_val) else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "RNG_NEXT_I32 lo and hi must be i32".to_string(),
                    ));
                };
                if lo >= hi {
                    return Err(RuntimeError::TypeMismatchRuntime(format!(
                        "random_next_i32: lo ({lo}) must be strictly less than hi ({hi})"
                    )));
                }
                // Compute range through i64 to handle the full i32 span without
                // overflow (e.g. lo=i32::MIN, hi=i32::MAX gives range=4294967295).
                let range = i64::from(hi) - i64::from(lo);
                let raw = xorshift64_step(&mut vm.prng_state);
                let offset = (raw % (range as u64)) as i64;
                let result = i64::from(lo) + offset;
                let result = i32::try_from(result)
                    .map_err(|_| RuntimeError::Trap(RuntimeTrap::ArithmeticOverflow))?;
                set_reg(vm, frame_idx, dst, Value::I32(result))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::LoadVar => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let symbol = lookup_symbol(&f, sid)?;
                let val = vm.callstack[frame_idx]
                    .locals
                    .get(&symbol)
                    .cloned()
                    .ok_or_else(|| {
                        let name = lookup_str(&f, sid).unwrap_or("<unknown>");
                        RuntimeError::UnknownVariable(name.to_string())
                    })?;
                set_reg(vm, frame_idx, dst, val)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::StoreVar => {
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let symbol = lookup_symbol(&f, sid)?;
                let val = get_reg(vm, frame_idx, src)?;
                let next_write_path = {
                    let frame = &vm.callstack[frame_idx];
                    if frame.locals.contains_key(&symbol) {
                        f.write_paths
                            .get(frame.next_write_path)
                            .filter(|path| path.root == symbol)
                            .cloned()
                    } else {
                        None
                    }
                };
                if let Some(write_path) = next_write_path {
                    let symbol_name = lookup_str(&f, sid).unwrap_or("<unknown>");
                    let frame = &vm.callstack[frame_idx];
                    ensure_write_path_allowed(symbol_name, &write_path, &frame.borrowed_paths)?;
                    vm.callstack[frame_idx].next_write_path += 1;
                }
                vm.callstack[frame_idx].locals.insert(symbol, val);
                next_pc = cur - f.instr_start;
            }
            Opcode::QAnd | Opcode::QOr | Opcode::QImpl => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let rhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lq = as_quad(get_reg(vm, frame_idx, lhs)?)?;
                let rq = as_quad(get_reg(vm, frame_idx, rhs)?)?;
                let out_q = match opcode {
                    Opcode::QAnd => quad_and(lq, rq),
                    Opcode::QOr => quad_or(lq, rq),
                    Opcode::QImpl => quad_or(quad_not(lq), rq),
                    _ => unreachable!(),
                };
                set_reg(vm, frame_idx, dst, Value::Quad(out_q))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::QNot => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let q = as_quad(get_reg(vm, frame_idx, src)?)?;
                set_reg(vm, frame_idx, dst, Value::Quad(quad_not(q)))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::BoolAnd | Opcode::BoolOr => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let rhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lb = as_bool(get_reg(vm, frame_idx, lhs)?)?;
                let rb = as_bool(get_reg(vm, frame_idx, rhs)?)?;
                let out_b = if opcode == Opcode::BoolAnd {
                    lb && rb
                } else {
                    lb || rb
                };
                set_reg(vm, frame_idx, dst, Value::Bool(out_b))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::BoolNot => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let b = as_bool(get_reg(vm, frame_idx, src)?)?;
                set_reg(vm, frame_idx, dst, Value::Bool(!b))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::CmpEq | Opcode::CmpNe => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let rhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lv = get_reg(vm, frame_idx, lhs)?;
                let rv = get_reg(vm, frame_idx, rhs)?;
                let eq = value_eq(&lv, &rv)?;
                let out = if opcode == Opcode::CmpEq { eq } else { !eq };
                set_reg(vm, frame_idx, dst, Value::Bool(out))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::CmpI32Lt | Opcode::CmpI32Le => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let rhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let l = as_i32(get_reg(vm, frame_idx, lhs)?)?;
                let r = as_i32(get_reg(vm, frame_idx, rhs)?)?;
                let out = if opcode == Opcode::CmpI32Lt {
                    l < r
                } else {
                    l <= r
                };
                set_reg(vm, frame_idx, dst, Value::Bool(out))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::AddF64 | Opcode::SubF64 | Opcode::MulF64 | Opcode::DivF64 => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let rhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let l = as_f64(get_reg(vm, frame_idx, lhs)?)?;
                let r = as_f64(get_reg(vm, frame_idx, rhs)?)?;
                let out = match opcode {
                    Opcode::AddF64 => l + r,
                    Opcode::SubF64 => l - r,
                    Opcode::MulF64 => l * r,
                    Opcode::DivF64 => l / r,
                    _ => unreachable!(),
                };
                set_reg(vm, frame_idx, dst, Value::F64(out))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::AddFx | Opcode::SubFx | Opcode::MulFx | Opcode::DivFx => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let rhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let l = as_fx(get_reg(vm, frame_idx, lhs)?)?;
                let r = as_fx(get_reg(vm, frame_idx, rhs)?)?;
                let out = match opcode {
                    Opcode::AddFx => fx_add_raw(l, r)?,
                    Opcode::SubFx => fx_sub_raw(l, r)?,
                    Opcode::MulFx => fx_mul_raw(l, r)?,
                    Opcode::DivFx => fx_div_raw(l, r)?,
                    _ => unreachable!(),
                };
                set_reg(vm, frame_idx, dst, Value::Fx(out))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::Jmp => {
                let addr = read_u32_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                if addr >= instr_rel_len {
                    return Err(RuntimeError::InvalidJumpAddress {
                        func: func_name,
                        addr,
                    });
                }
                next_pc = addr;
            }
            Opcode::JmpIf => {
                let cond = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let addr = read_u32_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                let b = as_bool(get_reg(vm, frame_idx, cond)?)?;
                if b {
                    if addr >= instr_rel_len {
                        return Err(RuntimeError::InvalidJumpAddress {
                            func: func_name,
                            addr,
                        });
                    }
                    next_pc = addr;
                } else {
                    next_pc = cur - f.instr_start;
                }
            }
            Opcode::Call => {
                let has_dst = read_u8(&f.code, &mut cur).map_err(map_format_err)? != 0;
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let callee_sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let argc = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                let callee = lookup_str(&f, callee_sid)?.to_string();
                let mut args = Vec::with_capacity(argc);
                for _ in 0..argc {
                    let r = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                    args.push(get_reg(vm, frame_idx, r)?);
                }
                if let Some(result) = try_eval_builtin_call(&callee, &args)? {
                    if has_dst {
                        set_reg(vm, frame_idx, dst, result)?;
                    }
                    next_pc = cur - f.instr_start;
                } else {
                    vm.callstack[frame_idx].pc = cur - f.instr_start;
                    push_frame(vm, &callee, args, if has_dst { Some(dst) } else { None })?;
                    continue;
                }
            }
            Opcode::ClosureCall => {
                let has_dst = read_u8(&f.code, &mut cur).map_err(map_format_err)? != 0;
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let closure_reg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let arg_reg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let closure = get_reg(vm, frame_idx, closure_reg)?;
                let Value::Closure(closure) = closure else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "CLOSURE_CALL source must be closure".to_string(),
                    ));
                };
                let mut args = closure.captures;
                args.push(get_reg(vm, frame_idx, arg_reg)?);
                vm.callstack[frame_idx].pc = cur - f.instr_start;
                push_frame(
                    vm,
                    &closure.function_name,
                    args,
                    if has_dst { Some(dst) } else { None },
                )?;
                continue;
            }
            Opcode::Assert => {
                let cond = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                match get_reg(vm, frame_idx, cond)? {
                    Value::Bool(true) => {
                        next_pc = cur - f.instr_start;
                    }
                    Value::Bool(false) => {
                        return Err(RuntimeError::Trap(RuntimeTrap::AssertionFailed));
                    }
                    other => {
                        return Err(RuntimeError::TypeMismatchRuntime(format!(
                            "ASSERT requires bool register, got {:?}",
                            other
                        )));
                    }
                }
            }
            Opcode::GateRead => {
                bump_effect_calls(vm)?;
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let device_id = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let port = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let value = host.gate_read(device_id, port)?;
                set_reg(vm, frame_idx, dst, value)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::GateWrite => {
                bump_effect_calls(vm)?;
                let device_id = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let port = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let value = get_reg(vm, frame_idx, src)?;
                host.gate_write(device_id, port, value)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::PulseEmit => {
                bump_effect_calls(vm)?;
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let signal = lookup_str(&f, sid)?;
                host.pulse_emit(signal)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::StateQuery => {
                bump_effect_calls(vm)?;
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let key = lookup_str(&f, sid)?;
                let value = host.state_query(key)?;
                set_reg(vm, frame_idx, dst, value)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::StateUpdate => {
                bump_effect_calls(vm)?;
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let key = lookup_str(&f, sid)?;
                let value = get_reg(vm, frame_idx, src)?;
                host.state_update(key, value)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::EventPost => {
                bump_effect_calls(vm)?;
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let signal = lookup_str(&f, sid)?;
                host.event_post(signal)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::ClockRead => {
                bump_effect_calls(vm)?;
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let value = host.clock_read()?;
                set_reg(vm, frame_idx, dst, value)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::Ret => {
                let has_src = read_u8(&f.code, &mut cur).map_err(map_format_err)? != 0;
                let ret_val = if has_src {
                    let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                    get_reg(vm, frame_idx, src)?
                } else {
                    Value::Unit
                };
                let finished = vm.callstack.pop().ok_or(RuntimeError::StackUnderflow)?;
                if let Some(caller) = vm.callstack.last_mut() {
                    if let Some(dst) = finished.return_dst {
                        write_reg(caller, dst as usize, ret_val, &vm.config.quotas)?;
                    }
                } else {
                    return Ok(());
                }
                continue;
            }
        }

        vm.callstack[frame_idx].pc = next_pc;
    }
}

fn value_to_abi(value: Value) -> Result<AbiValue, RuntimeError> {
    match value {
        Value::Quad(q) => Ok(AbiValue::Quad(quad_to_u8(q))),
        Value::Bool(v) => Ok(AbiValue::Bool(v)),
        Value::Text(_) => Err(RuntimeError::TypeMismatchRuntime(
            "text values are not part of the PROMETHEUS host ABI surface".to_string(),
        )),
        Value::Sequence(_) => Err(RuntimeError::TypeMismatchRuntime(
            "sequence values are not part of the PROMETHEUS host ABI surface".to_string(),
        )),
        Value::Map(_) => Err(RuntimeError::TypeMismatchRuntime(
            "map values are not part of the PROMETHEUS host ABI surface".to_string(),
        )),
        Value::Closure(_) => Err(RuntimeError::TypeMismatchRuntime(
            "closure values are not part of the PROMETHEUS host ABI surface".to_string(),
        )),
        Value::I32(v) => Ok(AbiValue::I32(v)),
        Value::F64(v) => Ok(AbiValue::F64(v)),
        Value::U32(v) => Ok(AbiValue::U32(v)),
        Value::Fx(v) => Ok(AbiValue::Fx(v)),
        Value::Tuple(_) => Err(RuntimeError::TypeMismatchRuntime(
            "tuple values are not part of the PROMETHEUS host ABI surface".to_string(),
        )),
        Value::Record(_) => Err(RuntimeError::TypeMismatchRuntime(
            "record values are not part of the PROMETHEUS host ABI surface".to_string(),
        )),
        Value::Adt(_) => Err(RuntimeError::TypeMismatchRuntime(
            "enum values are not part of the PROMETHEUS host ABI surface".to_string(),
        )),
        Value::Unit => Ok(AbiValue::Unit),
    }
}

fn value_from_abi(value: AbiValue) -> Value {
    match value {
        AbiValue::Quad(q) => Value::Quad(u8_to_quad(q)),
        AbiValue::Bool(v) => Value::Bool(v),
        AbiValue::I32(v) => Value::I32(v),
        AbiValue::F64(v) => Value::F64(v),
        AbiValue::U32(v) => Value::U32(v),
        AbiValue::Fx(v) => Value::Fx(v),
        AbiValue::Unit => Value::Unit,
    }
}

fn stable_state_query_fallback(key: &str) -> i32 {
    key.bytes().fold(0i32, |acc, byte| {
        acc.wrapping_mul(31).wrapping_add(i32::from(byte))
    })
}

fn push_frame(
    vm: &mut VM,
    func_name: &str,
    args: Vec<Value>,
    return_dst: Option<u16>,
) -> Result<(), RuntimeError> {
    let f = vm
        .functions
        .get(func_name)
        .ok_or_else(|| RuntimeError::UnknownFunction(func_name.to_string()))?;
    let next_depth = vm.callstack.len() + 1;
    enforce_quota(&vm.config.quotas, QuotaKind::Frames, next_depth)?;
    match enforce_quota(&vm.config.quotas, QuotaKind::StackDepth, next_depth) {
        Err(RuntimeError::QuotaExceeded(_)) => return Err(RuntimeError::StackOverflow),
        other => other?,
    }
    let initial_reg_count = 16usize.max(args.len());
    enforce_quota(&vm.config.quotas, QuotaKind::Registers, initial_reg_count)?;
    let mut regs = vec![Value::Unit; initial_reg_count];
    for (i, v) in args.into_iter().enumerate() {
        regs[i] = v;
    }
    let frame = Frame {
        pc: 0,
        regs,
        locals: HashMap::new(),
        borrowed_paths: f.borrowed_paths.clone(),
        next_write_path: 0,
        func: f.name.clone(),
        return_dst,
    };
    vm.callstack.push(frame);
    Ok(())
}

fn access_paths_overlap(lhs: &AccessPath, rhs: &AccessPath) -> bool {
    if lhs.root != rhs.root {
        return false;
    }
    let shared_len = lhs.components.len().min(rhs.components.len());
    lhs.components[..shared_len] == rhs.components[..shared_len]
}

fn ensure_write_path_allowed(
    _symbol_name: &str,
    write_path: &AccessPath,
    borrowed_paths: &[AccessPath],
) -> Result<(), RuntimeError> {
    if borrowed_paths
        .iter()
        .any(|borrowed_path| access_paths_overlap(write_path, borrowed_path))
    {
        return Err(RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict));
    }
    Ok(())
}

fn lookup_str<'a>(f: &'a FunctionBytecode, sid: u16) -> Result<&'a str, RuntimeError> {
    f.strings
        .get(sid as usize)
        .map(|s| s.as_str())
        .ok_or(RuntimeError::InvalidStringId(sid))
}

fn lookup_symbol(f: &FunctionBytecode, sid: u16) -> Result<SymbolId, RuntimeError> {
    f.symbol_ids
        .get(sid as usize)
        .copied()
        .ok_or(RuntimeError::InvalidStringId(sid))
}

fn get_reg(vm: &VM, frame_idx: usize, r: u16) -> Result<Value, RuntimeError> {
    vm.callstack
        .get(frame_idx)
        .and_then(|fr| fr.regs.get(r as usize))
        .cloned()
        .ok_or_else(|| RuntimeError::BadFormat(format!("read invalid reg r{}", r)))
}

fn set_reg(vm: &mut VM, frame_idx: usize, r: u16, v: Value) -> Result<(), RuntimeError> {
    if let Some(frame) = vm.callstack.get_mut(frame_idx) {
        write_reg(frame, r as usize, v, &vm.config.quotas)?;
    }
    Ok(())
}

fn write_reg(
    frame: &mut Frame,
    r: usize,
    v: Value,
    quotas: &RuntimeQuotas,
) -> Result<(), RuntimeError> {
    if frame.regs.len() <= r {
        let required = r + 1;
        enforce_quota(quotas, QuotaKind::Registers, required)?;
        frame.regs.resize(required, Value::Unit);
    }
    frame.regs[r] = v;
    Ok(())
}

fn enforce_quota(quotas: &RuntimeQuotas, kind: QuotaKind, used: usize) -> Result<(), RuntimeError> {
    if let Some(exceeded) = quotas.exceed(kind, used) {
        return Err(RuntimeError::QuotaExceeded(exceeded));
    }
    Ok(())
}

fn bump_effect_calls(vm: &mut VM) -> Result<(), RuntimeError> {
    let next = vm.effect_calls + 1;
    enforce_quota(&vm.config.quotas, QuotaKind::EffectCalls, next)?;
    vm.effect_calls = next;
    Ok(())
}

fn as_quad(v: Value) -> Result<QuadVal, RuntimeError> {
    if let Value::Quad(q) = v {
        Ok(q)
    } else {
        Err(RuntimeError::TypeMismatchRuntime(
            "expected quad".to_string(),
        ))
    }
}

fn as_bool(v: Value) -> Result<bool, RuntimeError> {
    if let Value::Bool(b) = v {
        Ok(b)
    } else {
        Err(RuntimeError::TypeMismatchRuntime(
            "expected bool".to_string(),
        ))
    }
}

fn as_i32(v: Value) -> Result<i32, RuntimeError> {
    if let Value::I32(x) = v {
        Ok(x)
    } else {
        Err(RuntimeError::TypeMismatchRuntime(
            "expected i32".to_string(),
        ))
    }
}

fn as_f64(v: Value) -> Result<f64, RuntimeError> {
    if let Value::F64(x) = v {
        Ok(x)
    } else {
        Err(RuntimeError::TypeMismatchRuntime(
            "expected f64".to_string(),
        ))
    }
}

fn as_fx(v: Value) -> Result<i32, RuntimeError> {
    if let Value::Fx(x) = v {
        Ok(x)
    } else {
        Err(RuntimeError::TypeMismatchRuntime("expected fx".to_string()))
    }
}

fn map_key_from_value(v: Value) -> Result<MapKey, RuntimeError> {
    match v {
        Value::I32(x) => Ok(MapKey::I32(x)),
        Value::U32(x) => Ok(MapKey::U32(x)),
        Value::Bool(x) => Ok(MapKey::Bool(x)),
        Value::Text(x) => Ok(MapKey::Text(x)),
        Value::Quad(q) => Ok(MapKey::Quad(quad_to_u8(q))),
        other => Err(RuntimeError::TypeMismatchRuntime(format!(
            "map key must be a scalar (i32, u32, bool, text, or quad), got {:?}",
            other
        ))),
    }
}

/// Advance the xorshift64 PRNG state by one step and return the new raw value.
/// If state is 0 (unseeded), treat it as seed 1 to avoid the zero fixed point.
fn xorshift64_step(state: &mut u64) -> u64 {
    if *state == 0 {
        *state = 1;
    }
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

fn fx_add_raw(lhs: i32, rhs: i32) -> Result<i32, RuntimeError> {
    i32::try_from(i64::from(lhs) + i64::from(rhs))
        .map_err(|_| RuntimeError::Trap(RuntimeTrap::ArithmeticOverflow))
}

fn fx_sub_raw(lhs: i32, rhs: i32) -> Result<i32, RuntimeError> {
    i32::try_from(i64::from(lhs) - i64::from(rhs))
        .map_err(|_| RuntimeError::Trap(RuntimeTrap::ArithmeticOverflow))
}

fn fx_mul_raw(lhs: i32, rhs: i32) -> Result<i32, RuntimeError> {
    let scaled = (i64::from(lhs) * i64::from(rhs)) / 1_000;
    i32::try_from(scaled).map_err(|_| RuntimeError::Trap(RuntimeTrap::ArithmeticOverflow))
}

fn fx_div_raw(lhs: i32, rhs: i32) -> Result<i32, RuntimeError> {
    if rhs == 0 {
        return Err(RuntimeError::Trap(RuntimeTrap::DivisionByZero));
    }
    let scaled = (i64::from(lhs) * 1_000) / i64::from(rhs);
    i32::try_from(scaled).map_err(|_| RuntimeError::Trap(RuntimeTrap::ArithmeticOverflow))
}

fn i32_div_raw(lhs: i32, rhs: i32) -> Result<i32, RuntimeError> {
    if rhs == 0 {
        return Err(RuntimeError::Trap(RuntimeTrap::DivisionByZero));
    }
    lhs.checked_div(rhs)
        .ok_or(RuntimeError::Trap(RuntimeTrap::ArithmeticOverflow))
}

fn i32_mod_raw(lhs: i32, rhs: i32) -> Result<i32, RuntimeError> {
    if rhs == 0 {
        return Err(RuntimeError::Trap(RuntimeTrap::DivisionByZero));
    }
    lhs.checked_rem(rhs)
        .ok_or(RuntimeError::Trap(RuntimeTrap::ArithmeticOverflow))
}

fn quad_to_u8(q: QuadVal) -> u8 {
    match q {
        QuadVal::N => 0,
        QuadVal::F => 1,
        QuadVal::T => 2,
        QuadVal::S => 3,
    }
}

fn u8_to_quad(v: u8) -> QuadVal {
    match v & 0b11 {
        0 => QuadVal::N,
        1 => QuadVal::F,
        2 => QuadVal::T,
        _ => QuadVal::S,
    }
}

fn quad_and(a: QuadVal, b: QuadVal) -> QuadVal {
    u8_to_quad(quad_to_u8(a) & quad_to_u8(b))
}

fn quad_or(a: QuadVal, b: QuadVal) -> QuadVal {
    u8_to_quad(quad_to_u8(a) | quad_to_u8(b))
}

fn quad_not(a: QuadVal) -> QuadVal {
    let v = quad_to_u8(a);
    let r = ((v & 0b10) >> 1) | ((v & 0b01) << 1);
    u8_to_quad(r)
}

fn value_eq(a: &Value, b: &Value) -> Result<bool, RuntimeError> {
    match (a, b) {
        (Value::Quad(x), Value::Quad(y)) => Ok(x == y),
        (Value::Bool(x), Value::Bool(y)) => Ok(x == y),
        (Value::Text(x), Value::Text(y)) => Ok(x == y),
        (Value::Sequence(xs), Value::Sequence(ys)) => {
            if xs.len() != ys.len() {
                return Ok(false);
            }
            for (x, y) in xs.iter().zip(ys.iter()) {
                if !value_eq(x, y)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (Value::Map(_), Value::Map(_)) => Err(RuntimeError::TypeMismatchRuntime(
            "Map values are not comparable with == / !=".to_string(),
        )),
        (Value::Closure(_), Value::Closure(_)) => Err(RuntimeError::TypeMismatchRuntime(
            "closure values are not comparable with CmpEq/CmpNe".to_string(),
        )),
        (Value::I32(x), Value::I32(y)) => Ok(x == y),
        (Value::F64(x), Value::F64(y)) => Ok(x == y),
        (Value::U32(x), Value::U32(y)) => Ok(x == y),
        (Value::Fx(x), Value::Fx(y)) => Ok(x == y),
        (Value::Tuple(xs), Value::Tuple(ys)) => {
            if xs.len() != ys.len() {
                return Ok(false);
            }
            for (x, y) in xs.iter().zip(ys.iter()) {
                if !value_eq(x, y)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (Value::Record(xs), Value::Record(ys)) => {
            if xs.type_name != ys.type_name {
                return Ok(false);
            }
            if xs.slots.len() != ys.slots.len() {
                return Ok(false);
            }
            for (x, y) in xs.slots.iter().zip(ys.slots.iter()) {
                if !value_eq(x, y)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (Value::Adt(xs), Value::Adt(ys)) => {
            if xs.type_name != ys.type_name
                || xs.variant_name != ys.variant_name
                || xs.tag != ys.tag
                || xs.payload.len() != ys.payload.len()
            {
                return Ok(false);
            }
            for (x, y) in xs.payload.iter().zip(ys.payload.iter()) {
                if !value_eq(x, y)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (Value::Unit, Value::Unit) => Ok(true),
        _ => Err(RuntimeError::TypeMismatchRuntime(
            "CmpEq/CmpNe operands must have same runtime type".to_string(),
        )),
    }
}

fn try_eval_builtin_call(name: &str, args: &[Value]) -> Result<Option<Value>, RuntimeError> {
    let value = match name {
        "sin" => Value::F64(expect_builtin_unary_f64(name, args)?.sin()),
        "cos" => Value::F64(expect_builtin_unary_f64(name, args)?.cos()),
        "tan" => Value::F64(expect_builtin_unary_f64(name, args)?.tan()),
        "sqrt" => Value::F64(expect_builtin_unary_f64(name, args)?.sqrt()),
        "abs" => Value::F64(expect_builtin_unary_f64(name, args)?.abs()),
        "pow" => {
            let (lhs, rhs) = expect_builtin_binary_f64(name, args)?;
            Value::F64(lhs.powf(rhs))
        }
        "to_text" => Value::Text(value_to_text(expect_builtin_to_text_arg(name, args)?)?),
        _ => return Ok(None),
    };
    Ok(Some(value))
}

fn decode_text_literal(raw: &str) -> String {
    raw.strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(raw)
        .to_string()
}

fn as_text(v: Value) -> Result<String, RuntimeError> {
    if let Value::Text(x) = v {
        Ok(x)
    } else {
        Err(RuntimeError::TypeMismatchRuntime("expected text".to_string()))
    }
}

fn value_to_text(v: Value) -> Result<String, RuntimeError> {
    match v {
        Value::Text(x) => Ok(x),
        Value::Bool(b) => Ok(if b { "true" } else { "false" }.to_string()),
        Value::I32(x) => Ok(x.to_string()),
        Value::U32(x) => Ok(x.to_string()),
        Value::Quad(q) => Ok(match q {
            QuadVal::N => "N",
            QuadVal::T => "T",
            QuadVal::F => "F",
            QuadVal::S => "S",
        }
        .to_string()),
        other => Err(RuntimeError::TypeMismatchRuntime(format!(
            "builtin 'to_text' currently supports text, bool, i32, u32, and quad; got {:?}",
            other
        ))),
    }
}

fn expect_builtin_to_text_arg(name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeMismatchRuntime(format!(
            "builtin '{name}' expects 1 argument, got {}",
            args.len()
        )));
    }
    Ok(args[0].clone())
}

fn expect_builtin_unary_f64(name: &str, args: &[Value]) -> Result<f64, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeMismatchRuntime(format!(
            "builtin '{name}' expects 1 f64 argument, got {}",
            args.len()
        )));
    }
    as_f64(args[0].clone())
}

fn expect_builtin_binary_f64(name: &str, args: &[Value]) -> Result<(f64, f64), RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeMismatchRuntime(format!(
            "builtin '{name}' expects 2 f64 arguments, got {}",
            args.len()
        )));
    }
    Ok((as_f64(args[0].clone())?, as_f64(args[1].clone())?))
}

fn disasm_one(f: &FunctionBytecode, pc: usize) -> Result<(String, usize), RuntimeError> {
    let mut cur = f.instr_start + pc;
    let opcode = Opcode::from_byte(read_u8(&f.code, &mut cur).map_err(map_format_err)?)
        .map_err(map_format_err)?;
    let text = match opcode {
        Opcode::LoadQ => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let q = read_u8(&f.code, &mut cur).map_err(map_format_err)?;
            format!("LOAD_Q r{}, {}", d, q)
        }
        Opcode::LoadBool => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let b = read_u8(&f.code, &mut cur).map_err(map_format_err)?;
            format!("LOAD_BOOL r{}, {}", d, b)
        }
        Opcode::LoadText => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let text = lookup_str(f, sid)?;
            format!("LOAD_TEXT r{}, {:?}", d, text)
        }
        Opcode::ConcatText => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let l = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let r = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("CONCAT_TEXT r{}, r{}, r{}", d, l, r)
        }
        Opcode::LoadI32 => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let n = read_i32_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("LOAD_I32 r{}, {}", d, n)
        }
        Opcode::AddI32 => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let l = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let r = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("ADD_I32 r{}, r{}, r{}", d, l, r)
        }
        Opcode::SubI32 => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let l = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let r = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("SUB_I32 r{}, r{}, r{}", d, l, r)
        }
        Opcode::MulI32 => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let l = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let r = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("MUL_I32 r{}, r{}, r{}", d, l, r)
        }
        Opcode::DivI32 => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let l = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let r = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("DIV_I32 r{}, r{}, r{}", d, l, r)
        }
        Opcode::ModI32 => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let l = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let r = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("MOD_I32 r{}, r{}, r{}", d, l, r)
        }
        Opcode::LoadU32 => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let n = read_u32_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("LOAD_U32 r{}, {}", d, n)
        }
        Opcode::LoadF64 => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let n = read_f64_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("LOAD_F64 r{}, {}", d, n)
        }
        Opcode::LoadFx => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let n = read_i32_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("LOAD_FX r{}, raw:{}", d, n)
        }
        Opcode::MakeSequence => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
            let mut regs = Vec::with_capacity(count);
            for _ in 0..count {
                regs.push(read_u16_le(&f.code, &mut cur).map_err(map_format_err)?);
            }
            let regs = regs
                .iter()
                .map(|reg| format!("r{}", reg))
                .collect::<Vec<_>>()
                .join(", ");
            format!("MAKE_SEQUENCE r{}, [{}]", d, regs)
        }
        Opcode::MakeClosure => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
            let mut regs = Vec::with_capacity(count);
            for _ in 0..count {
                regs.push(read_u16_le(&f.code, &mut cur).map_err(map_format_err)?);
            }
            let regs = regs
                .iter()
                .map(|reg| format!("r{}", reg))
                .collect::<Vec<_>>()
                .join(", ");
            let target = lookup_str(f, sid)?;
            format!("MAKE_CLOSURE r{}, {}, [{}]", d, target, regs)
        }
        Opcode::AddFx | Opcode::SubFx | Opcode::MulFx | Opcode::DivFx => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let l = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let r = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let op = match opcode {
                Opcode::AddFx => "ADD_FX",
                Opcode::SubFx => "SUB_FX",
                Opcode::MulFx => "MUL_FX",
                Opcode::DivFx => "DIV_FX",
                _ => unreachable!(),
            };
            format!("{} r{}, r{}, r{}", op, d, l, r)
        }
        Opcode::MakeTuple => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
            let mut regs = Vec::with_capacity(count);
            for _ in 0..count {
                regs.push(read_u16_le(&f.code, &mut cur).map_err(map_format_err)?);
            }
            let regs = regs
                .iter()
                .map(|reg| format!("r{}", reg))
                .collect::<Vec<_>>()
                .join(", ");
            format!("MAKE_TUPLE r{}, [{}]", d, regs)
        }
        Opcode::MakeRecord => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
            let mut regs = Vec::with_capacity(count);
            for _ in 0..count {
                regs.push(read_u16_le(&f.code, &mut cur).map_err(map_format_err)?);
            }
            let regs = regs
                .iter()
                .map(|reg| format!("r{}", reg))
                .collect::<Vec<_>>()
                .join(", ");
            let name = lookup_str(f, sid)?;
            format!("MAKE_RECORD r{}, {}, [{}]", d, name, regs)
        }
        Opcode::MakeAdt => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let variant_sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let tag = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
            let mut regs = Vec::with_capacity(count);
            for _ in 0..count {
                regs.push(read_u16_le(&f.code, &mut cur).map_err(map_format_err)?);
            }
            let regs = regs
                .iter()
                .map(|reg| format!("r{}", reg))
                .collect::<Vec<_>>()
                .join(", ");
            let name = lookup_str(f, sid)?;
            let variant = lookup_str(f, variant_sid)?;
            format!(
                "MAKE_ADT r{}, {}::{}, tag={}, [{}]",
                d, name, variant, tag, regs
            )
        }
        Opcode::AdtTag => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let name = lookup_str(f, sid)?;
            format!("ADT_TAG r{}, r{}, {}", d, s, name)
        }
        Opcode::AdtGet => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let i = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let name = lookup_str(f, sid)?;
            format!("ADT_GET r{}, r{}, {}, {}", d, s, name, i)
        }
        Opcode::RecordGet => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let i = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let name = lookup_str(f, sid)?;
            format!("RECORD_GET r{}, r{}, {}, {}", d, s, name, i)
        }
        Opcode::TupleGet => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let i = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("TUPLE_GET r{}, r{}, {}", d, s, i)
        }
        Opcode::SequenceGet => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let i = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("SEQUENCE_GET r{}, r{}, r{}", d, s, i)
        }
        Opcode::SequenceLen => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("SEQUENCE_LEN r{}, r{}", d, s)
        }
        Opcode::SequenceIsEmpty => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("SEQUENCE_IS_EMPTY r{}, r{}", d, s)
        }
        Opcode::SequenceContains => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let v = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("SEQUENCE_CONTAINS r{}, r{}, r{}", d, s, v)
        }
        Opcode::SequencePush => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let v = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("SEQUENCE_PUSH r{}, r{}, r{}", d, s, v)
        }
        Opcode::SequencePrepend => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let v = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("SEQUENCE_PREPEND r{}, r{}, r{}", d, s, v)
        }
        Opcode::SequencePop => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("SEQUENCE_POP r{}, r{}", d, s)
        }
        Opcode::MapEmpty => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("MAP_EMPTY r{}", d)
        }
        Opcode::MapContains => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let m = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let k = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("MAP_CONTAINS r{}, r{}, r{}", d, m, k)
        }
        Opcode::MapGet => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let m = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let k = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let dv = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("MAP_GET r{}, r{}, r{}, r{}", d, m, k, dv)
        }
        Opcode::MapSet => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let m = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let k = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let v = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("MAP_SET r{}, r{}, r{}, r{}", d, m, k, v)
        }
        Opcode::RngSeed => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("RNG_SEED r{}, r{}", d, s)
        }
        Opcode::RngNextI32 => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let lo = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let hi = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("RNG_NEXT_I32 r{}, r{}, r{}", d, lo, hi)
        }
        Opcode::ClosureCall => {
            let has_dst = read_u8(&f.code, &mut cur).map_err(map_format_err)? != 0;
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let closure = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let arg = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            if has_dst {
                format!("CLOSURE_CALL r{}, r{}, r{}", d, closure, arg)
            } else {
                format!("CLOSURE_CALL -, r{}, r{}", closure, arg)
            }
        }
        Opcode::LoadVar => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("LOAD_VAR r{}, s{}", d, s)
        }
        Opcode::StoreVar => {
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let r = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("STORE_VAR s{}, r{}", s, r)
        }
        Opcode::QAnd
        | Opcode::QOr
        | Opcode::QImpl
        | Opcode::BoolAnd
        | Opcode::BoolOr
        | Opcode::CmpI32Lt
        | Opcode::CmpI32Le
        | Opcode::AddF64
        | Opcode::SubF64
        | Opcode::MulF64
        | Opcode::DivF64
        | Opcode::CmpEq
        | Opcode::CmpNe => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let l = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let r = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let op = match opcode {
                Opcode::QAnd => "Q_AND",
                Opcode::QOr => "Q_OR",
                Opcode::QImpl => "Q_IMPL",
                Opcode::BoolAnd => "BOOL_AND",
                Opcode::BoolOr => "BOOL_OR",
                Opcode::CmpI32Lt => "CMP_I32_LT",
                Opcode::CmpI32Le => "CMP_I32_LE",
                Opcode::AddI32 => "ADD_I32",
                Opcode::SubI32 => "SUB_I32",
                Opcode::MulI32 => "MUL_I32",
                Opcode::DivI32 => "DIV_I32",
                Opcode::ModI32 => "MOD_I32",
                Opcode::AddF64 => "ADD_F64",
                Opcode::SubF64 => "SUB_F64",
                Opcode::MulF64 => "MUL_F64",
                Opcode::DivF64 => "DIV_F64",
                Opcode::AddFx => "ADD_FX",
                Opcode::SubFx => "SUB_FX",
                Opcode::MulFx => "MUL_FX",
                Opcode::DivFx => "DIV_FX",
                Opcode::CmpEq => "CMP_EQ",
                _ => "CMP_NE",
            };
            format!("{} r{}, r{}, r{}", op, d, l, r)
        }
        Opcode::QNot | Opcode::BoolNot => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let op = if opcode == Opcode::QNot {
                "Q_NOT"
            } else {
                "BOOL_NOT"
            };
            format!("{} r{}, r{}", op, d, s)
        }
        Opcode::Jmp => {
            let a = read_u32_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("JMP {}", a)
        }
        Opcode::JmpIf => {
            let c = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let a = read_u32_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("JMP_IF r{}, {}", c, a)
        }
        Opcode::Call => {
            let has = read_u8(&f.code, &mut cur).map_err(map_format_err)?;
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let n = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let argc = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
            for _ in 0..argc {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            format!("CALL dst?{} r{} fn#{} argc={}", has, d, n, argc)
        }
        Opcode::Assert => {
            let r = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("ASSERT r{}", r)
        }
        Opcode::GateRead => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let dev = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let port = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("GATE_READ r{}, dev={}, port={}", d, dev, port)
        }
        Opcode::GateWrite => {
            let dev = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let port = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("GATE_WRITE dev={}, port={}, r{}", dev, port, s)
        }
        Opcode::PulseEmit => {
            let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("PULSE_EMIT s{}", sid)
        }
        Opcode::StateQuery => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("STATE_QUERY r{}, s{}", d, sid)
        }
        Opcode::StateUpdate => {
            let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("STATE_UPDATE s{}, r{}", sid, src)
        }
        Opcode::EventPost => {
            let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("EVENT_POST s{}", sid)
        }
        Opcode::ClockRead => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("CLOCK_READ r{}", d)
        }
        Opcode::Ret => {
            let has = read_u8(&f.code, &mut cur).map_err(map_format_err)?;
            if has != 0 {
                let r = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                format!("RET r{}", r)
            } else {
                "RET".to_string()
            }
        }
    };
    Ok((text, cur - f.instr_start))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sm_emit::compile_program_to_semcode;
    use sm_runtime_core::{
        ExecutionConfig, ExecutionContext, PathComponent, QuotaExceeded, QuotaKind, RuntimeTrap,
    };

    #[test]
    fn vm_runs_empty_main() {
        let src = "fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_assert_statement_when_condition_holds() {
        let src = r#"
            fn main() {
                assert(true);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        run_semcode(&bytes).expect("assert(true) should pass");
    }

    #[test]
    fn vm_traps_on_failed_assert() {
        let src = r#"
            fn main() {
                assert(false);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let err = run_semcode(&bytes).expect_err("assert(false) should trap");
        assert!(matches!(
            err,
            RuntimeError::Trap(RuntimeTrap::AssertionFailed)
        ));
    }

    #[test]
    fn vm_runs_function_requires_clause_when_condition_holds() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn decide(ctx: DecisionContext, expected: quad) -> quad
                requires(ctx.camera == expected)
                requires(ctx.quality == 0.75) {
                return ctx.camera;
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let seen: quad = decide(ctx, T);
                assert(seen == T);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        run_semcode(&bytes).expect("requires clause should pass");
    }

    #[test]
    fn vm_traps_on_failed_function_requires_clause() {
        let src = r#"
            fn must_be_true(flag: bool) -> bool requires(flag == true) {
                return flag;
            }

            fn main() {
                let seen: bool = must_be_true(false);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let err = run_semcode(&bytes).expect_err("requires clause should trap");
        assert!(matches!(
            err,
            RuntimeError::Trap(RuntimeTrap::AssertionFailed)
        ));
    }

    #[test]
    fn vm_runs_function_ensures_clause_when_condition_holds() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn decide(ctx: DecisionContext) -> quad
                ensures(result == ctx.camera)
                ensures(ctx.quality == 0.75) {
                return ctx.camera;
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let seen: quad = decide(ctx);
                assert(seen == T);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        run_semcode(&bytes).expect("ensures clause should pass");
    }

    #[test]
    fn vm_traps_on_failed_function_ensures_clause() {
        let src = r#"
            fn must_return_true(flag: bool) -> bool ensures(result == true) {
                return flag;
            }

            fn main() {
                let seen: bool = must_return_true(false);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let err = run_semcode(&bytes).expect_err("ensures clause should trap");
        assert!(matches!(
            err,
            RuntimeError::Trap(RuntimeTrap::AssertionFailed)
        ));
    }

    #[test]
    fn vm_runs_function_invariant_clauses_when_conditions_hold() {
        let src = r#"
            fn keep(flag: bool) -> bool
                invariant(flag == true)
                invariant(result == flag) {
                return flag;
            }

            fn main() {
                let seen: bool = keep(true);
                assert(seen == true);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        run_semcode(&bytes).expect("invariant clauses should pass");
    }

    #[test]
    fn vm_traps_on_failed_function_invariant_clause() {
        let src = r#"
            fn must_stay_true(flag: bool) -> bool invariant(result == true) {
                return flag;
            }

            fn main() {
                let seen: bool = must_stay_true(false);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let err = run_semcode(&bytes).expect_err("invariant clause should trap");
        assert!(matches!(
            err,
            RuntimeError::Trap(RuntimeTrap::AssertionFailed)
        ));
    }

    #[test]
    fn vm_runs_bool_ops() {
        let src = r#"
			fn main() {
				let a: bool = true;
				let b: bool = false;
				let c = a && b;
				if c == false { return; } else { return; }
			}
		"#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_quad_ops() {
        let src = r#"
			fn main() {
				let a: quad = T;
				let b: quad = S;
				let c = a && b;
				if c == T { return; } else { return; }
			}
		"#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_call_ret() {
        let src = r#"
			fn one() -> i32 { return 1; }
			fn main() { let x: i32 = one(); return; }
		"#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_fx_literal_call_and_compare_path() {
        let src = r#"
            fn id(x: fx) -> fx { return x; }

            fn make() -> fx {
                return -1.25;
            }

            fn main() {
                let x: fx = 1.25;
                let y: fx = id(2);
                let z: fx = make();
                let a = x == x;
                let b = y != z;
                if a == b { return; } else { return; }
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("LOAD_FX"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_plain_fx_arithmetic_path() {
        let src = r#"
            fn main() {
                let a: fx = 2.5;
                let b: fx = 1.5;
                let sum: fx = a + b;
                let diff: fx = a - b;
                let prod: fx = a * b;
                let quo: fx = a / b;
                let neg: fx = -a;
                let expected_sum: fx = 4.0;
                let expected_diff: fx = 1.0;
                let expected_prod: fx = 3.75;
                let expected_quo: fx = 1.666;
                let expected_neg: fx = -2.5;
                assert(sum == expected_sum);
                assert(diff == expected_diff);
                assert(prod == expected_prod);
                assert(quo == expected_quo);
                assert(neg == expected_neg);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("ADD_FX"));
        assert!(disasm.contains("SUB_FX"));
        assert!(disasm.contains("MUL_FX"));
        assert!(disasm.contains("DIV_FX"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_text_literal_and_equality_path() {
        let src = r#"
            fn echo(x: text) -> text { return x; }

            fn main() {
                let a: text = "alpha";
                let b: text = echo("alpha");
                assert(a == b);
                assert(a != "beta");
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("LOAD_TEXT"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_sequence_literal_index_and_equality_path() {
        let src = r#"
            fn main() {
                let values: Sequence(i32) = [1, 2, 3];
                let head: i32 = values[0];
                assert(head == 1);
                assert(values == [1, 2, 3]);
                assert(values != [1, 2, 4]);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("MAKE_SEQUENCE"));
        assert!(disasm.contains("SEQUENCE_GET"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_first_class_closure_direct_invocation_path() {
        let src = r#"
            fn main() {
                let offset: f64 = 1.0;
                let add: Closure(f64 -> f64) = (x => x + offset);
                let total: f64 = add(2.0);
                assert(total == 3.0);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("MAKE_CLOSURE"));
        assert!(disasm.contains("CLOSURE_CALL"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_traps_on_fx_division_by_zero() {
        let src = r#"
            fn main() {
                let a: fx = 1.0;
                let b: fx = 0.0;
                let bad: fx = a / b;
                assert(bad == a);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let err = run_semcode(&bytes).expect_err("fx division by zero should trap");
        assert!(matches!(
            err,
            RuntimeError::Trap(RuntimeTrap::DivisionByZero)
        ));
    }

    #[test]
    fn vm_runs_u32_literal_compare_path() {
        let src = r#"
            fn main() {
                let left: u32 = 1_000u32;
                let right: u32 = 0x3e8u32;
                assert(left == right);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("LOAD_U32"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_tuple_return_and_equality_path() {
        let src = r#"
            fn pair(flag: bool) -> (i32, bool) {
                return (1, flag);
            }

            fn main() {
                let left: (i32, bool) = pair(true);
                let right: (i32, bool) = (1, true);
                assert(left == right);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("MAKE_TUPLE"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_tuple_destructuring_bind_path() {
        let src = r#"
            fn pair(flag: bool) -> (i32, bool) = (1, flag);

            fn main() {
                let (count, ready): (i32, bool) = pair(true);
                assert(count == 1);
                assert(ready == true);
                return;
            }
        "#;

        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("TUPLE_GET"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_tracks_borrowed_paths_on_frame_push() {
        let bytes = ownership_tracking_bytes();
        let (_, symbols, functions) = parse_semcode(&bytes).expect("parse");
        let mut vm = VM {
            functions,
            callstack: Vec::new(),
            config: ExecutionConfig::for_context(ExecutionContext::VerifiedLocal),
            effect_calls: 0,
            symbols,
            prng_state: 0,
        };

        push_frame(&mut vm, "main", Vec::new(), None).expect("push frame");

        assert_eq!(vm.callstack.len(), 1);
        let frame = &vm.callstack[0];
        assert_eq!(frame.borrowed_paths.len(), 1);
        assert_eq!(
            frame.borrowed_paths[0].components,
            vec![PathComponent::TupleIndex(0)]
        );
        assert_eq!(vm.symbols.resolve(frame.borrowed_paths[0].root), Some("pair"));
    }

    #[test]
    fn vm_clears_borrowed_paths_on_frame_exit() {
        let bytes = helper_borrow_bytes();
        let (_, symbols, functions) = parse_semcode(&bytes).expect("parse");
        let mut vm = VM {
            functions,
            callstack: Vec::new(),
            config: ExecutionConfig::for_context(ExecutionContext::VerifiedLocal),
            effect_calls: 0,
            symbols,
            prng_state: 0,
        };

        push_frame(&mut vm, "main", Vec::new(), None).expect("push main");
        push_frame(
            &mut vm,
            "helper",
            vec![Value::Tuple(vec![Value::I32(1), Value::Bool(true)])],
            None,
        )
        .expect("push helper");

        assert_eq!(vm.callstack.len(), 2);
        assert_eq!(vm.callstack[1].borrowed_paths.len(), 1);

        let finished = vm.callstack.pop().expect("helper frame");
        assert_eq!(finished.borrowed_paths.len(), 1);
        assert_eq!(vm.callstack.len(), 1);
        assert!(vm.callstack[0].borrowed_paths.is_empty());
    }

    #[test]
    fn vm_tracks_record_field_borrowed_paths_on_frame_push() {
        let bytes = record_field_borrow_tracking_bytes();
        let (_, symbols, functions) = parse_semcode(&bytes).expect("parse");
        let mut vm = VM {
            functions,
            callstack: Vec::new(),
            config: ExecutionConfig::for_context(ExecutionContext::VerifiedLocal),
            effect_calls: 0,
            symbols,
            prng_state: 0,
        };

        push_frame(&mut vm, "main", Vec::new(), None).expect("push frame");

        assert_eq!(vm.callstack.len(), 1);
        let frame = &vm.callstack[0];
        assert_eq!(frame.borrowed_paths.len(), 1);
        assert!(matches!(
            frame.borrowed_paths[0].components.as_slice(),
            [PathComponent::Field(_)]
        ));
    }

    #[test]
    fn vm_clears_record_field_borrowed_paths_on_frame_exit() {
        let bytes = helper_record_field_borrow_bytes();
        let (_, symbols, functions) = parse_semcode(&bytes).expect("parse");
        let mut vm = VM {
            functions,
            callstack: Vec::new(),
            config: ExecutionConfig::for_context(ExecutionContext::VerifiedLocal),
            effect_calls: 0,
            symbols,
            prng_state: 0,
        };

        push_frame(&mut vm, "main", Vec::new(), None).expect("push main");
        push_frame(
            &mut vm,
            "helper",
            vec![Value::Record(RecordCarrier {
                type_name: "DecisionContext".to_string(),
                slots: vec![Value::Quad(QuadVal::T), Value::F64(0.75)],
            })],
            None,
        )
        .expect("push helper");

        assert_eq!(vm.callstack.len(), 2);
        assert_eq!(vm.callstack[1].borrowed_paths.len(), 1);
        assert!(matches!(
            vm.callstack[1].borrowed_paths[0].components.as_slice(),
            [PathComponent::Field(_)]
        ));

        let finished = vm.callstack.pop().expect("helper frame");
        assert_eq!(finished.borrowed_paths.len(), 1);
        assert_eq!(vm.callstack.len(), 1);
        assert!(vm.callstack[0].borrowed_paths.is_empty());
    }

    #[test]
    fn vm_rejects_write_after_borrow_same_path() {
        let bytes = ownership_write_overlap_bytes(&[0], &[0]);
        let err = run_semcode(&bytes).expect_err("overlapping write must fail");
        assert!(matches!(
            err,
            RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict)
        ));
        assert_eq!(format!("{err}"), "write path overlaps active borrow");
    }

    #[test]
    fn vm_rejects_write_when_borrowed_parent_overlaps_child_path() {
        let bytes = ownership_write_overlap_bytes(&[], &[0]);
        let err = run_semcode(&bytes).expect_err("parent-child overlap must fail");
        assert!(matches!(
            err,
            RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict)
        ));
        assert_eq!(format!("{err}"), "write path overlaps active borrow");
    }

    #[test]
    fn vm_rejects_write_when_borrowed_child_overlaps_parent_path() {
        let bytes = ownership_write_overlap_bytes(&[0], &[]);
        let err = run_semcode(&bytes).expect_err("child-parent overlap must fail");
        assert!(matches!(
            err,
            RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict)
        ));
        assert_eq!(format!("{err}"), "write path overlaps active borrow");
    }

    #[test]
    fn vm_allows_write_to_sibling_path_with_active_borrow() {
        let bytes = ownership_write_overlap_bytes(&[0], &[1]);
        run_semcode(&bytes).expect("sibling write must stay allowed");
    }

    #[test]
    fn vm_rejects_record_field_write_after_borrow_same_field() {
        let bytes = record_field_write_overlap_bytes(Some("camera"), Some("camera"));
        let err = run_semcode(&bytes).expect_err("same-field record write must fail");
        assert!(matches!(
            err,
            RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict)
        ));
        assert_eq!(format!("{err}"), "write path overlaps active borrow");
    }

    #[test]
    fn vm_rejects_record_field_write_when_borrowed_parent_overlaps_child_field() {
        let bytes = record_field_write_overlap_bytes(None, Some("camera"));
        let err = run_semcode(&bytes).expect_err("record parent-child overlap must fail");
        assert!(matches!(
            err,
            RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict)
        ));
        assert_eq!(format!("{err}"), "write path overlaps active borrow");
    }

    #[test]
    fn vm_rejects_record_parent_write_when_borrowed_child_field() {
        let bytes = record_field_write_overlap_bytes(Some("camera"), None);
        let err = run_semcode(&bytes).expect_err("record child-parent overlap must fail");
        assert!(matches!(
            err,
            RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict)
        ));
        assert_eq!(format!("{err}"), "write path overlaps active borrow");
    }

    #[test]
    fn vm_allows_record_field_write_to_sibling_field_with_active_borrow() {
        let bytes = record_field_write_overlap_bytes(Some("camera"), Some("quality"));
        run_semcode(&bytes).expect("sibling record field write must stay allowed");
    }

    #[test]
    fn vm_runs_stage1_record_literal_path() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { quality: 0.75, camera: T };
                let shadow: DecisionContext = ctx;
                let _ = shadow;
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("MAKE_RECORD"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_stage1_enum_constructor_path() {
        let src = r#"
            enum Maybe {
                None,
                Some(bool),
            }

            fn choose(flag: bool) -> Maybe {
                return Maybe::Some(flag);
            }

            fn main() {
                let left: Maybe = choose(true);
                let right: Maybe = Maybe::None;
                let _ = left;
                let _ = right;
                return;
            }
        "#;

        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("MAKE_ADT"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_option_and_result_standard_form_paths() {
        let src = r#"
            fn keep(flag: bool) -> Option(bool) {
                let fallback: Option(bool) = Option::None;
                let _ = fallback;
                return Option::Some(flag);
            }

            fn settle(flag: bool) -> Result(bool, quad) {
                if flag {
                    let value: Result(bool, quad) = Result::Ok(true);
                    return value;
                }
                let value: Result(bool, quad) = Result::Err(N);
                return value;
            }

            fn main() {
                let left: Option(bool) = keep(true);
                let right: Result(bool, quad) = settle(false);
                let _ = left;
                let _ = right;
                return;
            }
        "#;

        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("MAKE_ADT"));
        run_semcode(&bytes).expect("Option/Result standard-form paths should run");
    }

    #[test]
    fn vm_runs_option_and_result_match_ergonomics_paths() {
        let src = r#"
            fn unwrap(opt: Option(bool)) -> bool {
                let out: bool = match opt {
                    Option::Some(value) => { value }
                    Option::None => { false }
                };
                return out;
            }

            fn settle(res: Result(quad, quad)) -> quad {
                let out: quad = match res {
                    Result::Ok(value) => { value }
                    Result::Err(code) => { code }
                };
                return out;
            }

            fn main() {
                let left: bool = unwrap(Option::Some(true));
                let right: bool = unwrap(Option::None);
                let code: quad = settle(Result::Err(S));
                assert(left == true);
                assert(right == false);
                assert(code == S);
                return;
            }
        "#;

        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("ADT_TAG"));
        assert!(disasm.contains("ADT_GET"));
        run_semcode(&bytes).expect("Option/Result match ergonomics paths should run");
    }

    #[test]
    fn vm_runs_stage1_adt_match_path() {
        let src = r#"
            enum Maybe {
                None,
                Some(f64),
            }

            fn unwrap(value: Maybe) -> f64 {
                let total: f64 = match value {
                    Maybe::Some(inner) => { inner }
                    _ => { 0.0 }
                };
                return total;
            }

            fn main() {
                let total: f64 = unwrap(Maybe::Some(2.5));
                assert(total == 2.5);
                return;
            }
        "#;

        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("ADT_TAG"));
        assert!(disasm.contains("ADT_GET"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_exhaustive_adt_match_without_default_path() {
        let src = r#"
            enum Maybe {
                None,
                Some(f64),
            }

            fn unwrap(value: Maybe) -> f64 {
                let total: f64 = match value {
                    Maybe::None => { 0.0 }
                    Maybe::Some(inner) => { inner }
                };
                return total;
            }

            fn main() {
                let total: f64 = unwrap(Maybe::Some(2.5));
                assert(total == 2.5);
                return;
            }
        "#;

        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("ADT_TAG"));
        assert!(disasm.contains("ASSERT"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_stage1_record_field_access_path() {
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
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("RECORD_GET"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_record_pass_return_and_safe_equality_path() {
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
        run_semcode(&bytes).expect("record pass/return/equality should run");
    }

    #[test]
    fn vm_runs_record_access_policy_scenario() {
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
        run_semcode(&bytes).expect("record access-policy scenario should run");
    }

    #[test]
    fn vm_runs_record_destructuring_bind_path() {
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
        let disasm = disasm_semcode(&bytes).expect("disassemble");
        assert!(disasm.contains("RECORD_GET"));
        run_semcode(&bytes).expect("record destructuring bind path should run");
    }

    #[test]
    fn vm_runs_record_let_else_path() {
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
        let disasm = disasm_semcode(&bytes).expect("disassemble");
        assert!(disasm.contains("RECORD_GET"));
        run_semcode(&bytes).expect("record let-else path should run");
    }

    #[test]
    fn vm_runs_record_copy_with_path() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let patched: DecisionContext = ctx with { quality: 1.0 };
                assert(patched.camera == T);
                assert(patched.quality == 1.0);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disassemble");
        assert!(disasm.contains("RECORD_GET"));
        assert!(disasm.contains("MAKE_RECORD"));
        run_semcode(&bytes).expect("record copy-with path should run");
    }

    #[test]
    fn vm_runs_record_stage2_ergonomics_scenario() {
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
                let DecisionContext { camera, quality: _ } = ctx;
                let DecisionContext { camera: T, override_state, quality } =
                    patched else return;
                assert(camera == T);
                assert(override_state == N);
                assert(quality == 0.75);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disassemble");
        assert!(disasm.contains("RECORD_GET"));
        assert!(disasm.contains("MAKE_RECORD"));
        run_semcode(&bytes).expect("record stage-2 ergonomics scenario should run");
    }

    #[test]
    fn vm_runs_for_range_inclusive_path() {
        let src = r#"
            fn main() {
                let saw_start: bool = false;
                let saw_end: bool = false;
                for i in 0..=2 {
                    if i == 0 {
                        saw_start ||= true;
                    } else {
                        saw_start ||= false;
                    }
                    if i == 2 {
                        saw_end ||= true;
                    } else {
                        saw_end ||= false;
                    }
                }
                assert(saw_start == true);
                assert(saw_end == true);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("CMP_I32_LE"));
        assert!(disasm.contains("ADD_I32"));
        run_semcode(&bytes).expect("inclusive for-range should run");
    }

    #[test]
    fn vm_runs_for_range_empty_half_open_path() {
        let src = r#"
            fn main() {
                let visited: bool = false;
                for i in 3..3 {
                    visited ||= true;
                }
                assert(visited == false);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("CMP_I32_LT"));
        run_semcode(&bytes).expect("empty half-open for-range should skip body");
    }

    #[test]
    fn vm_runs_iterable_for_over_sequence_path() {
        let src = r#"
            fn main() {
                let items: Sequence(i32) = [1, 2, 3];
                let saw_two: bool = false;
                for item in items {
                    if item == 2 {
                        saw_two ||= true;
                    }
                }
                assert(saw_two == true);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("SEQUENCE_LEN"));
        assert!(disasm.contains("SEQUENCE_GET"));
        run_semcode(&bytes).expect("Sequence(T) iterable loop should run");
    }

    #[test]
    fn vm_runs_iterable_for_over_explicit_record_impl_path() {
        let src = r#"
            trait Iterable {
                fn next(self: Self, index: i32) -> Option(i32);
            }

            record Numbers {
                limit: i32,
            }

            impl Iterable for Numbers {
                fn next(self: Self, index: i32) -> Option(i32) {
                    let _ = self.limit;
                    if index == 0 {
                        return Option::Some(0);
                    }
                    if index == 1 {
                        return Option::Some(1);
                    }
                    if index == 2 {
                        return Option::Some(2);
                    }
                    return Option::None;
                }
            }

            fn main() {
                let numbers: Numbers = Numbers { limit: 4 };
                let saw_two: bool = false;
                for value in numbers {
                    if value == 2 {
                        saw_two ||= true;
                    }
                }
                assert(saw_two == true);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("__impl::Iterable::Numbers::next"));
        assert!(disasm.contains("ADT_TAG"));
        assert!(disasm.contains("ADT_GET"));
        run_semcode(&bytes).expect("direct record Iterable loop should run");
    }

    #[test]
    fn vm_rejects_unknown_opcode_on_load() {
        let src = "fn main() { return; }";
        let mut bytes = compile_program_to_semcode(src).expect("compile");
        let opcode_pos = 8 + 2 + 4 + 4 + 2;
        bytes[opcode_pos] = 0xff;
        let err = run_semcode(&bytes).expect_err("must fail");
        assert!(matches!(err, RuntimeError::BadFormat(_)));
    }

    #[test]
    fn vm_rejects_unsupported_bytecode_version_with_hint() {
        let src = "fn main() { return; }";
        let mut bytes = compile_program_to_semcode(src).expect("compile");
        bytes[7] = b'X';
        let err = run_semcode(&bytes).expect_err("must fail");
        match err {
            RuntimeError::UnsupportedBytecodeVersion { found, supported } => {
                assert!(found.starts_with("SEMCODE"));
                assert!(supported.contains("SEMCODE0"));
                assert!(supported.contains("SEMCODE1"));
                assert!(supported.contains("SEMCODE2"));
                assert!(supported.contains("SEMCODE3"));
                assert!(supported.contains("SEMCODE4"));
                assert!(supported.contains("SEMCODE5"));
                assert!(supported.contains("SEMCODE6"));
                assert!(supported.contains("SEMCODE7"));
                assert!(supported.contains("SEMCODE8"));
                assert!(supported.contains("SEMCODE9"));
                assert!(supported.contains("SEMCOD10"));
                assert!(supported.contains("SEMCOD11"));
                assert!(supported.contains("SEMCOD12"));
                assert!(supported.contains("SEMCOD13"));
            }
            other => panic!("expected UnsupportedBytecodeVersion, got {other:?}"),
        }
    }

    #[test]
    fn vm_enforces_configured_stack_depth() {
        let src = r#"
            fn helper() { return; }
            fn main() { helper(); return; }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let mut config = ExecutionConfig::for_context(ExecutionContext::VerifiedLocal);
        config.quotas.max_stack_depth = 1;
        let err = run_semcode_with_config(&bytes, config).expect_err("must fail");
        assert_eq!(err, RuntimeError::StackOverflow);
    }

    #[test]
    fn vm_enforces_configured_register_budget() {
        let src = r#"
            fn main() {
                let a: bool = true;
                let b: bool = false;
                let c = a && b;
                if c == false { return; } else { return; }
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let mut config = ExecutionConfig::for_context(ExecutionContext::VerifiedLocal);
        config.quotas.max_registers = 2;
        let err = run_semcode_with_config(&bytes, config).expect_err("must fail");
        assert_eq!(
            err,
            RuntimeError::QuotaExceeded(QuotaExceeded {
                kind: QuotaKind::Registers,
                limit: 2,
                used: 16,
            })
        );
    }

    #[test]
    fn verified_run_rejects_invalid_bytecode_before_execution() {
        let src = "fn main() { return; }";
        let mut bytes = compile_program_to_semcode(src).expect("compile");
        let opcode_pos = 8 + 2 + 4 + 4 + 2;
        bytes[opcode_pos] = 0xff;
        let err = run_verified_semcode(&bytes).expect_err("must fail");
        assert!(matches!(err, RuntimeError::VerifierRejected(_)));
    }

    fn ownership_tracking_bytes() -> Vec<u8> {
        let src = r#"
            fn pair(flag: bool) -> (i32, bool) = (1, flag);

            fn main() {
                let pair: (i32, bool) = pair(true);
                let (ref left, _): (i32, bool) = pair;
                let _ = left;
                return;
            }
        "#;
        compile_program_to_semcode(src).expect("compile")
    }

    fn helper_borrow_bytes() -> Vec<u8> {
        let src = r#"
            fn helper(pair: (i32, bool)) {
                let (ref left, _): (i32, bool) = pair;
                let _ = left;
                return;
            }

            fn main() {
                return;
            }
        "#;
        compile_program_to_semcode(src).expect("compile")
    }

    fn record_field_borrow_tracking_bytes() -> Vec<u8> {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let DecisionContext { camera: ref seen_camera, quality: _ } = ctx;
                let _ = seen_camera;
                return;
            }
        "#;
        compile_program_to_semcode(src).expect("compile")
    }

    fn helper_record_field_borrow_bytes() -> Vec<u8> {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn helper(ctx: DecisionContext) {
                let DecisionContext { camera: ref seen_camera, quality: _ } = ctx;
                let _ = seen_camera;
                return;
            }

            fn main() {
                return;
            }
        "#;
        compile_program_to_semcode(src).expect("compile")
    }

    fn record_field_write_overlap_bytes(
        borrowed_field: Option<&str>,
        write_field: Option<&str>,
    ) -> Vec<u8> {
        let src = r#"
            fn main() {
                let camera: f64 = 0.0;
                let quality: f64 = 1.0;
                let ctx: f64 = 1.0;
                ctx += 2.0;
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        rewrite_record_main_ownership_section(bytes, borrowed_field, write_field)
    }

    fn rewrite_record_main_ownership_section(
        bytes: Vec<u8>,
        borrowed_field: Option<&str>,
        write_field: Option<&str>,
    ) -> Vec<u8> {
        let mut cursor = 8usize;
        let name_len = read_u16_le(&bytes, &mut cursor).expect("name len") as usize;
        let name = read_utf8(&bytes, &mut cursor, name_len).expect("name");
        assert_eq!(name, "main");
        let code_len_pos = cursor;
        let code_len = read_u32_le(&bytes, &mut cursor).expect("code len") as usize;
        let code_start = cursor;
        let code_end = code_start + code_len;
        let code = &bytes[code_start..code_end];
        let (strings, _, _, _, instr_start) =
            parse_string_table_debug_and_ownership(code).expect("parse");
        let ctx_root = strings
            .iter()
            .position(|s| s == "ctx")
            .expect("ctx root index") as u32;
        let ownership_start = code[..instr_start]
            .windows(OWNERSHIP_SECTION_TAG.len())
            .position(|window| window == OWNERSHIP_SECTION_TAG)
            .expect("OWN0 section");
        let mut new_code = Vec::with_capacity(code.len());
        new_code.extend_from_slice(&code[..ownership_start]);
        new_code.extend_from_slice(&record_field_ownership_section_bytes(
            ctx_root,
            &strings,
            borrowed_field,
            write_field,
        ));
        new_code.extend_from_slice(&code[instr_start..]);

        let mut out = Vec::with_capacity(bytes.len() + new_code.len().saturating_sub(code.len()));
        out.extend_from_slice(&bytes[..code_len_pos]);
        out.extend_from_slice(&(new_code.len() as u32).to_le_bytes());
        out.extend_from_slice(&new_code);
        out.extend_from_slice(&bytes[code_end..]);
        out
    }

    fn record_field_ownership_section_bytes(
        root: u32,
        strings: &[String],
        borrowed_field: Option<&str>,
        write_field: Option<&str>,
    ) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        out.extend_from_slice(&2u16.to_le_bytes());
        append_record_field_ownership_event(
            &mut out,
            OWNERSHIP_EVENT_KIND_BORROW,
            root,
            strings,
            borrowed_field,
        );
        append_record_field_ownership_event(
            &mut out,
            OWNERSHIP_EVENT_KIND_WRITE,
            root,
            strings,
            write_field,
        );
        out
    }

    fn append_record_field_ownership_event(
        out: &mut Vec<u8>,
        kind: u8,
        root: u32,
        strings: &[String],
        field: Option<&str>,
    ) {
        out.push(kind);
        out.extend_from_slice(&root.to_le_bytes());
        match field {
            Some(field_name) => {
                out.extend_from_slice(&1u16.to_le_bytes());
                out.push(OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL);
                let field_symbol = strings
                    .iter()
                    .position(|s| s == field_name)
                    .expect("field symbol") as u32;
                out.extend_from_slice(&field_symbol.to_le_bytes());
            }
            None => out.extend_from_slice(&0u16.to_le_bytes()),
        }
    }

    fn ownership_write_overlap_bytes(
        borrowed_components: &[u16],
        write_components: &[u16],
    ) -> Vec<u8> {
        let src = r#"
            fn main() {
                let total: f64 = 1.0;
                total += 2.0;
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        rewrite_main_ownership_section(bytes, borrowed_components, write_components)
    }

    fn rewrite_main_ownership_section(
        bytes: Vec<u8>,
        borrowed_components: &[u16],
        write_components: &[u16],
    ) -> Vec<u8> {
        let mut cursor = 8usize;
        let name_len = read_u16_le(&bytes, &mut cursor).expect("name len") as usize;
        let name = read_utf8(&bytes, &mut cursor, name_len).expect("name");
        assert_eq!(name, "main");
        let code_len_pos = cursor;
        let code_len = read_u32_le(&bytes, &mut cursor).expect("code len") as usize;
        let code_start = cursor;
        let code_end = code_start + code_len;
        let code = &bytes[code_start..code_end];
        let (strings, _, _, _, instr_start) =
            parse_string_table_debug_and_ownership(code).expect("parse");
        let total_root = strings
            .iter()
            .position(|s| s == "total")
            .expect("total root index") as u32;
        let ownership_start = code[..instr_start]
            .windows(OWNERSHIP_SECTION_TAG.len())
            .position(|window| window == OWNERSHIP_SECTION_TAG)
            .expect("OWN0 section");
        let mut new_code = Vec::with_capacity(code.len());
        new_code.extend_from_slice(&code[..ownership_start]);
        new_code.extend_from_slice(&ownership_section_bytes(
            total_root,
            borrowed_components,
            write_components,
        ));
        new_code.extend_from_slice(&code[instr_start..]);

        let mut out = Vec::with_capacity(bytes.len() + new_code.len().saturating_sub(code.len()));
        out.extend_from_slice(&bytes[..code_len_pos]);
        out.extend_from_slice(&(new_code.len() as u32).to_le_bytes());
        out.extend_from_slice(&new_code);
        out.extend_from_slice(&bytes[code_end..]);
        out
    }

    fn ownership_section_bytes(
        root: u32,
        borrowed_components: &[u16],
        write_components: &[u16],
    ) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&OWNERSHIP_SECTION_TAG);
        out.extend_from_slice(&2u16.to_le_bytes());
        append_ownership_event(
            &mut out,
            OWNERSHIP_EVENT_KIND_BORROW,
            root,
            borrowed_components,
        );
        append_ownership_event(
            &mut out,
            OWNERSHIP_EVENT_KIND_WRITE,
            root,
            write_components,
        );
        out
    }

    fn append_ownership_event(out: &mut Vec<u8>, kind: u8, root: u32, components: &[u16]) {
        out.push(kind);
        out.extend_from_slice(&root.to_le_bytes());
        out.extend_from_slice(&(components.len() as u16).to_le_bytes());
        for index in components {
            out.push(OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX);
            out.extend_from_slice(&index.to_le_bytes());
        }
    }
}
