//! UI DNA v2 projection patch contract foundation.
//!
//! This module defines inert, deterministic projection patch values and
//! structural validation only. It does not own Semantic truth, admission,
//! dispatch, runtime execution, shell mutation, or renderer commands.
#![allow(dead_code)]

use alloc::string::String;
use alloc::vec::Vec;

use crate::binding_graph::BindingTarget;
use crate::contract_primitives::{
    CollectionKey, DiagnosticCode, DiagnosticCoordinate, Epoch, Revision, StaticDocumentId,
    StaticNodeId, StaticSurfaceId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ProjectionPatchId(u64);

impl ProjectionPatchId {
    pub(crate) const fn new(raw: u64) -> Result<Self, ProjectionPatchIdentityError> {
        if raw == 0 {
            Err(ProjectionPatchIdentityError::ZeroPatchId)
        } else {
            Ok(Self(raw))
        }
    }

    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ProjectionPatchStreamId(u64);

impl ProjectionPatchStreamId {
    pub(crate) const fn new(raw: u64) -> Result<Self, ProjectionPatchIdentityError> {
        if raw == 0 {
            Err(ProjectionPatchIdentityError::ZeroStreamId)
        } else {
            Ok(Self(raw))
        }
    }

    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ProjectionPatchSequence(u64);

impl ProjectionPatchSequence {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub(crate) const fn raw(self) -> u64 {
        self.0
    }

    const fn checked_next(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(next) => Some(Self(next)),
            None => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProjectionPatchIdentityError {
    ZeroPatchId,
    ZeroStreamId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ProjectionPatchEnvelope {
    patch_id: ProjectionPatchId,
    stream_id: ProjectionPatchStreamId,
    document_id: StaticDocumentId,
    surface_id: Option<StaticSurfaceId>,
    previous_projection_revision: Revision,
    projection_revision: Revision,
    epoch: Epoch,
    sequence: ProjectionPatchSequence,
}

impl ProjectionPatchEnvelope {
    pub(crate) const fn new(
        patch_id: ProjectionPatchId,
        stream_id: ProjectionPatchStreamId,
        document_id: StaticDocumentId,
        surface_id: Option<StaticSurfaceId>,
        previous_projection_revision: Revision,
        projection_revision: Revision,
        epoch: Epoch,
        sequence: ProjectionPatchSequence,
    ) -> Self {
        Self {
            patch_id,
            stream_id,
            document_id,
            surface_id,
            previous_projection_revision,
            projection_revision,
            epoch,
            sequence,
        }
    }

    pub(crate) const fn patch_id(self) -> ProjectionPatchId {
        self.patch_id
    }

    pub(crate) const fn stream_id(self) -> ProjectionPatchStreamId {
        self.stream_id
    }

    pub(crate) const fn document_id(self) -> StaticDocumentId {
        self.document_id
    }

    pub(crate) const fn surface_id(self) -> Option<StaticSurfaceId> {
        self.surface_id
    }

    pub(crate) const fn previous_projection_revision(self) -> Revision {
        self.previous_projection_revision
    }

    pub(crate) const fn projection_revision(self) -> Revision {
        self.projection_revision
    }

    pub(crate) const fn epoch(self) -> Epoch {
        self.epoch
    }

    pub(crate) const fn sequence(self) -> ProjectionPatchSequence {
        self.sequence
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ProjectionQuadState {
    N,
    F,
    T,
    S,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ProjectionPatchValue {
    Quad(ProjectionQuadState),
    SignedScalar(i64),
    Text(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ProjectionNodeAvailability {
    Available,
    Unavailable,
    Hidden,
    Pending,
    Stale,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ProjectionPatchOperation {
    SetBindingValue {
        target: BindingTarget,
        value: ProjectionPatchValue,
    },
    SetNodeAvailability {
        node: StaticNodeId,
        availability: ProjectionNodeAvailability,
    },
    CollectionInsert {
        collection: StaticNodeId,
        key: CollectionKey,
        before: Option<CollectionKey>,
        value: ProjectionPatchValue,
    },
    CollectionUpdate {
        collection: StaticNodeId,
        key: CollectionKey,
        value: ProjectionPatchValue,
    },
    CollectionRemove {
        collection: StaticNodeId,
        key: CollectionKey,
    },
    CollectionMove {
        collection: StaticNodeId,
        key: CollectionKey,
        before: Option<CollectionKey>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProjectionPatch {
    envelope: ProjectionPatchEnvelope,
    operations: Vec<ProjectionPatchOperation>,
}

impl ProjectionPatch {
    pub(crate) fn new(
        envelope: ProjectionPatchEnvelope,
        operations: Vec<ProjectionPatchOperation>,
    ) -> Result<Self, ProjectionPatchDiagnostics> {
        let diagnostics = validate_patch(envelope, &operations);
        if diagnostics.is_empty() {
            Ok(Self {
                envelope,
                operations,
            })
        } else {
            Err(diagnostics)
        }
    }

    pub(crate) const fn envelope(&self) -> ProjectionPatchEnvelope {
        self.envelope
    }

    pub(crate) fn operations(&self) -> &[ProjectionPatchOperation] {
        &self.operations
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProjectionPatchSet {
    patches: Vec<ProjectionPatch>,
}

impl ProjectionPatchSet {
    pub(crate) fn new(patches: Vec<ProjectionPatch>) -> Result<Self, ProjectionPatchDiagnostics> {
        let diagnostics = validate_patch_set(&patches);
        if diagnostics.is_empty() {
            Ok(Self { patches })
        } else {
            Err(diagnostics)
        }
    }

    pub(crate) fn patches(&self) -> &[ProjectionPatch] {
        &self.patches
    }

    /// Internal deterministic qualification encoding only.
    ///
    /// This is not a public serialization format, bundle artifact, ABI,
    /// cryptographic digest, signature, or compatibility promise.
    pub(crate) fn canonical_bytes(&self) -> Result<Vec<u8>, ProjectionPatchEncodingError> {
        let mut bytes = Vec::new();
        push_bytes(&mut bytes, b"UI_DNA2_PROJECTION_PATCH_V1")?;
        push_len(&mut bytes, self.patches.len())?;
        for patch in &self.patches {
            let envelope = patch.envelope();
            push_u64(&mut bytes, envelope.patch_id().raw());
            push_u64(&mut bytes, envelope.stream_id().raw());
            push_u64(&mut bytes, envelope.document_id().raw());
            push_optional_u64(&mut bytes, envelope.surface_id().map(StaticSurfaceId::raw));
            push_u64(&mut bytes, envelope.previous_projection_revision().raw());
            push_u64(&mut bytes, envelope.projection_revision().raw());
            push_u64(&mut bytes, envelope.epoch().raw());
            push_u64(&mut bytes, envelope.sequence().raw());
            push_len(&mut bytes, patch.operations().len())?;
            for operation in patch.operations() {
                match operation {
                    ProjectionPatchOperation::SetBindingValue { target, value } => {
                        push_u8(&mut bytes, 1);
                        push_u64(&mut bytes, target.node.raw());
                        push_u32(&mut bytes, target.slot.0);
                        push_value(&mut bytes, value)?;
                    }
                    ProjectionPatchOperation::SetNodeAvailability { node, availability } => {
                        push_u8(&mut bytes, 2);
                        push_u64(&mut bytes, node.raw());
                        push_u8(&mut bytes, availability_tag(*availability));
                    }
                    ProjectionPatchOperation::CollectionInsert {
                        collection,
                        key,
                        before,
                        value,
                    } => {
                        push_u8(&mut bytes, 3);
                        push_u64(&mut bytes, collection.raw());
                        push_u64(&mut bytes, key.raw());
                        push_optional_u64(&mut bytes, before.map(CollectionKey::raw));
                        push_value(&mut bytes, value)?;
                    }
                    ProjectionPatchOperation::CollectionUpdate {
                        collection,
                        key,
                        value,
                    } => {
                        push_u8(&mut bytes, 4);
                        push_u64(&mut bytes, collection.raw());
                        push_u64(&mut bytes, key.raw());
                        push_value(&mut bytes, value)?;
                    }
                    ProjectionPatchOperation::CollectionRemove { collection, key } => {
                        push_u8(&mut bytes, 5);
                        push_u64(&mut bytes, collection.raw());
                        push_u64(&mut bytes, key.raw());
                    }
                    ProjectionPatchOperation::CollectionMove {
                        collection,
                        key,
                        before,
                    } => {
                        push_u8(&mut bytes, 6);
                        push_u64(&mut bytes, collection.raw());
                        push_u64(&mut bytes, key.raw());
                        push_optional_u64(&mut bytes, before.map(CollectionKey::raw));
                    }
                }
            }
        }
        Ok(bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ProjectionPatchDiagnosticKind {
    ZeroPatchId,
    ZeroStreamId,
    EmptyOperations,
    DuplicatePatchId,
    MixedStream,
    MixedDocument,
    MixedEpoch,
    NonIncreasingRevision,
    BrokenRevisionChain,
    DuplicateSequence,
    OutOfOrderSequence,
    SequenceOverflow,
    DuplicateMutationTarget,
    CollectionBeforeSelf,
}

impl ProjectionPatchDiagnosticKind {
    const fn code(self) -> DiagnosticCode {
        match self {
            Self::ZeroPatchId => DiagnosticCode::new("PP_ZERO_PATCH_ID"),
            Self::ZeroStreamId => DiagnosticCode::new("PP_ZERO_STREAM_ID"),
            Self::EmptyOperations => DiagnosticCode::new("PP_EMPTY_OPERATIONS"),
            Self::DuplicatePatchId => DiagnosticCode::new("PP_DUP_PATCH_ID"),
            Self::MixedStream => DiagnosticCode::new("PP_MIXED_STREAM"),
            Self::MixedDocument => DiagnosticCode::new("PP_MIXED_DOCUMENT"),
            Self::MixedEpoch => DiagnosticCode::new("PP_MIXED_EPOCH"),
            Self::NonIncreasingRevision => DiagnosticCode::new("PP_NON_INCREASING_REVISION"),
            Self::BrokenRevisionChain => DiagnosticCode::new("PP_BROKEN_REVISION_CHAIN"),
            Self::DuplicateSequence => DiagnosticCode::new("PP_DUP_SEQUENCE"),
            Self::OutOfOrderSequence => DiagnosticCode::new("PP_OUT_OF_ORDER_SEQUENCE"),
            Self::SequenceOverflow => DiagnosticCode::new("PP_SEQUENCE_OVERFLOW"),
            Self::DuplicateMutationTarget => DiagnosticCode::new("PP_DUP_MUTATION_TARGET"),
            Self::CollectionBeforeSelf => DiagnosticCode::new("PP_COLLECTION_BEFORE_SELF"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ProjectionPatchDiagnostic {
    coordinate: DiagnosticCoordinate,
}

impl ProjectionPatchDiagnostic {
    pub(crate) const fn new(
        code: ProjectionPatchDiagnosticKind,
        domain: u64,
        secondary: u64,
    ) -> Self {
        Self {
            coordinate: DiagnosticCoordinate::new(None, code.code(), domain, secondary),
        }
    }

    pub(crate) const fn coordinate(&self) -> DiagnosticCoordinate {
        self.coordinate
    }
}

impl Ord for ProjectionPatchDiagnostic {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.coordinate.cmp(&other.coordinate)
    }
}

impl PartialOrd for ProjectionPatchDiagnostic {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub(crate) type ProjectionPatchDiagnostics = Vec<ProjectionPatchDiagnostic>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProjectionPatchEncodingError {
    LengthOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum MutationTarget {
    Binding {
        node: StaticNodeId,
        slot: u32,
    },
    NodeAvailability {
        node: StaticNodeId,
    },
    CollectionItem {
        kind: CollectionOperationKind,
        collection: StaticNodeId,
        key: CollectionKey,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum CollectionOperationKind {
    Insert,
    Update,
    Remove,
    Move,
}

fn validate_patch(
    envelope: ProjectionPatchEnvelope,
    operations: &[ProjectionPatchOperation],
) -> ProjectionPatchDiagnostics {
    let mut diagnostics = Vec::new();

    if envelope.patch_id().raw() == 0 {
        diagnostics.push(ProjectionPatchDiagnostic::new(
            ProjectionPatchDiagnosticKind::ZeroPatchId,
            envelope.patch_id().raw(),
            0,
        ));
    }
    if envelope.stream_id().raw() == 0 {
        diagnostics.push(ProjectionPatchDiagnostic::new(
            ProjectionPatchDiagnosticKind::ZeroStreamId,
            envelope.patch_id().raw(),
            0,
        ));
    }
    if envelope.projection_revision().raw() <= envelope.previous_projection_revision().raw() {
        diagnostics.push(ProjectionPatchDiagnostic::new(
            ProjectionPatchDiagnosticKind::NonIncreasingRevision,
            envelope.patch_id().raw(),
            0,
        ));
    }
    if operations.is_empty() {
        diagnostics.push(ProjectionPatchDiagnostic::new(
            ProjectionPatchDiagnosticKind::EmptyOperations,
            envelope.patch_id().raw(),
            0,
        ));
    }

    let mut seen_targets = Vec::new();
    for (index, operation) in operations.iter().enumerate() {
        let op_index = u64::try_from(index).unwrap_or(u64::MAX);
        if let Some(target) = mutation_target(operation) {
            if seen_targets.contains(&target) {
                diagnostics.push(ProjectionPatchDiagnostic::new(
                    ProjectionPatchDiagnosticKind::DuplicateMutationTarget,
                    envelope.patch_id().raw(),
                    op_index,
                ));
            } else {
                seen_targets.push(target);
            }
        }

        match operation {
            ProjectionPatchOperation::CollectionInsert {
                key, before, ..
            }
            | ProjectionPatchOperation::CollectionMove {
                key, before, ..
            } => {
                if before == &Some(*key) {
                    diagnostics.push(ProjectionPatchDiagnostic::new(
                        ProjectionPatchDiagnosticKind::CollectionBeforeSelf,
                        envelope.patch_id().raw(),
                        op_index,
                    ));
                }
            }
            ProjectionPatchOperation::SetBindingValue { .. }
            | ProjectionPatchOperation::SetNodeAvailability { .. }
            | ProjectionPatchOperation::CollectionUpdate { .. }
            | ProjectionPatchOperation::CollectionRemove { .. } => {}
        }
    }

    sort_and_dedup_diagnostics(diagnostics)
}

fn validate_patch_set(patches: &[ProjectionPatch]) -> ProjectionPatchDiagnostics {
    let mut diagnostics = Vec::new();

    for (index, patch) in patches.iter().enumerate() {
        if patches[..index]
            .iter()
            .any(|previous| previous.envelope().patch_id() == patch.envelope().patch_id())
        {
            diagnostics.push(ProjectionPatchDiagnostic::new(
                ProjectionPatchDiagnosticKind::DuplicatePatchId,
                patch.envelope().patch_id().raw(),
                patch.envelope().sequence().raw(),
            ));
        }

        if index > 0 {
            let previous = &patches[index - 1];
            let current = patch;
            let mut sequence_chain_valid = true;

            if current.envelope().stream_id() != previous.envelope().stream_id() {
                diagnostics.push(ProjectionPatchDiagnostic::new(
                    ProjectionPatchDiagnosticKind::MixedStream,
                    current.envelope().patch_id().raw(),
                    previous.envelope().patch_id().raw(),
                ));
            }

            if current.envelope().document_id() != previous.envelope().document_id() {
                diagnostics.push(ProjectionPatchDiagnostic::new(
                    ProjectionPatchDiagnosticKind::MixedDocument,
                    current.envelope().patch_id().raw(),
                    previous.envelope().patch_id().raw(),
                ));
            }

            if current.envelope().epoch() != previous.envelope().epoch() {
                diagnostics.push(ProjectionPatchDiagnostic::new(
                    ProjectionPatchDiagnosticKind::MixedEpoch,
                    current.envelope().patch_id().raw(),
                    previous.envelope().patch_id().raw(),
                ));
            }

            match previous.envelope().sequence().checked_next() {
                None => {
                    diagnostics.push(ProjectionPatchDiagnostic::new(
                        ProjectionPatchDiagnosticKind::SequenceOverflow,
                        previous.envelope().patch_id().raw(),
                        previous.envelope().sequence().raw(),
                    ));
                    sequence_chain_valid = false;
                }
                Some(expected) => match current
                    .envelope()
                    .sequence()
                    .cmp(&previous.envelope().sequence())
                {
                    core::cmp::Ordering::Equal => {
                        diagnostics.push(ProjectionPatchDiagnostic::new(
                    ProjectionPatchDiagnosticKind::DuplicateSequence,
                    current.envelope().patch_id().raw(),
                    current.envelope().sequence().raw(),
                ));
                        sequence_chain_valid = false;
                    }
                    core::cmp::Ordering::Less => {
                        diagnostics.push(ProjectionPatchDiagnostic::new(
                    ProjectionPatchDiagnosticKind::OutOfOrderSequence,
                    current.envelope().patch_id().raw(),
                    current.envelope().sequence().raw(),
                ));
                        sequence_chain_valid = false;
                    }
                    core::cmp::Ordering::Greater if current.envelope().sequence() != expected => {
                        diagnostics.push(ProjectionPatchDiagnostic::new(
                            ProjectionPatchDiagnosticKind::OutOfOrderSequence,
                            current.envelope().patch_id().raw(),
                            current.envelope().sequence().raw(),
                        ));
                        sequence_chain_valid = false;
                    }
                    core::cmp::Ordering::Greater => {}
                },
            }

            if sequence_chain_valid
                && current.envelope().previous_projection_revision()
                    != previous.envelope().projection_revision()
            {
                diagnostics.push(ProjectionPatchDiagnostic::new(
                    ProjectionPatchDiagnosticKind::BrokenRevisionChain,
                    current.envelope().patch_id().raw(),
                    previous.envelope().patch_id().raw(),
                ));
            }
        }
    }

    sort_and_dedup_diagnostics(diagnostics)
}

fn mutation_target(operation: &ProjectionPatchOperation) -> Option<MutationTarget> {
    match operation {
        ProjectionPatchOperation::SetBindingValue { target, .. } => Some(MutationTarget::Binding {
            node: target.node,
            slot: target.slot.0,
        }),
        ProjectionPatchOperation::SetNodeAvailability { node, .. } => {
            Some(MutationTarget::NodeAvailability { node: *node })
        }
        ProjectionPatchOperation::CollectionInsert {
            collection, key, ..
        } => Some(MutationTarget::CollectionItem {
            kind: CollectionOperationKind::Insert,
            collection: *collection,
            key: *key,
        }),
        ProjectionPatchOperation::CollectionUpdate {
            collection, key, ..
        } => Some(MutationTarget::CollectionItem {
            kind: CollectionOperationKind::Update,
            collection: *collection,
            key: *key,
        }),
        ProjectionPatchOperation::CollectionRemove { collection, key } => Some(
            MutationTarget::CollectionItem {
                kind: CollectionOperationKind::Remove,
                collection: *collection,
                key: *key,
            },
        ),
        ProjectionPatchOperation::CollectionMove {
            collection, key, ..
        } => Some(MutationTarget::CollectionItem {
            kind: CollectionOperationKind::Move,
            collection: *collection,
            key: *key,
        }),
    }
}

fn sort_and_dedup_diagnostics(
    mut diagnostics: ProjectionPatchDiagnostics,
) -> ProjectionPatchDiagnostics {
    diagnostics.sort();
    diagnostics.dedup_by(|right, left| {
        right.coordinate().code().as_str() == left.coordinate().code().as_str()
    });
    diagnostics
}

fn push_value(
    bytes: &mut Vec<u8>,
    value: &ProjectionPatchValue,
) -> Result<(), ProjectionPatchEncodingError> {
    match value {
        ProjectionPatchValue::Quad(state) => {
            push_u8(bytes, 1);
            push_u8(bytes, quad_state_tag(*state));
        }
        ProjectionPatchValue::SignedScalar(value) => {
            push_u8(bytes, 2);
            push_i64(bytes, *value);
        }
        ProjectionPatchValue::Text(value) => {
            push_u8(bytes, 3);
            push_string(bytes, value)?;
        }
    }
    Ok(())
}

const fn quad_state_tag(state: ProjectionQuadState) -> u8 {
    match state {
        ProjectionQuadState::N => 0,
        ProjectionQuadState::F => 1,
        ProjectionQuadState::T => 2,
        ProjectionQuadState::S => 3,
    }
}

const fn availability_tag(state: ProjectionNodeAvailability) -> u8 {
    match state {
        ProjectionNodeAvailability::Available => 0,
        ProjectionNodeAvailability::Unavailable => 1,
        ProjectionNodeAvailability::Hidden => 2,
        ProjectionNodeAvailability::Pending => 3,
        ProjectionNodeAvailability::Stale => 4,
    }
}

fn push_bytes(bytes: &mut Vec<u8>, value: &[u8]) -> Result<(), ProjectionPatchEncodingError> {
    push_len(bytes, value.len())?;
    bytes.extend_from_slice(value);
    Ok(())
}

fn push_string(bytes: &mut Vec<u8>, value: &str) -> Result<(), ProjectionPatchEncodingError> {
    push_len(bytes, value.len())?;
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn push_len(bytes: &mut Vec<u8>, len: usize) -> Result<(), ProjectionPatchEncodingError> {
    let len = u32::try_from(len).map_err(|_| ProjectionPatchEncodingError::LengthOverflow)?;
    push_u32(bytes, len);
    Ok(())
}

fn push_optional_u64(bytes: &mut Vec<u8>, value: Option<u64>) {
    match value {
        Some(value) => {
            push_u8(bytes, 1);
            push_u64(bytes, value);
        }
        None => push_u8(bytes, 0),
    }
}

fn push_u8(bytes: &mut Vec<u8>, value: u8) {
    bytes.push(value);
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_i64(bytes: &mut Vec<u8>, value: i64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}
