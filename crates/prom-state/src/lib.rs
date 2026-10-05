#![allow(clippy::derivable_impls)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FactValue {
    Bool(bool),
    I32(i32),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FactResolution {
    Certain(FactValue),
    Uncertain(Vec<FactValue>),
    Conflicted(Vec<FactValue>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextWindow {
    pub name: String,
}

impl ContextWindow {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateEpoch(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionMetadata {
    pub key: String,
    pub from_epoch: StateEpoch,
    pub to_epoch: StateEpoch,
    pub reason: String,
}

pub type StateTransitionMetadata = TransitionMetadata;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateRecord {
    pub key: String,
    pub resolution: FactResolution,
    pub context: ContextWindow,
    pub epoch: StateEpoch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateUpdate {
    pub key: String,
    pub resolution: FactResolution,
    pub context: ContextWindow,
    pub reason: String,
}

impl StateUpdate {
    pub fn new(
        key: impl Into<String>,
        resolution: FactResolution,
        context: ContextWindow,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            resolution,
            context,
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateSnapshot {
    pub epoch: StateEpoch,
    pub records: BTreeMap<String, StateRecord>,
}

pub const STATE_SNAPSHOT_ARCHIVE_FORMAT_VERSION: u32 = 1;
pub const STATE_ROLLBACK_ARTIFACT_FORMAT_VERSION: u32 = 1;
const STATE_SNAPSHOT_ARCHIVE_MAGIC: &str = "semantic_state_snapshot_archive";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateSnapshotArchive {
    pub format_version: u32,
    pub snapshot: StateSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateRollbackCheckpoint {
    pub checkpoint_ordinal: u32,
    pub snapshot: StateSnapshotArchive,
    pub applied_transition_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateRollbackArtifact {
    pub format_version: u32,
    pub head_epoch: StateEpoch,
    pub checkpoints: Vec<StateRollbackCheckpoint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateSnapshotArchiveFormatError {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateRollbackCode {
    UnsupportedFormatVersion,
    StoreHistoryMismatch,
    HeadEpochMismatch,
    MissingCheckpoint,
    NonMonotonicCheckpointOrdinal,
    NonMonotonicTransitionCount,
    SnapshotEpochMismatch,
    TransitionCountOutOfRange,
    /// The selected checkpoint snapshot violates `StateSnapshot::validate`
    /// (#1782); rollback is not a bypass around snapshot admission.
    InvalidCheckpointSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateRollbackError {
    pub code: StateRollbackCode,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateRollbackAdvance {
    pub checkpoint_ordinal: u32,
    pub from_epoch: StateEpoch,
    pub to_epoch: StateEpoch,
    pub retained_transition_count: usize,
    pub dropped_transition_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateValidationCode {
    EmptyKey,
    EmptyContext,
    EmptyReason,
    EmptyAlternatives,
    DuplicateAlternatives,
    NotEnoughAlternatives,
    /// A snapshot record is stored under a different key than its own (#1782).
    KeyMismatch,
    /// A snapshot record claims an epoch later than the snapshot (#1782).
    RecordEpochAfterSnapshot,
    /// No further epoch can be issued (#1783).
    EpochExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateValidationError {
    pub code: StateValidationCode,
    pub message: String,
}

impl StateValidationError {
    pub fn new(code: StateValidationCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl StateRollbackError {
    pub fn new(code: StateRollbackCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl core::fmt::Display for StateValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for StateValidationError {}

impl core::fmt::Display for StateRollbackError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for StateRollbackError {}

impl StateSnapshotArchiveFormatError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl core::fmt::Display for StateSnapshotArchiveFormatError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "state snapshot archive format error: {}", self.message)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for StateSnapshotArchiveFormatError {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SemanticStateStore {
    epoch: StateEpoch,
    records: BTreeMap<String, StateRecord>,
    transitions: Vec<TransitionMetadata>,
}

impl SemanticStateStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn epoch(&self) -> StateEpoch {
        self.epoch
    }

    pub fn get(&self, key: &str) -> Option<&StateRecord> {
        self.records.get(key)
    }

    pub fn records(&self) -> &BTreeMap<String, StateRecord> {
        &self.records
    }

    pub fn transitions(&self) -> &[TransitionMetadata] {
        &self.transitions
    }

    pub fn snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            epoch: self.epoch,
            records: self.records.clone(),
        }
    }

    pub fn apply(
        &mut self,
        update: StateUpdate,
    ) -> Result<TransitionMetadata, StateValidationError> {
        validate_update(&update)?;
        let from_epoch = self.epoch;
        // #1783: checked, not saturating — saturating would report an accepted
        // update without advancing identity. Fails before any mutation.
        let to_epoch = StateEpoch(self.epoch.0.checked_add(1).ok_or_else(epoch_exhausted)?);
        let record = StateRecord {
            key: update.key.clone(),
            resolution: update.resolution,
            context: update.context,
            epoch: to_epoch,
        };
        self.records.insert(update.key.clone(), record);
        let transition = TransitionMetadata {
            key: update.key,
            from_epoch,
            to_epoch,
            reason: update.reason,
        };
        self.transitions.push(transition.clone());
        self.epoch = to_epoch;
        Ok(transition)
    }

    /// Installs a snapshot only if it is valid (#1782). An invalid snapshot
    /// returns `Err` and leaves the store unchanged.
    pub fn restore(&mut self, snapshot: StateSnapshot) -> Result<(), StateValidationError> {
        snapshot.validate()?;
        self.epoch = snapshot.epoch;
        self.records = snapshot.records;
        Ok(())
    }

    pub fn apply_rollback(
        &mut self,
        artifact: &StateRollbackArtifact,
        checkpoint_ordinal: u32,
    ) -> Result<StateRollbackAdvance, StateRollbackError> {
        if artifact.format_version != STATE_ROLLBACK_ARTIFACT_FORMAT_VERSION {
            return Err(StateRollbackError::new(
                StateRollbackCode::UnsupportedFormatVersion,
                format!(
                    "unsupported rollback artifact format version {}; expected {}",
                    artifact.format_version, STATE_ROLLBACK_ARTIFACT_FORMAT_VERSION
                ),
            ));
        }

        let current_transition_count = self.transitions.len();
        if self.epoch.0 != current_transition_count as u64 {
            return Err(StateRollbackError::new(
                StateRollbackCode::StoreHistoryMismatch,
                format!(
                    "store epoch {} does not match transition count {}",
                    self.epoch.0, current_transition_count
                ),
            ));
        }
        if artifact.head_epoch != self.epoch {
            return Err(StateRollbackError::new(
                StateRollbackCode::HeadEpochMismatch,
                format!(
                    "rollback artifact head epoch {} does not match store epoch {}",
                    artifact.head_epoch.0, self.epoch.0
                ),
            ));
        }

        let mut previous_ordinal = None;
        let mut previous_transition_count = None;
        let mut selected_checkpoint = None;

        for checkpoint in &artifact.checkpoints {
            if let Some(prev) = previous_ordinal {
                if checkpoint.checkpoint_ordinal <= prev {
                    return Err(StateRollbackError::new(
                        StateRollbackCode::NonMonotonicCheckpointOrdinal,
                        "rollback checkpoints must be strictly ordered by checkpoint ordinal",
                    ));
                }
            }
            if let Some(prev) = previous_transition_count {
                if checkpoint.applied_transition_count <= prev {
                    return Err(StateRollbackError::new(
                        StateRollbackCode::NonMonotonicTransitionCount,
                        "rollback checkpoints must be strictly ordered by applied transition count",
                    ));
                }
            }

            let expected_epoch = StateEpoch(checkpoint.applied_transition_count as u64);
            if checkpoint.snapshot.snapshot.epoch != expected_epoch {
                return Err(StateRollbackError::new(
                    StateRollbackCode::SnapshotEpochMismatch,
                    format!(
                        "rollback checkpoint {} carries snapshot epoch {} but transition count {}",
                        checkpoint.checkpoint_ordinal,
                        checkpoint.snapshot.snapshot.epoch.0,
                        checkpoint.applied_transition_count
                    ),
                ));
            }
            if checkpoint.applied_transition_count > current_transition_count {
                return Err(StateRollbackError::new(
                    StateRollbackCode::TransitionCountOutOfRange,
                    format!(
                        "rollback checkpoint {} references transition count {} beyond current store history {}",
                        checkpoint.checkpoint_ordinal,
                        checkpoint.applied_transition_count,
                        current_transition_count
                    ),
                ));
            }
            if checkpoint.checkpoint_ordinal == checkpoint_ordinal {
                selected_checkpoint = Some(checkpoint.clone());
            }

            previous_ordinal = Some(checkpoint.checkpoint_ordinal);
            previous_transition_count = Some(checkpoint.applied_transition_count);
        }

        let checkpoint = selected_checkpoint.ok_or_else(|| {
            StateRollbackError::new(
                StateRollbackCode::MissingCheckpoint,
                format!(
                    "rollback artifact does not contain checkpoint ordinal {}",
                    checkpoint_ordinal
                ),
            )
        })?;

        checkpoint.snapshot.snapshot.validate().map_err(|err| {
            StateRollbackError::new(
                StateRollbackCode::InvalidCheckpointSnapshot,
                format!(
                    "rollback checkpoint {} carries an invalid snapshot: {}",
                    checkpoint.checkpoint_ordinal, err
                ),
            )
        })?;

        let from_epoch = self.epoch;
        let to_epoch = checkpoint.snapshot.snapshot.epoch;
        let retained_transition_count = checkpoint.applied_transition_count;
        let dropped_transition_count = current_transition_count - retained_transition_count;

        self.records = checkpoint.snapshot.snapshot.records;
        self.epoch = to_epoch;
        self.transitions.truncate(retained_transition_count);

        Ok(StateRollbackAdvance {
            checkpoint_ordinal,
            from_epoch,
            to_epoch,
            retained_transition_count,
            dropped_transition_count,
        })
    }
}

fn epoch_exhausted() -> StateValidationError {
    StateValidationError::new(
        StateValidationCode::EpochExhausted,
        "state epoch space is exhausted; no further update can be accepted",
    )
}

fn validate_key(key: &str) -> Result<(), StateValidationError> {
    if key.trim().is_empty() {
        return Err(StateValidationError::new(
            StateValidationCode::EmptyKey,
            "state key must not be empty",
        ));
    }
    Ok(())
}

fn validate_context(context: &ContextWindow) -> Result<(), StateValidationError> {
    if context.name.trim().is_empty() {
        return Err(StateValidationError::new(
            StateValidationCode::EmptyContext,
            "context window must not be empty",
        ));
    }
    Ok(())
}

fn validate_update(update: &StateUpdate) -> Result<(), StateValidationError> {
    validate_key(&update.key)?;
    validate_context(&update.context)?;
    if update.reason.trim().is_empty() {
        return Err(StateValidationError::new(
            StateValidationCode::EmptyReason,
            "transition reason must not be empty",
        ));
    }
    validate_resolution(&update.resolution)
}

fn validate_resolution(resolution: &FactResolution) -> Result<(), StateValidationError> {
    match resolution {
        FactResolution::Certain(_) => Ok(()),
        FactResolution::Uncertain(values) | FactResolution::Conflicted(values) => {
            if values.is_empty() {
                return Err(StateValidationError::new(
                    StateValidationCode::EmptyAlternatives,
                    "uncertain/conflicted resolution requires alternatives",
                ));
            }
            if values.len() < 2 {
                return Err(StateValidationError::new(
                    StateValidationCode::NotEnoughAlternatives,
                    "uncertain/conflicted resolution requires at least two alternatives",
                ));
            }
            let unique = values.iter().cloned().collect::<BTreeSet<_>>();
            if unique.len() != values.len() {
                return Err(StateValidationError::new(
                    StateValidationCode::DuplicateAlternatives,
                    "uncertain/conflicted resolution alternatives must be unique",
                ));
            }
            Ok(())
        }
    }
}

impl Default for StateEpoch {
    fn default() -> Self {
        Self(0)
    }
}

impl StateSnapshot {
    pub fn archive(&self) -> StateSnapshotArchive {
        StateSnapshotArchive::new(self.clone())
    }

    /// The single semantic admission rule for snapshots (#1782, #1783), shared
    /// by restore, archive load, canonical archive write and rollback. It
    /// reuses the update validators, so a snapshot can only hold records an
    /// update could have produced.
    pub fn validate(&self) -> Result<(), StateValidationError> {
        // The store has no frozen mode: a snapshot from which no update could
        // ever be accepted is not admissible (terminal epoch policy).
        if self.epoch.0 == u64::MAX {
            return Err(epoch_exhausted());
        }
        for (key, record) in &self.records {
            validate_key(key)?;
            if record.key != *key {
                return Err(StateValidationError::new(
                    StateValidationCode::KeyMismatch,
                    format!(
                        "snapshot record stored under '{}' carries key '{}'",
                        key, record.key
                    ),
                ));
            }
            validate_context(&record.context)?;
            validate_resolution(&record.resolution)?;
            if record.epoch > self.epoch {
                return Err(StateValidationError::new(
                    StateValidationCode::RecordEpochAfterSnapshot,
                    format!(
                        "snapshot record '{}' has epoch {} after snapshot epoch {}",
                        key, record.epoch.0, self.epoch.0
                    ),
                ));
            }
        }
        Ok(())
    }
}

impl StateSnapshotArchive {
    pub fn new(snapshot: StateSnapshot) -> Self {
        Self {
            format_version: STATE_SNAPSHOT_ARCHIVE_FORMAT_VERSION,
            snapshot,
        }
    }

    /// Serializes a valid archive (#1782). Invalid evidence is rejected rather
    /// than written as text the same owner's reader would refuse; valid
    /// archives serialize byte-for-byte as before.
    pub fn to_canonical_text(&self) -> Result<String, StateSnapshotArchiveFormatError> {
        if self.format_version != STATE_SNAPSHOT_ARCHIVE_FORMAT_VERSION {
            return Err(StateSnapshotArchiveFormatError::new(format!(
                "unsupported archive format version {}; expected {}",
                self.format_version, STATE_SNAPSHOT_ARCHIVE_FORMAT_VERSION
            )));
        }
        self.snapshot
            .validate()
            .map_err(|err| StateSnapshotArchiveFormatError::new(err.to_string()))?;
        let mut out = String::new();
        out.push_str(STATE_SNAPSHOT_ARCHIVE_MAGIC);
        out.push('\t');
        out.push_str(&self.format_version.to_string());
        out.push('\n');
        out.push_str("epoch\t");
        out.push_str(&self.snapshot.epoch.0.to_string());
        out.push('\n');
        out.push_str("records\t");
        out.push_str(&self.snapshot.records.len().to_string());
        out.push('\n');

        for (key, record) in &self.snapshot.records {
            out.push_str("record\t");
            out.push_str(&escape_archive_field(key));
            out.push('\t');
            out.push_str(&record.epoch.0.to_string());
            out.push('\t');
            out.push_str(&escape_archive_field(&record.context.name));
            encode_resolution(&mut out, &record.resolution);
            out.push('\n');
        }

        Ok(out)
    }

    pub fn from_canonical_text(src: &str) -> Result<Self, StateSnapshotArchiveFormatError> {
        let mut lines = src.lines();
        let header = lines
            .next()
            .ok_or_else(|| StateSnapshotArchiveFormatError::new("missing archive header"))?;
        let header_parts = split_archive_line(header);
        if header_parts.len() != 2 || header_parts[0] != STATE_SNAPSHOT_ARCHIVE_MAGIC {
            return Err(StateSnapshotArchiveFormatError::new(
                "invalid archive header",
            ));
        }
        let format_version = parse_u32_field(header_parts[1], "archive format version")?;
        if format_version != STATE_SNAPSHOT_ARCHIVE_FORMAT_VERSION {
            return Err(StateSnapshotArchiveFormatError::new(format!(
                "unsupported archive format version {}; expected {}",
                format_version, STATE_SNAPSHOT_ARCHIVE_FORMAT_VERSION
            )));
        }

        let epoch_line = lines
            .next()
            .ok_or_else(|| StateSnapshotArchiveFormatError::new("missing archive epoch line"))?;
        let epoch_parts = split_archive_line(epoch_line);
        if epoch_parts.len() != 2 || epoch_parts[0] != "epoch" {
            return Err(StateSnapshotArchiveFormatError::new(
                "invalid archive epoch line",
            ));
        }
        let epoch = StateEpoch(parse_u64_field(epoch_parts[1], "archive epoch")?);

        let record_count_line = lines.next().ok_or_else(|| {
            StateSnapshotArchiveFormatError::new("missing archive record-count line")
        })?;
        let record_count_parts = split_archive_line(record_count_line);
        if record_count_parts.len() != 2 || record_count_parts[0] != "records" {
            return Err(StateSnapshotArchiveFormatError::new(
                "invalid archive record-count line",
            ));
        }
        let expected_record_count =
            parse_usize_field(record_count_parts[1], "archive record count")?;

        let mut records = BTreeMap::new();
        let mut seen_records = 0usize;
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            let parts = split_archive_line(line);
            if parts.len() < 7 || parts[0] != "record" {
                return Err(StateSnapshotArchiveFormatError::new(
                    "invalid archive record line",
                ));
            }
            let key = unescape_archive_field(parts[1])?;
            let record_epoch = StateEpoch(parse_u64_field(parts[2], "record epoch")?);
            let context = ContextWindow::new(unescape_archive_field(parts[3])?);
            let resolution = decode_resolution(&parts[4..])?;
            let previous = records.insert(
                key.clone(),
                StateRecord {
                    key,
                    resolution,
                    context,
                    epoch: record_epoch,
                },
            );
            if previous.is_some() {
                return Err(StateSnapshotArchiveFormatError::new(
                    "duplicate record key in archive",
                ));
            }
            seen_records += 1;
        }

        if seen_records != expected_record_count {
            return Err(StateSnapshotArchiveFormatError::new(format!(
                "record count mismatch: header says {}, parsed {}",
                expected_record_count, seen_records
            )));
        }

        let snapshot = StateSnapshot { epoch, records };
        // #1782: structural decode alone is not admission.
        snapshot
            .validate()
            .map_err(|err| StateSnapshotArchiveFormatError::new(err.to_string()))?;
        Ok(Self {
            format_version,
            snapshot,
        })
    }
}

impl StateRollbackCheckpoint {
    pub fn new(
        checkpoint_ordinal: u32,
        snapshot: StateSnapshotArchive,
        applied_transition_count: usize,
    ) -> Self {
        Self {
            checkpoint_ordinal,
            snapshot,
            applied_transition_count,
        }
    }
}

impl StateRollbackArtifact {
    pub fn new(head_epoch: StateEpoch, checkpoints: Vec<StateRollbackCheckpoint>) -> Self {
        Self {
            format_version: STATE_ROLLBACK_ARTIFACT_FORMAT_VERSION,
            head_epoch,
            checkpoints,
        }
    }
}

fn encode_resolution(out: &mut String, resolution: &FactResolution) {
    match resolution {
        FactResolution::Certain(value) => {
            out.push_str("\tcertain\t1\t");
            encode_value(out, value);
        }
        FactResolution::Uncertain(values) => {
            out.push_str("\tuncertain\t");
            out.push_str(&values.len().to_string());
            for value in values {
                out.push('\t');
                encode_value(out, value);
            }
        }
        FactResolution::Conflicted(values) => {
            out.push_str("\tconflicted\t");
            out.push_str(&values.len().to_string());
            for value in values {
                out.push('\t');
                encode_value(out, value);
            }
        }
    }
}

fn encode_value(out: &mut String, value: &FactValue) {
    match value {
        FactValue::Bool(value) => {
            out.push_str("bool:");
            out.push_str(if *value { "true" } else { "false" });
        }
        FactValue::I32(value) => {
            out.push_str("i32:");
            out.push_str(&value.to_string());
        }
        FactValue::Text(value) => {
            out.push_str("text:");
            out.push_str(&escape_archive_field(value));
        }
    }
}

fn decode_resolution(parts: &[&str]) -> Result<FactResolution, StateSnapshotArchiveFormatError> {
    if parts.len() < 3 {
        return Err(StateSnapshotArchiveFormatError::new(
            "record resolution payload is incomplete",
        ));
    }
    let count = parse_usize_field(parts[1], "resolution value count")?;
    // Checked: a persisted count of usize::MAX must not overflow. Once this
    // holds, `count` is bounded by the structurally present payload.
    if parts.len().checked_sub(2) != Some(count) {
        return Err(StateSnapshotArchiveFormatError::new(
            "resolution value count does not match payload length",
        ));
    }
    let mut values = Vec::with_capacity(count);
    for raw in &parts[2..] {
        values.push(decode_value(raw)?);
    }
    match parts[0] {
        "certain" => {
            if values.len() != 1 {
                return Err(StateSnapshotArchiveFormatError::new(
                    "certain resolution requires exactly one value",
                ));
            }
            Ok(FactResolution::Certain(values.remove(0)))
        }
        "uncertain" => Ok(FactResolution::Uncertain(values)),
        "conflicted" => Ok(FactResolution::Conflicted(values)),
        _ => Err(StateSnapshotArchiveFormatError::new(
            "unknown resolution kind in archive",
        )),
    }
}

fn decode_value(raw: &str) -> Result<FactValue, StateSnapshotArchiveFormatError> {
    let (kind, payload) = raw
        .split_once(':')
        .ok_or_else(|| StateSnapshotArchiveFormatError::new("invalid encoded fact value"))?;
    match kind {
        "bool" => match payload {
            "true" => Ok(FactValue::Bool(true)),
            "false" => Ok(FactValue::Bool(false)),
            _ => Err(StateSnapshotArchiveFormatError::new(
                "invalid bool fact payload",
            )),
        },
        "i32" => payload
            .parse::<i32>()
            .map(FactValue::I32)
            .map_err(|_| StateSnapshotArchiveFormatError::new("invalid i32 fact payload")),
        "text" => Ok(FactValue::Text(unescape_archive_field(payload)?)),
        _ => Err(StateSnapshotArchiveFormatError::new(
            "unknown fact value kind in archive",
        )),
    }
}

fn split_archive_line(line: &str) -> Vec<&str> {
    line.split('\t').collect()
}

fn parse_u32_field(raw: &str, label: &str) -> Result<u32, StateSnapshotArchiveFormatError> {
    raw.parse::<u32>()
        .map_err(|_| StateSnapshotArchiveFormatError::new(format!("invalid {}", label)))
}

fn parse_u64_field(raw: &str, label: &str) -> Result<u64, StateSnapshotArchiveFormatError> {
    raw.parse::<u64>()
        .map_err(|_| StateSnapshotArchiveFormatError::new(format!("invalid {}", label)))
}

fn parse_usize_field(raw: &str, label: &str) -> Result<usize, StateSnapshotArchiveFormatError> {
    raw.parse::<usize>()
        .map_err(|_| StateSnapshotArchiveFormatError::new(format!("invalid {}", label)))
}

fn escape_archive_field(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '\t' => escaped.push_str("\\t"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            ':' => escaped.push_str("\\:"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn unescape_archive_field(value: &str) -> Result<String, StateSnapshotArchiveFormatError> {
    let mut out = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        let escaped = chars
            .next()
            .ok_or_else(|| StateSnapshotArchiveFormatError::new("unterminated archive escape"))?;
        match escaped {
            '\\' => out.push('\\'),
            't' => out.push('\t'),
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            ':' => out.push(':'),
            _ => {
                return Err(StateSnapshotArchiveFormatError::new(
                    "unsupported archive escape sequence",
                ))
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_store_applies_updates_and_advances_epoch() {
        let mut store = SemanticStateStore::new();
        let transition = store
            .apply(StateUpdate::new(
                "fact.alpha",
                FactResolution::Certain(FactValue::Bool(true)),
                ContextWindow::new("root"),
                "seed fact",
            ))
            .expect("apply");

        assert_eq!(transition.from_epoch, StateEpoch(0));
        assert_eq!(transition.to_epoch, StateEpoch(1));
        assert_eq!(store.epoch(), StateEpoch(1));
        assert_eq!(
            store.get("fact.alpha").expect("record").resolution,
            FactResolution::Certain(FactValue::Bool(true))
        );
    }

    #[test]
    fn state_store_rejects_duplicate_alternatives_in_conflict() {
        let mut store = SemanticStateStore::new();
        let err = store
            .apply(StateUpdate::new(
                "fact.alpha",
                FactResolution::Conflicted(vec![FactValue::I32(1), FactValue::I32(1)]),
                ContextWindow::new("root"),
                "conflict",
            ))
            .expect_err("must reject duplicate alternatives");
        assert_eq!(err.code, StateValidationCode::DuplicateAlternatives);
    }

    #[test]
    fn state_store_requires_context_and_reason() {
        let mut store = SemanticStateStore::new();
        let err = store
            .apply(StateUpdate::new(
                "fact.alpha",
                FactResolution::Certain(FactValue::Bool(true)),
                ContextWindow::new(""),
                "",
            ))
            .expect_err("must reject empty context");
        assert_eq!(err.code, StateValidationCode::EmptyContext);
    }

    #[test]
    fn state_store_snapshot_roundtrip_restores_epoch_and_records() {
        let mut store = SemanticStateStore::new();
        store
            .apply(StateUpdate::new(
                "fact.alpha",
                FactResolution::Certain(FactValue::Text("ready".to_string())),
                ContextWindow::new("window.alpha"),
                "set initial fact",
            ))
            .expect("apply");
        let snapshot = store.snapshot();

        store
            .apply(StateUpdate::new(
                "fact.beta",
                FactResolution::Certain(FactValue::Bool(false)),
                ContextWindow::new("window.beta"),
                "mutate after snapshot",
            ))
            .expect("apply");
        store.restore(snapshot).expect("valid snapshot");

        assert_eq!(store.epoch(), StateEpoch(1));
        assert!(store.get("fact.alpha").is_some());
        assert!(store.get("fact.beta").is_none());
    }

    #[test]
    fn state_snapshot_archive_uses_canonical_format_and_carries_snapshot() {
        let mut store = SemanticStateStore::new();
        store
            .apply(StateUpdate::new(
                "fact.alpha",
                FactResolution::Certain(FactValue::Bool(true)),
                ContextWindow::new("root"),
                "seed snapshot",
            ))
            .expect("apply");

        let archive = store.snapshot().archive();

        assert_eq!(
            archive.format_version,
            STATE_SNAPSHOT_ARCHIVE_FORMAT_VERSION
        );
        assert_eq!(archive.snapshot.epoch, StateEpoch(1));
        assert!(archive.snapshot.records.contains_key("fact.alpha"));
    }

    #[test]
    fn state_snapshot_archive_roundtrips_through_canonical_text() {
        let mut store = SemanticStateStore::new();
        store
            .apply(StateUpdate::new(
                "fact.beta",
                FactResolution::Conflicted(vec![
                    FactValue::I32(7),
                    FactValue::Text("a:b\tc".to_string()),
                ]),
                ContextWindow::new("window.beta"),
                "seed beta",
            ))
            .expect("apply");
        store
            .apply(StateUpdate::new(
                "fact.alpha",
                FactResolution::Certain(FactValue::Bool(true)),
                ContextWindow::new("root"),
                "seed alpha",
            ))
            .expect("apply");
        let archive = store.snapshot().archive();

        let text = archive.to_canonical_text().expect("valid archive");
        let parsed = StateSnapshotArchive::from_canonical_text(&text).expect("parse");

        assert_eq!(parsed, archive);
        let lines = text.lines().collect::<Vec<_>>();
        assert!(lines[3].contains("fact.alpha"));
        assert!(lines[4].contains("fact.beta"));
    }

    #[test]
    fn state_snapshot_archive_rejects_record_count_mismatch() {
        let text = "\
semantic_state_snapshot_archive\t1\n\
epoch\t1\n\
records\t2\n\
record\tfact.alpha\t1\troot\tcertain\t1\tbool:true\n";

        let err = StateSnapshotArchive::from_canonical_text(text).expect_err("must reject");

        assert!(err.message.contains("record count mismatch"));
    }

    #[test]
    fn state_snapshot_archive_rejects_unknown_format_version() {
        let text = "\
semantic_state_snapshot_archive\t2\n\
epoch\t0\n\
records\t0\n";

        let err = StateSnapshotArchive::from_canonical_text(text).expect_err("must reject");

        assert!(err.message.contains("unsupported archive format version"));
    }

    #[test]
    fn state_rollback_artifact_uses_explicit_format_version_and_head_epoch() {
        let mut store = SemanticStateStore::new();
        store
            .apply(StateUpdate::new(
                "fact.alpha",
                FactResolution::Certain(FactValue::Bool(true)),
                ContextWindow::new("root"),
                "seed alpha",
            ))
            .expect("apply");
        let checkpoint = StateRollbackCheckpoint::new(0, store.snapshot().archive(), 1);
        let artifact = StateRollbackArtifact::new(store.epoch(), vec![checkpoint.clone()]);

        assert_eq!(
            artifact.format_version,
            STATE_ROLLBACK_ARTIFACT_FORMAT_VERSION
        );
        assert_eq!(artifact.head_epoch, StateEpoch(1));
        assert_eq!(artifact.checkpoints, vec![checkpoint]);
    }

    #[test]
    fn state_rollback_artifact_preserves_declared_checkpoint_order() {
        let mut store = SemanticStateStore::new();
        store
            .apply(StateUpdate::new(
                "fact.alpha",
                FactResolution::Certain(FactValue::Bool(true)),
                ContextWindow::new("root"),
                "seed alpha",
            ))
            .expect("apply");
        let checkpoint0 = StateRollbackCheckpoint::new(0, store.snapshot().archive(), 1);

        store
            .apply(StateUpdate::new(
                "fact.beta",
                FactResolution::Certain(FactValue::Bool(false)),
                ContextWindow::new("window.beta"),
                "seed beta",
            ))
            .expect("apply");
        let checkpoint1 = StateRollbackCheckpoint::new(1, store.snapshot().archive(), 2);

        let artifact = StateRollbackArtifact::new(
            store.epoch(),
            vec![checkpoint0.clone(), checkpoint1.clone()],
        );

        assert_eq!(artifact.checkpoints[0].checkpoint_ordinal, 0);
        assert_eq!(
            artifact.checkpoints[0].snapshot.snapshot.epoch,
            StateEpoch(1)
        );
        assert_eq!(artifact.checkpoints[1].checkpoint_ordinal, 1);
        assert_eq!(
            artifact.checkpoints[1].snapshot.snapshot.epoch,
            StateEpoch(2)
        );
    }

    #[test]
    fn state_store_apply_rollback_restores_checkpoint_snapshot_and_truncates_history() {
        let mut store = SemanticStateStore::new();
        store
            .apply(StateUpdate::new(
                "fact.alpha",
                FactResolution::Certain(FactValue::Bool(true)),
                ContextWindow::new("root"),
                "seed alpha",
            ))
            .expect("apply");
        let checkpoint0 = StateRollbackCheckpoint::new(0, store.snapshot().archive(), 1);

        store
            .apply(StateUpdate::new(
                "fact.beta",
                FactResolution::Certain(FactValue::Bool(false)),
                ContextWindow::new("window.beta"),
                "seed beta",
            ))
            .expect("apply");
        let checkpoint1 = StateRollbackCheckpoint::new(1, store.snapshot().archive(), 2);
        let artifact =
            StateRollbackArtifact::new(store.epoch(), vec![checkpoint0.clone(), checkpoint1]);

        let advance = store.apply_rollback(&artifact, 0).expect("rollback");

        assert_eq!(advance.checkpoint_ordinal, 0);
        assert_eq!(advance.from_epoch, StateEpoch(2));
        assert_eq!(advance.to_epoch, StateEpoch(1));
        assert_eq!(advance.retained_transition_count, 1);
        assert_eq!(advance.dropped_transition_count, 1);
        assert_eq!(store.epoch(), StateEpoch(1));
        assert!(store.get("fact.alpha").is_some());
        assert!(store.get("fact.beta").is_none());
        assert_eq!(store.transitions().len(), 1);
        assert_eq!(checkpoint0.snapshot.snapshot, store.snapshot());
    }

    #[test]
    fn state_store_apply_rollback_rejects_head_epoch_mismatch() {
        let mut store = SemanticStateStore::new();
        store
            .apply(StateUpdate::new(
                "fact.alpha",
                FactResolution::Certain(FactValue::Bool(true)),
                ContextWindow::new("root"),
                "seed alpha",
            ))
            .expect("apply");
        let checkpoint = StateRollbackCheckpoint::new(0, store.snapshot().archive(), 1);
        let artifact = StateRollbackArtifact::new(StateEpoch(99), vec![checkpoint]);

        let err = store.apply_rollback(&artifact, 0).expect_err("must reject");

        assert_eq!(err.code, StateRollbackCode::HeadEpochMismatch);
    }

    #[test]
    fn state_store_apply_rollback_rejects_checkpoint_epoch_transition_mismatch() {
        let mut store = SemanticStateStore::new();
        store
            .apply(StateUpdate::new(
                "fact.alpha",
                FactResolution::Certain(FactValue::Bool(true)),
                ContextWindow::new("root"),
                "seed alpha",
            ))
            .expect("apply");
        let mut archive = store.snapshot().archive();
        archive.snapshot.epoch = StateEpoch(7);
        let checkpoint = StateRollbackCheckpoint::new(0, archive, 1);
        let artifact = StateRollbackArtifact::new(store.epoch(), vec![checkpoint]);

        let err = store.apply_rollback(&artifact, 0).expect_err("must reject");

        assert_eq!(err.code, StateRollbackCode::SnapshotEpochMismatch);
    }

    #[test]
    fn state_store_apply_rollback_rejects_store_history_mismatch_after_manual_restore() {
        let mut store = SemanticStateStore::new();
        store
            .apply(StateUpdate::new(
                "fact.alpha",
                FactResolution::Certain(FactValue::Bool(true)),
                ContextWindow::new("root"),
                "seed alpha",
            ))
            .expect("apply");
        let checkpoint0 = StateRollbackCheckpoint::new(0, store.snapshot().archive(), 1);

        store
            .apply(StateUpdate::new(
                "fact.beta",
                FactResolution::Certain(FactValue::Bool(false)),
                ContextWindow::new("window.beta"),
                "seed beta",
            ))
            .expect("apply");
        let artifact = StateRollbackArtifact::new(store.epoch(), vec![checkpoint0]);

        store
            .restore(StateSnapshot {
                epoch: StateEpoch(1),
                records: store
                    .snapshot()
                    .records
                    .into_iter()
                    .filter(|(key, _)| key == "fact.alpha")
                    .collect(),
            })
            .expect("valid snapshot");

        let err = store.apply_rollback(&artifact, 0).expect_err("must reject");

        assert_eq!(err.code, StateRollbackCode::StoreHistoryMismatch);
    }

    fn pb07_record(key: &str, resolution: FactResolution, epoch: u64) -> StateRecord {
        StateRecord {
            key: key.to_string(),
            resolution,
            context: ContextWindow::new("root"),
            epoch: StateEpoch(epoch),
        }
    }

    fn pb07_snapshot(epoch: u64, records: Vec<(&str, StateRecord)>) -> StateSnapshot {
        StateSnapshot {
            epoch: StateEpoch(epoch),
            records: records
                .into_iter()
                .map(|(key, record)| (key.to_string(), record))
                .collect(),
        }
    }

    fn pb07_seeded_store() -> SemanticStateStore {
        let mut store = SemanticStateStore::new();
        store
            .apply(StateUpdate::new(
                "fact.alpha",
                FactResolution::Certain(FactValue::Bool(true)),
                ContextWindow::new("root"),
                "seed",
            ))
            .expect("seed");
        store
    }

    fn pb07_invalid_snapshots() -> Vec<(StateValidationCode, StateSnapshot)> {
        let certain = || FactResolution::Certain(FactValue::Bool(true));
        vec![
            (
                StateValidationCode::KeyMismatch,
                pb07_snapshot(1, vec![("fact.a", pb07_record("fact.b", certain(), 1))]),
            ),
            (
                StateValidationCode::EmptyKey,
                pb07_snapshot(1, vec![(" ", pb07_record(" ", certain(), 1))]),
            ),
            (
                StateValidationCode::NotEnoughAlternatives,
                pb07_snapshot(
                    1,
                    vec![(
                        "fact.a",
                        pb07_record(
                            "fact.a",
                            FactResolution::Uncertain(vec![FactValue::I32(1)]),
                            1,
                        ),
                    )],
                ),
            ),
            (
                StateValidationCode::DuplicateAlternatives,
                pb07_snapshot(
                    1,
                    vec![(
                        "fact.a",
                        pb07_record(
                            "fact.a",
                            FactResolution::Conflicted(vec![FactValue::I32(1), FactValue::I32(1)]),
                            1,
                        ),
                    )],
                ),
            ),
            (
                StateValidationCode::RecordEpochAfterSnapshot,
                pb07_snapshot(1, vec![("fact.a", pb07_record("fact.a", certain(), 2))]),
            ),
            (
                StateValidationCode::EmptyContext,
                pb07_snapshot(
                    1,
                    vec![(
                        "fact.a",
                        StateRecord {
                            context: ContextWindow::new(""),
                            ..pb07_record("fact.a", certain(), 1)
                        },
                    )],
                ),
            ),
            (
                StateValidationCode::EpochExhausted,
                pb07_snapshot(u64::MAX, Vec::new()),
            ),
        ]
    }

    // #1782/#1783: restore validates fully before mutation.
    #[test]
    fn pb07_restore_rejects_invalid_snapshots_and_leaves_store_unchanged() {
        for (code, snapshot) in pb07_invalid_snapshots() {
            let mut store = pb07_seeded_store();
            let before = store.clone();
            let err = store.restore(snapshot).expect_err("invalid snapshot");
            assert_eq!(err.code, code);
            assert_eq!(store, before, "{code:?}: store must be unchanged");
        }
    }

    // #1782: archive reader and canonical writer share the snapshot rule.
    #[test]
    fn pb07_archive_writer_and_reader_reject_invalid_snapshots() {
        for (code, snapshot) in pb07_invalid_snapshots() {
            let archive = StateSnapshotArchive::new(snapshot);
            assert!(archive.to_canonical_text().is_err(), "{code:?}: writer");
        }
        let valid = pb07_seeded_store().snapshot().archive();
        let text = valid.to_canonical_text().expect("valid archive");
        assert_eq!(
            StateSnapshotArchive::from_canonical_text(&text).expect("valid"),
            valid
        );
        for (needle, replacement) in [
            ("\tcertain\t1\tbool:true", "\tuncertain\t1\tbool:true"),
            ("epoch\t1\n", "epoch\t18446744073709551615\n"),
            ("record\tfact.alpha\t1\t", "record\tfact.alpha\t9\t"),
        ] {
            assert!(text.contains(needle), "fixture drift: {needle:?}");
            let bad = text.replace(needle, replacement);
            assert!(
                StateSnapshotArchive::from_canonical_text(&bad).is_err(),
                "reader must reject {replacement:?}"
            );
        }
    }

    // #1783: defense in depth even if a future ingress bypassed restore.
    #[test]
    fn pb07_apply_at_max_epoch_fails_closed_without_mutation() {
        let mut store = pb07_seeded_store();
        store.epoch = StateEpoch(u64::MAX);
        let before = store.clone();
        let err = store
            .apply(StateUpdate::new(
                "fact.beta",
                FactResolution::Certain(FactValue::I32(1)),
                ContextWindow::new("root"),
                "would wrap",
            ))
            .expect_err("epoch exhausted");
        assert_eq!(err.code, StateValidationCode::EpochExhausted);
        assert_eq!(store, before, "no mutation, no wrap");
    }

    // #1782: rollback is not a bypass around snapshot admission.
    #[test]
    fn pb07_rollback_rejects_invalid_checkpoint_snapshot() {
        let mut store = pb07_seeded_store();
        let bad = pb07_snapshot(
            1,
            vec![(
                "fact.a",
                pb07_record("fact.b", FactResolution::Certain(FactValue::I32(1)), 1),
            )],
        );
        let artifact = StateRollbackArtifact::new(
            store.epoch(),
            vec![StateRollbackCheckpoint::new(
                0,
                StateSnapshotArchive::new(bad),
                1,
            )],
        );
        let before = store.clone();
        let err = store
            .apply_rollback(&artifact, 0)
            .expect_err("invalid checkpoint");
        assert_eq!(err.code, StateRollbackCode::InvalidCheckpointSnapshot);
        assert_eq!(store, before);
    }

    // Same-root (#1782): a hostile resolution count is a typed error.
    #[test]
    fn pb07_resolution_count_overflow_is_a_typed_error() {
        let max = usize::MAX.to_string();
        let parts = ["uncertain", max.as_str(), "i32:1", "i32:2"];
        assert!(decode_resolution(&parts).is_err());
    }
}
