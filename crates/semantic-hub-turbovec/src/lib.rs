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

use std::path::PathBuf;

use semantic_hub::runtime::{HubTool, RestrictedHubContext};
use semantic_hub::{
    HubCapability, HubDeterminismClass, HubExecutionMode, HubOperationDescriptor, HubOperationId,
    HubResourceBudget, HubToolDescriptor, HubToolError, HubToolId, HubToolVersion, HubTrustClass,
};

use payload::{
    check_result_count_bounds, check_vector_batch_bounds, parse, to_bytes, CreateIndexReply,
    CreateIndexRequest, DescribeIndexReply, IndexNameOnlyRequest, InsertReply, InsertRequest,
    RemoveReply, RemoveRequest, ResetReply, ResetRequest, SearchHit, SearchReply, SearchRequest,
};
use storage::{IndexName, ScopedStorage, MAX_INDEX_COUNT};

pub const TOOL_ID: &str = "vector.turbovec";
pub const TURBOVEC_DEPENDENCY_VERSION: &str = "0.9.0";
pub const TURBOVEC_DEPENDENCY_LICENSE: &str = "MIT";
pub const TURBOVEC_DEPENDENCY_SOURCE: &str = "https://github.com/RyanCodrai/turbovec";
pub const ADAPTER_VERSION: &str = env!("CARGO_PKG_VERSION");

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
                    HubCapability::PrivateStorageWrite,
                ],
                HubDeterminismClass::Deterministic,
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
    fn load(&self, name: &IndexName, max_dim: usize) -> Result<turbovec::IdMapIndex, HubToolError> {
        let path = self
            .storage
            .checked_index_path(name)
            .map_err(|e| HubToolError::new("ScopedStorageViolation", e.to_string()))?;
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

    fn save_atomic(
        &self,
        name: &IndexName,
        index: &turbovec::IdMapIndex,
    ) -> Result<(), HubToolError> {
        self.storage.ensure_root_checked().map_err(|e| {
            HubToolError::new(
                "StorageUnavailable",
                format!("could not prepare scoped storage directory: {e}"),
            )
        })?;
        let final_path = self
            .storage
            .checked_index_path(name)
            .map_err(|e| HubToolError::new("ScopedStorageViolation", e.to_string()))?;
        let tmp_path = final_path.with_extension(format!(
            "tvim.tmp.{}.{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        index.write(&tmp_path).map_err(|e| {
            HubToolError::new("IndexWriteFailed", format!("could not write index: {e}"))
        })?;
        std::fs::rename(&tmp_path, &final_path).map_err(|e| {
            let _ = std::fs::remove_file(&tmp_path);
            HubToolError::new(
                "IndexWriteFailed",
                format!("could not finalize index write: {e}"),
            )
        })
    }

    fn index_name(raw: &str) -> Result<IndexName, HubToolError> {
        IndexName::new(raw).map_err(|e| HubToolError::new("InvalidIndexName", e.to_string()))
    }

    fn handle_create(&self, payload: &[u8], max_dim: usize) -> Result<Vec<u8>, HubToolError> {
        let req: CreateIndexRequest =
            parse(payload).map_err(|e| HubToolError::new("MalformedInput", e.to_string()))?;
        let name = Self::index_name(&req.index)?;
        if self.storage.exists(&name) {
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
        let index = turbovec::IdMapIndex::new(req.dim, req.bit_width)
            .map_err(|e| HubToolError::new("InvalidIndexParameters", e.to_string()))?;
        self.save_atomic(&name, &index)?;
        Ok(to_bytes(&CreateIndexReply {
            index: name.as_str().to_string(),
            dim: req.dim,
            bit_width: req.bit_width,
        }))
    }

    fn handle_describe(&self, payload: &[u8], max_dim: usize) -> Result<Vec<u8>, HubToolError> {
        let req: IndexNameOnlyRequest =
            parse(payload).map_err(|e| HubToolError::new("MalformedInput", e.to_string()))?;
        let name = Self::index_name(&req.index)?;
        let index = self.load(&name, max_dim)?;
        Ok(to_bytes(&DescribeIndexReply {
            index: name.as_str().to_string(),
            dim: index.dim(),
            bit_width: index.bit_width(),
            len: index.len(),
        }))
    }

    fn handle_insert(
        &self,
        payload: &[u8],
        max_vectors: usize,
        max_dim: usize,
    ) -> Result<Vec<u8>, HubToolError> {
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
        let mut index = self.load(&name, max_dim)?;
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
        let flat: Vec<f32> = req.vectors.iter().flatten().copied().collect();
        index
            .add_with_ids_2d(&flat, dim, &req.ids)
            .map_err(|e| HubToolError::new("InsertRejected", e.to_string()))?;
        let inserted = req.ids.len();
        let len = index.len();
        self.save_atomic(&name, &index)?;
        Ok(to_bytes(&InsertReply {
            index: name.as_str().to_string(),
            inserted,
            len,
        }))
    }

    fn handle_remove(&self, payload: &[u8], max_dim: usize) -> Result<Vec<u8>, HubToolError> {
        let req: RemoveRequest =
            parse(payload).map_err(|e| HubToolError::new("MalformedInput", e.to_string()))?;
        let name = Self::index_name(&req.index)?;
        let mut index = self.load(&name, max_dim)?;
        let mut removed = Vec::new();
        let mut not_found = Vec::new();
        for id in req.ids {
            if index.remove(id) {
                removed.push(id);
            } else {
                not_found.push(id);
            }
        }
        let len = index.len();
        self.save_atomic(&name, &index)?;
        Ok(to_bytes(&RemoveReply {
            index: name.as_str().to_string(),
            removed,
            not_found,
            len,
        }))
    }

    fn handle_search(
        &self,
        payload: &[u8],
        max_results: usize,
        filtered: bool,
        max_dim: usize,
    ) -> Result<Vec<u8>, HubToolError> {
        let req: SearchRequest =
            parse(payload).map_err(|e| HubToolError::new("MalformedInput", e.to_string()))?;
        let name = Self::index_name(&req.index)?;
        check_result_count_bounds(req.k, max_results)
            .map_err(|e| HubToolError::new("TooManyResults", e.to_string()))?;
        if req.queries.is_empty() {
            return Err(HubToolError::new("EmptyQuery", "queries must not be empty"));
        }
        let index = self.load(&name, max_dim)?;
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

        let allowed_ids = if filtered {
            let ids = req.allowed_ids.clone().unwrap_or_default();
            if ids.is_empty() {
                return Err(HubToolError::new(
                    "EmptyFilter",
                    "vector.search.filtered requires a non-empty allowed_ids list",
                ));
            }
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
            None
        };

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
        Ok(to_bytes(&SearchReply {
            index: name.as_str().to_string(),
            index_version: 1,
            hits,
        }))
    }

    fn handle_reset(&self, payload: &[u8], max_dim: usize) -> Result<Vec<u8>, HubToolError> {
        let req: ResetRequest =
            parse(payload).map_err(|e| HubToolError::new("MalformedInput", e.to_string()))?;
        let name = Self::index_name(&req.index)?;
        let existing = self.load(&name, max_dim)?;
        let (dim, bit_width) = (existing.dim(), existing.bit_width());
        // turbovec has no in-place clear/truncate: a v0 "reset" is
        // constructing a fresh empty index with the same shape and
        // overwriting the persisted file, documented explicitly rather
        // than inventing an unsupported private truncation format.
        let fresh = turbovec::IdMapIndex::new(dim, bit_width)
            .map_err(|e| HubToolError::new("InvalidIndexParameters", e.to_string()))?;
        self.save_atomic(&name, &fresh)?;
        Ok(to_bytes(&ResetReply {
            index: name.as_str().to_string(),
            dim,
            bit_width,
        }))
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
    ) -> Result<Vec<u8>, HubToolError> {
        let max_vectors = context.resource_budget.index_item_count as usize;
        let max_results = context.resource_budget.result_count as usize;
        let max_dim = context.resource_budget.vector_dimensions as usize;
        match operation_id.as_str() {
            "vector.index.create" => self.handle_create(payload, max_dim),
            "vector.index.describe" => self.handle_describe(payload, max_dim),
            "vector.index.insert" => self.handle_insert(payload, max_vectors, max_dim),
            "vector.index.remove" => self.handle_remove(payload, max_dim),
            "vector.search" => self.handle_search(payload, max_results, false, max_dim),
            "vector.search.filtered" => self.handle_search(payload, max_results, true, max_dim),
            "vector.index.reset" => self.handle_reset(payload, max_dim),
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
        let create: CreateIndexReply = serde_json::from_slice(&create).unwrap();
        assert_eq!(create.dim, 8);

        let describe = a
            .handle(
                &op("vector.index.describe"),
                br#"{"index":"docs"}"#,
                &ctx(&budget, &caps),
            )
            .unwrap();
        let describe: DescribeIndexReply = serde_json::from_slice(&describe).unwrap();
        assert_eq!(describe.dim, 8);
        assert_eq!(describe.bit_width, 4);
        assert_eq!(describe.len, 0);
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
        let insert_reply: InsertReply = serde_json::from_slice(&insert_reply).unwrap();
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
        let search_reply: SearchReply = serde_json::from_slice(&search_reply).unwrap();
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
        let remove_reply: RemoveReply = serde_json::from_slice(&remove_reply).unwrap();
        assert_eq!(remove_reply.removed, vec![20]);
        assert_eq!(remove_reply.len, 3);

        let search_again = a
            .handle(
                &op("vector.search"),
                serde_json::to_vec(&search_req).unwrap().as_slice(),
                &ctx(&budget, &caps),
            )
            .unwrap();
        let search_again: SearchReply = serde_json::from_slice(&search_again).unwrap();
        assert!(!search_again.hits[0].iter().any(|h| h.external_id == 20));
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
        let reset: ResetReply = serde_json::from_slice(&reset).unwrap();
        assert_eq!(reset.dim, 8);

        let describe = a
            .handle(
                &op("vector.index.describe"),
                br#"{"index":"docs"}"#,
                &ctx(&budget, &caps),
            )
            .unwrap();
        let describe: DescribeIndexReply = serde_json::from_slice(&describe).unwrap();
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
