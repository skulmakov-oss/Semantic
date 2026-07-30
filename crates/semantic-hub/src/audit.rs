//! Bounded, deterministic audit evidence for every admitted invocation.
//!
//! This is a new canonical truth store for a domain `prom-audit` does not
//! cover today (external-tool invocation evidence, as opposed to
//! SemCode/host-ABI runtime events) -- it does not compete with or
//! duplicate `prom_audit::AuditTrail`'s authority over SemCode runtime
//! audit truth. It mirrors that crate's (and `prom-state`'s) canonical
//! convention: a versioned magic header, tab-delimited fields with explicit
//! escaping, and strict round-trip validation, rather than reaching for
//! serde/JSON for a wire format that must stay byte-stable across releases.

use std::fmt;
use std::io::BufRead;
use std::path::Path;

use crate::capability::HubCapability;
use crate::execution::{HubDeterminismClass, HubExecutionMode, HubPrivacyClass, HubTrustClass};
use crate::ids::{
    HubCallerIdentity, HubOperationId, HubRequestId, HubSessionId, HubToolId, HubToolVersion,
};
use crate::provenance::HubDigest;
use crate::resource::{HubResourceBudget, HubResourceUsage};
use crate::worker::HubWorkerState;

/// Format version of the canonical audit text produced by [`HubAuditTrail`].
pub const HUB_AUDIT_FORMAT_VERSION: u32 = 2;
const MAGIC: &str = "semantic-hub.audit.v2";
const LEGACY_MAGIC_V1: &str = "semantic-hub.audit.v1";
const LEGACY_FORMAT_VERSION_V1: u32 = 1;
const MAX_AUDIT_HEADER_LINE_BYTES: usize = 128;
const MAX_AUDIT_TRANSACTION_ID_BYTES: usize = 256;

/// Maximum canonical-text byte length of one audit record. Bounds evidence
/// size regardless of what a tool tried to return.
pub const MAX_AUDIT_RECORD_BYTES: usize = 8 * 1024;

fn escape_field(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            _ => out.push(c),
        }
    }
    out
}

fn unescape_field(s: &str) -> Result<String, AuditParseError> {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\\') => out.push('\\'),
                Some('t') => out.push('\t'),
                Some('n') => out.push('\n'),
                _ => return Err(AuditParseError::Malformed("invalid escape sequence")),
            }
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

/// One admitted invocation's bounded audit evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubAuditRecord {
    pub sequence: u64,
    pub request_id: HubRequestId,
    pub session_id: HubSessionId,
    pub caller_identity: HubCallerIdentity,
    pub tool_id: HubToolId,
    pub tool_version: HubToolVersion,
    pub adapter_provenance: String,
    pub operation_id: HubOperationId,
    pub execution_mode: HubExecutionMode,
    pub determinism: HubDeterminismClass,
    pub trust_class: HubTrustClass,
    pub privacy_class: HubPrivacyClass,
    pub capabilities_requested: Vec<HubCapability>,
    pub capabilities_granted: Vec<HubCapability>,
    pub input_digest: HubDigest,
    pub output_digest: HubDigest,
    /// Correlates a mutating reply with its recoverable persistence
    /// transaction. It is an opaque bounded identifier, never a path,
    /// secret, or payload.
    pub transaction_id: Option<String>,
    pub resource_budget: HubResourceBudget,
    pub resource_usage: HubResourceUsage,
    pub worker_state_after: HubWorkerState,
    pub status_code: &'static str,
    pub fault_code: Option<&'static str>,
}

/// Reasons canonical audit text fails to parse back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditParseError {
    MissingMagicHeader,
    UnsupportedFormatVersion(u32),
    Malformed(&'static str),
    FieldCount { expected: usize, actual: usize },
    NonMonotonicSequence { previous: u64, next: u64 },
    RecordTooLarge { max: usize, actual: usize },
    TransactionIdTooLarge { max: usize, actual: usize },
}

impl fmt::Display for AuditParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditParseError::MissingMagicHeader => {
                write!(f, "missing canonical audit magic header")
            }
            AuditParseError::UnsupportedFormatVersion(v) => {
                write!(f, "unsupported audit format version {v}")
            }
            AuditParseError::Malformed(m) => write!(f, "malformed audit record: {m}"),
            AuditParseError::FieldCount { expected, actual } => {
                write!(f, "audit record has {actual} fields, expected {expected}")
            }
            AuditParseError::NonMonotonicSequence { previous, next } => write!(
                f,
                "audit sequence {next} does not follow {previous} monotonically"
            ),
            AuditParseError::RecordTooLarge { max, actual } => {
                write!(f, "audit record {actual} bytes exceeds maximum {max} bytes")
            }
            AuditParseError::TransactionIdTooLarge { max, actual } => write!(
                f,
                "audit transaction id {actual} bytes exceeds maximum {max} bytes"
            ),
        }
    }
}

/// Errors from streaming canonical audit reads.
#[derive(Debug)]
pub enum AuditFileError {
    Io(std::io::Error),
    Parse(AuditParseError),
    /// The file ended after bytes belonging to a record but before its
    /// terminating newline. `valid_records` earlier records remain valid
    /// and untouched; append refuses to modify this file.
    PartialFinalRecord {
        valid_records: usize,
        bytes: usize,
    },
}

impl fmt::Display for AuditFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditFileError::Io(e) => write!(f, "audit log I/O error: {e}"),
            AuditFileError::Parse(e) => write!(f, "audit log parse error: {e}"),
            AuditFileError::PartialFinalRecord {
                valid_records,
                bytes,
            } => write!(
                f,
                "audit log has a partial final record ({bytes} bytes) after {valid_records} valid records"
            ),
        }
    }
}

impl std::error::Error for AuditFileError {}

/// Summary produced by a complete, validated file scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HubAuditFileSummary {
    pub format_version: u32,
    pub record_count: usize,
    pub next_sequence: Option<u64>,
    pub streaming: bool,
}

/// Errors from [`HubAuditTrail::append_records_to_file`].
#[derive(Debug)]
pub enum AuditAppendError {
    Io(std::io::Error),
    Parse(AuditParseError),
    PartialFinalRecord { valid_records: usize, bytes: usize },
}

impl fmt::Display for AuditAppendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditAppendError::Io(e) => write!(f, "audit log I/O error: {e}"),
            AuditAppendError::Parse(e) => write!(f, "audit log parse error: {e}"),
            AuditAppendError::PartialFinalRecord {
                valid_records,
                bytes,
            } => write!(
                f,
                "audit log has a partial final record ({bytes} bytes) after {valid_records} valid records"
            ),
        }
    }
}

impl std::error::Error for AuditAppendError {}

impl From<AuditFileError> for AuditAppendError {
    fn from(value: AuditFileError) -> Self {
        match value {
            AuditFileError::Io(error) => Self::Io(error),
            AuditFileError::Parse(error) => Self::Parse(error),
            AuditFileError::PartialFinalRecord {
                valid_records,
                bytes,
            } => Self::PartialFinalRecord {
                valid_records,
                bytes,
            },
        }
    }
}

const FIELD_COUNT_V1: usize = 20;
const FIELD_COUNT_V2: usize = 22;

fn pack_budget(b: &HubResourceBudget) -> String {
    b.canonical_text()
}

fn unpack_budget(s: &str) -> Result<HubResourceBudget, AuditParseError> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 12 {
        return Err(AuditParseError::Malformed("resource_budget"));
    }
    let n = |i: usize| -> Result<u64, AuditParseError> {
        parts[i]
            .parse()
            .map_err(|_| AuditParseError::Malformed("resource_budget"))
    };
    let n32 = |i: usize| -> Result<u32, AuditParseError> {
        parts[i]
            .parse()
            .map_err(|_| AuditParseError::Malformed("resource_budget"))
    };
    Ok(HubResourceBudget {
        wall_time_millis: n(0)?,
        memory_bytes: n(1)?,
        input_bytes: n(2)?,
        output_bytes: n(3)?,
        index_item_count: n(4)?,
        vector_dimensions: n32(5)?,
        result_count: n32(6)?,
        queue_depth: n32(7)?,
        concurrent_requests: n32(8)?,
        storage_read_bytes: n(9)?,
        storage_write_bytes: n(10)?,
        audit_bytes: n(11)?,
    })
}

fn pack_usage(u: &HubResourceUsage) -> String {
    fn opt<T: ToString>(v: Option<T>) -> String {
        v.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string())
    }
    [
        opt(u.wall_time_millis),
        opt(u.peak_memory_bytes),
        opt(u.input_bytes),
        opt(u.output_bytes),
        opt(u.index_item_count),
        opt(u.result_count),
        opt(u.storage_read_bytes),
        opt(u.storage_write_bytes),
        opt(u.audit_bytes),
    ]
    .join(",")
}

fn unpack_usage(s: &str) -> Result<HubResourceUsage, AuditParseError> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 9 {
        return Err(AuditParseError::Malformed("resource_usage"));
    }
    fn opt_u64(s: &str) -> Result<Option<u64>, AuditParseError> {
        if s == "-" {
            Ok(None)
        } else {
            s.parse()
                .map(Some)
                .map_err(|_| AuditParseError::Malformed("resource_usage"))
        }
    }
    fn opt_u32(s: &str) -> Result<Option<u32>, AuditParseError> {
        if s == "-" {
            Ok(None)
        } else {
            s.parse()
                .map(Some)
                .map_err(|_| AuditParseError::Malformed("resource_usage"))
        }
    }
    Ok(HubResourceUsage {
        wall_time_millis: opt_u64(parts[0])?,
        peak_memory_bytes: opt_u64(parts[1])?,
        input_bytes: opt_u64(parts[2])?,
        output_bytes: opt_u64(parts[3])?,
        index_item_count: opt_u64(parts[4])?,
        result_count: opt_u32(parts[5])?,
        storage_read_bytes: opt_u64(parts[6])?,
        storage_write_bytes: opt_u64(parts[7])?,
        audit_bytes: opt_u64(parts[8])?,
    })
}

impl HubAuditRecord {
    pub fn to_canonical_line(&self) -> String {
        let requested = pack_capabilities(&self.capabilities_requested);
        let granted = pack_capabilities(&self.capabilities_granted);
        let transaction_id = self
            .transaction_id
            .as_deref()
            .map(escape_field)
            .unwrap_or_else(|| "-".to_string());
        let fields = [
            self.sequence.to_string(),
            escape_field(self.request_id.as_str()),
            escape_field(self.session_id.as_str()),
            escape_field(self.caller_identity.as_str()),
            escape_field(self.tool_id.as_str()),
            self.tool_version.to_string(),
            escape_field(&self.adapter_provenance),
            escape_field(self.operation_id.as_str()),
            self.execution_mode.to_string(),
            self.determinism.to_string(),
            self.trust_class.to_string(),
            self.privacy_class.to_string(),
            escape_field(&requested),
            escape_field(&granted),
            self.input_digest.to_string(),
            self.output_digest.to_string(),
            transaction_id,
            pack_budget(&self.resource_budget),
            pack_usage(&self.resource_usage),
            self.worker_state_after.to_string(),
            self.status_code.to_string(),
            self.fault_code.unwrap_or("-").to_string(),
        ];
        debug_assert_eq!(fields.len(), FIELD_COUNT_V2);
        fields.join("\t")
    }

    pub fn from_canonical_line(
        line: &str,
        previous_sequence: Option<u64>,
    ) -> Result<Self, AuditParseError> {
        Self::from_canonical_line_version(line, previous_sequence, HUB_AUDIT_FORMAT_VERSION)
    }

    fn from_canonical_line_version(
        line: &str,
        previous_sequence: Option<u64>,
        format_version: u32,
    ) -> Result<Self, AuditParseError> {
        if line.len() > MAX_AUDIT_RECORD_BYTES {
            return Err(AuditParseError::RecordTooLarge {
                max: MAX_AUDIT_RECORD_BYTES,
                actual: line.len(),
            });
        }
        let raw: Vec<&str> = line.split('\t').collect();
        let expected_fields = match format_version {
            LEGACY_FORMAT_VERSION_V1 => FIELD_COUNT_V1,
            HUB_AUDIT_FORMAT_VERSION => FIELD_COUNT_V2,
            other => return Err(AuditParseError::UnsupportedFormatVersion(other)),
        };
        if raw.len() != expected_fields {
            return Err(AuditParseError::FieldCount {
                expected: expected_fields,
                actual: raw.len(),
            });
        }
        let sequence: u64 = raw[0]
            .parse()
            .map_err(|_| AuditParseError::Malformed("sequence is not a valid u64"))?;
        if let Some(prev) = previous_sequence {
            if sequence <= prev {
                return Err(AuditParseError::NonMonotonicSequence {
                    previous: prev,
                    next: sequence,
                });
            }
        }
        let request_id = HubRequestId::new(unescape_field(raw[1])?)
            .map_err(|_| AuditParseError::Malformed("request_id"))?;
        let session_id = HubSessionId::new(unescape_field(raw[2])?)
            .map_err(|_| AuditParseError::Malformed("session_id"))?;
        let caller_identity = HubCallerIdentity::new(unescape_field(raw[3])?)
            .map_err(|_| AuditParseError::Malformed("caller_identity"))?;
        let tool_id = HubToolId::new(unescape_field(raw[4])?)
            .map_err(|_| AuditParseError::Malformed("tool_id"))?;
        let tool_version: HubToolVersion = raw[5]
            .parse()
            .map_err(|_| AuditParseError::Malformed("tool_version"))?;
        let adapter_provenance = unescape_field(raw[6])?;
        let operation_id = HubOperationId::new(unescape_field(raw[7])?)
            .map_err(|_| AuditParseError::Malformed("operation_id"))?;

        let execution_mode = match raw[8] {
            "InProcess" => HubExecutionMode::InProcess,
            "Subprocess" => HubExecutionMode::Subprocess,
            "Wasm" => HubExecutionMode::Wasm,
            "Remote" => HubExecutionMode::Remote,
            _ => return Err(AuditParseError::Malformed("execution_mode")),
        };
        let determinism = match raw[9] {
            "Deterministic" => HubDeterminismClass::Deterministic,
            "DeterministicWithSeed" => HubDeterminismClass::DeterministicWithSeed,
            "EnvironmentDependent" => HubDeterminismClass::EnvironmentDependent,
            "Unknown" => HubDeterminismClass::Unknown,
            _ => return Err(AuditParseError::Malformed("determinism")),
        };
        let trust_class = match raw[10] {
            "InProcessUnisolated" => HubTrustClass::InProcessUnisolated,
            "ProcessIsolated" => HubTrustClass::ProcessIsolated,
            "SandboxIsolated" => HubTrustClass::SandboxIsolated,
            _ => return Err(AuditParseError::Malformed("trust_class")),
        };
        let privacy_class = match raw[11] {
            "PublicSafe" => HubPrivacyClass::PublicSafe,
            "ProjectLocal" => HubPrivacyClass::ProjectLocal,
            "PrivateSource" => HubPrivacyClass::PrivateSource,
            "OrganizationPrivate" => HubPrivacyClass::OrganizationPrivate,
            "SecretSuspected" => HubPrivacyClass::SecretSuspected,
            _ => return Err(AuditParseError::Malformed("privacy_class")),
        };
        let parse_digest = |s: &str| -> Result<HubDigest, AuditParseError> {
            let rest = s
                .strip_prefix("fnv1a64:")
                .ok_or(AuditParseError::Malformed("digest prefix"))?;
            let (hex, len) = rest
                .split_once(':')
                .ok_or(AuditParseError::Malformed("digest shape"))?;
            let fnv1a64 = u64::from_str_radix(hex, 16)
                .map_err(|_| AuditParseError::Malformed("digest hex"))?;
            let byte_len: u64 = len
                .parse()
                .map_err(|_| AuditParseError::Malformed("digest length"))?;
            Ok(HubDigest { fnv1a64, byte_len })
        };
        let (
            capabilities_requested,
            mut capabilities_granted,
            input_digest_index,
            output_digest_index,
            transaction_id,
            budget_index,
            usage_index,
            worker_state_index,
            status_index,
            fault_index,
        ) = if format_version == LEGACY_FORMAT_VERSION_V1 {
            (
                unpack_capabilities(raw[12], "capabilities")?,
                Vec::new(),
                13,
                14,
                None,
                15,
                16,
                17,
                18,
                19,
            )
        } else {
            (
                unpack_capabilities(raw[12], "capabilities_requested")?,
                unpack_capabilities(raw[13], "capabilities_granted")?,
                14,
                15,
                unpack_transaction_id(raw[16])?,
                17,
                18,
                19,
                20,
                21,
            )
        };

        let input_digest = parse_digest(raw[input_digest_index])?;
        let output_digest = parse_digest(raw[output_digest_index])?;
        let resource_budget = unpack_budget(raw[budget_index])?;
        let resource_usage = unpack_usage(raw[usage_index])?;
        let worker_state_after = match raw[worker_state_index] {
            "Registered" => HubWorkerState::Registered,
            "Starting" => HubWorkerState::Starting,
            "Ready" => HubWorkerState::Ready,
            "Busy" => HubWorkerState::Busy,
            "Degraded" => HubWorkerState::Degraded,
            "Restarting" => HubWorkerState::Restarting,
            "Quarantined" => HubWorkerState::Quarantined,
            "Disabled" => HubWorkerState::Disabled,
            "Stopped" => HubWorkerState::Stopped,
            _ => return Err(AuditParseError::Malformed("worker_state_after")),
        };
        let status_code: &'static str = leak_known_status(raw[status_index])?;
        let fault_code = if raw[fault_index] == "-" {
            None
        } else {
            Some(leak_known_fault_code(raw[fault_index])?)
        };

        if format_version == LEGACY_FORMAT_VERSION_V1 {
            // v1 had one field named "granted", but its writer actually
            // serialized the caller's requested context. The best
            // reconstructable semantics are therefore: preserve it as
            // requested; pre-dispatch rejection granted nothing; otherwise
            // remove capabilities Hub v0 can never grant. v2 stores both
            // fields explicitly and needs no inference.
            capabilities_granted = if status_code == "Rejected" {
                Vec::new()
            } else {
                capabilities_requested
                    .iter()
                    .filter(|capability| !capability.is_sensitive())
                    .copied()
                    .collect()
            };
        }

        Ok(HubAuditRecord {
            sequence,
            request_id,
            session_id,
            caller_identity,
            tool_id,
            tool_version,
            adapter_provenance,
            operation_id,
            execution_mode,
            determinism,
            trust_class,
            privacy_class,
            capabilities_requested,
            capabilities_granted,
            input_digest,
            output_digest,
            transaction_id,
            resource_budget,
            resource_usage,
            worker_state_after,
            status_code,
            fault_code,
        })
    }
}

fn pack_capabilities(capabilities: &[HubCapability]) -> String {
    capabilities
        .iter()
        .map(HubCapability::as_str)
        .collect::<Vec<_>>()
        .join(";")
}

fn unpack_capabilities(
    raw: &str,
    field_name: &'static str,
) -> Result<Vec<HubCapability>, AuditParseError> {
    let raw = unescape_field(raw)?;
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    raw.split(';')
        .map(|part| HubCapability::parse(part).ok_or(AuditParseError::Malformed(field_name)))
        .collect()
}

fn unpack_transaction_id(raw: &str) -> Result<Option<String>, AuditParseError> {
    if raw == "-" {
        return Ok(None);
    }
    let transaction_id = unescape_field(raw)?;
    if transaction_id.is_empty() {
        return Err(AuditParseError::Malformed("transaction_id"));
    }
    if transaction_id.len() > MAX_AUDIT_TRANSACTION_ID_BYTES {
        return Err(AuditParseError::TransactionIdTooLarge {
            max: MAX_AUDIT_TRANSACTION_ID_BYTES,
            actual: transaction_id.len(),
        });
    }
    Ok(Some(transaction_id))
}

/// Maps a parsed status string back to the `&'static str` constants used
/// elsewhere, so round-tripped records stay comparable by pointer-free value
/// equality without inventing a fresh heap string per parse.
fn leak_known_status(s: &str) -> Result<&'static str, AuditParseError> {
    match s {
        "Success" => Ok("Success"),
        "Rejected" => Ok("Rejected"),
        "ToolFailed" => Ok("ToolFailed"),
        "Crashed" => Ok("Crashed"),
        "HubFault" => Ok("HubFault"),
        _ => Err(AuditParseError::Malformed("status_code")),
    }
}

fn leak_known_fault_code(s: &str) -> Result<&'static str, AuditParseError> {
    // Must list every `HubFault::code()` string -- this is a hand-written
    // duplicate of that exhaustive list (not derived from it, since the
    // audit format needs a `&'static str` the parser can hand back
    // without allocating, and `HubFault::code()` already returns
    // `&'static str` constants defined next to each variant). A fault
    // code added to `HubFault` without a matching entry here makes any
    // audit record carrying it unparseable on the very next read --
    // exactly the class of bug `a_record_granting_every_sensitive_capability_round_trips`
    // guards for the capability list; there is no test that can catch a
    // *missing* fault code here other than exercising every variant
    // end-to-end, which `tests/hub_cli.rs` does for the ones reachable
    // through the CLI.
    const CODES: &[&str] = &[
        "UnknownTool",
        "UnknownOperation",
        "ApiVersionUnsupported",
        "SchemaVersionUnsupported",
        "DescriptorIncompatible",
        "InputRejected",
        "CapabilityDenied",
        "PrivacyDenied",
        "SensitiveCapabilityDenied",
        "WorkerDegraded",
        "SessionLimitExceeded",
        "ResourceBudgetInvalid",
        "QueueFull",
        "ToolDisabled",
        "ToolQuarantined",
        "DeadlineExceeded",
        "Cancelled",
        "ResourceExhausted",
        "WorkerBusy",
        "ToolDeclaredFailure",
        "WorkerPanicked",
        "ProtocolViolation",
        "OutputRejected",
        "AuditProvenanceFailure",
        "PersistenceFailed",
        "RecoveryRequired",
        "SequenceExhausted",
        "InternalHubFault",
    ];
    CODES
        .iter()
        .find(|c| **c == s)
        .copied()
        .ok_or(AuditParseError::Malformed("fault_code"))
}

#[derive(Debug, Clone, Copy)]
struct AuditHeader {
    format_version: u32,
    declared_count: Option<usize>,
}

fn parse_header(magic: &str, version: &str, count: &str) -> Result<AuditHeader, AuditParseError> {
    if magic != MAGIC && magic != LEGACY_MAGIC_V1 {
        return Err(AuditParseError::MissingMagicHeader);
    }
    let format_version: u32 = version
        .parse()
        .map_err(|_| AuditParseError::Malformed("format version is not a u32"))?;
    if format_version != LEGACY_FORMAT_VERSION_V1 && format_version != HUB_AUDIT_FORMAT_VERSION {
        return Err(AuditParseError::UnsupportedFormatVersion(format_version));
    }
    if (format_version == LEGACY_FORMAT_VERSION_V1 && magic != LEGACY_MAGIC_V1)
        || (format_version == HUB_AUDIT_FORMAT_VERSION && magic != MAGIC)
    {
        return Err(AuditParseError::Malformed("magic/version mismatch"));
    }
    let declared_count = if count == "unbounded" {
        None
    } else {
        Some(
            count
                .parse()
                .map_err(|_| AuditParseError::Malformed("record count is not a usize"))?,
        )
    };
    Ok(AuditHeader {
        format_version,
        declared_count,
    })
}

fn validate_declared_count(
    declared_count: Option<usize>,
    actual: usize,
) -> Result<(), AuditParseError> {
    if let Some(expected) = declared_count {
        if actual != expected {
            return Err(AuditParseError::FieldCount { expected, actual });
        }
    }
    Ok(())
}

fn scan_text(
    text: &str,
    mut visit: impl FnMut(&HubAuditRecord),
) -> Result<HubAuditFileSummary, AuditParseError> {
    let mut lines = text.lines();
    let header = parse_header(
        lines.next().ok_or(AuditParseError::MissingMagicHeader)?,
        lines
            .next()
            .ok_or(AuditParseError::Malformed("missing format version"))?,
        lines
            .next()
            .ok_or(AuditParseError::Malformed("missing record count"))?,
    )?;
    let mut previous_sequence = None;
    let mut record_count = 0usize;
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let record = HubAuditRecord::from_canonical_line_version(
            line,
            previous_sequence,
            header.format_version,
        )?;
        previous_sequence = Some(record.sequence);
        record_count = record_count
            .checked_add(1)
            .ok_or(AuditParseError::Malformed("record count overflow"))?;
        visit(&record);
    }
    validate_declared_count(header.declared_count, record_count)?;
    Ok(HubAuditFileSummary {
        format_version: header.format_version,
        record_count,
        next_sequence: previous_sequence
            .map(|sequence| sequence.checked_add(1))
            .unwrap_or(Some(0)),
        streaming: header.declared_count.is_none(),
    })
}

enum BoundedLine {
    Eof,
    Complete(Vec<u8>),
    Partial { bytes: usize },
    TooLarge { actual: usize },
}

/// Reads one newline-terminated line without ever retaining more than
/// `max` bytes. Oversized input is consumed through its newline so the
/// diagnostic can report its exact byte length without allocating it.
fn read_bounded_line(reader: &mut impl BufRead, max: usize) -> Result<BoundedLine, std::io::Error> {
    let mut retained = Vec::with_capacity(max.min(256));
    let mut total = 0usize;
    let mut too_large = false;
    loop {
        let (chunk_len, has_newline) = {
            let available = reader.fill_buf()?;
            if available.is_empty() {
                return if total == 0 {
                    Ok(BoundedLine::Eof)
                } else {
                    Ok(BoundedLine::Partial { bytes: total })
                };
            }
            let chunk_len = available
                .iter()
                .position(|byte| *byte == b'\n')
                .unwrap_or(available.len());
            total = total.saturating_add(chunk_len);
            if !too_large && total <= max {
                retained.extend_from_slice(&available[..chunk_len]);
            } else {
                too_large = true;
            }
            (chunk_len, chunk_len < available.len())
        };
        reader.consume(chunk_len + usize::from(has_newline));
        if has_newline {
            return if too_large {
                Ok(BoundedLine::TooLarge { actual: total })
            } else {
                Ok(BoundedLine::Complete(retained))
            };
        }
    }
}

fn decode_line(bytes: Vec<u8>) -> Result<String, AuditParseError> {
    String::from_utf8(bytes).map_err(|_| AuditParseError::Malformed("audit log is not valid UTF-8"))
}

fn read_header_line(
    reader: &mut impl BufRead,
    missing: &'static str,
) -> Result<String, AuditFileError> {
    match read_bounded_line(reader, MAX_AUDIT_HEADER_LINE_BYTES).map_err(AuditFileError::Io)? {
        BoundedLine::Complete(bytes) => decode_line(bytes).map_err(AuditFileError::Parse),
        BoundedLine::Eof | BoundedLine::Partial { .. } => {
            Err(AuditFileError::Parse(AuditParseError::Malformed(missing)))
        }
        BoundedLine::TooLarge { actual } => {
            Err(AuditFileError::Parse(AuditParseError::RecordTooLarge {
                max: MAX_AUDIT_HEADER_LINE_BYTES,
                actual,
            }))
        }
    }
}

fn write_streaming_header(writer: &mut impl std::io::Write) -> Result<(), std::io::Error> {
    writeln!(writer, "{MAGIC}")?;
    writeln!(writer, "{HUB_AUDIT_FORMAT_VERSION}")?;
    writeln!(writer, "unbounded")
}

fn write_record_line(
    writer: &mut impl std::io::Write,
    record: &HubAuditRecord,
) -> Result<(), std::io::Error> {
    writeln!(writer, "{}", record.to_canonical_line())
}

fn validate_records(
    records: &[HubAuditRecord],
    mut previous_sequence: Option<u64>,
) -> Result<(), AuditParseError> {
    for record in records {
        let line = record.to_canonical_line();
        HubAuditRecord::from_canonical_line(&line, previous_sequence)?;
        previous_sequence = Some(record.sequence);
    }
    Ok(())
}

fn atomic_replace(
    path: &Path,
    write: impl FnOnce(&mut std::fs::File) -> Result<(), AuditAppendError>,
) -> Result<(), AuditAppendError> {
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let parent = path.parent().filter(|path| !path.as_os_str().is_empty());
    let parent = parent.unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent).map_err(AuditAppendError::Io)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("semantic-hub-audit");
    let temporary = parent.join(format!(
        ".{file_name}.tmp.{}.{}",
        std::process::id(),
        TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .map_err(AuditAppendError::Io)?;
        write(&mut file)?;
        file.sync_all().map_err(AuditAppendError::Io)?;
        drop(file);
        std::fs::rename(&temporary, path).map_err(AuditAppendError::Io)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

/// An ordered, append-only sequence of audit records for one Hub process
/// lifetime, with strict monotonic sequencing and canonical text encoding.
#[derive(Debug, Clone, Default)]
pub struct HubAuditTrail {
    records: Vec<HubAuditRecord>,
}

impl HubAuditTrail {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// `None` means the sequence space is exhausted (the last record's
    /// sequence is already `u64::MAX`) -- the caller must treat this as a
    /// hard failure, not silently wrap to `0` (which would violate the
    /// trail's own monotonic-sequence invariant) or panic (in an
    /// overflow-checked build). Reaching this is astronomically unlikely
    /// in practice, but the checked arithmetic costs nothing and this
    /// type's whole purpose is never accepting a silently-wrong sequence.
    pub fn next_sequence(&self) -> Option<u64> {
        self.records
            .last()
            .map(|r| r.sequence.checked_add(1))
            .unwrap_or(Some(0))
    }

    pub fn push(&mut self, record: HubAuditRecord) {
        self.records.push(record);
    }

    pub fn records(&self) -> &[HubAuditRecord] {
        &self.records
    }

    pub fn find_by_request(&self, request_id: &HubRequestId) -> Option<&HubAuditRecord> {
        self.records.iter().find(|r| &r.request_id == request_id)
    }

    /// Find the record matching `request_id` by scanning `text` one line
    /// at a time, stopping as soon as a match is found instead of first
    /// parsing every record into a `Vec<HubAuditRecord>` (what
    /// `from_canonical_text` followed by `find_by_request` does). For a
    /// caller like `smc hub audit --request <id>` that only ever wants
    /// one record, this avoids allocating (capability vectors, cloned
    /// strings, ...) for every OTHER record in a long-lived project's
    /// history just to discard them immediately afterward. The full
    /// header is still validated first, exactly like
    /// `from_canonical_text`. `text` itself must already be resident in
    /// memory (this is not a zero-copy file-seek implementation) -- the
    /// improvement here is bounded per-record allocation, not zero I/O.
    pub fn find_by_request_streaming(
        text: &str,
        request_id: &HubRequestId,
    ) -> Result<Option<HubAuditRecord>, AuditParseError> {
        let mut found = None;
        scan_text(text, |record| {
            if found.is_none() && &record.request_id == request_id {
                found = Some(record.clone());
            }
        })?;
        Ok(found)
    }

    /// Learn the next sequence number by parsing only the LAST record
    /// line in `text`, without building a `Vec<HubAuditRecord>` for the
    /// entire history first (what `from_canonical_text` followed by
    /// `next_sequence` does). Still validates the header exactly like
    /// `from_canonical_text`, and still requires `text` to already be
    /// resident in memory -- see `find_by_request_streaming`'s doc
    /// comment for the same caveat.
    pub fn next_sequence_streaming(text: &str) -> Result<Option<u64>, AuditParseError> {
        Ok(scan_text(text, |_| {})?.next_sequence)
    }

    /// Stream and validate an audit file without loading the whole history
    /// into memory. At most one bounded record line is retained at a time.
    /// The visitor is called in canonical sequence order only after each
    /// record validates; the scan still reaches EOF so declared counts and
    /// a partial final record cannot be hidden by an early match.
    pub fn scan_file(
        path: &Path,
        mut visit: impl FnMut(&HubAuditRecord),
    ) -> Result<HubAuditFileSummary, AuditFileError> {
        let file = std::fs::File::open(path).map_err(AuditFileError::Io)?;
        let mut reader = std::io::BufReader::new(file);
        let header = parse_header(
            &read_header_line(&mut reader, "missing magic header")?,
            &read_header_line(&mut reader, "missing format version")?,
            &read_header_line(&mut reader, "missing record count")?,
        )
        .map_err(AuditFileError::Parse)?;

        let mut previous_sequence = None;
        let mut record_count = 0usize;
        loop {
            let line = read_bounded_line(&mut reader, MAX_AUDIT_RECORD_BYTES)
                .map_err(AuditFileError::Io)?;
            let bytes = match line {
                BoundedLine::Eof => break,
                BoundedLine::Complete(bytes) if bytes.is_empty() => continue,
                BoundedLine::Complete(bytes) => bytes,
                BoundedLine::Partial { bytes } => {
                    return Err(AuditFileError::PartialFinalRecord {
                        valid_records: record_count,
                        bytes,
                    });
                }
                BoundedLine::TooLarge { actual } => {
                    return Err(AuditFileError::Parse(AuditParseError::RecordTooLarge {
                        max: MAX_AUDIT_RECORD_BYTES,
                        actual,
                    }));
                }
            };
            let line = decode_line(bytes).map_err(AuditFileError::Parse)?;
            let record = HubAuditRecord::from_canonical_line_version(
                &line,
                previous_sequence,
                header.format_version,
            )
            .map_err(AuditFileError::Parse)?;
            previous_sequence = Some(record.sequence);
            record_count = record_count.checked_add(1).ok_or(AuditFileError::Parse(
                AuditParseError::Malformed("record count overflow"),
            ))?;
            visit(&record);
        }
        validate_declared_count(header.declared_count, record_count)
            .map_err(AuditFileError::Parse)?;
        Ok(HubAuditFileSummary {
            format_version: header.format_version,
            record_count,
            next_sequence: previous_sequence
                .map(|sequence| sequence.checked_add(1))
                .unwrap_or(Some(0)),
            streaming: header.declared_count.is_none(),
        })
    }

    pub fn find_by_request_file(
        path: &Path,
        request_id: &HubRequestId,
    ) -> Result<Option<HubAuditRecord>, AuditFileError> {
        let mut found = None;
        Self::scan_file(path, |record| {
            if found.is_none() && &record.request_id == request_id {
                found = Some(record.clone());
            }
        })?;
        Ok(found)
    }

    pub fn scan_file_by_session(
        path: &Path,
        session_id: &HubSessionId,
        mut visit: impl FnMut(&HubAuditRecord),
    ) -> Result<HubAuditFileSummary, AuditFileError> {
        Self::scan_file(path, |record| {
            if &record.session_id == session_id {
                visit(record);
            }
        })
    }

    pub fn next_sequence_file(path: &Path) -> Result<Option<u64>, AuditFileError> {
        Ok(Self::scan_file(path, |_| {})?.next_sequence)
    }

    /// Append `new_records` to the canonical text file at `path`,
    /// avoiding a full read-parse-rewrite of the entire prior history
    /// whenever possible -- the fix for the single biggest cost in the
    /// pre-completion implementation, where every invocation re-read,
    /// fully parsed, and fully re-wrote the ENTIRE audit history just to
    /// add one record. `new_records` must already be in strictly
    /// increasing sequence order (not verified against prior on-disk
    /// records here -- the caller is expected to have derived the first
    /// new sequence from [`Self::next_sequence_streaming`] against the
    /// same file immediately beforehand, under the same project lock).
    ///
    /// Three cases:
    /// - **No file yet**: written from scratch using
    ///   [`Self::to_canonical_text_streaming`]'s header shape (the
    ///   `"unbounded"` sentinel, not an exact count), via an atomic
    ///   write.
    /// - **Existing file already in streaming form** (its header already
    ///   carries the `"unbounded"` sentinel -- checked by peeking only
    ///   its first three lines, not the whole file): a true cheap append,
    ///   `OpenOptions::append` plus new lines.
    /// - **Existing file still in the older, exact-count form** (written
    ///   by [`Self::to_canonical_text`]): a one-time migration -- the
    ///   whole file is read, parsed, and rewritten once in streaming
    ///   form with `new_records` included, so every *subsequent* append
    ///   against the same file takes the cheap path above.
    ///
    /// A crash mid-append can at worst leave a truncated FINAL line, never
    /// corrupt an earlier one -- `HubAuditRecord::from_canonical_line`'s
    /// own field-count/length checks already reject a truncated line
    /// distinctly from a valid one, and a reader can still recover every
    /// earlier, complete record.
    pub fn append_records_to_file(
        path: &std::path::Path,
        new_records: &[HubAuditRecord],
    ) -> Result<(), AuditAppendError> {
        if new_records.is_empty() {
            return Ok(());
        }
        match std::fs::metadata(path) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                validate_records(new_records, None).map_err(AuditAppendError::Parse)?;
                return atomic_replace(path, |file| {
                    write_streaming_header(file).map_err(AuditAppendError::Io)?;
                    for record in new_records {
                        write_record_line(file, record).map_err(AuditAppendError::Io)?;
                    }
                    Ok(())
                });
            }
            Err(error) => return Err(AuditAppendError::Io(error)),
        }

        let summary = Self::scan_file(path, |_| {}).map_err(AuditAppendError::from)?;
        let previous_sequence = match summary.next_sequence {
            Some(0) if summary.record_count == 0 => None,
            Some(next) => Some(next - 1),
            None => Some(u64::MAX),
        };
        validate_records(new_records, previous_sequence).map_err(AuditAppendError::Parse)?;

        if summary.format_version == HUB_AUDIT_FORMAT_VERSION && summary.streaming {
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(path)
                .map_err(AuditAppendError::Io)?;
            for record in new_records {
                write_record_line(&mut file, record).map_err(AuditAppendError::Io)?;
            }
            file.sync_all().map_err(AuditAppendError::Io)
        } else {
            // v1 cannot represent requested/granted capability evidence
            // separately. Stream every validated legacy record through
            // the documented reconstruction into a durable v2 replacement.
            atomic_replace(path, |file| {
                write_streaming_header(file).map_err(AuditAppendError::Io)?;
                let mut write_error = None;
                Self::scan_file(path, |record| {
                    if write_error.is_none() {
                        write_error = write_record_line(file, record).err();
                    }
                })
                .map_err(AuditAppendError::from)?;
                if let Some(error) = write_error {
                    return Err(AuditAppendError::Io(error));
                }
                for record in new_records {
                    write_record_line(file, record).map_err(AuditAppendError::Io)?;
                }
                Ok(())
            })
        }
    }

    pub fn to_canonical_text(&self) -> String {
        let mut out = String::new();
        out.push_str(MAGIC);
        out.push('\n');
        out.push_str(&HUB_AUDIT_FORMAT_VERSION.to_string());
        out.push('\n');
        out.push_str(&self.records.len().to_string());
        out.push('\n');
        for record in &self.records {
            out.push_str(&record.to_canonical_line());
            out.push('\n');
        }
        out
    }

    pub fn from_canonical_text(text: &str) -> Result<Self, AuditParseError> {
        let mut records = Vec::new();
        scan_text(text, |record| records.push(record.clone()))?;
        Ok(Self { records })
    }

    /// Same shape as [`Self::to_canonical_text`], but writes the
    /// `"unbounded"` sentinel in place of an exact record count --
    /// produces a file [`Self::append_records_to_file`] can cheaply
    /// append further records to later without a full rewrite. Prefer
    /// this over `to_canonical_text` for any file that may later be
    /// appended to; `to_canonical_text` remains the right choice for a
    /// one-shot, complete, final serialization (e.g. a golden fixture).
    pub fn to_canonical_text_streaming(&self) -> String {
        let mut out = String::new();
        out.push_str(MAGIC);
        out.push('\n');
        out.push_str(&HUB_AUDIT_FORMAT_VERSION.to_string());
        out.push('\n');
        out.push_str("unbounded");
        out.push('\n');
        for record in &self.records {
            out.push_str(&record.to_canonical_line());
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::HubToolVersion;

    fn sample_record(sequence: u64) -> HubAuditRecord {
        HubAuditRecord {
            sequence,
            request_id: HubRequestId::new("req-1").unwrap(),
            session_id: HubSessionId::new("sess-1").unwrap(),
            caller_identity: HubCallerIdentity::new("cli:local").unwrap(),
            tool_id: HubToolId::new("vector.turbovec").unwrap(),
            tool_version: HubToolVersion::new(0, 1, 0),
            adapter_provenance: "semantic-hub-turbovec 0.1.0\twith\ttabs".into(),
            operation_id: HubOperationId::new("vector.search").unwrap(),
            execution_mode: HubExecutionMode::InProcess,
            determinism: HubDeterminismClass::Unknown,
            trust_class: HubTrustClass::InProcessUnisolated,
            privacy_class: HubPrivacyClass::ProjectLocal,
            capabilities_requested: vec![HubCapability::VectorSearch, HubCapability::CpuCompute],
            capabilities_granted: vec![HubCapability::VectorSearch, HubCapability::CpuCompute],
            input_digest: HubDigest::of(b"input"),
            output_digest: HubDigest::of(b"output"),
            transaction_id: Some("txn-1-2".into()),
            resource_budget: HubResourceBudget::V0_CEILING,
            resource_usage: HubResourceUsage::default(),
            worker_state_after: HubWorkerState::Ready,
            status_code: "Success",
            fault_code: None,
        }
    }

    fn legacy_v1_text(records: &[HubAuditRecord], count: &str) -> String {
        let mut text = format!("{LEGACY_MAGIC_V1}\n{LEGACY_FORMAT_VERSION_V1}\n{count}\n");
        for record in records {
            let v2 = record.to_canonical_line();
            let fields: Vec<&str> = v2.split('\t').collect();
            assert_eq!(fields.len(), FIELD_COUNT_V2);
            let legacy = [&fields[0..13], &fields[14..16], &fields[17..22]].concat();
            assert_eq!(legacy.len(), FIELD_COUNT_V1);
            text.push_str(&legacy.join("\t"));
            text.push('\n');
        }
        text
    }

    #[test]
    fn single_record_round_trips_through_canonical_line() {
        let record = sample_record(0);
        let line = record.to_canonical_line();
        let parsed = HubAuditRecord::from_canonical_line(&line, None).unwrap();
        assert_eq!(parsed, record);
    }

    #[test]
    fn v2_keeps_requested_granted_and_transaction_identity_distinct() {
        let mut record = sample_record(0);
        record
            .capabilities_requested
            .push(HubCapability::NetworkAccess);
        record.capabilities_granted = vec![HubCapability::VectorSearch];
        record.transaction_id = Some("txn-escaped\\id".into());
        let parsed =
            HubAuditRecord::from_canonical_line(&record.to_canonical_line(), None).unwrap();
        assert_eq!(parsed.capabilities_requested, record.capabilities_requested);
        assert_eq!(parsed.capabilities_granted, record.capabilities_granted);
        assert_eq!(parsed.transaction_id, record.transaction_id);
    }

    #[test]
    fn v2_rejects_an_oversized_transaction_identity() {
        let mut record = sample_record(0);
        record.transaction_id = Some("x".repeat(MAX_AUDIT_TRANSACTION_ID_BYTES + 1));
        assert!(matches!(
            HubAuditRecord::from_canonical_line(&record.to_canonical_line(), None),
            Err(AuditParseError::TransactionIdTooLarge { .. })
        ));
    }

    #[test]
    fn legacy_v1_success_reconstructs_requested_and_non_sensitive_granted() {
        let mut record = sample_record(0);
        record.capabilities_requested =
            vec![HubCapability::VectorSearch, HubCapability::NetworkAccess];
        let parsed = HubAuditTrail::from_canonical_text(&legacy_v1_text(&[record], "1")).unwrap();
        let record = &parsed.records()[0];
        assert_eq!(
            record.capabilities_requested,
            vec![HubCapability::VectorSearch, HubCapability::NetworkAccess]
        );
        assert_eq!(
            record.capabilities_granted,
            vec![HubCapability::VectorSearch]
        );
        assert_eq!(record.transaction_id, None);
    }

    #[test]
    fn legacy_v1_rejection_reconstructs_no_granted_capabilities() {
        let mut record = sample_record(0);
        record.status_code = "Rejected";
        record.fault_code = Some("SensitiveCapabilityDenied");
        record.capabilities_requested = vec![HubCapability::NetworkAccess];
        let parsed = HubAuditTrail::from_canonical_text(&legacy_v1_text(&[record], "1")).unwrap();
        assert_eq!(
            parsed.records()[0].capabilities_requested,
            vec![HubCapability::NetworkAccess]
        );
        assert!(parsed.records()[0].capabilities_granted.is_empty());
    }

    #[test]
    fn a_record_granting_every_sensitive_capability_round_trips() {
        // Regression test: a caller can grant (not just require) a
        // sensitive capability such as NetworkAccess -- admission denies
        // using it, but HubCapabilitySet::grant() does not filter it out
        // of what gets recorded in the audit trail. An earlier version of
        // this parser only recognized the 11 non-sensitive names, so any
        // request granting one sensitive capability corrupted the whole
        // audit log for every subsequent read.
        let mut record = sample_record(0);
        record.capabilities_granted = vec![
            HubCapability::NetworkAccess,
            HubCapability::ArbitraryFilesystemRead,
            HubCapability::ArbitraryFilesystemWrite,
            HubCapability::ProcessSpawn,
            HubCapability::DeviceAccess,
            HubCapability::EnvironmentRead,
            HubCapability::SecretRead,
            HubCapability::ProjectMutation,
            HubCapability::SemanticStateMutation,
        ];
        let line = record.to_canonical_line();
        let parsed = HubAuditRecord::from_canonical_line(&line, None).unwrap();
        assert_eq!(parsed, record);
    }

    #[test]
    fn tab_and_backslash_in_free_text_fields_survive_round_trip() {
        let record = sample_record(0);
        let line = record.to_canonical_line();
        let parsed = HubAuditRecord::from_canonical_line(&line, None).unwrap();
        assert_eq!(
            parsed.adapter_provenance,
            "semantic-hub-turbovec 0.1.0\twith\ttabs"
        );
    }

    #[test]
    fn trail_round_trips_multiple_records_through_canonical_text() {
        let mut trail = HubAuditTrail::new();
        trail.push(sample_record(0));
        trail.push(sample_record(1));
        let text = trail.to_canonical_text();
        let parsed = HubAuditTrail::from_canonical_text(&text).unwrap();
        assert_eq!(parsed.records().len(), 2);
        assert_eq!(parsed.records(), trail.records());
    }

    #[test]
    fn trail_rejects_non_monotonic_sequence() {
        let mut trail = HubAuditTrail::new();
        trail.push(sample_record(1));
        trail.push(sample_record(0)); // out of order
        let text = trail.to_canonical_text();
        assert!(matches!(
            HubAuditTrail::from_canonical_text(&text),
            Err(AuditParseError::NonMonotonicSequence { .. })
        ));
    }

    #[test]
    fn trail_rejects_wrong_magic_header() {
        assert!(matches!(
            HubAuditTrail::from_canonical_text("not-the-magic\n1\n0\n"),
            Err(AuditParseError::MissingMagicHeader)
        ));
    }

    #[test]
    fn trail_rejects_unsupported_format_version() {
        let text = format!("{MAGIC}\n999\n0\n");
        assert!(matches!(
            HubAuditTrail::from_canonical_text(&text),
            Err(AuditParseError::UnsupportedFormatVersion(999))
        ));
    }

    #[test]
    fn trail_rejects_declared_count_mismatch() {
        let mut trail = HubAuditTrail::new();
        trail.push(sample_record(0));
        let text = trail.to_canonical_text();
        // Header is exactly `MAGIC\n<version>\n<count>\n<records...>`; rewrite
        // just the count line (line index 2) rather than a fragile string
        // replace, since both the format version and the record count for a
        // single-record trail happen to render as the digit "1".
        let mut lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[2], "1");
        lines[2] = "2";
        let corrupted = lines.join("\n") + "\n";
        assert!(matches!(
            HubAuditTrail::from_canonical_text(&corrupted),
            Err(AuditParseError::FieldCount { .. })
        ));
    }

    #[test]
    fn oversized_record_line_is_rejected_before_parsing_fields() {
        let huge = "x".repeat(MAX_AUDIT_RECORD_BYTES + 1);
        assert!(matches!(
            HubAuditRecord::from_canonical_line(&huge, None),
            Err(AuditParseError::RecordTooLarge { .. })
        ));
    }

    #[test]
    fn next_sequence_is_zero_for_empty_trail_and_increments_after_push() {
        let mut trail = HubAuditTrail::new();
        assert_eq!(trail.next_sequence(), Some(0));
        trail.push(sample_record(0));
        assert_eq!(trail.next_sequence(), Some(1));
    }

    #[test]
    fn next_sequence_is_none_when_the_last_record_is_at_u64_max() {
        // Regression test: a plain `+ 1` would panic in an
        // overflow-checked build or silently wrap to 0 in release,
        // violating the trail's own monotonic-sequence invariant.
        let mut trail = HubAuditTrail::new();
        trail.push(sample_record(u64::MAX));
        assert_eq!(trail.next_sequence(), None);
    }

    #[test]
    fn every_hub_fault_code_is_recognized_by_the_audit_parser() {
        // Regression test: `leak_known_fault_code`'s CODES list is a
        // hand-maintained duplicate of every `HubFault::code()` string,
        // not derived from the enum -- a fault variant added to
        // `crate::fault::HubFault` without a matching entry here makes any
        // audit record carrying it unparseable on the very next read. This
        // constructs one instance of every variant (using placeholder
        // payloads for the ones that carry data) and confirms each
        // survives a full record round-trip, not just a bare
        // `leak_known_fault_code` call.
        use crate::fault::HubFault;
        use crate::resource::{HubBudgetExceeded, HubResourceKind};

        let budget_err = HubBudgetExceeded {
            kind: HubResourceKind::InputBytes,
            limit: 1,
            attempted: 2,
        };
        let all_faults: Vec<HubFault> = vec![
            HubFault::UnknownTool,
            HubFault::UnknownOperation,
            HubFault::ApiVersionUnsupported,
            HubFault::SchemaVersionUnsupported,
            HubFault::DescriptorIncompatible,
            HubFault::InputRejected("x".into()),
            HubFault::CapabilityDenied("x".into()),
            HubFault::PrivacyDenied("x".into()),
            HubFault::SensitiveCapabilityDenied("x".into()),
            HubFault::WorkerDegraded,
            HubFault::SessionLimitExceeded(budget_err),
            HubFault::ResourceBudgetInvalid(budget_err),
            HubFault::QueueFull,
            HubFault::ToolDisabled,
            HubFault::ToolQuarantined,
            HubFault::DeadlineExceeded,
            HubFault::Cancelled,
            HubFault::ResourceExhausted(budget_err),
            HubFault::WorkerBusy,
            HubFault::ToolDeclaredFailure("x".into()),
            HubFault::WorkerPanicked("x".into()),
            HubFault::ProtocolViolation("x".into()),
            HubFault::OutputRejected("x".into()),
            HubFault::AuditProvenanceFailure("x".into()),
            HubFault::PersistenceFailed("x".into()),
            HubFault::RecoveryRequired("x".into()),
            HubFault::SequenceExhausted,
            HubFault::InternalHubFault("x".into()),
        ];
        for (i, fault) in all_faults.iter().enumerate() {
            let mut record = sample_record(i as u64);
            record.status_code = "Rejected";
            record.fault_code = Some(fault.code());
            let line = record.to_canonical_line();
            let parsed = HubAuditRecord::from_canonical_line(
                &line,
                if i == 0 { None } else { Some(i as u64 - 1) },
            )
            .unwrap_or_else(|e| panic!("fault code {:?} did not round-trip: {e}", fault.code()));
            assert_eq!(parsed.fault_code, Some(fault.code()));
        }
    }

    #[test]
    fn find_by_request_locates_the_matching_record() {
        let mut trail = HubAuditTrail::new();
        trail.push(sample_record(0));
        let found = trail
            .find_by_request(&HubRequestId::new("req-1").unwrap())
            .unwrap();
        assert_eq!(found.sequence, 0);
        assert!(trail
            .find_by_request(&HubRequestId::new("req-missing").unwrap())
            .is_none());
    }

    fn temp_audit_path(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "semantic-hub-audit-streaming-test-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("audit.log")
    }

    #[test]
    fn file_scan_filters_request_and_session_while_deriving_next_sequence() {
        let path = temp_audit_path("file-scan");
        let first = sample_record(2);
        let mut second = sample_record(4);
        second.request_id = HubRequestId::new("req-2").unwrap();
        second.session_id = HubSessionId::new("sess-2").unwrap();
        HubAuditTrail::append_records_to_file(&path, &[first, second]).unwrap();

        let found =
            HubAuditTrail::find_by_request_file(&path, &HubRequestId::new("req-2").unwrap())
                .unwrap()
                .unwrap();
        assert_eq!(found.sequence, 4);

        let mut session_sequences = Vec::new();
        let summary = HubAuditTrail::scan_file_by_session(
            &path,
            &HubSessionId::new("sess-1").unwrap(),
            |record| session_sequences.push(record.sequence),
        )
        .unwrap();
        assert_eq!(session_sequences, vec![2]);
        assert_eq!(summary.record_count, 2);
        assert_eq!(summary.next_sequence, Some(5));
        assert_eq!(summary.format_version, HUB_AUDIT_FORMAT_VERSION);
        assert!(summary.streaming);
        assert_eq!(HubAuditTrail::next_sequence_file(&path).unwrap(), Some(5));
    }

    #[test]
    fn file_lookup_validates_records_after_an_early_match() {
        let path = temp_audit_path("validate-after-match");
        HubAuditTrail::append_records_to_file(&path, &[sample_record(0)]).unwrap();
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        file.write_all(b"truncated-but-newline-terminated\n")
            .unwrap();
        file.sync_all().unwrap();

        assert!(matches!(
            HubAuditTrail::find_by_request_file(&path, &HubRequestId::new("req-1").unwrap()),
            Err(AuditFileError::Parse(AuditParseError::FieldCount { .. }))
        ));
    }

    #[test]
    fn partial_final_record_is_explicit_and_append_preserves_the_file() {
        let path = temp_audit_path("partial-tail");
        HubAuditTrail::append_records_to_file(&path, &[sample_record(0)]).unwrap();
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        file.write_all(b"partial").unwrap();
        file.sync_all().unwrap();
        let before = std::fs::read(&path).unwrap();

        assert!(matches!(
            HubAuditTrail::scan_file(&path, |_| {}),
            Err(AuditFileError::PartialFinalRecord {
                valid_records: 1,
                bytes: 7
            })
        ));
        assert!(matches!(
            HubAuditTrail::append_records_to_file(&path, &[sample_record(1)]),
            Err(AuditAppendError::PartialFinalRecord {
                valid_records: 1,
                bytes: 7
            })
        ));
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }

    #[test]
    fn file_scan_rejects_an_oversized_line_with_bounded_retention() {
        let path = temp_audit_path("oversized-line");
        let text = format!(
            "{MAGIC}\n{HUB_AUDIT_FORMAT_VERSION}\nunbounded\n{}\n",
            "x".repeat(MAX_AUDIT_RECORD_BYTES + 1)
        );
        std::fs::write(&path, text).unwrap();
        assert!(matches!(
            HubAuditTrail::scan_file(&path, |_| {}),
            Err(AuditFileError::Parse(AuditParseError::RecordTooLarge {
                max: MAX_AUDIT_RECORD_BYTES,
                actual
            })) if actual == MAX_AUDIT_RECORD_BYTES + 1
        ));
    }

    #[test]
    fn streaming_text_round_trips_through_from_canonical_text() {
        let mut trail = HubAuditTrail::new();
        trail.push(sample_record(0));
        trail.push(sample_record(1));
        let text = trail.to_canonical_text_streaming();
        assert!(text.lines().nth(2) == Some("unbounded"));
        let parsed = HubAuditTrail::from_canonical_text(&text).unwrap();
        assert_eq!(parsed.records(), trail.records());
    }

    #[test]
    fn find_by_request_streaming_matches_the_non_streaming_result() {
        let mut trail = HubAuditTrail::new();
        trail.push(sample_record(0));
        trail.push(sample_record(1));
        let text = trail.to_canonical_text();
        let target = HubRequestId::new("req-1").unwrap();
        let streaming = HubAuditTrail::find_by_request_streaming(&text, &target)
            .unwrap()
            .unwrap();
        let non_streaming = trail.find_by_request(&target).unwrap();
        assert_eq!(&streaming, non_streaming);
    }

    #[test]
    fn find_by_request_streaming_accepts_the_unbounded_streaming_header() {
        // Regression test: an earlier version of this function still
        // required the count line to parse as a plain usize, so it
        // rejected any file written in the "unbounded" streaming form
        // (the DEFAULT form since `save_audit_trail` started always
        // using `to_canonical_text_streaming`) with a spurious
        // "record count is not a usize" error -- reproduced via a real
        // `smc hub audit` CLI call against a project whose log had
        // already been touched by `smc hub session`, not just at the
        // unit level (this crate's own tests for this function
        // previously only ever exercised the numeric-count form).
        let mut trail = HubAuditTrail::new();
        trail.push(sample_record(0));
        trail.push(sample_record(1));
        let text = trail.to_canonical_text_streaming();
        let target = HubRequestId::new("req-1").unwrap();
        let found = HubAuditTrail::find_by_request_streaming(&text, &target)
            .unwrap()
            .unwrap();
        assert_eq!(found.sequence, 0);
    }

    #[test]
    fn next_sequence_streaming_accepts_the_unbounded_streaming_header() {
        let mut trail = HubAuditTrail::new();
        trail.push(sample_record(0));
        trail.push(sample_record(4));
        let text = trail.to_canonical_text_streaming();
        assert_eq!(
            HubAuditTrail::next_sequence_streaming(&text).unwrap(),
            Some(5)
        );
    }

    #[test]
    fn find_by_request_streaming_returns_none_for_an_unknown_id() {
        let trail_text = HubAuditTrail::new().to_canonical_text();
        assert_eq!(
            HubAuditTrail::find_by_request_streaming(
                &trail_text,
                &HubRequestId::new("req-missing").unwrap()
            )
            .unwrap(),
            None
        );
    }

    #[test]
    fn next_sequence_streaming_matches_the_non_streaming_result() {
        let mut trail = HubAuditTrail::new();
        trail.push(sample_record(0));
        trail.push(sample_record(5));
        let text = trail.to_canonical_text();
        assert_eq!(
            HubAuditTrail::next_sequence_streaming(&text).unwrap(),
            trail.next_sequence()
        );
    }

    #[test]
    fn next_sequence_streaming_is_zero_for_an_empty_trail() {
        let text = HubAuditTrail::new().to_canonical_text();
        assert_eq!(
            HubAuditTrail::next_sequence_streaming(&text).unwrap(),
            Some(0)
        );
    }

    #[test]
    fn append_records_to_file_creates_a_fresh_streaming_file() {
        let path = temp_audit_path("fresh");
        HubAuditTrail::append_records_to_file(&path, &[sample_record(0), sample_record(1)])
            .unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let trail = HubAuditTrail::from_canonical_text(&text).unwrap();
        assert_eq!(trail.records().len(), 2);
        assert_eq!(text.lines().nth(2), Some("unbounded"));
    }

    #[test]
    fn append_records_to_file_cheaply_appends_to_an_already_streaming_file() {
        let path = temp_audit_path("append");
        HubAuditTrail::append_records_to_file(&path, &[sample_record(0)]).unwrap();
        HubAuditTrail::append_records_to_file(&path, &[sample_record(1), sample_record(2)])
            .unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let trail = HubAuditTrail::from_canonical_text(&text).unwrap();
        assert_eq!(trail.records().len(), 3);
        assert_eq!(
            trail
                .records()
                .iter()
                .map(|r| r.sequence)
                .collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
    }

    #[test]
    fn append_rejects_a_non_monotonic_new_sequence_without_modifying_the_file() {
        let path = temp_audit_path("append-sequence");
        HubAuditTrail::append_records_to_file(&path, &[sample_record(1)]).unwrap();
        let before = std::fs::read(&path).unwrap();
        assert!(matches!(
            HubAuditTrail::append_records_to_file(&path, &[sample_record(1)]),
            Err(AuditAppendError::Parse(
                AuditParseError::NonMonotonicSequence { .. }
            ))
        ));
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }

    #[test]
    fn append_migrates_legacy_v1_to_explicit_v2_fields() {
        let path = temp_audit_path("migrate-v1");
        let mut legacy = sample_record(0);
        legacy.capabilities_requested =
            vec![HubCapability::VectorSearch, HubCapability::NetworkAccess];
        std::fs::write(&path, legacy_v1_text(&[legacy], "1")).unwrap();

        HubAuditTrail::append_records_to_file(&path, &[sample_record(1)]).unwrap();

        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(text.lines().next(), Some(MAGIC));
        assert_eq!(text.lines().nth(1), Some("2"));
        let parsed = HubAuditTrail::from_canonical_text(&text).unwrap();
        assert_eq!(parsed.records().len(), 2);
        assert_eq!(
            parsed.records()[0].capabilities_requested,
            vec![HubCapability::VectorSearch, HubCapability::NetworkAccess]
        );
        assert_eq!(
            parsed.records()[0].capabilities_granted,
            vec![HubCapability::VectorSearch]
        );
        assert_eq!(parsed.records()[0].transaction_id, None);
    }

    #[test]
    fn append_records_to_file_migrates_an_exact_count_file_then_appends() {
        // A file written by the OLD full-rewrite `to_canonical_text` (an
        // exact numeric count, not the "unbounded" sentinel) must still
        // be appendable -- the one-time migration path.
        let path = temp_audit_path("migrate");
        let mut legacy = HubAuditTrail::new();
        legacy.push(sample_record(0));
        std::fs::write(&path, legacy.to_canonical_text().as_bytes()).unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap().lines().nth(2),
            Some("1"),
            "precondition: legacy file uses an exact numeric count"
        );

        HubAuditTrail::append_records_to_file(&path, &[sample_record(1)]).unwrap();

        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            text.lines().nth(2),
            Some("unbounded"),
            "migration must convert the header to the streaming sentinel"
        );
        let trail = HubAuditTrail::from_canonical_text(&text).unwrap();
        assert_eq!(trail.records().len(), 2);
        assert_eq!(
            trail
                .records()
                .iter()
                .map(|r| r.sequence)
                .collect::<Vec<_>>(),
            vec![0, 1]
        );

        // And the now-migrated file supports a further cheap append too.
        HubAuditTrail::append_records_to_file(&path, &[sample_record(2)]).unwrap();
        let final_text = std::fs::read_to_string(&path).unwrap();
        let final_trail = HubAuditTrail::from_canonical_text(&final_text).unwrap();
        assert_eq!(final_trail.records().len(), 3);
    }

    #[test]
    fn append_records_to_file_with_no_new_records_is_a_harmless_no_op() {
        let path = temp_audit_path("noop");
        // No file exists yet, and nothing is appended -- must not create
        // an empty/malformed file.
        HubAuditTrail::append_records_to_file(&path, &[]).unwrap();
        assert!(!path.is_file());
    }
}
