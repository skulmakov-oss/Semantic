//! Explicit `CollectionAnchor` declaration qualification.
//!
//! This module implements the crate-private, pure in-memory path frozen in
//! `docs/spec/ui/projection_source_model.md` section 12:
//!
//! ```text
//! ProjectionSourceNodeId declaration
//!     -> deterministic source-to-static lowering
//!     -> QualifiedCollectionAnchorDeclarations
//! ```
//!
//! It does not implement `PreparedActiveProjectionTargets`,
//! `PreparedProjectionPatchTargets`, `ActiveProjectionTargetCatalog`,
//! `ActivatedShellSessionContext` expansion, stage-4/5/6 evaluation or
//! orchestration, `ProjectionPatch` application, replay-cursor advancement,
//! renderer/backend integration, or any public API.
#![allow(dead_code)]

use alloc::vec::Vec;

use crate::contract_primitives::{
    DiagnosticCode, DiagnosticCoordinate, Epoch, Revision, SourceRef, StaticDocumentId,
    StaticNodeId,
};
use crate::projection_compile::lower_projection_source_node_id;
use crate::projection_source::ProjectionSourceDocument;
use crate::static_ir::StaticUiDocument;

/// The deterministically qualified, whole-set collection-anchor evidence for
/// exactly one validated projection-owned static document version.
///
/// Same-crate private construction is allowed only through
/// [`qualify_collection_anchor_declarations`]. It is immutable, non-runtime,
/// non-authoritative, and it is not `PreparedActiveProjectionTargets`, the
/// runtime catalog, or prepared activation evidence by itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct QualifiedCollectionAnchorDeclarations {
    document_id: StaticDocumentId,
    revision: Revision,
    epoch: Epoch,
    anchors: Vec<StaticNodeId>,
}

impl QualifiedCollectionAnchorDeclarations {
    /// Rebuilds already-verified evidence from one `ProjectionBundle v0`
    /// decode pass. Callable only from `crate::projection_bundle`, and only
    /// after that module's own verification has independently proven: (a)
    /// the anchors arrived in strictly ascending `StaticNodeId` order
    /// (equivalent to Rule 2 duplicate rejection plus Rule 7 ordering), and
    /// (b) every anchor references a node present in the same bundle's
    /// verified Static UI IR (equivalent to Rule 1/Rule 3 existence). This
    /// is not a public constructor and does not accept caller-asserted
    /// evidence from outside the verified bundle decode path.
    pub(crate) fn from_verified_bundle_evidence(
        document_id: StaticDocumentId,
        revision: Revision,
        epoch: Epoch,
        anchors: Vec<StaticNodeId>,
    ) -> Self {
        Self::new(document_id, revision, epoch, anchors)
    }

    fn new(
        document_id: StaticDocumentId,
        revision: Revision,
        epoch: Epoch,
        anchors: Vec<StaticNodeId>,
    ) -> Self {
        Self {
            document_id,
            revision,
            epoch,
            anchors,
        }
    }

    pub(crate) const fn document_id(&self) -> StaticDocumentId {
        self.document_id
    }

    pub(crate) const fn revision(&self) -> Revision {
        self.revision
    }

    pub(crate) const fn epoch(&self) -> Epoch {
        self.epoch
    }

    pub(crate) fn anchors(&self) -> &[StaticNodeId] {
        &self.anchors
    }

    pub(crate) fn len(&self) -> usize {
        self.anchors.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.anchors.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum CollectionAnchorDeclarationDiagnosticKind {
    MissingTargetNode,
    DuplicateDeclaration,
    MissingStaticNode,
    DocumentVersionMismatch,
}

impl CollectionAnchorDeclarationDiagnosticKind {
    const fn code(self) -> DiagnosticCode {
        match self {
            Self::MissingTargetNode => DiagnosticCode::new("CAD_MISSING_TARGET_NODE"),
            Self::DuplicateDeclaration => DiagnosticCode::new("CAD_DUPLICATE_DECLARATION"),
            Self::MissingStaticNode => DiagnosticCode::new("CAD_MISSING_STATIC_NODE"),
            Self::DocumentVersionMismatch => DiagnosticCode::new("CAD_DOCUMENT_VERSION_MISMATCH"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct CollectionAnchorDeclarationDiagnostic {
    coordinate: DiagnosticCoordinate,
    kind: CollectionAnchorDeclarationDiagnosticKind,
}

impl CollectionAnchorDeclarationDiagnostic {
    fn new(
        source: Option<SourceRef>,
        kind: CollectionAnchorDeclarationDiagnosticKind,
        domain: u64,
        secondary: u64,
    ) -> Self {
        Self {
            coordinate: DiagnosticCoordinate::new(source, kind.code(), domain, secondary),
            kind,
        }
    }

    pub(crate) const fn kind(self) -> CollectionAnchorDeclarationDiagnosticKind {
        self.kind
    }

    pub(crate) const fn coordinate(self) -> DiagnosticCoordinate {
        self.coordinate
    }
}

impl Ord for CollectionAnchorDeclarationDiagnostic {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (self.coordinate, self.kind).cmp(&(other.coordinate, other.kind))
    }
}

impl PartialOrd for CollectionAnchorDeclarationDiagnostic {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct CollectionAnchorDeclarationDiagnostics {
    diagnostics: Vec<CollectionAnchorDeclarationDiagnostic>,
}

impl CollectionAnchorDeclarationDiagnostics {
    fn new(mut diagnostics: Vec<CollectionAnchorDeclarationDiagnostic>) -> Self {
        diagnostics.sort();
        diagnostics.dedup();
        Self { diagnostics }
    }

    pub(crate) fn diagnostics(&self) -> &[CollectionAnchorDeclarationDiagnostic] {
        &self.diagnostics
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Deterministically qualifies every authored explicit `CollectionAnchor`
/// declaration in `source` against `static_document`.
///
/// Qualification is atomic: either every declaration is valid and the
/// complete deterministically ordered `QualifiedCollectionAnchorDeclarations`
/// is returned, or the complete deterministically ordered diagnostic
/// sequence is returned and no qualified set escapes.
pub(crate) fn qualify_collection_anchor_declarations(
    expected_document_id: StaticDocumentId,
    source: &ProjectionSourceDocument,
    static_document: &StaticUiDocument,
) -> Result<QualifiedCollectionAnchorDeclarations, CollectionAnchorDeclarationDiagnostics> {
    let mut diagnostics = Vec::new();

    // Pass 1: Rule 1 (source target existence) and compiler-owned lowering.
    // Every declaration that passes contributes one (StaticNodeId, SourceRef)
    // pair, independent of whether its lowered target later turns out to
    // exist in `static_document` (Rule 3). Duplicate identity (Rule 2) is
    // already fully known at this point and must not depend on Rule 3
    // succeeding.
    let mut lowered: Vec<(StaticNodeId, SourceRef)> = Vec::new();
    for declaration in source.collection_anchor_declarations() {
        let target = declaration.target();

        if !source.nodes().iter().any(|node| node.id() == target) {
            diagnostics.push(CollectionAnchorDeclarationDiagnostic::new(
                Some(declaration.source()),
                CollectionAnchorDeclarationDiagnosticKind::MissingTargetNode,
                target.raw(),
                0,
            ));
            continue;
        }

        // Compiler-owned deterministic lowering; the same function used by
        // `compile_projection_source_to_static_ir`. A non-zero
        // `ProjectionSourceNodeId` always lowers, so this branch is
        // defensive rather than reachable in practice.
        let Some(static_id) = lower_projection_source_node_id(target) else {
            diagnostics.push(CollectionAnchorDeclarationDiagnostic::new(
                Some(declaration.source()),
                CollectionAnchorDeclarationDiagnosticKind::MissingTargetNode,
                target.raw(),
                0,
            ));
            continue;
        };

        lowered.push((static_id, declaration.source()));
    }

    // Rule 2 -- duplicate qualified declaration, computed over every
    // successfully lowered declaration regardless of static membership.
    // Duplicate identity is the qualified StaticNodeId; the diagnostic's
    // provenance is the least SourceRef among the contributing declarations
    // (`SourceRef: Ord` over `(SourceId, SourceSpan)`), which is deterministic
    // and independent of authored declaration order or `Vec` insertion order.
    let mut duplicated_ids: Vec<StaticNodeId> = Vec::new();
    {
        let mut seen: Vec<StaticNodeId> = Vec::new();
        for (id, _) in &lowered {
            if seen.contains(id) {
                if !duplicated_ids.contains(id) {
                    duplicated_ids.push(*id);
                }
            } else {
                seen.push(*id);
            }
        }
    }
    for id in &duplicated_ids {
        let provenance = lowered
            .iter()
            .filter(|(candidate, _)| candidate == id)
            .map(|(_, source_ref)| *source_ref)
            .min()
            .expect("a duplicated id has at least two contributing declarations");
        diagnostics.push(CollectionAnchorDeclarationDiagnostic::new(
            Some(provenance),
            CollectionAnchorDeclarationDiagnosticKind::DuplicateDeclaration,
            id.raw(),
            0,
        ));
    }

    // Rule 3 -- static target existence, evaluated independently of Rule 2
    // over the same lowered set.
    let mut qualified: Vec<StaticNodeId> = Vec::new();
    for (id, source_ref) in &lowered {
        if !static_document.nodes().iter().any(|node| node.id() == *id) {
            diagnostics.push(CollectionAnchorDeclarationDiagnostic::new(
                Some(*source_ref),
                CollectionAnchorDeclarationDiagnosticKind::MissingStaticNode,
                id.raw(),
                0,
            ));
            continue;
        }
        qualified.push(*id);
    }

    // Rule 4 -- document-version coherence. The expected revision and epoch
    // are the source document's own revision and epoch.
    if static_document.id() != expected_document_id
        || static_document.revision() != source.revision()
        || static_document.epoch() != source.epoch()
    {
        diagnostics.push(CollectionAnchorDeclarationDiagnostic::new(
            None,
            CollectionAnchorDeclarationDiagnosticKind::DocumentVersionMismatch,
            expected_document_id.raw(),
            static_document.id().raw(),
        ));
    }

    if !diagnostics.is_empty() {
        return Err(CollectionAnchorDeclarationDiagnostics::new(diagnostics));
    }

    // Rule 7 -- deterministic ordering by ascending StaticNodeId, independent
    // of authored declaration order.
    let mut anchors = qualified;
    anchors.sort();

    Ok(QualifiedCollectionAnchorDeclarations::new(
        expected_document_id,
        source.revision(),
        source.epoch(),
        anchors,
    ))
}
