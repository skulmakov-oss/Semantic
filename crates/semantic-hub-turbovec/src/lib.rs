//! Semantic Hub v0 reference adapter: `vector.turbovec`.
//!
//! Wraps the real [`turbovec`] crate (exact pinned version `=0.9.0`, MIT
//! licensed, <https://github.com/RyanCodrai/turbovec>) behind the generic
//! `semantic-hub` [`HubTool`] contract. All mutable index state is
//! adapter-private; callers only ever see a validated index name and
//! JSON request/reply payloads through the Hub.
//!
//! Search results are labeled `SearchHit` -- candidates/evidence, never a
//! Semantic-truth judgment, verified relevance claim, or permission to act.

pub mod payload;
pub mod storage;
pub mod transaction;

use std::path::PathBuf;

use semantic_hub::runtime::{HubTool, HubToolOutcome, RestrictedHubContext};
use semantic_hub::{
    HubArtifactProvenance, HubCapability, HubDeterminismClass, HubDigest, HubExecutionMode,
    HubOperationDescriptor, HubOperationId, HubResourceBudget, HubToolDescriptor, HubToolError,
    HubToolId, HubToolVersion, HubTrustClass,
};

use payload::{
    check_result_count_bounds, check_vector_batch_bounds, parse, to_bytes, CreateIndexReply,
    CreateIndexRequest, DescribeIndexReply, IndexNameOnlyRequest, InsertReply, InsertRequest,
    RecoverReply, RemoveReply, RemoveRequest, ResetReply, ResetRequest, SearchHit, SearchReply,
    SearchRequest,
};
use storage::{IndexName, ScopedStorage, MAX_INDEX_COUNT};
use transaction::RecoveryOutcome;

pub const TOOL_ID: &str = "vector.turbovec";
pub const TURBOVEC_DEPENDENCY_VERSION: &str = "0.9.0";
pub const TURBOVEC_DEPENDENCY_LICENSE: &str = "MIT";
pub const TURBOVEC_DEPENDENCY_SOURCE: &str = "https://github.com/RyanCodrai/turbovec";
pub const ADAPTER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Reads just enough of a `.tvim` file's header to learn its stored
/// dimension, without reconstructing the full index (rotation matrix,
/// codebook, packed codes, id table -- the ~250-300ms-for-a-cold-4096-dim
/// -index cost measured in this crate's own benchmarks). Duplicates
/// turbovec `=0.9.0`'s exact, documented on-disk layout for `.tvim`
/// (4-byte `"TVIM"` magic + 1-byte format version + a 9-byte core header:
/// `bit_width: u8`, `dim: u32` LE, `n_vectors: u32` LE) -- safe because
/// the dependency is exact-pinned, so this layout cannot silently drift
/// out from under this function. Exists purely so `load()` can reject an
/// over-budget index *before* paying the full reconstruction cost, not
/// after; `load()` still re-checks `index.dim()` against the same
/// budget post-load as a correctness backstop, so a bug in this
/// duplicated parsing can never let an over-budget index through --
/// only, at worst, cost the reconstruction this function exists to
/// avoid.
fn peek_tvim_dim(path: &std::path::Path) -> std::io::Result<usize> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut header = [0u8; 14]; // 4 (magic) + 1 (version) + 9 (core header)
    f.read_exact(&mut header)?;
    if &header[0..4] != b"TVIM" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "not a TVIM file: wrong magic",
        ));
    }
    let dim = u32::from_le_bytes([header[6], header[7], header[8], header[9]]);
    Ok(dim as usize)
}

fn op(id: &str) -> HubOperationId {
    HubOperationId::new(id).expect("operation ids declared in this module are valid")
}

fn adapter_provenance() -> String {
    format!(
        "semantic-hub-turbovec {ADAPTER_VERSION}; turbovec {TURBOVEC_DEPENDENCY_VERSION} \
         ({TURBOVEC_DEPENDENCY_LICENSE}, {TURBOVEC_DEPENDENCY_SOURCE})"
    )
}

/// Build the static descriptor for `vector.turbovec`. Determinism is
/// declared per-operation from the qualification evidence in
/// `docs/spec/hub/turbovec_adapter_v0.md`, not inferred from the tool name.
pub fn descriptor(resource_ceiling: HubResourceBudget) -> HubToolDescriptor {
    HubToolDescriptor {
        tool_id: HubToolId::new(TOOL_ID).expect("TOOL_ID is a valid HubToolId"),
        name: "TurboVec".into(),
        tool_version: HubToolVersion::new(0, 9, 0),
        hub_api_version: semantic_hub::HubApiVersion::CURRENT,
        execution_mode: HubExecutionMode::InProcess,
        trust_class: HubTrustClass::InProcessUnisolated,
        operations: vec![
            HubOperationDescriptor::new(
                op("vector.index.create"),
                [
                    HubCapability::VectorIndexCreate,
                    HubCapability::PrivateStorageRead,
                    HubCapability::PrivateStorageWrite,
                ],
                HubDeterminismClass::Deterministic,
                true,
            ),
            HubOperationDescriptor::new(
                op("vector.index.describe"),
                [
                    HubCapability::VectorIndexRead,
                    HubCapability::PrivateStorageRead,
                ],
                HubDeterminismClass::Deterministic,
                false,
            ),
            HubOperationDescriptor::new(
                op("vector.index.insert"),
                [
                    HubCapability::VectorIndexMutate,
                    HubCapability::PrivateStorageRead,
                    HubCapability::PrivateStorageWrite,
                ],
                HubDeterminismClass::DeterministicWithSeed,
                true,
            ),
            HubOperationDescriptor::new(
                op("vector.index.remove"),
                [
                    HubCapability::VectorIndexMutate,
                    HubCapability::PrivateStorageRead,
                    HubCapability::PrivateStorageWrite,
                ],
                HubDeterminismClass::Deterministic,
                true,
            ),
            HubOperationDescriptor::new(
                op("vector.search"),
                [
                    HubCapability::VectorSearch,
                    HubCapability::PrivateStorageRead,
                ],
                HubDeterminismClass::DeterministicWithSeed,
                false,
            ),
            HubOperationDescriptor::new(
                op("vector.search.filtered"),
                [
                    HubCapability::VectorFilteredSearch,
                    HubCapability::PrivateStorageRead,
                ],
                HubDeterminismClass::DeterministicWithSeed,
                false,
            ),
            HubOperationDescriptor::new(
                op("vector.index.reset"),
                [
                    HubCapability::VectorIndexMutate,
                    HubCapability::PrivateStorageRead,
                    HubCapability::PrivateStorageWrite,
                ],
                HubDeterminismClass::Deterministic,
                true,
            ),
            HubOperationDescriptor::new(
                op("vector.index.recover"),
                [
                    HubCapability::VectorIndexMutate,
                    HubCapability::PrivateStorageRead,
                    HubCapability::PrivateStorageWrite,
                ],
                // The outcome depends on which files happen to exist on
                // disk at call time, not purely on the request payload --
                // honestly classified as environment-dependent rather than
                // deterministic, even though it is idempotent once the
                // underlying state stops changing.
                HubDeterminismClass::EnvironmentDependent,
                true,
            ),
        ],
        resource_ceiling,
        adapter_provenance: adapter_provenance(),
    }
}

/// The `vector.turbovec` reference adapter. Holds only its scoped storage
/// root -- there is no other adapter-private mutable state cached across
/// calls in v0, since each Hub CLI invocation is a fresh process and the
/// on-disk `.tvim` file is the actual source of truth between invocations.
pub struct TurboVecAdapter {
    descriptor: HubToolDescriptor,
    storage: ScopedStorage,
}

impl TurboVecAdapter {
    pub fn new(data_dir: impl Into<PathBuf>, resource_ceiling: HubResourceBudget) -> Self {
        Self {
            descriptor: descriptor(resource_ceiling),
            storage: ScopedStorage::new(data_dir),
        }
    }

    /// Loads the index and rejects it if its stored dimension exceeds
    /// `max_dim` -- the admitted budget's `vector_dimensions` ceiling for
    /// *this* invocation. Only `vector.index.create` gets to choose an
    /// index's dimension; every other operation reaches its data through
    /// this one function, so checking here (rather than per-caller)
    /// enforces the budget against an index that was created under a
    /// wider budget in an earlier, separate CLI invocation.
    fn transaction_error(error: transaction::TransactionError, default_code: &str) -> HubToolError {
        match error {
            transaction::TransactionError::ScopedStorage(message) => {
                HubToolError::new("ScopedStorageViolation", message)
            }
            transaction::TransactionError::Unresolved(phase) => HubToolError::new(
                "RecoveryRequired",
                format!(
                    "index has an unresolved transaction in phase {phase}; run vector.index.recover"
                ),
            ),
            other => HubToolError::new(default_code, other.to_string()),
        }
    }

    fn ensure_transaction_resolved(&self, name: &IndexName) -> Result<(), HubToolError> {
        transaction::ensure_resolved(&self.storage, name)
            .map_err(|error| Self::transaction_error(error, "RecoveryRequired"))
    }

    fn sync_file(path: &std::path::Path) -> std::io::Result<()> {
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?
            .sync_all()
    }

    fn check_deadline(context: &RestrictedHubContext) -> Result<(), HubToolError> {
        if context.deadline_exceeded() {
            Err(HubToolError::new(
                "DeadlineExceeded",
                "operation deadline exceeded",
            ))
        } else {
            Ok(())
        }
    }

    fn load(
        &self,
        name: &IndexName,
        max_dim: usize,
        context: &RestrictedHubContext,
    ) -> Result<turbovec::IdMapIndex, HubToolError> {
        Self::check_deadline(context)?;
        self.ensure_transaction_resolved(name)?;
        let path = self
            .storage
            .checked_index_path(name)
            .map_err(|e| HubToolError::new("ScopedStorageViolation", e.to_string()))?;
        // Reject an over-budget index via a cheap header peek before
        // paying for a full reconstruction that would only be discarded
        // by the post-load check below -- see peek_tvim_dim's own doc
        // comment for why this is safe to duplicate from turbovec's
        // exact-pinned format. A peek failure (missing file, wrong
        // magic, truncated header) is not itself the rejection reason;
        // fall through to the real load, which reports it properly.
        if let Ok(peeked_dim) = peek_tvim_dim(&path) {
            if peeked_dim > max_dim {
                return Err(HubToolError::new(
                    "DimensionExceedsBudget",
                    format!(
                        "index {:?} has dimension {peeked_dim} which exceeds the admitted budget's vector_dimensions ceiling {max_dim}",
                        name.as_str()
                    ),
                ));
            }
        }
        Self::check_deadline(context)?;
        let index = turbovec::IdMapIndex::load(path).map_err(|e| {
            HubToolError::new(
                "IndexLoadFailed",
                format!("index {:?} could not be loaded: {e}", name.as_str()),
            )
        })?;
        if index.dim() > max_dim {
            return Err(HubToolError::new(
                "DimensionExceedsBudget",
                format!(
                    "index {:?} has dimension {} which exceeds the admitted budget's vector_dimensions ceiling {max_dim}",
                    name.as_str(),
                    index.dim()
                ),
            ));
        }
        Ok(index)
    }

    /// Write-ahead, recoverable commit of one index mutation. Follows the
    /// intent -> candidate write -> validate/digest -> atomic rename ->
    /// completion-record sequence documented in `transaction`'s module doc
    /// comment, and returns the artifact provenance the caller embeds in
    /// its `HubToolOutcome` -- the digest is always of the bytes actually
    /// read back from the committed file, never the in-memory buffer that
    /// was written.
    fn save_atomic(
        &self,
        name: &IndexName,
        index: &turbovec::IdMapIndex,
        operation: &str,
        context: &RestrictedHubContext,
    ) -> Result<HubArtifactProvenance, HubToolError> {
        Self::check_deadline(context)?;
        self.storage.ensure_root_checked().map_err(|e| {
            HubToolError::new(
                "StorageUnavailable",
                format!("could not prepare scoped storage directory: {e}"),
            )
        })?;
        self.storage
            .checked_index_path(name)
            .map_err(|e| HubToolError::new("ScopedStorageViolation", e.to_string()))?;

        let txn_record = transaction::begin(&self.storage, name, operation)
            .map_err(|error| Self::transaction_error(error, "PersistenceFailed"))?;
        let tmp_path = transaction::checked_candidate_path(&self.storage, name, &txn_record)
            .map_err(|error| Self::transaction_error(error, "PersistenceFailed"))?;

        Self::check_deadline(context)?;
        index.write(&tmp_path).map_err(|e| {
            HubToolError::new("IndexWriteFailed", format!("could not write index: {e}"))
        })?;
        Self::sync_file(&tmp_path).map_err(|e| {
            HubToolError::new(
                "PersistenceFailed",
                format!("could not sync candidate index: {e}"),
            )
        })?;
        Self::check_deadline(context)?;
        let validated = turbovec::IdMapIndex::load(&tmp_path).map_err(|e| {
            HubToolError::new(
                "CandidateValidationFailed",
                format!("candidate index could not be loaded: {e}"),
            )
        })?;
        if validated.dim() != index.dim()
            || validated.bit_width() != index.bit_width()
            || validated.len() != index.len()
        {
            return Err(HubToolError::new(
                "CandidateValidationFailed",
                "candidate index shape does not match the in-memory index",
            ));
        }
        drop(validated);
        Self::check_deadline(context)?;
        let candidate_bytes = std::fs::read(&tmp_path).map_err(|e| {
            HubToolError::new(
                "PersistenceFailed",
                format!("could not read back candidate index: {e}"),
            )
        })?;
        let candidate_digest = HubDigest::of(&candidate_bytes);
        Self::check_deadline(context)?;
        let txn_record = transaction::prepare(&self.storage, name, txn_record, candidate_digest)
            .map_err(|error| Self::transaction_error(error, "PersistenceFailed"))?;

        // This is the last cooperative cancellation point before the
        // externally visible atomic rename. Once rename starts, finish the
        // transaction record even if the deadline expires meanwhile.
        Self::check_deadline(context)?;
        let tmp_path = transaction::checked_candidate_path(&self.storage, name, &txn_record)
            .map_err(|error| Self::transaction_error(error, "PersistenceFailed"))?;
        let final_path = self
            .storage
            .checked_index_path(name)
            .map_err(|e| HubToolError::new("ScopedStorageViolation", e.to_string()))?;
        std::fs::rename(&tmp_path, &final_path).map_err(|e| {
            HubToolError::new(
                "IndexWriteFailed",
                format!("could not finalize index write: {e}"),
            )
        })?;
        let final_path = self
            .storage
            .checked_index_path(name)
            .map_err(|e| HubToolError::new("ScopedStorageViolation", e.to_string()))?;
        Self::sync_file(&final_path).map_err(|e| {
            HubToolError::new(
                "PersistenceFailed",
                format!("could not sync committed index: {e}"),
            )
        })?;

        let committed_bytes = std::fs::read(&final_path).map_err(|e| {
            HubToolError::new(
                "PersistenceFailed",
                format!("could not read back committed artifact for provenance: {e}"),
            )
        })?;
        let digest = HubDigest::of(&committed_bytes);
        if digest != candidate_digest {
            return Err(HubToolError::new(
                "PersistenceFailed",
                "committed artifact does not match the prepared candidate",
            ));
        }

        let transaction_id = txn_record.transaction_id.clone();
        transaction::commit(&self.storage, name, txn_record, digest)
            .map_err(|error| Self::transaction_error(error, "PersistenceFailed"))?;

        Ok(HubArtifactProvenance {
            kind: "turbovec.index".to_string(),
            id: name.as_str().to_string(),
            digest,
            transaction_id: Some(transaction_id),
        })
    }

    fn index_name(raw: &str) -> Result<IndexName, HubToolError> {
        IndexName::new(raw).map_err(|e| HubToolError::new("InvalidIndexName", e.to_string()))
    }

    fn handle_create(
        &self,
        payload: &[u8],
        max_dim: usize,
        context: &RestrictedHubContext,
    ) -> Result<HubToolOutcome, HubToolError> {
        let req: CreateIndexRequest =
            parse(payload).map_err(|e| HubToolError::new("MalformedInput", e.to_string()))?;
        let name = Self::index_name(&req.index)?;
        // Validate the scoped root before the index-count preflight, then
        // use the checked final leaf for the existence test below.
        self.storage
            .ensure_root_checked()
            .map_err(|e| HubToolError::new("ScopedStorageViolation", e.to_string()))?;
        self.ensure_transaction_resolved(&name)?;
        let index_path = self
            .storage
            .checked_index_path(&name)
            .map_err(|e| HubToolError::new("ScopedStorageViolation", e.to_string()))?;
        if index_path.is_file() {
            return Err(HubToolError::new(
                "IndexAlreadyExists",
                format!("index {:?} already exists", name.as_str()),
            ));
        }
        if self.storage.index_count() >= MAX_INDEX_COUNT {
            return Err(HubToolError::new(
                "TooManyIndexes",
                format!("index count already at bound {MAX_INDEX_COUNT}"),
            ));
        }
        // Enforce the admitted budget's vector_dimensions ceiling before
        // constructing anything -- turbovec's own MAX_DIM (65536) is far
        // looser than the Hub's admitted budget, and dimension drives a
        // dim^2 rotation-matrix allocation, so this must be checked here,
        // not left to turbovec's own (much larger) internal limit.
        if req.dim > max_dim {
            return Err(HubToolError::new(
                "DimensionExceedsBudget",
                format!("requested dim {} exceeds the admitted budget's vector_dimensions ceiling {max_dim}", req.dim),
            ));
        }
        Self::check_deadline(context)?;
        let index = turbovec::IdMapIndex::new(req.dim, req.bit_width)
            .map_err(|e| HubToolError::new("InvalidIndexParameters", e.to_string()))?;
        let artifact = self.save_atomic(&name, &index, "create", context)?;
        Ok(HubToolOutcome {
            payload: to_bytes(&CreateIndexReply {
                index: name.as_str().to_string(),
                dim: req.dim,
                bit_width: req.bit_width,
            }),
            artifact: Some(artifact),
        })
    }

    fn handle_describe(
        &self,
        payload: &[u8],
        max_dim: usize,
        context: &RestrictedHubContext,
    ) -> Result<HubToolOutcome, HubToolError> {
        let req: IndexNameOnlyRequest =
            parse(payload).map_err(|e| HubToolError::new("MalformedInput", e.to_string()))?;
        let name = Self::index_name(&req.index)?;
        let index = self.load(&name, max_dim, context)?;
        Ok(HubToolOutcome::payload_only(to_bytes(
            &DescribeIndexReply {
                index: name.as_str().to_string(),
                dim: index.dim(),
                bit_width: index.bit_width(),
                len: index.len(),
            },
        )))
    }

    fn handle_insert(
        &self,
        payload: &[u8],
        max_vectors: usize,
        max_dim: usize,
        context: &RestrictedHubContext,
    ) -> Result<HubToolOutcome, HubToolError> {
        let req: InsertRequest =
            parse(payload).map_err(|e| HubToolError::new("MalformedInput", e.to_string()))?;
        let name = Self::index_name(&req.index)?;
        check_vector_batch_bounds(req.vectors.len(), max_vectors)
            .map_err(|e| HubToolError::new("TooManyVectors", e.to_string()))?;
        if req.ids.len() != req.vectors.len() {
            return Err(HubToolError::new(
                "IdsVectorsCountMismatch",
                format!(
                    "ids count {} does not match vectors count {}",
                    req.ids.len(),
                    req.vectors.len()
                ),
            ));
        }
        let mut index = self.load(&name, max_dim, context)?;
        let dim = index.dim();
        for (i, v) in req.vectors.iter().enumerate() {
            if v.len() != dim {
                return Err(HubToolError::new(
                    "VectorDimensionMismatch",
                    format!(
                        "vector {i} has length {} but index dimension is {dim}",
                        v.len()
                    ),
                ));
            }
        }
        Self::check_deadline(context)?;
        let flat: Vec<f32> = req.vectors.iter().flatten().copied().collect();
        Self::check_deadline(context)?;
        index
            .add_with_ids_2d(&flat, dim, &req.ids)
            .map_err(|e| HubToolError::new("InsertRejected", e.to_string()))?;
        let inserted = req.ids.len();
        let len = index.len();
        let artifact = self.save_atomic(&name, &index, "insert", context)?;
        Ok(HubToolOutcome {
            payload: to_bytes(&InsertReply {
                index: name.as_str().to_string(),
                inserted,
                len,
            }),
            artifact: Some(artifact),
        })
    }

    fn handle_remove(
        &self,
        payload: &[u8],
        max_vectors: usize,
        max_dim: usize,
        context: &RestrictedHubContext,
    ) -> Result<HubToolOutcome, HubToolError> {
        let req: RemoveRequest =
            parse(payload).map_err(|e| HubToolError::new("MalformedInput", e.to_string()))?;
        let name = Self::index_name(&req.index)?;
        check_vector_batch_bounds(req.ids.len(), max_vectors)
            .map_err(|e| HubToolError::new("TooManyVectors", e.to_string()))?;
        let mut index = self.load(&name, max_dim, context)?;
        let mut removed = Vec::new();
        let mut not_found = Vec::new();
        Self::check_deadline(context)?;
        for id in req.ids {
            if index.remove(id) {
                removed.push(id);
            } else {
                not_found.push(id);
            }
        }
        let len = index.len();
        let artifact = self.save_atomic(&name, &index, "remove", context)?;
        Ok(HubToolOutcome {
            payload: to_bytes(&RemoveReply {
                index: name.as_str().to_string(),
                removed,
                not_found,
                len,
            }),
            artifact: Some(artifact),
        })
    }

    fn handle_search(
        &self,
        payload: &[u8],
        max_results: usize,
        max_vectors: usize,
        filtered: bool,
        max_dim: usize,
        context: &RestrictedHubContext,
    ) -> Result<HubToolOutcome, HubToolError> {
        let req: SearchRequest =
            parse(payload).map_err(|e| HubToolError::new("MalformedInput", e.to_string()))?;
        let name = Self::index_name(&req.index)?;
        if req.queries.is_empty() {
            return Err(HubToolError::new("EmptyQuery", "queries must not be empty"));
        }
        // Bound the TOTAL result count (queries.len() * k), not just k
        // alone: turbovec allocates score/id buffers proportional to
        // queries.len() * min(k, index.len()), and the Hub's own `hits`
        // construction below does the same. A payload well under the
        // input-bytes budget can still contain hundreds of thousands of
        // small queries; checking only k left the total unbounded before
        // ever calling into turbovec. Checked multiplication: nq is
        // always >= 1 here, so this is at least as strict as (and, for
        // nq > 1, strictly stricter than) the old k-only check.
        let total_results =
            req.queries.len().checked_mul(req.k).ok_or_else(|| {
                HubToolError::new("TooManyResults", "queries.len() * k overflowed")
            })?;
        check_result_count_bounds(total_results, max_results)
            .map_err(|e| HubToolError::new("TooManyResults", e.to_string()))?;
        let index = self.load(&name, max_dim, context)?;
        let dim = index.dim();
        for (i, q) in req.queries.iter().enumerate() {
            if q.len() != dim {
                return Err(HubToolError::new(
                    "VectorDimensionMismatch",
                    format!(
                        "query {i} has length {} but index dimension is {dim}",
                        q.len()
                    ),
                ));
            }
        }
        Self::check_deadline(context)?;
        let flat: Vec<f32> = req.queries.iter().flatten().copied().collect();
        // turbovec::search / search_with_allowlist panic on non-finite
        // input; validate up front using turbovec's own exported check so
        // adversarial NaN/Inf/huge-magnitude queries are a typed rejection
        // rather than a caught panic.
        if let Some((vi, ci, v)) = turbovec::first_invalid_coord(&flat, dim) {
            return Err(HubToolError::new(
                "InvalidQueryValue",
                format!("query {vi} coord {ci} is invalid: {v}"),
            ));
        }

        Self::check_deadline(context)?;
        let allowed_ids = if filtered {
            let ids = req.allowed_ids.clone().unwrap_or_default();
            if ids.is_empty() {
                return Err(HubToolError::new(
                    "EmptyFilter",
                    "vector.search.filtered requires a non-empty allowed_ids list",
                ));
            }
            // Bound the allowlist against the admitted item-count budget
            // before cloning/iterating any further: a compact allowlist
            // of several million ids fits comfortably under the 8 MiB
            // request-file cap, and nothing upstream of this point checks
            // its length against resource_budget.index_item_count.
            check_vector_batch_bounds(ids.len(), max_vectors)
                .map_err(|e| HubToolError::new("TooManyVectors", e.to_string()))?;
            for id in &ids {
                if !index.contains(*id) {
                    return Err(HubToolError::new(
                        "UnknownFilterId",
                        format!("allowed_ids contains {id}, which is not present in the index"),
                    ));
                }
            }
            Some(ids)
        } else {
            // SearchRequest is shared by vector.search and
            // vector.search.filtered; a caller mistakenly setting
            // allowed_ids on the unfiltered operation must not have it
            // silently discarded -- that would return hits outside what
            // the caller's own request declared it wanted restricted to.
            if req.allowed_ids.is_some() {
                return Err(HubToolError::new(
                    "UnexpectedFilter",
                    "vector.search does not accept allowed_ids -- use vector.search.filtered instead",
                ));
            }
            None
        };

        Self::check_deadline(context)?;
        let (scores, ids) = index.search_with_allowlist(&flat, req.k, allowed_ids.as_deref());
        let nq = req.queries.len();
        let k = ids.len().checked_div(nq).unwrap_or(0);
        let mut hits = Vec::with_capacity(nq);
        for qi in 0..nq {
            let mut row = Vec::with_capacity(k);
            for r in 0..k {
                let idx = qi * k + r;
                row.push(SearchHit {
                    external_id: ids[idx],
                    score: scores[idx],
                    rank: r,
                });
            }
            hits.push(row);
        }
        Ok(HubToolOutcome::payload_only(to_bytes(&SearchReply {
            index: name.as_str().to_string(),
            index_version: 1,
            hits,
        })))
    }

    fn handle_reset(
        &self,
        payload: &[u8],
        max_dim: usize,
        context: &RestrictedHubContext,
    ) -> Result<HubToolOutcome, HubToolError> {
        let req: ResetRequest =
            parse(payload).map_err(|e| HubToolError::new("MalformedInput", e.to_string()))?;
        let name = Self::index_name(&req.index)?;
        let existing = self.load(&name, max_dim, context)?;
        let (dim, bit_width) = (existing.dim(), existing.bit_width());
        // turbovec has no in-place clear/truncate: a v0 "reset" is
        // constructing a fresh empty index with the same shape and
        // overwriting the persisted file, documented explicitly rather
        // than inventing an unsupported private truncation format.
        Self::check_deadline(context)?;
        let fresh = turbovec::IdMapIndex::new(dim, bit_width)
            .map_err(|e| HubToolError::new("InvalidIndexParameters", e.to_string()))?;
        let artifact = self.save_atomic(&name, &fresh, "reset", context)?;
        Ok(HubToolOutcome {
            payload: to_bytes(&ResetReply {
                index: name.as_str().to_string(),
                dim,
                bit_width,
            }),
            artifact: Some(artifact),
        })
    }

    /// Inspect (and, where provably safe, resolve) the transaction state
    /// for one index -- see `transaction::recover`'s own doc comment for
    /// the exact recovery algorithm. Does not itself call `load()`: an
    /// index with an unresolved `Intent` transaction is exactly the case
    /// this operation exists to unblock, so it must not be gated by the
    /// same check it is meant to resolve.
    fn handle_recover(
        &self,
        payload: &[u8],
        context: &RestrictedHubContext,
    ) -> Result<HubToolOutcome, HubToolError> {
        let req: IndexNameOnlyRequest =
            parse(payload).map_err(|e| HubToolError::new("MalformedInput", e.to_string()))?;
        let name = Self::index_name(&req.index)?;
        self.storage
            .ensure_root_checked()
            .map_err(|e| HubToolError::new("ScopedStorageViolation", e.to_string()))?;
        Self::check_deadline(context)?;
        let outcome = transaction::recover(&self.storage, &name)
            .map_err(|error| Self::transaction_error(error, "PersistenceFailed"))?;
        let (outcome_str, artifact_digest, reason) = match &outcome {
            RecoveryOutcome::NoTransaction => ("NoTransaction", None, None),
            RecoveryOutcome::AlreadyCommitted { artifact_digest } => {
                ("AlreadyCommitted", artifact_digest.clone(), None)
            }
            RecoveryOutcome::RolledBackAbandonedCandidate { artifact_digest } => (
                "RolledBackAbandonedCandidate",
                artifact_digest.clone(),
                None,
            ),
            RecoveryOutcome::FinalizedCommit { artifact_digest } => {
                ("FinalizedCommit", Some(artifact_digest.clone()), None)
            }
            RecoveryOutcome::Indeterminate { reason } => {
                let reason: String = reason.chars().take(256).collect();
                return Err(HubToolError::new(
                    "RecoveryRequired",
                    format!("transaction remains indeterminate: {reason}"),
                ));
            }
        };
        Ok(HubToolOutcome::payload_only(to_bytes(&RecoverReply {
            index: name.as_str().to_string(),
            outcome: outcome_str.to_string(),
            artifact_digest,
            reason,
        })))
    }
}

impl HubTool for TurboVecAdapter {
    fn descriptor(&self) -> &HubToolDescriptor {
        &self.descriptor
    }

    fn handle(
        &mut self,
        operation_id: &HubOperationId,
        payload: &[u8],
        context: &RestrictedHubContext,
    ) -> Result<HubToolOutcome, HubToolError> {
        Self::check_deadline(context)?;
        let max_vectors = context.resource_budget.index_item_count as usize;
        let max_results = context.resource_budget.result_count as usize;
        let max_dim = context.resource_budget.vector_dimensions as usize;
        match operation_id.as_str() {
            "vector.index.create" => self.handle_create(payload, max_dim, context),
            "vector.index.describe" => self.handle_describe(payload, max_dim, context),
            "vector.index.insert" => self.handle_insert(payload, max_vectors, max_dim, context),
            "vector.index.remove" => self.handle_remove(payload, max_vectors, max_dim, context),
            "vector.search" => {
                self.handle_search(payload, max_results, max_vectors, false, max_dim, context)
            }
            "vector.search.filtered" => {
                self.handle_search(payload, max_results, max_vectors, true, max_dim, context)
            }
            "vector.index.reset" => self.handle_reset(payload, max_dim, context),
            "vector.index.recover" => self.handle_recover(payload, context),
            other => Err(HubToolError::new(
                "UnknownOperation",
                format!("vector.turbovec has no operation {other:?}"),
            )),
        }
    }

    fn validate_reply(&self, _operation_id: &HubOperationId, payload: &[u8]) -> Result<(), String> {
        // Structural check: every reply is required to be valid JSON. This
        // cannot fail in practice (all reply paths build via `to_bytes` on
        // well-formed types), but it is the adapter's declared contract
        // check, exercised directly by protocol-violation fault-injection
        // tests rather than left unverified.
        serde_json::from_slice::<serde_json::Value>(payload)
            .map(|_| ())
            .map_err(|e| format!("adapter reply is not valid JSON: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semantic_hub::{HubCapabilitySet, HubResourceBudget};
    use std::path::PathBuf;

    fn temp_dir(tag: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "semantic-hub-turbovec-test-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        dir
    }

    fn adapter(tag: &str) -> TurboVecAdapter {
        TurboVecAdapter::new(temp_dir(tag), HubResourceBudget::V0_CEILING)
    }

    #[test]
    fn peek_tvim_dim_matches_the_real_index_dim_after_a_real_write() {
        // Regression test for the header-peek's own correctness, not
        // just its presence: writes a real turbovec index to disk and
        // confirms peek_tvim_dim (this crate's own duplicated,
        // format-locked parsing) agrees with the value the real
        // IdMapIndex::load / .dim() reports for the same file.
        let dir = temp_dir("peek-tvim-dim");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("probe.tvim");
        let index = turbovec::IdMapIndex::new(16, 4).unwrap();
        index.write(&path).unwrap();

        assert_eq!(peek_tvim_dim(&path).unwrap(), 16);
        let reloaded = turbovec::IdMapIndex::load(&path).unwrap();
        assert_eq!(peek_tvim_dim(&path).unwrap(), reloaded.dim());
    }

    fn ctx<'a>(
        budget: &'a HubResourceBudget,
        caps: &'a HubCapabilitySet,
    ) -> RestrictedHubContext<'a> {
        RestrictedHubContext {
            resource_budget: budget,
            capability_context: caps,
            deadline: None,
        }
    }

    fn unit_vectors(n: usize, dim: usize) -> Vec<Vec<f32>> {
        (0..n)
            .map(|i| {
                let mut v = vec![0.0f32; dim];
                v[i % dim] = 1.0;
                v
            })
            .collect()
    }

    #[test]
    fn create_then_describe_round_trips_shape() {
        let mut a = adapter("create-describe");
        let budget = HubResourceBudget::V0_CEILING;
        let caps = HubCapabilitySet::empty();

        let create = a
            .handle(
                &op("vector.index.create"),
                br#"{"index":"docs","dim":8,"bit_width":4}"#,
                &ctx(&budget, &caps),
            )
            .unwrap();
        assert!(create
            .artifact
            .as_ref()
            .and_then(|artifact| artifact.transaction_id.as_deref())
            .is_some_and(|id| id.starts_with("txn-")));
        let create: CreateIndexReply = serde_json::from_slice(&create.payload).unwrap();
        assert_eq!(create.dim, 8);

        let describe = a
            .handle(
                &op("vector.index.describe"),
                br#"{"index":"docs"}"#,
                &ctx(&budget, &caps),
            )
            .unwrap();
        let describe: DescribeIndexReply = serde_json::from_slice(&describe.payload).unwrap();
        assert_eq!(describe.dim, 8);
        assert_eq!(describe.bit_width, 4);
        assert_eq!(describe.len, 0);
    }

    #[test]
    fn create_fails_closed_on_an_unresolved_or_malformed_transaction_record() {
        let budget = HubResourceBudget::V0_CEILING;
        let caps = HubCapabilitySet::empty();

        let mut unresolved = adapter("create-unresolved-transaction");
        unresolved.storage.ensure_root_checked().unwrap();
        let name = IndexName::new("docs").unwrap();
        transaction::begin(&unresolved.storage, &name, "create").unwrap();
        let error = unresolved
            .handle(
                &op("vector.index.create"),
                br#"{"index":"docs","dim":8,"bit_width":4}"#,
                &ctx(&budget, &caps),
            )
            .unwrap_err();
        assert_eq!(error.code, "RecoveryRequired");

        let mut malformed = adapter("create-malformed-transaction");
        malformed.storage.ensure_root_checked().unwrap();
        std::fs::write(
            transaction::txn_path(&malformed.storage, &name),
            b"{truncated",
        )
        .unwrap();
        let error = malformed
            .handle(
                &op("vector.index.create"),
                br#"{"index":"docs","dim":8,"bit_width":4}"#,
                &ctx(&budget, &caps),
            )
            .unwrap_err();
        assert_eq!(error.code, "RecoveryRequired");
    }

    #[test]
    fn indeterminate_transaction_blocks_both_load_and_create() {
        let mut a = adapter("indeterminate-blocks");
        let budget = HubResourceBudget::V0_CEILING;
        let caps = HubCapabilitySet::empty();
        a.handle(
            &op("vector.index.create"),
            br#"{"index":"docs","dim":8,"bit_width":4}"#,
            &ctx(&budget, &caps),
        )
        .unwrap();

        let name = IndexName::new("docs").unwrap();
        let record = transaction::begin(&a.storage, &name, "insert").unwrap();
        let candidate = transaction::checked_candidate_path(&a.storage, &name, &record).unwrap();
        std::fs::write(&candidate, b"different bytes").unwrap();
        transaction::prepare(&a.storage, &name, record, HubDigest::of(b"different bytes")).unwrap();
        std::fs::remove_file(candidate).unwrap();
        let recover_error = a
            .handle(
                &op("vector.index.recover"),
                br#"{"index":"docs"}"#,
                &ctx(&budget, &caps),
            )
            .unwrap_err();
        assert_eq!(recover_error.code, "RecoveryRequired");
        assert!(!recover_error
            .message
            .contains(&a.storage.root().display().to_string()));

        let describe_error = a
            .handle(
                &op("vector.index.describe"),
                br#"{"index":"docs"}"#,
                &ctx(&budget, &caps),
            )
            .unwrap_err();
        assert_eq!(describe_error.code, "RecoveryRequired");

        let create_error = a
            .handle(
                &op("vector.index.create"),
                br#"{"index":"docs","dim":8,"bit_width":4}"#,
                &ctx(&budget, &caps),
            )
            .unwrap_err();
        assert_eq!(create_error.code, "RecoveryRequired");
    }

    #[test]
    fn committed_digest_mismatch_blocks_load_before_turbovec_parses_the_file() {
        let mut a = adapter("committed-digest-mismatch");
        let budget = HubResourceBudget::V0_CEILING;
        let caps = HubCapabilitySet::empty();
        a.handle(
            &op("vector.index.create"),
            br#"{"index":"docs","dim":8,"bit_width":4}"#,
            &ctx(&budget, &caps),
        )
        .unwrap();

        let name = IndexName::new("docs").unwrap();
        std::fs::write(a.storage.index_path(&name), b"tampered").unwrap();
        let error = a
            .handle(
                &op("vector.index.describe"),
                br#"{"index":"docs"}"#,
                &ctx(&budget, &caps),
            )
            .unwrap_err();
        assert_eq!(error.code, "RecoveryRequired");
    }

    #[test]
    fn expired_deadline_is_a_typed_error_before_adapter_io() {
        let mut a = adapter("deadline-before-io");
        let budget = HubResourceBudget::V0_CEILING;
        let caps = HubCapabilitySet::empty();
        let expired = RestrictedHubContext {
            resource_budget: &budget,
            capability_context: &caps,
            deadline: Some(std::time::Instant::now() - std::time::Duration::from_millis(1)),
        };

        let error = a
            .handle(
                &op("vector.index.create"),
                br#"{"index":"docs","dim":8,"bit_width":4}"#,
                &expired,
            )
            .unwrap_err();
        assert_eq!(error.code, "DeadlineExceeded");
        let name = IndexName::new("docs").unwrap();
        assert!(!transaction::txn_path(&a.storage, &name).exists());
        assert!(!a.storage.index_path(&name).exists());
    }

    #[test]
    fn create_rejects_dimension_exceeding_the_admitted_budget() {
        // Regression test: `handle_create` previously never consulted the
        // admitted resource_budget.vector_dimensions at all, so a caller
        // could request a dim up to turbovec's own much looser MAX_DIM
        // (65536) despite a narrower admitted ceiling (e.g. the documented
        // default of 4096).
        let mut a = adapter("dim-budget");
        let caps = HubCapabilitySet::empty();
        let budget = HubResourceBudget {
            vector_dimensions: 8,
            ..HubResourceBudget::V0_CEILING
        };
        let err = a
            .handle(
                &op("vector.index.create"),
                br#"{"index":"docs","dim":16,"bit_width":4}"#,
                &ctx(&budget, &caps),
            )
            .unwrap_err();
        assert_eq!(err.code, "DimensionExceedsBudget");
    }

    #[test]
    fn create_within_the_admitted_dimension_budget_is_accepted() {
        let mut a = adapter("dim-budget-ok");
        let caps = HubCapabilitySet::empty();
        let budget = HubResourceBudget {
            vector_dimensions: 16,
            ..HubResourceBudget::V0_CEILING
        };
        assert!(a
            .handle(
                &op("vector.index.create"),
                br#"{"index":"docs","dim":16,"bit_width":4}"#,
                &ctx(&budget, &caps),
            )
            .is_ok());
    }

    #[test]
    fn describe_rejects_an_existing_index_whose_dimension_exceeds_the_current_budget() {
        // Regression test: only vector.index.create ever consulted
        // vector_dimensions -- every other operation loaded an existing
        // index with no check at all, so an index created under a wide
        // budget in one CLI invocation could later be operated on by a
        // separate invocation admitted with a much narrower dimension
        // budget, with no enforcement.
        let mut a = adapter("dim-budget-existing");
        let wide_budget = HubResourceBudget::V0_CEILING;
        let caps = HubCapabilitySet::empty();
        a.handle(
            &op("vector.index.create"),
            br#"{"index":"docs","dim":16,"bit_width":4}"#,
            &ctx(&wide_budget, &caps),
        )
        .unwrap();

        let narrow_budget = HubResourceBudget {
            vector_dimensions: 8,
            ..HubResourceBudget::V0_CEILING
        };
        let err = a
            .handle(
                &op("vector.index.describe"),
                br#"{"index":"docs"}"#,
                &ctx(&narrow_budget, &caps),
            )
            .unwrap_err();
        assert_eq!(err.code, "DimensionExceedsBudget");
    }

    #[cfg(unix)]
    #[test]
    fn create_rejects_a_symlinked_scoped_root_end_to_end() {
        // End-to-end companion to storage.rs's own unit test: confirms the
        // adapter surfaces the violation as a typed HubToolError through
        // the real handle() path, and that nothing was written through
        // the symlink target.
        let target = temp_dir("symlink-target-e2e");
        std::fs::create_dir_all(&target).unwrap();
        let link = temp_dir("symlink-root-e2e");
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let mut a = TurboVecAdapter::new(link, HubResourceBudget::V0_CEILING);
        let budget = HubResourceBudget::V0_CEILING;
        let caps = HubCapabilitySet::empty();
        let err = a
            .handle(
                &op("vector.index.create"),
                br#"{"index":"docs","dim":8,"bit_width":4}"#,
                &ctx(&budget, &caps),
            )
            .unwrap_err();
        assert_eq!(err.code, "ScopedStorageViolation");
        assert!(std::fs::read_dir(&target).unwrap().next().is_none());
    }

    #[cfg(unix)]
    #[test]
    fn create_rejects_a_symlinked_root_even_when_the_target_already_has_a_matching_file() {
        // Regression test: the test above uses an EMPTY symlink target,
        // which happens to still surface ScopedStorageViolation via
        // save_atomic's later check -- but handle_create's own earlier,
        // unchecked exists()/index_count() preflight reads would
        // silently follow the symlink first. With a pre-existing
        // <name>.tvim already in the target, the unfixed code returned
        // IndexAlreadyExists (proof it had already read outside the
        // scoped directory) instead of rejecting the root itself.
        let target = temp_dir("symlink-root-preexisting-target");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join("docs.tvim"), b"not a real index").unwrap();

        let link = temp_dir("symlink-root-preexisting-link");
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let mut a = TurboVecAdapter::new(link, HubResourceBudget::V0_CEILING);
        let budget = HubResourceBudget::V0_CEILING;
        let caps = HubCapabilitySet::empty();
        let err = a
            .handle(
                &op("vector.index.create"),
                br#"{"index":"docs","dim":8,"bit_width":4}"#,
                &ctx(&budget, &caps),
            )
            .unwrap_err();
        assert_eq!(err.code, "ScopedStorageViolation");
    }

    #[test]
    fn filtered_search_rejects_an_allowlist_exceeding_the_admitted_item_count_budget() {
        // Regression test: allowed_ids was cloned and iterated with no
        // check against resource_budget.index_item_count at all -- a
        // compact allowlist of several million ids fits comfortably
        // under the 8 MiB request-file cap, well beyond a narrow
        // admitted item budget.
        let mut a = adapter("filtered-allowlist-budget");
        let caps = HubCapabilitySet::empty();
        a.handle(
            &op("vector.index.create"),
            br#"{"index":"docs","dim":8,"bit_width":4}"#,
            &ctx(&HubResourceBudget::V0_CEILING, &caps),
        )
        .unwrap();
        let insert_req = serde_json::json!({
            "index": "docs",
            "vectors": unit_vectors(4, 8),
            "ids": [1, 2, 3, 4],
        });
        a.handle(
            &op("vector.index.insert"),
            serde_json::to_vec(&insert_req).unwrap().as_slice(),
            &ctx(&HubResourceBudget::V0_CEILING, &caps),
        )
        .unwrap();

        let narrow_budget = HubResourceBudget {
            index_item_count: 2,
            ..HubResourceBudget::V0_CEILING
        };
        let search_req = serde_json::json!({
            "index": "docs",
            "queries": [unit_vectors(1, 8)[0].clone()],
            "k": 1,
            "allowed_ids": [1, 2, 3, 4],
        });
        let err = a
            .handle(
                &op("vector.search.filtered"),
                serde_json::to_vec(&search_req).unwrap().as_slice(),
                &ctx(&narrow_budget, &caps),
            )
            .unwrap_err();
        assert_eq!(err.code, "TooManyVectors");
    }

    #[test]
    fn creating_an_already_existing_index_is_rejected() {
        let mut a = adapter("dup-create");
        let budget = HubResourceBudget::V0_CEILING;
        let caps = HubCapabilitySet::empty();
        let req = br#"{"index":"docs","dim":8,"bit_width":4}"#;
        a.handle(&op("vector.index.create"), req, &ctx(&budget, &caps))
            .unwrap();
        let err = a
            .handle(&op("vector.index.create"), req, &ctx(&budget, &caps))
            .unwrap_err();
        assert_eq!(err.code, "IndexAlreadyExists");
    }

    #[test]
    fn full_insert_search_remove_search_again_workflow() {
        let mut a = adapter("full-workflow");
        let budget = HubResourceBudget::V0_CEILING;
        let caps = HubCapabilitySet::empty();

        a.handle(
            &op("vector.index.create"),
            br#"{"index":"docs","dim":8,"bit_width":4}"#,
            &ctx(&budget, &caps),
        )
        .unwrap();

        let vectors = unit_vectors(4, 8);
        let insert_req = serde_json::json!({
            "index": "docs",
            "vectors": vectors,
            "ids": [10, 20, 30, 40],
        });
        let insert_reply = a
            .handle(
                &op("vector.index.insert"),
                serde_json::to_vec(&insert_req).unwrap().as_slice(),
                &ctx(&budget, &caps),
            )
            .unwrap();
        let insert_reply: InsertReply = serde_json::from_slice(&insert_reply.payload).unwrap();
        assert_eq!(insert_reply.inserted, 4);
        assert_eq!(insert_reply.len, 4);

        let search_req = serde_json::json!({
            "index": "docs",
            "queries": [vectors[1].clone()],
            "k": 2,
        });
        let search_reply = a
            .handle(
                &op("vector.search"),
                serde_json::to_vec(&search_req).unwrap().as_slice(),
                &ctx(&budget, &caps),
            )
            .unwrap();
        let search_reply: SearchReply = serde_json::from_slice(&search_reply.payload).unwrap();
        assert_eq!(search_reply.hits.len(), 1);
        assert_eq!(search_reply.hits[0][0].external_id, 20);

        let remove_req = serde_json::json!({"index": "docs", "ids": [20]});
        let remove_reply = a
            .handle(
                &op("vector.index.remove"),
                serde_json::to_vec(&remove_req).unwrap().as_slice(),
                &ctx(&budget, &caps),
            )
            .unwrap();
        let remove_reply: RemoveReply = serde_json::from_slice(&remove_reply.payload).unwrap();
        assert_eq!(remove_reply.removed, vec![20]);
        assert_eq!(remove_reply.len, 3);

        let search_again = a
            .handle(
                &op("vector.search"),
                serde_json::to_vec(&search_req).unwrap().as_slice(),
                &ctx(&budget, &caps),
            )
            .unwrap();
        let search_again: SearchReply = serde_json::from_slice(&search_again.payload).unwrap();
        assert!(!search_again.hits[0].iter().any(|h| h.external_id == 20));
    }

    #[test]
    fn plain_search_rejects_an_allowed_ids_field_instead_of_silently_ignoring_it() {
        // Regression test: vector.search and vector.search.filtered share
        // SearchRequest. A caller mistakenly setting allowed_ids on the
        // unfiltered vector.search previously had it silently discarded,
        // so the invocation succeeded with hits outside the allowlist
        // the caller's own request declared -- surprising, not just
        // permissive, since the field was accepted but its stated intent
        // was ignored.
        let mut a = adapter("plain-search-with-filter");
        let budget = HubResourceBudget::V0_CEILING;
        let caps = HubCapabilitySet::empty();
        a.handle(
            &op("vector.index.create"),
            br#"{"index":"docs","dim":8,"bit_width":4}"#,
            &ctx(&budget, &caps),
        )
        .unwrap();
        let insert_req = serde_json::json!({
            "index": "docs",
            "vectors": unit_vectors(2, 8),
            "ids": [1, 2],
        });
        a.handle(
            &op("vector.index.insert"),
            serde_json::to_vec(&insert_req).unwrap().as_slice(),
            &ctx(&budget, &caps),
        )
        .unwrap();

        let search_req = serde_json::json!({
            "index": "docs",
            "queries": [unit_vectors(1, 8)[0].clone()],
            "k": 1,
            "allowed_ids": [1],
        });
        let err = a
            .handle(
                &op("vector.search"),
                serde_json::to_vec(&search_req).unwrap().as_slice(),
                &ctx(&budget, &caps),
            )
            .unwrap_err();
        assert_eq!(err.code, "UnexpectedFilter");
    }

    #[test]
    fn filtered_search_rejects_empty_allowlist_without_panicking() {
        let mut a = adapter("filtered-empty");
        let budget = HubResourceBudget::V0_CEILING;
        let caps = HubCapabilitySet::empty();
        a.handle(
            &op("vector.index.create"),
            br#"{"index":"docs","dim":8,"bit_width":4}"#,
            &ctx(&budget, &caps),
        )
        .unwrap();
        let insert_req = serde_json::json!({
            "index": "docs",
            "vectors": unit_vectors(2, 8),
            "ids": [1, 2],
        });
        a.handle(
            &op("vector.index.insert"),
            serde_json::to_vec(&insert_req).unwrap().as_slice(),
            &ctx(&budget, &caps),
        )
        .unwrap();

        let search_req = serde_json::json!({
            "index": "docs",
            "queries": [unit_vectors(1, 8)[0].clone()],
            "k": 1,
            "allowed_ids": [],
        });
        let err = a
            .handle(
                &op("vector.search.filtered"),
                serde_json::to_vec(&search_req).unwrap().as_slice(),
                &ctx(&budget, &caps),
            )
            .unwrap_err();
        assert_eq!(err.code, "EmptyFilter");
    }

    #[test]
    fn filtered_search_rejects_unknown_id_without_panicking() {
        let mut a = adapter("filtered-unknown");
        let budget = HubResourceBudget::V0_CEILING;
        let caps = HubCapabilitySet::empty();
        a.handle(
            &op("vector.index.create"),
            br#"{"index":"docs","dim":8,"bit_width":4}"#,
            &ctx(&budget, &caps),
        )
        .unwrap();
        let insert_req = serde_json::json!({
            "index": "docs",
            "vectors": unit_vectors(2, 8),
            "ids": [1, 2],
        });
        a.handle(
            &op("vector.index.insert"),
            serde_json::to_vec(&insert_req).unwrap().as_slice(),
            &ctx(&budget, &caps),
        )
        .unwrap();

        let search_req = serde_json::json!({
            "index": "docs",
            "queries": [unit_vectors(1, 8)[0].clone()],
            "k": 1,
            "allowed_ids": [999],
        });
        let err = a
            .handle(
                &op("vector.search.filtered"),
                serde_json::to_vec(&search_req).unwrap().as_slice(),
                &ctx(&budget, &caps),
            )
            .unwrap_err();
        assert_eq!(err.code, "UnknownFilterId");
    }

    #[test]
    fn huge_magnitude_query_is_rejected_as_typed_error_not_a_panic() {
        // Standard JSON has no literal for NaN/Infinity at all -- serde_json's
        // `json!` macro silently turns a NaN f32 into `null`, which fails at
        // the parse stage (see `raw_nan_text_is_rejected_as_malformed_json`
        // below), never reaching the semantic finite-value check. A huge
        // magnitude (turbovec's other `first_invalid_coord` rejection case,
        // `|value| >= 1e16`) *is* representable as a valid JSON number, so
        // this is the real way to exercise `InvalidQueryValue` through JSON.
        let mut a = adapter("huge-query");
        let budget = HubResourceBudget::V0_CEILING;
        let caps = HubCapabilitySet::empty();
        a.handle(
            &op("vector.index.create"),
            br#"{"index":"docs","dim":8,"bit_width":4}"#,
            &ctx(&budget, &caps),
        )
        .unwrap();
        let insert_req = serde_json::json!({
            "index": "docs",
            "vectors": unit_vectors(1, 8),
            "ids": [1],
        });
        a.handle(
            &op("vector.index.insert"),
            serde_json::to_vec(&insert_req).unwrap().as_slice(),
            &ctx(&budget, &caps),
        )
        .unwrap();

        let mut bad_query = vec![0.0f32; 8];
        bad_query[0] = 1e20;
        let search_req = serde_json::json!({"index": "docs", "queries": [bad_query], "k": 1});
        let err = a
            .handle(
                &op("vector.search"),
                serde_json::to_vec(&search_req).unwrap().as_slice(),
                &ctx(&budget, &caps),
            )
            .unwrap_err();
        assert_eq!(err.code, "InvalidQueryValue");
    }

    #[test]
    fn raw_nan_text_is_rejected_as_malformed_json() {
        // A literal bareword `NaN` token is not valid JSON syntax, so
        // serde_json's strict parser rejects it before any field is even
        // read -- this is the actual "NaN input" rejection path a caller
        // hits over the wire, distinct from (and prior to) semantic
        // per-coordinate validation.
        let mut a = adapter("raw-nan-text");
        let budget = HubResourceBudget::V0_CEILING;
        let caps = HubCapabilitySet::empty();
        a.handle(
            &op("vector.index.create"),
            br#"{"index":"docs","dim":8,"bit_width":4}"#,
            &ctx(&budget, &caps),
        )
        .unwrap();
        let raw = br#"{"index":"docs","queries":[[NaN,0,0,0,0,0,0,0]],"k":1}"#;
        let err = a
            .handle(&op("vector.search"), raw, &ctx(&budget, &caps))
            .unwrap_err();
        assert_eq!(err.code, "MalformedInput");
    }

    #[test]
    fn remove_rejects_a_batch_exceeding_the_admitted_item_count_budget() {
        // Regression test: handle_remove previously received only max_dim
        // and never checked req.ids.len() against the admitted
        // index_item_count budget at all, unlike handle_insert's
        // analogous check_vector_batch_bounds call -- a request admitted
        // with a tiny item budget could still remove an unbounded number
        // of ids in one call.
        let mut a = adapter("remove-item-budget");
        let caps = HubCapabilitySet::empty();
        a.handle(
            &op("vector.index.create"),
            br#"{"index":"docs","dim":8,"bit_width":4}"#,
            &ctx(&HubResourceBudget::V0_CEILING, &caps),
        )
        .unwrap();
        let insert_req = serde_json::json!({
            "index": "docs",
            "vectors": unit_vectors(4, 8),
            "ids": [1, 2, 3, 4],
        });
        a.handle(
            &op("vector.index.insert"),
            serde_json::to_vec(&insert_req).unwrap().as_slice(),
            &ctx(&HubResourceBudget::V0_CEILING, &caps),
        )
        .unwrap();

        let narrow_budget = HubResourceBudget {
            index_item_count: 2,
            ..HubResourceBudget::V0_CEILING
        };
        let remove_req = serde_json::json!({"index": "docs", "ids": [1, 2, 3, 4]});
        let err = a
            .handle(
                &op("vector.index.remove"),
                serde_json::to_vec(&remove_req).unwrap().as_slice(),
                &ctx(&narrow_budget, &caps),
            )
            .unwrap_err();
        assert_eq!(err.code, "TooManyVectors");
    }

    #[test]
    fn remove_within_the_admitted_item_count_budget_is_accepted() {
        let mut a = adapter("remove-item-budget-ok");
        let caps = HubCapabilitySet::empty();
        a.handle(
            &op("vector.index.create"),
            br#"{"index":"docs","dim":8,"bit_width":4}"#,
            &ctx(&HubResourceBudget::V0_CEILING, &caps),
        )
        .unwrap();
        let insert_req = serde_json::json!({
            "index": "docs",
            "vectors": unit_vectors(4, 8),
            "ids": [1, 2, 3, 4],
        });
        a.handle(
            &op("vector.index.insert"),
            serde_json::to_vec(&insert_req).unwrap().as_slice(),
            &ctx(&HubResourceBudget::V0_CEILING, &caps),
        )
        .unwrap();

        let narrow_budget = HubResourceBudget {
            index_item_count: 2,
            ..HubResourceBudget::V0_CEILING
        };
        let remove_req = serde_json::json!({"index": "docs", "ids": [1, 2]});
        assert!(a
            .handle(
                &op("vector.index.remove"),
                serde_json::to_vec(&remove_req).unwrap().as_slice(),
                &ctx(&narrow_budget, &caps),
            )
            .is_ok());
    }

    #[test]
    fn dimension_mismatch_on_insert_is_a_typed_error() {
        let mut a = adapter("dim-mismatch");
        let budget = HubResourceBudget::V0_CEILING;
        let caps = HubCapabilitySet::empty();
        a.handle(
            &op("vector.index.create"),
            br#"{"index":"docs","dim":8,"bit_width":4}"#,
            &ctx(&budget, &caps),
        )
        .unwrap();
        let insert_req = serde_json::json!({
            "index": "docs",
            "vectors": [[1.0, 2.0, 3.0]],
            "ids": [1],
        });
        let err = a
            .handle(
                &op("vector.index.insert"),
                serde_json::to_vec(&insert_req).unwrap().as_slice(),
                &ctx(&budget, &caps),
            )
            .unwrap_err();
        assert_eq!(err.code, "VectorDimensionMismatch");
    }

    #[test]
    fn search_rejects_a_query_batch_whose_total_result_count_exceeds_the_budget() {
        // Regression test: only k was checked against max_results, never
        // queries.len() * k -- a multi-query batch could request a total
        // result count far beyond the budget while each individual k
        // stayed under it, since turbovec allocates score/id buffers
        // proportional to the total across all queries, not per query.
        let mut a = adapter("total-results-budget");
        let caps = HubCapabilitySet::empty();
        a.handle(
            &op("vector.index.create"),
            br#"{"index":"docs","dim":8,"bit_width":4}"#,
            &ctx(&HubResourceBudget::V0_CEILING, &caps),
        )
        .unwrap();

        let narrow_budget = HubResourceBudget {
            result_count: 10,
            ..HubResourceBudget::V0_CEILING
        };
        // 3 queries * k=5 = 15 total results, exceeding the budget of 10
        // even though k=5 alone is well under it.
        let search_req = serde_json::json!({
            "index": "docs",
            "queries": [
                [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            ],
            "k": 5,
        });
        let err = a
            .handle(
                &op("vector.search"),
                serde_json::to_vec(&search_req).unwrap().as_slice(),
                &ctx(&narrow_budget, &caps),
            )
            .unwrap_err();
        assert_eq!(err.code, "TooManyResults");
    }

    #[test]
    fn search_accepts_a_query_batch_whose_total_result_count_is_within_the_budget() {
        let mut a = adapter("total-results-budget-ok");
        let caps = HubCapabilitySet::empty();
        a.handle(
            &op("vector.index.create"),
            br#"{"index":"docs","dim":8,"bit_width":4}"#,
            &ctx(&HubResourceBudget::V0_CEILING, &caps),
        )
        .unwrap();
        let insert_req = serde_json::json!({
            "index": "docs",
            "vectors": [[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]],
            "ids": [1],
        });
        a.handle(
            &op("vector.index.insert"),
            serde_json::to_vec(&insert_req).unwrap().as_slice(),
            &ctx(&HubResourceBudget::V0_CEILING, &caps),
        )
        .unwrap();

        let narrow_budget = HubResourceBudget {
            result_count: 10,
            ..HubResourceBudget::V0_CEILING
        };
        // 2 queries * k=5 = 10 total results, exactly at the budget.
        let search_req = serde_json::json!({
            "index": "docs",
            "queries": [
                [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            ],
            "k": 5,
        });
        assert!(a
            .handle(
                &op("vector.search"),
                serde_json::to_vec(&search_req).unwrap().as_slice(),
                &ctx(&narrow_budget, &caps),
            )
            .is_ok());
    }

    #[test]
    fn describing_a_missing_index_is_a_typed_error_not_a_panic() {
        let mut a = adapter("missing-index");
        let budget = HubResourceBudget::V0_CEILING;
        let caps = HubCapabilitySet::empty();
        let err = a
            .handle(
                &op("vector.index.describe"),
                br#"{"index":"nonexistent"}"#,
                &ctx(&budget, &caps),
            )
            .unwrap_err();
        assert_eq!(err.code, "IndexLoadFailed");
    }

    #[test]
    fn reset_replaces_index_with_empty_index_of_same_shape() {
        let mut a = adapter("reset");
        let budget = HubResourceBudget::V0_CEILING;
        let caps = HubCapabilitySet::empty();
        a.handle(
            &op("vector.index.create"),
            br#"{"index":"docs","dim":8,"bit_width":4}"#,
            &ctx(&budget, &caps),
        )
        .unwrap();
        let insert_req = serde_json::json!({
            "index": "docs",
            "vectors": unit_vectors(2, 8),
            "ids": [1, 2],
        });
        a.handle(
            &op("vector.index.insert"),
            serde_json::to_vec(&insert_req).unwrap().as_slice(),
            &ctx(&budget, &caps),
        )
        .unwrap();

        let reset = a
            .handle(
                &op("vector.index.reset"),
                br#"{"index":"docs"}"#,
                &ctx(&budget, &caps),
            )
            .unwrap();
        let reset: ResetReply = serde_json::from_slice(&reset.payload).unwrap();
        assert_eq!(reset.dim, 8);

        let describe = a
            .handle(
                &op("vector.index.describe"),
                br#"{"index":"docs"}"#,
                &ctx(&budget, &caps),
            )
            .unwrap();
        let describe: DescribeIndexReply = serde_json::from_slice(&describe.payload).unwrap();
        assert_eq!(describe.len, 0);
    }

    #[test]
    fn unknown_operation_is_a_typed_error() {
        let mut a = adapter("unknown-op");
        let budget = HubResourceBudget::V0_CEILING;
        let caps = HubCapabilitySet::empty();
        let err = a
            .handle(&op("vector.unknown"), b"{}", &ctx(&budget, &caps))
            .unwrap_err();
        assert_eq!(err.code, "UnknownOperation");
    }

    #[test]
    fn descriptor_validates_against_current_hub_api() {
        let d = descriptor(HubResourceBudget::V0_CEILING);
        assert!(d.validate(semantic_hub::HubApiVersion::CURRENT).is_ok());
    }
}
