//! Controlled read-only cross-crate bridge for Shell Player integration.
//!
//! This is the first public bridge surface between `prom-ui` and
//! `prom-ui-runtime`, implementing the prepared cross-crate handoff frozen
//! in `docs/spec/ui/shell_player_session_state_v0.md` section 7.1.9.
//!
//! Every prepared-evidence type (`ProjectionPatch`, `ProjectionPatchSet`,
//! `PreparedProjectionPatchTargets`, `PreparedActiveProjectionTargets`,
//! `LocalProjectionState`) remains entirely crate-private to `prom-ui` and
//! is never named here. This module exposes only:
//!
//! - controlled *ingress*: admitting a raw patch batch expressed in
//!   bridge-safe raw types, producing an opaque [`AdmittedProjectionPatchBatch`];
//! - controlled *read-only egress*: the bounded evidence the frozen contract
//!   permits `prom-ui-runtime` to consume — raw target-reference
//!   coordinates, the actual manifest count, declared activation-target
//!   coordinates, and local projected values — always as freshly computed
//!   plain data, never as a handle into the opaque evidence objects
//!   themselves.
//!
//! Nothing in this module lets a caller construct prepared evidence from
//! raw parts, `Default`, a raw `Vec`, `From`/`Into`, or deserialization:
//! every value returned here is computed by calling back into this crate's
//! existing deterministic producers ([`derive_prepared_projection_patch_targets`],
//! [`derive_prepared_active_projection_targets`], [`apply_projection_patch_set`]).
//! The runtime catalog these snapshots feed is not constructed here — it
//! remains `prom-ui-runtime`'s own responsibility (contract section
//! 7.1.9.6).

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::binding_graph::{
    BindingDeclaration, BindingGraphDocument, BindingId, BindingSlot, BindingTarget,
    BindingValueDomain,
};
use crate::collection_anchor_declarations::qualify_collection_anchor_declarations;
use crate::contract_primitives::{
    ChildOrder, CollectionKey, ContractVersion, Epoch, ProjectionSourceNodeId,
    ProjectionSourceSurfaceId, Revision, SchemaVersion, SourceId, SourceRef, SourceSpan,
    StaticDocumentId, StaticNodeId,
};
use crate::local_projection_state::{
    apply_projection_patch_set, LocalProjectionApplicationError, LocalProjectionState,
};
use crate::prepared_active_projection_targets::derive_prepared_active_projection_targets;
use crate::prepared_projection_patch_targets::derive_prepared_projection_patch_targets;
use crate::projection_compile::compile_projection_source_to_static_ir;
use crate::projection_patch::{
    ProjectionNodeAvailability, ProjectionPatch, ProjectionPatchEnvelope, ProjectionPatchId,
    ProjectionPatchOperation, ProjectionPatchSequence, ProjectionPatchSet, ProjectionPatchStreamId,
    ProjectionPatchValue, ProjectionQuadState,
};
use crate::projection_source::{
    ProjectionSourceChild, ProjectionSourceCollectionAnchorDeclaration, ProjectionSourceDocument,
    ProjectionSourceNode, ProjectionSourceRoleName, ProjectionSourceSurface,
};
use crate::role_dictionary::RoleDictionary;
use crate::target_anchor::TargetAnchor;

// ---- Bridge-safe raw value types ------------------------------------------

/// Bridge-safe mirror of `ProjectionQuadState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeQuad {
    N,
    F,
    T,
    S,
}

/// Bridge-safe mirror of `ProjectionPatchValue`.
#[derive(Debug, Clone, PartialEq)]
pub enum BridgeValue {
    Quad(BridgeQuad),
    Signed(i64),
    Unsigned(u64),
    Text(String),
}

/// Bridge-safe mirror of `ProjectionNodeAvailability`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeAvailability {
    Available,
    Unavailable,
    Hidden,
    Pending,
    Stale,
}

fn build_value(value: BridgeValue) -> ProjectionPatchValue {
    match value {
        BridgeValue::Quad(q) => ProjectionPatchValue::Quad(match q {
            BridgeQuad::N => ProjectionQuadState::N,
            BridgeQuad::F => ProjectionQuadState::F,
            BridgeQuad::T => ProjectionQuadState::T,
            BridgeQuad::S => ProjectionQuadState::S,
        }),
        BridgeValue::Signed(v) => ProjectionPatchValue::SignedScalar(v),
        BridgeValue::Unsigned(v) => ProjectionPatchValue::UnsignedScalar(v),
        BridgeValue::Text(v) => ProjectionPatchValue::Text(v),
    }
}

fn read_value(value: &ProjectionPatchValue) -> BridgeValue {
    match value {
        ProjectionPatchValue::Quad(q) => BridgeValue::Quad(match q {
            ProjectionQuadState::N => BridgeQuad::N,
            ProjectionQuadState::F => BridgeQuad::F,
            ProjectionQuadState::T => BridgeQuad::T,
            ProjectionQuadState::S => BridgeQuad::S,
        }),
        ProjectionPatchValue::SignedScalar(v) => BridgeValue::Signed(*v),
        ProjectionPatchValue::UnsignedScalar(v) => BridgeValue::Unsigned(*v),
        ProjectionPatchValue::Text(v) => BridgeValue::Text(v.clone()),
    }
}

fn read_availability(availability: ProjectionNodeAvailability) -> BridgeAvailability {
    match availability {
        ProjectionNodeAvailability::Available => BridgeAvailability::Available,
        ProjectionNodeAvailability::Unavailable => BridgeAvailability::Unavailable,
        ProjectionNodeAvailability::Hidden => BridgeAvailability::Hidden,
        ProjectionNodeAvailability::Pending => BridgeAvailability::Pending,
        ProjectionNodeAvailability::Stale => BridgeAvailability::Stale,
    }
}

fn build_availability(availability: BridgeAvailability) -> ProjectionNodeAvailability {
    match availability {
        BridgeAvailability::Available => ProjectionNodeAvailability::Available,
        BridgeAvailability::Unavailable => ProjectionNodeAvailability::Unavailable,
        BridgeAvailability::Hidden => ProjectionNodeAvailability::Hidden,
        BridgeAvailability::Pending => ProjectionNodeAvailability::Pending,
        BridgeAvailability::Stale => ProjectionNodeAvailability::Stale,
    }
}

// ---- Patch admission (ingress) --------------------------------------------

/// Bridge-safe patch operation input. Mirrors `ProjectionPatchOperation`
/// using only raw, publicly constructible field types. This is real patch
/// content supplied by a transition caller; it is not prepared evidence.
#[derive(Debug, Clone, PartialEq)]
pub enum BridgePatchOperation {
    SetBindingValue {
        node: u64,
        slot: u32,
        value: BridgeValue,
    },
    SetNodeAvailability {
        node: u64,
        availability: BridgeAvailability,
    },
    CollectionInsert {
        collection: u64,
        key: u64,
        before: Option<u64>,
        value: BridgeValue,
    },
    CollectionUpdate {
        collection: u64,
        key: u64,
        value: BridgeValue,
    },
    CollectionRemove {
        collection: u64,
        key: u64,
    },
    CollectionMove {
        collection: u64,
        key: u64,
        before: Option<u64>,
    },
}

/// Bridge-safe patch envelope input. Mirrors `ProjectionPatchEnvelope`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgePatchEnvelope {
    pub patch_id: u64,
    pub stream_id: u64,
    pub document_id: u64,
    pub previous_revision: u64,
    pub revision: u64,
    pub epoch: u64,
    pub sequence: u64,
}

/// Admission failure. The complete input is rejected; no partial batch is
/// ever produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeAdmissionError {
    InvalidIdentity,
    InvalidPatch,
}

/// Opaque admitted ordered patch batch. Wraps a validated, admitted private
/// `ProjectionPatchSet`. Opaque outside `prom-ui`: no field access and no
/// public constructor other than [`admit_projection_patch_batch`].
#[derive(Debug, Clone)]
pub struct AdmittedProjectionPatchBatch {
    inner: ProjectionPatchSet,
}

fn raw_node(raw: u64) -> Result<StaticNodeId, BridgeAdmissionError> {
    StaticNodeId::new(raw).map_err(|_| BridgeAdmissionError::InvalidIdentity)
}

fn raw_key(raw: u64) -> Result<CollectionKey, BridgeAdmissionError> {
    CollectionKey::new(raw).map_err(|_| BridgeAdmissionError::InvalidIdentity)
}

fn build_operation(
    op: BridgePatchOperation,
) -> Result<ProjectionPatchOperation, BridgeAdmissionError> {
    Ok(match op {
        BridgePatchOperation::SetBindingValue { node, slot, value } => {
            ProjectionPatchOperation::SetBindingValue {
                target: BindingTarget {
                    node: raw_node(node)?,
                    slot: BindingSlot(slot),
                },
                value: build_value(value),
            }
        }
        BridgePatchOperation::SetNodeAvailability { node, availability } => {
            ProjectionPatchOperation::SetNodeAvailability {
                node: raw_node(node)?,
                availability: build_availability(availability),
            }
        }
        BridgePatchOperation::CollectionInsert {
            collection,
            key,
            before,
            value,
        } => ProjectionPatchOperation::CollectionInsert {
            collection: raw_node(collection)?,
            key: raw_key(key)?,
            before: before.map(raw_key).transpose()?,
            value: build_value(value),
        },
        BridgePatchOperation::CollectionUpdate {
            collection,
            key,
            value,
        } => ProjectionPatchOperation::CollectionUpdate {
            collection: raw_node(collection)?,
            key: raw_key(key)?,
            value: build_value(value),
        },
        BridgePatchOperation::CollectionRemove { collection, key } => {
            ProjectionPatchOperation::CollectionRemove {
                collection: raw_node(collection)?,
                key: raw_key(key)?,
            }
        }
        BridgePatchOperation::CollectionMove {
            collection,
            key,
            before,
        } => ProjectionPatchOperation::CollectionMove {
            collection: raw_node(collection)?,
            key: raw_key(key)?,
            before: before.map(raw_key).transpose()?,
        },
    })
}

fn build_envelope(
    input: BridgePatchEnvelope,
) -> Result<ProjectionPatchEnvelope, BridgeAdmissionError> {
    let patch_id = ProjectionPatchId::new(input.patch_id)
        .map_err(|_| BridgeAdmissionError::InvalidIdentity)?;
    let stream_id = ProjectionPatchStreamId::new(input.stream_id)
        .map_err(|_| BridgeAdmissionError::InvalidIdentity)?;
    let document_id = StaticDocumentId::new(input.document_id)
        .map_err(|_| BridgeAdmissionError::InvalidIdentity)?;
    Ok(ProjectionPatchEnvelope::new(
        patch_id,
        stream_id,
        document_id,
        None,
        Revision::new(input.previous_revision),
        Revision::new(input.revision),
        Epoch::new(input.epoch),
        ProjectionPatchSequence::new(input.sequence),
    ))
}

/// Admits one ordered patch batch (currently exactly one `ProjectionPatch`)
/// from bridge-safe raw input, running the complete existing crate-private
/// `ProjectionPatch`/`ProjectionPatchSet` validation. Rejects the whole
/// input on any structural or identity failure; no partial batch escapes.
pub fn admit_projection_patch_batch(
    envelope: BridgePatchEnvelope,
    operations: Vec<BridgePatchOperation>,
) -> Result<AdmittedProjectionPatchBatch, BridgeAdmissionError> {
    let envelope = build_envelope(envelope)?;
    let operations = operations
        .into_iter()
        .map(build_operation)
        .collect::<Result<Vec<_>, _>>()?;
    let patch = ProjectionPatch::new(envelope, operations)
        .map_err(|_| BridgeAdmissionError::InvalidPatch)?;
    let inner =
        ProjectionPatchSet::new(vec![patch]).map_err(|_| BridgeAdmissionError::InvalidPatch)?;
    Ok(AdmittedProjectionPatchBatch { inner })
}

// ---- Prepared transition-target evidence (read-only egress) ---------------

/// Bridge-safe target role for one manifest entry. Mirrors [`TargetAnchor`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeTargetRole {
    Node { node: u64 },
    Binding { node: u64, slot: u32 },
    Collection { collection: u64 },
}

/// One read-only stable-target manifest entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgeManifestEntry {
    pub patch_ordinal: u64,
    pub operation_ordinal: u64,
    pub role: BridgeTargetRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeManifestError {
    ReplayIndexOverflow,
}

/// One coherent submission: an admitted patch batch bound atomically to the
/// prepared stable-target manifest, actual target-reference count, and
/// actual patch count derived from that exact batch.
///
/// Opaque outside `prom-ui`: no field access, and the only public
/// constructor is [`prepare_patch_submission`], which consumes the admitted
/// batch by value and derives every other field from that same value in the
/// same call. There is no accessor that returns the inner
/// `AdmittedProjectionPatchBatch` separately, and no constructor that
/// accepts a manifest and a batch as independent parameters — a manifest
/// derived from one batch can never be paired with a different batch's
/// operations, because the two are never representable apart from each
/// other after this point.
#[derive(Debug, Clone)]
pub struct PreparedPatchSubmission {
    batch: AdmittedProjectionPatchBatch,
    entries: Vec<BridgeManifestEntry>,
    target_reference_count: usize,
    patch_count: usize,
}

/// Deterministically derives the complete [`PreparedPatchSubmission`] for
/// one admitted patch batch, via [`derive_prepared_projection_patch_targets`].
pub fn prepare_patch_submission(
    batch: AdmittedProjectionPatchBatch,
) -> Result<PreparedPatchSubmission, BridgeManifestError> {
    let prepared = derive_prepared_projection_patch_targets(&batch.inner)
        .map_err(|_| BridgeManifestError::ReplayIndexOverflow)?;

    let entries = prepared
        .entries()
        .iter()
        .map(|entry| {
            let coordinate = entry.coordinate();
            let role = match entry.anchor() {
                TargetAnchor::Node(anchor) => BridgeTargetRole::Node {
                    node: anchor.node().raw(),
                },
                TargetAnchor::Binding(anchor) => BridgeTargetRole::Binding {
                    node: anchor.node().raw(),
                    slot: anchor.slot().0,
                },
                TargetAnchor::Collection(anchor) => BridgeTargetRole::Collection {
                    collection: anchor.collection().raw(),
                },
            };
            BridgeManifestEntry {
                patch_ordinal: coordinate.patch_ordinal(),
                operation_ordinal: coordinate.operation_ordinal(),
                role,
            }
        })
        .collect();

    let target_reference_count = prepared.target_reference_count();
    let patch_count = submission_patch_count(&batch);

    Ok(PreparedPatchSubmission {
        batch,
        entries,
        target_reference_count,
        patch_count,
    })
}

fn submission_patch_count(batch: &AdmittedProjectionPatchBatch) -> usize {
    batch.inner.patches().len()
}

/// The ordered stable-target manifest entries derived from `submission`'s
/// own admitted batch.
pub fn prepared_submission_entries(submission: &PreparedPatchSubmission) -> &[BridgeManifestEntry] {
    &submission.entries
}

/// The actual target-reference count derived from `submission`'s own
/// admitted batch.
pub fn prepared_submission_target_reference_count(submission: &PreparedPatchSubmission) -> usize {
    submission.target_reference_count
}

/// The actual patch count derived from `submission`'s own admitted batch.
pub fn prepared_submission_patch_count(submission: &PreparedPatchSubmission) -> usize {
    submission.patch_count
}

// ---- Prepared activation-target evidence (read-only egress) ---------------

/// Freshly computed, read-only snapshot of `PreparedActiveProjectionTargets`.
/// `prom-ui-runtime` builds its own `ActiveProjectionTargetCatalog` from
/// this snapshot exactly once, at activation (contract section 7.1.9.6);
/// this snapshot itself is not the runtime catalog.
///
/// Opaque outside `prom-ui`: fields are private, there is no `Default`, no
/// public raw-parts constructor, and no `From`/`Into` path. The only
/// constructor in this module is [`demo_activation_snapshot`], which derives
/// every id from a validated projection-owned structural fixture via
/// [`derive_prepared_active_projection_targets`] — never from caller-supplied
/// raw ids. Cross-crate consumers may only read the declared ids back out
/// through the accessor methods below; they cannot construct or mutate a
/// snapshot, so a runtime catalog built from one is never fabricable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivationTargetSnapshot {
    node_anchor_ids: Vec<u64>,
    binding_anchor_ids: Vec<(u64, u32)>,
    collection_anchor_ids: Vec<u64>,
}

impl ActivationTargetSnapshot {
    /// Declared `NodeAnchor` ids, in deterministic ascending order.
    pub fn node_anchor_ids(&self) -> &[u64] {
        &self.node_anchor_ids
    }

    /// Declared `BindingAnchor` `(node, slot)` ids, in deterministic
    /// ascending order.
    pub fn binding_anchor_ids(&self) -> &[(u64, u32)] {
        &self.binding_anchor_ids
    }

    /// Declared `CollectionAnchor` ids, in deterministic ascending order.
    /// Sourced only from explicit `CollectionAnchor` declarations.
    pub fn collection_anchor_ids(&self) -> &[u64] {
        &self.collection_anchor_ids
    }
}

/// Returns a small, fixed, deterministic projection-owned structural
/// fixture (one root, a text-binding node, an availability-toggle node, and
/// one explicitly declared `CollectionAnchor` list node), and its derived
/// [`ActivationTargetSnapshot`].
///
/// This is a minimal built-in reference fixture for demonstrating Shell
/// Player activation end-to-end. It is not general `ProjectionBundle`
/// parsing or loading, which remain separately unauthorized.
pub fn demo_activation_snapshot() -> ActivationTargetSnapshot {
    let doc_id = StaticDocumentId::new(1).expect("demo document id");
    let root_source = ProjectionSourceNodeId::new(1).expect("demo root id");
    let label_source = ProjectionSourceNodeId::new(2).expect("demo label id");
    let status_source = ProjectionSourceNodeId::new(3).expect("demo status id");
    let list_source = ProjectionSourceNodeId::new(4).expect("demo list id");

    let span = SourceRef::new(
        SourceId::new(1).expect("demo source id"),
        SourceSpan::new(0, 1).expect("demo source span"),
    );

    let mut root = ProjectionSourceNode::new(
        root_source,
        ProjectionSourceRoleName::new("root"),
        CollectionKey::new(1).expect("demo root key"),
        span,
    );
    root.push_child(ProjectionSourceChild::new(label_source, ChildOrder::new(0)));
    root.push_child(ProjectionSourceChild::new(
        status_source,
        ChildOrder::new(1),
    ));
    root.push_child(ProjectionSourceChild::new(list_source, ChildOrder::new(2)));

    let mut source = ProjectionSourceDocument::new(
        SourceId::new(1).expect("demo source id"),
        Revision::new(1),
        Epoch::new(1),
    );
    source.push_node(root);
    source.push_node(ProjectionSourceNode::new(
        label_source,
        ProjectionSourceRoleName::new("text"),
        CollectionKey::new(2).expect("demo label key"),
        span,
    ));
    source.push_node(ProjectionSourceNode::new(
        status_source,
        ProjectionSourceRoleName::new("text"),
        CollectionKey::new(3).expect("demo status key"),
        span,
    ));
    source.push_node(ProjectionSourceNode::new(
        list_source,
        ProjectionSourceRoleName::new("evidence_panel"),
        CollectionKey::new(4).expect("demo list key"),
        span,
    ));
    source.push_surface(ProjectionSourceSurface::new(
        ProjectionSourceSurfaceId::new(1).expect("demo surface id"),
        root_source,
        CollectionKey::new(5).expect("demo surface key"),
        span,
    ));
    source.push_collection_anchor_declaration(ProjectionSourceCollectionAnchorDeclaration::new(
        list_source,
        span,
    ));

    let static_document =
        compile_projection_source_to_static_ir(doc_id, &source, RoleDictionary::current())
            .expect("demo fixture compiles");

    let collection_anchors =
        qualify_collection_anchor_declarations(doc_id, &source, &static_document)
            .expect("demo fixture qualifies");

    let bindings = vec![BindingDeclaration {
        id: BindingId(1),
        target: BindingTarget {
            node: raw_node(2).expect("demo label node"),
            slot: BindingSlot(0),
        },
        domain: BindingValueDomain::Text,
        dependencies: Vec::new(),
        collection_key: None,
        semantic_value: None,
        source_ref: None,
    }];
    let binding_graph = BindingGraphDocument::build(bindings, &static_document)
        .expect("demo binding graph is valid");

    let prepared = derive_prepared_active_projection_targets(
        &static_document,
        &binding_graph,
        &collection_anchors,
    )
    .expect("demo activation evidence derives");

    ActivationTargetSnapshot {
        node_anchor_ids: prepared
            .node_anchors()
            .iter()
            .map(|a| a.node().raw())
            .collect(),
        binding_anchor_ids: prepared
            .binding_anchors()
            .iter()
            .map(|a| (a.node().raw(), a.slot().0))
            .collect(),
        collection_anchor_ids: prepared
            .collection_anchors()
            .iter()
            .map(|a| a.collection().raw())
            .collect(),
    }
}

// ---- Local projection state (read-only egress + application) -------------

/// Opaque handle to committed local projected-value state for one Shell
/// Player session. Opaque outside `prom-ui`: no field access and no public
/// constructor other than [`initial_local_projection_state`] and
/// [`apply_prepared_patch_submission`].
#[derive(Debug, Clone, PartialEq)]
pub struct LocalProjectionStateHandle {
    inner: LocalProjectionState,
}

/// The canonical zero local projection state for a freshly activated
/// session that has not yet committed any patch.
pub fn initial_local_projection_state() -> LocalProjectionStateHandle {
    LocalProjectionStateHandle {
        inner: LocalProjectionState::empty(),
    }
}

/// Bridge-safe mirror of [`LocalProjectionApplicationError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeApplicationError {
    KeyAlreadyPresent { collection: u64, key: u64 },
    KeyMissing { collection: u64, key: u64 },
    BeforeKeyMissing { collection: u64, before: u64 },
}

fn map_application_error(error: LocalProjectionApplicationError) -> BridgeApplicationError {
    match error {
        LocalProjectionApplicationError::KeyAlreadyPresent { collection, key } => {
            BridgeApplicationError::KeyAlreadyPresent {
                collection: collection.raw(),
                key: key.raw(),
            }
        }
        LocalProjectionApplicationError::KeyMissing { collection, key } => {
            BridgeApplicationError::KeyMissing {
                collection: collection.raw(),
                key: key.raw(),
            }
        }
        LocalProjectionApplicationError::BeforeKeyMissing { collection, before } => {
            BridgeApplicationError::BeforeKeyMissing {
                collection: collection.raw(),
                before: before.raw(),
            }
        }
    }
}

/// Deterministically computes the candidate [`LocalProjectionStateHandle`]
/// from applying every operation in `submission`'s own admitted batch to
/// `previous`, via [`apply_projection_patch_set`]. Atomic: `previous` is
/// never mutated, and this returns `Ok` only if the complete batch applies
/// successfully. The applied operations are always exactly the operations
/// `submission`'s manifest was derived from (contract section 7.1.9.7): this
/// function takes no separate batch parameter through which a different
/// batch's operations could be substituted.
pub fn apply_prepared_patch_submission(
    previous: &LocalProjectionStateHandle,
    submission: &PreparedPatchSubmission,
) -> Result<LocalProjectionStateHandle, BridgeApplicationError> {
    let inner = apply_projection_patch_set(&previous.inner, &submission.batch.inner)
        .map_err(map_application_error)?;
    Ok(LocalProjectionStateHandle { inner })
}

/// Reads the current binding value at `(node, slot)`, or `None` if unset.
pub fn binding_value(
    state: &LocalProjectionStateHandle,
    node: u64,
    slot: u32,
) -> Option<BridgeValue> {
    let node = StaticNodeId::new(node).ok()?;
    state
        .inner
        .binding_value(node, BindingSlot(slot))
        .map(read_value)
}

/// Reads the current availability of `node`, or `None` if unset.
pub fn node_availability(
    state: &LocalProjectionStateHandle,
    node: u64,
) -> Option<BridgeAvailability> {
    let node = StaticNodeId::new(node).ok()?;
    state.inner.node_availability(node).map(read_availability)
}

/// Reads the current ordered `(key, value)` entries of `collection`. Empty
/// if the collection has never been touched or `collection` is invalid.
pub fn collection_entries(
    state: &LocalProjectionStateHandle,
    collection: u64,
) -> Vec<(u64, BridgeValue)> {
    let Ok(collection) = StaticNodeId::new(collection) else {
        return Vec::new();
    };
    state
        .inner
        .collection_entries(collection)
        .iter()
        .map(|(key, value)| (key.raw(), read_value(value)))
        .collect()
}

// ---- Gate D bounded ProjectionBundle activation (Issue #1543) ------------
//
// Bounded to the UI-DNA2-10 reference contour only. This is not general
// `ProjectionBundle` activation, does not imply production promotion, and
// uses a fixed, non-caller-configurable policy (accepted bundle schema/
// contract version, `RoleDictionary::current()`, and resource limits sized
// for the bounded reference contour) -- see
// `docs/spec/ui/gate_d_activation_policy_v0.md` for the exact limits and
// exclusions.

/// One node's bridge-safe structural evidence from an activated bundle:
/// identity, role name, and ordered child identities. Read-only; not the
/// internal `StaticUiNode` type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeActivatedNode {
    id: u64,
    role: String,
    children: Vec<u64>,
}

impl BridgeActivatedNode {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub fn children(&self) -> &[u64] {
        &self.children
    }
}

/// Bridge-safe mirror of `projection_bundle::AccessibilityRole`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeAccessibilityRole {
    GenericContainer,
    Label,
    Button,
}

/// One node's deterministically-derived accessibility evidence from an
/// activated bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeAccessibilityEntry {
    node: u64,
    role: BridgeAccessibilityRole,
    label: String,
}

impl BridgeAccessibilityEntry {
    pub fn node(&self) -> u64 {
        self.node
    }

    pub fn role(&self) -> BridgeAccessibilityRole {
        self.role
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

/// Opaque bounded Gate D activation result. Wraps every artifact class from
/// one verified `ProjectionBundle v0`, reduced to the bridge-safe evidence
/// the UI-DNA2-10 reference contour consumes. Construction is possible only
/// through [`activate_projection_bundle_v0_gate_d`]; there is no public
/// constructor, `Default`, `From`/`Into`, or deserialization path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateDActivatedBundle {
    nodes: Vec<BridgeActivatedNode>,
    accessibility: Vec<BridgeAccessibilityEntry>,
    snapshot: ActivationTargetSnapshot,
    root_node: u64,
}

impl GateDActivatedBundle {
    /// Every node in the activated bundle's Static UI IR, ascending by id.
    pub fn nodes(&self) -> &[BridgeActivatedNode] {
        &self.nodes
    }

    /// Deterministically-derived accessibility evidence, ascending by node
    /// id.
    pub fn accessibility_entries(&self) -> &[BridgeAccessibilityEntry] {
        &self.accessibility
    }

    /// The Shell Player activation-target snapshot derived from this
    /// bundle's Binding Graph and qualified `CollectionAnchor`
    /// declarations. Pass this directly to
    /// `prom_ui_runtime::shell_player::create_shell_session`.
    pub fn activation_snapshot(&self) -> &ActivationTargetSnapshot {
        &self.snapshot
    }

    /// The first declared surface's root node id, or `0` if the bundle
    /// declares no surface (structurally impossible for a verified bundle,
    /// since Static UI IR structural validation requires a root; `0` is a
    /// defensive fallback only).
    pub fn root_node(&self) -> u64 {
        self.root_node
    }
}

/// Bounded Gate D activation failed. The complete bundle is rejected; no
/// partial `GateDActivatedBundle` ever escapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateDActivationError {
    BundleRejected,
}

fn gate_d_reference_contour_limits() -> crate::projection_bundle::ProjectionBundleLimits {
    crate::projection_bundle::ProjectionBundleLimits {
        max_bundle_bytes: 256 * 1024,
        max_binding_count: 256,
        max_action_route_count: 256,
        max_collection_anchor_count: 256,
        max_accessibility_entry_count: 512,
        static_ir_limits: crate::static_ir_artifact::StaticUiArtifactV1Limits {
            max_input_bytes: 256 * 1024,
            max_surfaces: 16,
            max_nodes: 512,
            max_children_per_node: 64,
            max_role_bytes: 64,
        },
    }
}

/// Verifies one candidate `ProjectionBundle v0` byte sequence (structural,
/// cross-artifact, compatibility, and self-consistency verification, per
/// `crate::projection_bundle::verify_projection_bundle_v0`) against a
/// fixed, bounded Gate D policy, and -- only if every stage succeeds --
/// derives the bridge-safe activation evidence the UI-DNA2-10 reference
/// application needs. Fails closed: a rejected bundle never produces a
/// `GateDActivatedBundle`, an `ActivationTargetSnapshot`, or any Shell
/// Player state.
///
/// This is bounded Gate D activation for the reference contour only. It
/// does not select a cryptographic trust algorithm (none exists in this
/// workspace), does not imply general `ProjectionBundle` production
/// readiness, and does not imply production promotion.
pub fn activate_projection_bundle_v0_gate_d(
    bytes: &[u8],
) -> Result<GateDActivatedBundle, GateDActivationError> {
    let dictionary = RoleDictionary::current();
    let compat = crate::projection_bundle::ProjectionBundleCompatContext {
        accepted_schema_version: SchemaVersion::CURRENT,
        accepted_contract_version: ContractVersion::CURRENT,
        role_dictionary: dictionary,
    };
    let verified = crate::projection_bundle::verify_projection_bundle_v0(
        bytes,
        compat,
        gate_d_reference_contour_limits(),
    )
    .map_err(|_| GateDActivationError::BundleRejected)?;
    let bundle = verified.bundle();

    let prepared = derive_prepared_active_projection_targets(
        bundle.static_document(),
        bundle.binding_graph(),
        bundle.collection_anchors(),
    )
    .map_err(|_| GateDActivationError::BundleRejected)?;

    let snapshot = ActivationTargetSnapshot {
        node_anchor_ids: prepared
            .node_anchors()
            .iter()
            .map(|a| a.node().raw())
            .collect(),
        binding_anchor_ids: prepared
            .binding_anchors()
            .iter()
            .map(|a| (a.node().raw(), a.slot().0))
            .collect(),
        collection_anchor_ids: prepared
            .collection_anchors()
            .iter()
            .map(|a| a.collection().raw())
            .collect(),
    };

    let nodes = bundle
        .static_document()
        .nodes()
        .iter()
        .map(|node| BridgeActivatedNode {
            id: node.id().raw(),
            role: String::from(node.role().as_str()),
            children: node.children().iter().map(|c| c.child().raw()).collect(),
        })
        .collect();

    let accessibility = bundle
        .accessibility_entries()
        .iter()
        .map(|entry| BridgeAccessibilityEntry {
            node: entry.node().raw(),
            role: match entry.role() {
                crate::projection_bundle::AccessibilityRole::GenericContainer => {
                    BridgeAccessibilityRole::GenericContainer
                }
                crate::projection_bundle::AccessibilityRole::Label => {
                    BridgeAccessibilityRole::Label
                }
                crate::projection_bundle::AccessibilityRole::Button => {
                    BridgeAccessibilityRole::Button
                }
            },
            label: String::from(entry.label()),
        })
        .collect();

    let root_node = bundle
        .static_document()
        .surfaces()
        .first()
        .map(|s| s.root().raw())
        .unwrap_or(0);

    Ok(GateDActivatedBundle {
        nodes,
        accessibility,
        snapshot,
        root_node,
    })
}

// ---- ProjectionBundle v0 compilation from source text (Issue #1543) ------

/// Compilation failure. The complete input is rejected; no partial bundle
/// bytes are ever produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionBundleCompileError {
    ParseOrCompile,
    CollectionAnchorQualification,
    Canonicalization,
}

/// Compiles Projection Source text (Grammar v0, including the textual
/// `collection_anchor` declaration syntax) all the way through to canonical
/// `ProjectionBundle v0` bytes: parse, Static UI IR lowering, explicit
/// `CollectionAnchor` qualification, an empty (but present) Binding Graph
/// and Action IR section, deterministically-derived accessibility
/// evidence, and canonical serialization.
///
/// This is the public production-side counterpart to
/// [`activate_projection_bundle_v0_gate_d`], which consumes the bytes this
/// function produces. Together they realize the complete
/// `source text -> ... -> ProjectionBundle -> parse/validate/verify ->
/// inert load -> Gate D activation` pipeline through public API only.
pub fn compile_projection_source_to_bundle_v0(
    source_id: u64,
    source_text: &str,
    document_id: u64,
    bundle_revision: u64,
    bundle_epoch: u64,
    compiler_identity: &str,
) -> Result<Vec<u8>, ProjectionBundleCompileError> {
    let source_id = SourceId::new(source_id).map_err(|e| {
        println!("Parse Error: {:?}", e);
        ProjectionBundleCompileError::ParseOrCompile
    })?;
    let document_id = StaticDocumentId::new(document_id).map_err(|e| {
        println!("Parse Error: {:?}", e);
        ProjectionBundleCompileError::ParseOrCompile
    })?;
    let dictionary = RoleDictionary::current();

    let compiled = crate::projection_source_frontend::compile_projection_source_text_with_collection_anchors(
        source_id,
        source_text,
        document_id,
        dictionary,
    )
    .map_err(|err| match err {
        crate::projection_source_frontend::ProjectionSourceFrontendWithAnchorsError::CollectionAnchor(_) => {
            ProjectionBundleCompileError::CollectionAnchorQualification
        }
        _ => ProjectionBundleCompileError::ParseOrCompile,
    })?;

    let binding_graph = BindingGraphDocument::build(Vec::new(), compiled.static_document())
        .map_err(|e| {
            println!("Parse Error: {:?}", e);
            ProjectionBundleCompileError::ParseOrCompile
        })?;
    let action_ir =
        crate::action_ir::ActionIrDocument::build(Vec::new(), compiled.static_document()).map_err(
            |e| {
                println!("Parse Error: {:?}", e);
                ProjectionBundleCompileError::ParseOrCompile
            },
        )?;

    let bundle = crate::projection_bundle::ProjectionBundleV0::assemble(
        document_id,
        Revision::new(bundle_revision),
        Epoch::new(bundle_epoch),
        dictionary,
        compiled.static_document().clone(),
        binding_graph,
        action_ir,
        compiled.collection_anchors().clone(),
        crate::projection_bundle::BundleProvenance::new(
            String::from(compiler_identity),
            source_id,
            Revision::new(bundle_revision),
            Epoch::new(bundle_epoch),
        ),
    );

    bundle
        .canonical_bytes(dictionary)
        .map_err(|_| ProjectionBundleCompileError::Canonicalization)
}
