//! Recoverable write-ahead transaction protocol for durable `.tvim` index
//! mutations.
//!
//! `save_atomic`'s tmp-write-then-rename was already atomic *for the final
//! rename step itself*, but left no durable trace of *intent* -- if the
//! process crashed between writing the candidate file and renaming it into
//! place, the only evidence was an orphaned `.tvim.tmp.<pid>.<nanos>` file
//! nothing ever inspected. This module adds one small, bounded sibling
//! record per index (`<name>.tvim.txn`, always overwritten in place, never
//! appended) so a future invocation -- or an explicit `vector.index.recover`
//! call -- can tell, without guessing, whether the last mutation attempt
//! against that index committed, was safely abandoned, or is genuinely
//! indeterminate and must not be silently treated as success.
//!
//! This is not a general transaction engine: it tracks exactly one
//! in-flight candidate per index, matching the fact that Hub v0 admits at
//! most one state-mutating request at a time per stateful tool instance.

use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::storage::{IndexName, ScopedStorage};

const MAX_TRANSACTION_RECORD_BYTES: u64 = 16 * 1024;

/// Where a transaction record's own subject currently stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionPhase {
    /// A candidate artifact write was about to begin (or is in progress).
    /// If this is still the on-disk phase on a later read, the process
    /// that wrote it never reached `Committed` -- recovery must determine
    /// what actually happened, never assume either outcome.
    Intent,
    /// The complete candidate was synced and its exact digest was synced
    /// into this record before any rename was attempted.
    Prepared,
    /// The candidate was validated, atomically committed to the final
    /// path, and `artifact_digest` is the digest of exactly those
    /// committed bytes (verified by reading the final path back, not
    /// assumed from the in-memory buffer that was written).
    Committed,
    /// Recovery inspected this transaction and could not prove either
    /// outcome (e.g. neither the candidate nor the final artifact exists).
    /// A caller must not proceed as if this index were usable.
    Indeterminate,
}

impl fmt::Display for TransactionPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TransactionPhase::Intent => "Intent",
            TransactionPhase::Prepared => "Prepared",
            TransactionPhase::Committed => "Committed",
            TransactionPhase::Indeterminate => "Indeterminate",
        };
        f.write_str(s)
    }
}

/// The durable record for one index's most recent mutation attempt.
/// Overwritten in place on every new mutation -- bounded size, no
/// unbounded transaction history, matching v0's "at most one in-flight
/// mutation per stateful tool instance" concurrency model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub transaction_id: String,
    pub operation: String,
    pub phase: TransactionPhase,
    /// File name only (not a full path) of the candidate file, resolved
    /// against the same scoped storage root as the index itself -- never
    /// a caller-influenced path.
    pub candidate_file_name: String,
    /// Digest of the synced candidate bytes. Present from `Prepared`
    /// onward and used as the only proof that a missing candidate was
    /// renamed to the final path.
    #[serde(default)]
    pub candidate_digest: Option<String>,
    /// Present only once `phase == Committed`: the digest of the exact
    /// bytes read back from the final, committed artifact.
    pub artifact_digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionError {
    Io(String),
    Malformed(String),
    ScopedStorage(String),
    Unresolved(TransactionPhase),
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionError::Io(m) => write!(f, "transaction record I/O error: {m}"),
            TransactionError::Malformed(m) => write!(f, "malformed transaction record: {m}"),
            TransactionError::ScopedStorage(m) => write!(f, "{m}"),
            TransactionError::Unresolved(phase) => {
                write!(f, "transaction is unresolved in phase {phase}")
            }
        }
    }
}

/// A fresh, process/time-derived transaction id -- not a security token,
/// only a correlation string, same convention as `save_atomic`'s existing
/// tmp-file suffix.
pub fn new_transaction_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("txn-{}-{}", std::process::id(), nanos)
}

pub fn txn_path(storage: &ScopedStorage, name: &IndexName) -> PathBuf {
    storage.root().join(format!("{}.tvim.txn", name.as_str()))
}

fn checked_txn_path(
    storage: &ScopedStorage,
    name: &IndexName,
) -> Result<PathBuf, TransactionError> {
    storage
        .checked_private_file_path(&format!("{}.tvim.txn", name.as_str()))
        .map_err(|e| TransactionError::ScopedStorage(e.to_string()))
}

fn expected_candidate_file_name(name: &IndexName, transaction_id: &str) -> String {
    format!("{}.tvim.tmp.{transaction_id}", name.as_str())
}

pub fn checked_candidate_path(
    storage: &ScopedStorage,
    name: &IndexName,
    record: &TransactionRecord,
) -> Result<PathBuf, TransactionError> {
    validate_record(name, record)?;
    storage
        .checked_private_file_path(&record.candidate_file_name)
        .map_err(|e| TransactionError::ScopedStorage(e.to_string()))
}

fn sync_existing_file(path: &std::path::Path) -> Result<(), TransactionError> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .and_then(|file| file.sync_all())
        .map_err(|e| TransactionError::Io(e.to_string()))
}

/// Durably write a transaction record, replacing any prior one for this
/// index. Uses the same write-temp-then-rename pattern as the index
/// artifact itself, so the transaction record's own write can never be
/// observed half-written.
fn write_record(
    storage: &ScopedStorage,
    name: &IndexName,
    record: &TransactionRecord,
) -> Result<(), TransactionError> {
    storage
        .ensure_root_checked()
        .map_err(|e| TransactionError::ScopedStorage(e.to_string()))?;
    validate_record(name, record)?;
    let path = checked_txn_path(storage, name)?;
    let text = serde_json::to_string_pretty(record)
        .map_err(|e| TransactionError::Malformed(e.to_string()))?;
    if text.len() as u64 > MAX_TRANSACTION_RECORD_BYTES {
        return Err(TransactionError::Malformed(
            "transaction record exceeds the size bound".to_string(),
        ));
    }
    let tmp_file_name = format!(
        "{}.tvim.txn.tmp.{}.{}",
        name.as_str(),
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let tmp = storage
        .checked_private_file_path(&tmp_file_name)
        .map_err(|e| TransactionError::ScopedStorage(e.to_string()))?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp)
        .map_err(|e| TransactionError::Io(e.to_string()))?;
    let write_result = (|| {
        use std::io::Write;
        file.write_all(text.as_bytes())?;
        file.sync_all()
    })();
    drop(file);
    if let Err(error) = write_result {
        let _ = std::fs::remove_file(&tmp);
        return Err(TransactionError::Io(error.to_string()));
    }
    if let Err(error) = std::fs::rename(&tmp, &path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(TransactionError::Io(error.to_string()));
    }
    sync_existing_file(&checked_txn_path(storage, name)?)
}

fn is_valid_transaction_id(id: &str) -> bool {
    if id.len() > 64 {
        return false;
    }
    let Some(rest) = id.strip_prefix("txn-") else {
        return false;
    };
    let mut parts = rest.split('-');
    let pid = parts.next().unwrap_or_default();
    let nanos = parts.next().unwrap_or_default();
    !pid.is_empty()
        && pid.bytes().all(|b| b.is_ascii_digit())
        && !nanos.is_empty()
        && nanos.bytes().all(|b| b.is_ascii_digit())
        && parts.next().is_none()
}

fn is_valid_operation(operation: &str) -> bool {
    !operation.is_empty()
        && operation.len() <= 64
        && operation.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn validate_record(name: &IndexName, record: &TransactionRecord) -> Result<(), TransactionError> {
    if !is_valid_transaction_id(&record.transaction_id) {
        return Err(TransactionError::Malformed(
            "transaction_id is not a generated transaction id".to_string(),
        ));
    }
    if !is_valid_operation(&record.operation) {
        return Err(TransactionError::Malformed(
            "operation is not a bounded operation identifier".to_string(),
        ));
    }
    let expected = expected_candidate_file_name(name, &record.transaction_id);
    if record.candidate_file_name != expected {
        return Err(TransactionError::Malformed(
            "candidate file is not bound to this index and transaction".to_string(),
        ));
    }
    let valid_phase_fields = match record.phase {
        TransactionPhase::Intent => {
            record.candidate_digest.is_none() && record.artifact_digest.is_none()
        }
        TransactionPhase::Prepared => {
            record.candidate_digest.is_some() && record.artifact_digest.is_none()
        }
        TransactionPhase::Committed => {
            record.candidate_digest.is_some()
                && record.artifact_digest.is_some()
                && record.candidate_digest == record.artifact_digest
        }
        TransactionPhase::Indeterminate => true,
    };
    if !valid_phase_fields {
        return Err(TransactionError::Malformed(
            "transaction phase fields are inconsistent".to_string(),
        ));
    }
    Ok(())
}

/// Read the current transaction record for an index, if any. `Ok(None)`
/// means no mutation has ever been attempted against this index (or the
/// txn file predates this feature) -- not itself a problem. Returns
/// `Err(TransactionError::Malformed)` when the transaction id, candidate
/// binding, or phase fields are inconsistent -- deliberately loud rather
/// than trusting a private path or recovery claim read from disk.
pub fn read_record(
    storage: &ScopedStorage,
    name: &IndexName,
) -> Result<Option<TransactionRecord>, TransactionError> {
    let path = checked_txn_path(storage, name)?;
    if !path.is_file() {
        return Ok(None);
    }
    let metadata = std::fs::metadata(&path).map_err(|e| TransactionError::Io(e.to_string()))?;
    if metadata.len() > MAX_TRANSACTION_RECORD_BYTES {
        return Err(TransactionError::Malformed(
            "transaction record exceeds the size bound".to_string(),
        ));
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    let mut file = std::fs::File::open(&path).map_err(|e| TransactionError::Io(e.to_string()))?;
    {
        use std::io::Read;
        file.by_ref()
            .take(MAX_TRANSACTION_RECORD_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| TransactionError::Io(e.to_string()))?;
    }
    if bytes.len() as u64 > MAX_TRANSACTION_RECORD_BYTES {
        return Err(TransactionError::Malformed(
            "transaction record exceeds the size bound".to_string(),
        ));
    }
    let record: TransactionRecord =
        serde_json::from_slice(&bytes).map_err(|e| TransactionError::Malformed(e.to_string()))?;
    validate_record(name, &record)?;
    Ok(Some(record))
}

/// Begin a new transaction: durably record intent BEFORE the candidate
/// artifact write begins. Returns the record (including the candidate's
/// resolved file name) so the caller writes to exactly that path.
pub fn begin(
    storage: &ScopedStorage,
    name: &IndexName,
    operation: &str,
) -> Result<TransactionRecord, TransactionError> {
    ensure_resolved(storage, name)?;
    let transaction_id = new_transaction_id();
    let candidate_file_name = expected_candidate_file_name(name, &transaction_id);
    let record = TransactionRecord {
        transaction_id,
        operation: operation.to_string(),
        phase: TransactionPhase::Intent,
        candidate_file_name,
        candidate_digest: None,
        artifact_digest: None,
    };
    write_record(storage, name, &record)?;
    Ok(record)
}

/// Persist proof that the complete candidate was synced before rename.
pub fn prepare(
    storage: &ScopedStorage,
    name: &IndexName,
    mut record: TransactionRecord,
    candidate_digest: semantic_hub::HubDigest,
) -> Result<TransactionRecord, TransactionError> {
    validate_record(name, &record)?;
    if record.phase != TransactionPhase::Intent {
        return Err(TransactionError::Malformed(
            "only an Intent transaction can be prepared".to_string(),
        ));
    }
    let candidate_path = checked_candidate_path(storage, name, &record)?;
    sync_existing_file(&candidate_path)?;
    let actual_digest = std::fs::read(candidate_path)
        .map(|bytes| semantic_hub::HubDigest::of(&bytes))
        .map_err(|e| TransactionError::Io(e.to_string()))?;
    if actual_digest != candidate_digest {
        return Err(TransactionError::Malformed(
            "candidate digest does not match the synced candidate file".to_string(),
        ));
    }
    record.phase = TransactionPhase::Prepared;
    record.candidate_digest = Some(actual_digest.to_string());
    write_record(storage, name, &record)?;
    Ok(record)
}

/// Finalize a transaction after the candidate has been atomically
/// committed to the final path: durably record the committed artifact's
/// digest (computed from the final path's own bytes, read back fresh --
/// never assumed from the in-memory buffer that was written) as the
/// completion record. Success is only reported to the caller once this
/// write itself is durable.
pub fn commit(
    storage: &ScopedStorage,
    name: &IndexName,
    mut record: TransactionRecord,
    artifact_digest: semantic_hub::HubDigest,
) -> Result<(), TransactionError> {
    validate_record(name, &record)?;
    let artifact_digest = artifact_digest.to_string();
    if record.phase != TransactionPhase::Prepared
        || record.candidate_digest.as_deref() != Some(artifact_digest.as_str())
    {
        return Err(TransactionError::Malformed(
            "committed artifact does not match the prepared candidate".to_string(),
        ));
    }
    let final_path = storage
        .checked_index_path(name)
        .map_err(|e| TransactionError::ScopedStorage(e.to_string()))?;
    sync_existing_file(&final_path)?;
    let actual_digest = std::fs::read(final_path)
        .map(|bytes| semantic_hub::HubDigest::of(&bytes).to_string())
        .map_err(|e| TransactionError::Io(e.to_string()))?;
    if actual_digest != artifact_digest {
        return Err(TransactionError::Malformed(
            "final artifact does not match the prepared candidate".to_string(),
        ));
    }
    record.phase = TransactionPhase::Committed;
    record.artifact_digest = Some(actual_digest);
    write_record(storage, name, &record)
}

fn final_digest(
    storage: &ScopedStorage,
    name: &IndexName,
) -> Result<Option<String>, TransactionError> {
    let path = storage
        .checked_index_path(name)
        .map_err(|e| TransactionError::ScopedStorage(e.to_string()))?;
    if !path.is_file() {
        return Ok(None);
    }
    std::fs::read(path)
        .map(|bytes| Some(semantic_hub::HubDigest::of(&bytes).to_string()))
        .map_err(|e| TransactionError::Io(e.to_string()))
}

fn verify_committed(
    storage: &ScopedStorage,
    name: &IndexName,
    record: &TransactionRecord,
) -> Result<(), TransactionError> {
    validate_record(name, record)?;
    if record.phase != TransactionPhase::Committed {
        return Err(TransactionError::Unresolved(record.phase));
    }
    let candidate_path = checked_candidate_path(storage, name, record)?;
    if candidate_path.exists() {
        return Err(TransactionError::Malformed(
            "committed transaction still has a candidate file".to_string(),
        ));
    }
    let expected = record.artifact_digest.as_deref().ok_or_else(|| {
        TransactionError::Malformed("committed transaction has no artifact digest".to_string())
    })?;
    match final_digest(storage, name)? {
        Some(actual) if actual == expected => Ok(()),
        Some(_) => Err(TransactionError::Malformed(
            "committed artifact digest does not match the transaction record".to_string(),
        )),
        None => Err(TransactionError::Malformed(
            "committed transaction has no final artifact".to_string(),
        )),
    }
}

/// Shared fail-closed gate for every adapter operation except explicit
/// recovery. Malformed/unreadable records and every unresolved phase
/// prevent both reads and writes.
pub fn ensure_resolved(storage: &ScopedStorage, name: &IndexName) -> Result<(), TransactionError> {
    let Some(record) = read_record(storage, name)? else {
        return Ok(());
    };
    match record.phase {
        TransactionPhase::Committed => verify_committed(storage, name, &record),
        TransactionPhase::Intent | TransactionPhase::Prepared | TransactionPhase::Indeterminate => {
            Err(TransactionError::Unresolved(record.phase))
        }
    }
}

fn remove_record(storage: &ScopedStorage, name: &IndexName) -> Result<(), TransactionError> {
    let path = checked_txn_path(storage, name)?;
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(TransactionError::Io(error.to_string())),
    }
}

fn mark_indeterminate(
    storage: &ScopedStorage,
    name: &IndexName,
    mut record: TransactionRecord,
    reason: &'static str,
) -> Result<RecoveryOutcome, TransactionError> {
    record.phase = TransactionPhase::Indeterminate;
    write_record(storage, name, &record)?;
    Ok(RecoveryOutcome::Indeterminate {
        reason: reason.to_string(),
    })
}

/// Outcome of inspecting (and, where safe, resolving) one index's
/// transaction state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryOutcome {
    /// No transaction record exists for this index at all.
    NoTransaction,
    /// The last recorded transaction already reached `Committed`; nothing
    /// to do.
    AlreadyCommitted { artifact_digest: Option<String> },
    /// The last transaction never passed the durable `Prepared` proof, or
    /// its prepared candidate was still present (so rename never happened).
    /// Any candidate was removed and the intent record was cleared.
    RolledBackAbandonedCandidate { artifact_digest: Option<String> },
    /// A `Prepared` candidate was gone and the final artifact's digest
    /// exactly matched its synced candidate digest -- proof the rename
    /// completed before the completion record was written.
    FinalizedCommit { artifact_digest: String },
    /// Recovery could not prove either outcome. The transaction record is
    /// marked `Indeterminate` and all ordinary operations remain blocked
    /// until a human resolves it.
    Indeterminate { reason: String },
}

/// Inspect (and resolve where provably safe) the transaction state for one
/// index. Idempotent: calling this repeatedly against an already-resolved
/// transaction is a no-op that reports `AlreadyCommitted`.
pub fn recover(
    storage: &ScopedStorage,
    name: &IndexName,
) -> Result<RecoveryOutcome, TransactionError> {
    let Some(record) = read_record(storage, name)? else {
        return Ok(RecoveryOutcome::NoTransaction);
    };
    let candidate_path = checked_candidate_path(storage, name, &record)?;
    let final_path = storage
        .checked_index_path(name)
        .map_err(|e| TransactionError::ScopedStorage(e.to_string()))?;
    let candidate_exists = candidate_path.is_file();
    let final_exists = final_path.is_file();

    match record.phase {
        TransactionPhase::Committed => match verify_committed(storage, name, &record) {
            Ok(()) => Ok(RecoveryOutcome::AlreadyCommitted {
                artifact_digest: record.artifact_digest,
            }),
            Err(TransactionError::ScopedStorage(error)) => {
                Err(TransactionError::ScopedStorage(error))
            }
            Err(_) => mark_indeterminate(
                storage,
                name,
                record,
                "committed transaction could not be verified against the final artifact",
            ),
        },
        TransactionPhase::Indeterminate => Ok(RecoveryOutcome::Indeterminate {
            reason: "transaction remains indeterminate and requires manual resolution".to_string(),
        }),
        TransactionPhase::Intent => {
            if candidate_exists {
                std::fs::remove_file(&candidate_path)
                    .map_err(|e| TransactionError::Io(e.to_string()))?;
            }
            let digest = if final_exists {
                final_digest(storage, name)?
            } else {
                None
            };
            remove_record(storage, name)?;
            Ok(RecoveryOutcome::RolledBackAbandonedCandidate {
                artifact_digest: digest,
            })
        }
        TransactionPhase::Prepared => {
            let expected = record.candidate_digest.as_deref().ok_or_else(|| {
                TransactionError::Malformed(
                    "prepared transaction has no candidate digest".to_string(),
                )
            })?;
            if candidate_exists {
                let actual = std::fs::read(&candidate_path)
                    .map(|bytes| semantic_hub::HubDigest::of(&bytes).to_string())
                    .map_err(|e| TransactionError::Io(e.to_string()))?;
                if actual != expected {
                    return mark_indeterminate(
                        storage,
                        name,
                        record,
                        "prepared candidate digest does not match the transaction record",
                    );
                }
                std::fs::remove_file(&candidate_path)
                    .map_err(|e| TransactionError::Io(e.to_string()))?;
                let digest = if final_exists {
                    final_digest(storage, name)?
                } else {
                    None
                };
                remove_record(storage, name)?;
                return Ok(RecoveryOutcome::RolledBackAbandonedCandidate {
                    artifact_digest: digest,
                });
            }

            let Some(actual) = final_digest(storage, name)? else {
                return mark_indeterminate(
                    storage,
                    name,
                    record,
                    "prepared transaction has neither its candidate nor a final artifact",
                );
            };
            if actual != expected {
                return mark_indeterminate(
                    storage,
                    name,
                    record,
                    "final artifact does not match the prepared candidate digest",
                );
            }
            sync_existing_file(&final_path)?;
            let mut resolved = record;
            resolved.phase = TransactionPhase::Committed;
            resolved.artifact_digest = Some(actual.clone());
            write_record(storage, name, &resolved)?;
            Ok(RecoveryOutcome::FinalizedCommit {
                artifact_digest: actual,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_storage(tag: &str) -> ScopedStorage {
        let root = std::env::temp_dir().join(format!(
            "semantic-hub-turbovec-txn-test-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let storage = ScopedStorage::new(root);
        storage.ensure_root_checked().unwrap();
        storage
    }

    #[test]
    fn no_transaction_record_means_no_transaction() {
        let storage = temp_storage("none");
        let name = IndexName::new("docs").unwrap();
        assert_eq!(
            recover(&storage, &name).unwrap(),
            RecoveryOutcome::NoTransaction
        );
    }

    #[test]
    fn begin_then_commit_round_trips_and_recovery_reports_already_committed() {
        let storage = temp_storage("commit");
        let name = IndexName::new("docs").unwrap();
        let record = begin(&storage, &name, "create").unwrap();
        assert_eq!(record.phase, TransactionPhase::Intent);

        let candidate = checked_candidate_path(&storage, &name, &record).unwrap();
        std::fs::write(&candidate, b"final bytes").unwrap();
        let digest = semantic_hub::HubDigest::of(b"final bytes");
        let record = prepare(&storage, &name, record, digest).unwrap();
        std::fs::rename(candidate, storage.index_path(&name)).unwrap();
        commit(&storage, &name, record, digest).unwrap();

        ensure_resolved(&storage, &name).unwrap();
        match recover(&storage, &name).unwrap() {
            RecoveryOutcome::AlreadyCommitted { artifact_digest } => {
                assert_eq!(artifact_digest, Some(digest.to_string()));
            }
            other => panic!("expected AlreadyCommitted, got {other:?}"),
        }
    }

    #[test]
    fn recovery_rolls_back_an_abandoned_candidate_when_the_rename_never_happened() {
        let storage = temp_storage("rollback");
        let name = IndexName::new("docs").unwrap();
        // Pre-existing final artifact (as if a prior, unrelated,
        // successfully committed mutation already happened).
        std::fs::write(storage.index_path(&name), b"pre-existing committed bytes").unwrap();

        let record = begin(&storage, &name, "insert").unwrap();
        let candidate_path = checked_candidate_path(&storage, &name, &record).unwrap();
        std::fs::write(&candidate_path, b"never-committed candidate bytes").unwrap();
        let record = prepare(
            &storage,
            &name,
            record,
            semantic_hub::HubDigest::of(b"never-committed candidate bytes"),
        )
        .unwrap();
        assert_eq!(record.phase, TransactionPhase::Prepared);

        match recover(&storage, &name).unwrap() {
            RecoveryOutcome::RolledBackAbandonedCandidate { artifact_digest } => {
                assert_eq!(
                    artifact_digest,
                    Some(semantic_hub::HubDigest::of(b"pre-existing committed bytes").to_string())
                );
            }
            other => panic!("expected RolledBackAbandonedCandidate, got {other:?}"),
        }
        assert!(
            !candidate_path.is_file(),
            "abandoned candidate must be removed by recovery"
        );
        assert_eq!(
            recover(&storage, &name).unwrap(),
            RecoveryOutcome::NoTransaction
        );
    }

    #[test]
    fn recovery_finalizes_a_commit_whose_completion_record_never_got_written() {
        let storage = temp_storage("finalize");
        let name = IndexName::new("docs").unwrap();
        let record = begin(&storage, &name, "create").unwrap();
        let candidate_path = checked_candidate_path(&storage, &name, &record).unwrap();
        std::fs::write(&candidate_path, b"committed bytes").unwrap();
        prepare(
            &storage,
            &name,
            record,
            semantic_hub::HubDigest::of(b"committed bytes"),
        )
        .unwrap();
        // Simulate the rename half of save_atomic succeeding, but the
        // process crashing before `commit()`'s own record write ran.
        std::fs::rename(&candidate_path, storage.index_path(&name)).unwrap();

        match recover(&storage, &name).unwrap() {
            RecoveryOutcome::FinalizedCommit { artifact_digest } => {
                assert_eq!(
                    artifact_digest,
                    semantic_hub::HubDigest::of(b"committed bytes").to_string()
                );
            }
            other => panic!("expected FinalizedCommit, got {other:?}"),
        }
        assert_eq!(
            read_record(&storage, &name).unwrap().unwrap().phase,
            TransactionPhase::Committed
        );
    }

    #[test]
    fn intent_without_a_candidate_rolls_back_instead_of_claiming_an_old_final_committed() {
        let storage = temp_storage("intent-old-final");
        let name = IndexName::new("docs").unwrap();
        std::fs::write(storage.index_path(&name), b"old final").unwrap();
        begin(&storage, &name, "create").unwrap();

        match recover(&storage, &name).unwrap() {
            RecoveryOutcome::RolledBackAbandonedCandidate { artifact_digest } => assert_eq!(
                artifact_digest,
                Some(semantic_hub::HubDigest::of(b"old final").to_string())
            ),
            other => panic!("expected rollback, got {other:?}"),
        }
        assert!(read_record(&storage, &name).unwrap().is_none());
        assert_eq!(
            std::fs::read(storage.index_path(&name)).unwrap(),
            b"old final"
        );
    }

    #[test]
    fn prepared_missing_candidate_does_not_false_commit_an_old_final() {
        let storage = temp_storage("prepared-old-final");
        let name = IndexName::new("docs").unwrap();
        std::fs::write(storage.index_path(&name), b"old final").unwrap();
        let record = begin(&storage, &name, "insert").unwrap();
        let candidate = checked_candidate_path(&storage, &name, &record).unwrap();
        std::fs::write(&candidate, b"new candidate").unwrap();
        prepare(
            &storage,
            &name,
            record,
            semantic_hub::HubDigest::of(b"new candidate"),
        )
        .unwrap();
        std::fs::remove_file(candidate).unwrap();

        let outcome = recover(&storage, &name).unwrap();
        let RecoveryOutcome::Indeterminate { reason } = outcome else {
            panic!("expected Indeterminate, got {outcome:?}");
        };
        assert!(!reason.contains(&storage.root().display().to_string()));
        assert!(matches!(
            ensure_resolved(&storage, &name),
            Err(TransactionError::Unresolved(
                TransactionPhase::Indeterminate
            ))
        ));
    }

    #[test]
    fn committed_record_is_verified_against_the_current_final_artifact() {
        let storage = temp_storage("committed-verification");
        let name = IndexName::new("docs").unwrap();
        let record = begin(&storage, &name, "create").unwrap();
        let candidate = checked_candidate_path(&storage, &name, &record).unwrap();
        std::fs::write(&candidate, b"committed bytes").unwrap();
        let digest = semantic_hub::HubDigest::of(b"committed bytes");
        let record = prepare(&storage, &name, record, digest).unwrap();
        std::fs::rename(candidate, storage.index_path(&name)).unwrap();
        commit(&storage, &name, record, digest).unwrap();

        std::fs::write(storage.index_path(&name), b"tampered bytes").unwrap();
        assert!(matches!(
            ensure_resolved(&storage, &name),
            Err(TransactionError::Malformed(_))
        ));
        assert!(matches!(
            recover(&storage, &name).unwrap(),
            RecoveryOutcome::Indeterminate { .. }
        ));
        assert!(matches!(
            ensure_resolved(&storage, &name),
            Err(TransactionError::Unresolved(
                TransactionPhase::Indeterminate
            ))
        ));
    }

    #[test]
    fn malformed_transaction_record_fails_closed() {
        let storage = temp_storage("malformed-fail-closed");
        let name = IndexName::new("docs").unwrap();
        std::fs::write(txn_path(&storage, &name), b"{truncated").unwrap();

        assert!(matches!(
            ensure_resolved(&storage, &name),
            Err(TransactionError::Malformed(_))
        ));
        assert!(matches!(
            begin(&storage, &name, "create"),
            Err(TransactionError::Malformed(_))
        ));
    }

    #[test]
    fn oversized_or_unbounded_record_fields_fail_closed() {
        let storage = temp_storage("oversized-record");
        let name = IndexName::new("docs").unwrap();
        std::fs::write(
            txn_path(&storage, &name),
            vec![b'x'; MAX_TRANSACTION_RECORD_BYTES as usize + 1],
        )
        .unwrap();
        assert!(matches!(
            read_record(&storage, &name),
            Err(TransactionError::Malformed(_))
        ));

        let invalid_operation_storage = temp_storage("invalid-operation");
        assert!(matches!(
            begin(&invalid_operation_storage, &name, &"x".repeat(65)),
            Err(TransactionError::Malformed(_))
        ));
        assert!(matches!(
            begin(&invalid_operation_storage, &name, "../escape"),
            Err(TransactionError::Malformed(_))
        ));
    }

    #[test]
    fn each_new_transaction_overwrites_the_prior_record_in_place() {
        let storage = temp_storage("overwrite");
        let name = IndexName::new("docs").unwrap();
        let first = begin(&storage, &name, "create").unwrap();
        let candidate = checked_candidate_path(&storage, &name, &first).unwrap();
        std::fs::write(&candidate, b"v1").unwrap();
        let first = prepare(&storage, &name, first, semantic_hub::HubDigest::of(b"v1")).unwrap();
        std::fs::rename(candidate, storage.index_path(&name)).unwrap();
        commit(&storage, &name, first, semantic_hub::HubDigest::of(b"v1")).unwrap();

        let second = begin(&storage, &name, "insert").unwrap();
        // The txn file always reflects only the LATEST transaction -- no
        // unbounded history file, and the new record's id differs from
        // the first, now-superseded one.
        let on_disk = read_record(&storage, &name).unwrap().unwrap();
        assert_eq!(on_disk.transaction_id, second.transaction_id);
        assert_eq!(on_disk.phase, TransactionPhase::Intent);
    }

    #[test]
    fn generated_transaction_id_shape_is_validated() {
        assert!(is_valid_transaction_id("txn-1-2"));
        assert!(!is_valid_transaction_id("txn-attacker"));
        assert!(!is_valid_transaction_id("txn-1-2-extra"));
        assert!(!is_valid_transaction_id("../../outside"));
    }

    #[test]
    fn read_record_rejects_a_hand_edited_record_with_a_path_traversal_candidate_file_name() {
        // Regression test for a real path-escape finding: recover()'s
        // RolledBackAbandonedCandidate branch calls
        // std::fs::remove_file(&candidate_path) where candidate_path is
        // storage.root().join(&record.candidate_file_name). Without
        // validating candidate_file_name on the READ path, a
        // <name>.tvim.txn file corrupted or hand-edited by anyone with
        // filesystem write access to the project (the same class of
        // threat ScopedStorage::checked_index_path's symlink checks
        // already defend against elsewhere in this crate) could make
        // recover() delete an arbitrary file outside the scoped storage
        // root. This must be rejected loudly, not silently trusted.
        let storage = temp_storage("path-escape");
        let name = IndexName::new("docs").unwrap();

        // Plant a file OUTSIDE the scoped root that a successful attack
        // would have deleted.
        let victim = storage.root().parent().unwrap().join(format!(
            "victim-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&victim, b"do not delete me").unwrap();

        // Hand-craft a malicious transaction record, exactly as if
        // <name>.tvim.txn had been corrupted or tampered with.
        let malicious = TransactionRecord {
            transaction_id: "txn-1-2".to_string(),
            operation: "insert".to_string(),
            phase: TransactionPhase::Intent,
            candidate_file_name: format!("../{}", victim.file_name().unwrap().to_str().unwrap()),
            candidate_digest: None,
            artifact_digest: None,
        };
        std::fs::write(
            txn_path(&storage, &name),
            serde_json::to_vec(&malicious).unwrap(),
        )
        .unwrap();

        // read_record must reject this outright...
        let err = read_record(&storage, &name).unwrap_err();
        assert!(matches!(err, TransactionError::Malformed(_)));

        // ...and recover() (which calls read_record internally) must
        // propagate that rejection rather than silently proceeding to
        // delete the victim file.
        let recover_err = recover(&storage, &name).unwrap_err();
        assert!(matches!(recover_err, TransactionError::Malformed(_)));

        assert!(
            victim.is_file(),
            "the file outside the scoped storage root must survive recovery"
        );
        std::fs::remove_file(&victim).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn transaction_candidate_and_final_symlink_leaves_are_rejected() {
        let name = IndexName::new("docs").unwrap();

        let txn_storage = temp_storage("txn-symlink");
        let outside_txn = txn_storage.root().join("outside-txn");
        std::fs::write(&outside_txn, b"{}").unwrap();
        std::os::unix::fs::symlink(&outside_txn, txn_path(&txn_storage, &name)).unwrap();
        assert!(matches!(
            read_record(&txn_storage, &name),
            Err(TransactionError::ScopedStorage(_))
        ));

        let candidate_storage = temp_storage("candidate-symlink");
        let record = begin(&candidate_storage, &name, "create").unwrap();
        let candidate = candidate_storage
            .root()
            .join(record.candidate_file_name.as_str());
        let outside_candidate = candidate_storage.root().join("outside-candidate");
        std::fs::write(&outside_candidate, b"candidate").unwrap();
        std::os::unix::fs::symlink(&outside_candidate, candidate).unwrap();
        assert!(matches!(
            recover(&candidate_storage, &name),
            Err(TransactionError::ScopedStorage(_))
        ));

        let final_storage = temp_storage("final-symlink");
        begin(&final_storage, &name, "create").unwrap();
        let outside_final = final_storage.root().join("outside-final");
        std::fs::write(&outside_final, b"final").unwrap();
        std::os::unix::fs::symlink(&outside_final, final_storage.index_path(&name)).unwrap();
        assert!(matches!(
            recover(&final_storage, &name),
            Err(TransactionError::ScopedStorage(_))
        ));
    }
}
