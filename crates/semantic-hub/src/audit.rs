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

use crate::capability::HubCapability;
use crate::execution::{HubDeterminismClass, HubExecutionMode, HubPrivacyClass, HubTrustClass};
use crate::ids::{
    HubCallerIdentity, HubOperationId, HubRequestId, HubSessionId, HubToolId, HubToolVersion,
};
use crate::provenance::HubDigest;
use crate::resource::{HubResourceBudget, HubResourceUsage};
use crate::worker::HubWorkerState;

/// Format version of the canonical audit text produced by [`HubAuditTrail`].
pub const HUB_AUDIT_FORMAT_VERSION: u32 = 1;
const MAGIC: &str = "semantic-hub.audit.v1";

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
    pub capabilities_granted: Vec<HubCapability>,
    pub input_digest: HubDigest,
    pub output_digest: HubDigest,
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
        }
    }
}

const FIELD_COUNT: usize = 20;

fn pack_budget(b: &HubResourceBudget) -> String {
    [
        b.wall_time_millis.to_string(),
        b.memory_bytes.to_string(),
        b.input_bytes.to_string(),
        b.output_bytes.to_string(),
        b.index_item_count.to_string(),
        b.vector_dimensions.to_string(),
        b.result_count.to_string(),
        b.queue_depth.to_string(),
        b.concurrent_requests.to_string(),
        b.storage_read_bytes.to_string(),
        b.storage_write_bytes.to_string(),
        b.audit_bytes.to_string(),
    ]
    .join(",")
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
        let caps = self
            .capabilities_granted
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>()
            .join(";");
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
            escape_field(&caps),
            self.input_digest.to_string(),
            self.output_digest.to_string(),
            pack_budget(&self.resource_budget),
            pack_usage(&self.resource_usage),
            self.worker_state_after.to_string(),
            self.status_code.to_string(),
            self.fault_code.unwrap_or("-").to_string(),
        ];
        debug_assert_eq!(fields.len(), FIELD_COUNT);
        fields.join("\t")
    }

    pub fn from_canonical_line(
        line: &str,
        previous_sequence: Option<u64>,
    ) -> Result<Self, AuditParseError> {
        if line.len() > MAX_AUDIT_RECORD_BYTES {
            return Err(AuditParseError::RecordTooLarge {
                max: MAX_AUDIT_RECORD_BYTES,
                actual: line.len(),
            });
        }
        let raw: Vec<&str> = line.split('\t').collect();
        if raw.len() != FIELD_COUNT {
            return Err(AuditParseError::FieldCount {
                expected: FIELD_COUNT,
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
        let caps_raw = unescape_field(raw[12])?;
        let mut capabilities_granted = Vec::new();
        if !caps_raw.is_empty() {
            for part in caps_raw.split(';') {
                capabilities_granted.push(match part {
                    "VectorIndexCreate" => HubCapability::VectorIndexCreate,
                    "VectorIndexRead" => HubCapability::VectorIndexRead,
                    "VectorIndexMutate" => HubCapability::VectorIndexMutate,
                    "VectorSearch" => HubCapability::VectorSearch,
                    "VectorFilteredSearch" => HubCapability::VectorFilteredSearch,
                    "VectorIndexPersist" => HubCapability::VectorIndexPersist,
                    "CpuCompute" => HubCapability::CpuCompute,
                    "MemoryAllocateBounded" => HubCapability::MemoryAllocateBounded,
                    "PrivateStorageRead" => HubCapability::PrivateStorageRead,
                    "PrivateStorageWrite" => HubCapability::PrivateStorageWrite,
                    "ClockMonotonic" => HubCapability::ClockMonotonic,
                    _ => return Err(AuditParseError::Malformed("capabilities_granted")),
                });
            }
        }

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
        let input_digest = parse_digest(raw[13])?;
        let output_digest = parse_digest(raw[14])?;
        let resource_budget = unpack_budget(raw[15])?;
        let resource_usage = unpack_usage(raw[16])?;
        let worker_state_after = match raw[17] {
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
        let status_code: &'static str = leak_known_status(raw[18])?;
        let fault_code = if raw[19] == "-" {
            None
        } else {
            Some(leak_known_fault_code(raw[19])?)
        };

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
            capabilities_granted,
            input_digest,
            output_digest,
            resource_budget,
            resource_usage,
            worker_state_after,
            status_code,
            fault_code,
        })
    }
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
    const CODES: &[&str] = &[
        "UnknownTool",
        "UnknownOperation",
        "ApiVersionUnsupported",
        "SchemaVersionUnsupported",
        "DescriptorIncompatible",
        "InputRejected",
        "CapabilityDenied",
        "PrivacyDenied",
        "ResourceBudgetInvalid",
        "QueueFull",
        "ToolDisabled",
        "ToolQuarantined",
        "DeadlineExceeded",
        "Cancelled",
        "ResourceExhausted",
        "ToolDeclaredFailure",
        "WorkerPanicked",
        "ProtocolViolation",
        "OutputRejected",
        "AuditProvenanceFailure",
        "InternalHubFault",
    ];
    CODES
        .iter()
        .find(|c| **c == s)
        .copied()
        .ok_or(AuditParseError::Malformed("fault_code"))
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

    pub fn next_sequence(&self) -> u64 {
        self.records.last().map(|r| r.sequence + 1).unwrap_or(0)
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
        let mut lines = text.lines();
        let magic = lines.next().ok_or(AuditParseError::MissingMagicHeader)?;
        if magic != MAGIC {
            return Err(AuditParseError::MissingMagicHeader);
        }
        let version: u32 = lines
            .next()
            .ok_or(AuditParseError::Malformed("missing format version"))?
            .parse()
            .map_err(|_| AuditParseError::Malformed("format version is not a u32"))?;
        if version != HUB_AUDIT_FORMAT_VERSION {
            return Err(AuditParseError::UnsupportedFormatVersion(version));
        }
        let declared_count: usize = lines
            .next()
            .ok_or(AuditParseError::Malformed("missing record count"))?
            .parse()
            .map_err(|_| AuditParseError::Malformed("record count is not a usize"))?;

        let mut records = Vec::new();
        let mut previous_sequence = None;
        for line in lines {
            if line.is_empty() {
                continue;
            }
            let record = HubAuditRecord::from_canonical_line(line, previous_sequence)?;
            previous_sequence = Some(record.sequence);
            records.push(record);
        }
        if records.len() != declared_count {
            return Err(AuditParseError::FieldCount {
                expected: declared_count,
                actual: records.len(),
            });
        }
        Ok(Self { records })
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
            capabilities_granted: vec![HubCapability::VectorSearch, HubCapability::CpuCompute],
            input_digest: HubDigest::of(b"input"),
            output_digest: HubDigest::of(b"output"),
            resource_budget: HubResourceBudget::V0_CEILING,
            resource_usage: HubResourceUsage::default(),
            worker_state_after: HubWorkerState::Ready,
            status_code: "Success",
            fault_code: None,
        }
    }

    #[test]
    fn single_record_round_trips_through_canonical_line() {
        let record = sample_record(0);
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
        assert_eq!(trail.next_sequence(), 0);
        trail.push(sample_record(0));
        assert_eq!(trail.next_sequence(), 1);
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
}
