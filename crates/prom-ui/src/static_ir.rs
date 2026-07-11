//! UI DNA v2 Static UI IR document wrapper.
//!
//! `UiIr` remains the legacy structural substrate. This module owns the
//! versioned v2 Static UI IR contract and bounded legacy adaptation.
#![allow(dead_code)]

use alloc::vec::Vec;

use crate::contract_primitives::{
    ChildOrder, CollectionKey, ContractVersion, DiagnosticCode, DiagnosticCoordinate, Epoch,
    Revision, SchemaVersion, SourceRef, StaticDocumentId, StaticNodeId, StaticSurfaceId,
};
use crate::model::{UiAstNodeId, UiIr, UiIrNodeKind};
use crate::role_dictionary::{RoleDictionary, RoleId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct StaticUiDocument {
    schema_version: SchemaVersion,
    contract_version: ContractVersion,
    id: StaticDocumentId,
    revision: Revision,
    epoch: Epoch,
    surfaces: Vec<StaticUiSurface>,
    nodes: Vec<StaticUiNode>,
}

impl StaticUiDocument {
    pub(crate) fn new(id: StaticDocumentId, revision: Revision, epoch: Epoch) -> Self {
        Self {
            schema_version: SchemaVersion::CURRENT,
            contract_version: ContractVersion::CURRENT,
            id,
            revision,
            epoch,
            surfaces: Vec::new(),
            nodes: Vec::new(),
        }
    }

    pub(crate) const fn schema_version(&self) -> SchemaVersion {
        self.schema_version
    }

    pub(crate) const fn contract_version(&self) -> ContractVersion {
        self.contract_version
    }

    pub(crate) const fn id(&self) -> StaticDocumentId {
        self.id
    }

    pub(crate) const fn revision(&self) -> Revision {
        self.revision
    }

    pub(crate) const fn epoch(&self) -> Epoch {
        self.epoch
    }

    pub(crate) fn surfaces(&self) -> &[StaticUiSurface] {
        &self.surfaces
    }

    pub(crate) fn nodes(&self) -> &[StaticUiNode] {
        &self.nodes
    }

    pub(crate) fn push_surface(&mut self, surface: StaticUiSurface) {
        self.surfaces.push(surface);
    }

    pub(crate) fn push_node(&mut self, node: StaticUiNode) {
        self.nodes.push(node);
    }

    pub(crate) fn validate(&self, dictionary: RoleDictionary) -> Result<(), StaticUiIrDiagnostics> {
        let diagnostics = validate_static_ir(self, dictionary);
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }

    pub(crate) fn canonicalized(
        &self,
        dictionary: RoleDictionary,
    ) -> Result<Self, StaticUiIrDiagnostics> {
        self.validate(dictionary)?;
        let mut canonical = self.clone();
        canonical
            .surfaces
            .sort_by_key(|surface| (surface.key, surface.id));
        canonical.nodes.sort_by_key(|node| (node.key, node.id));
        for node in &mut canonical.nodes {
            node.normalize();
        }
        Ok(canonical)
    }

    // Internal deterministic qualification encoding; not a public artifact,
    // cryptographic digest, signature, integrity guarantee, or compatibility promise.
    pub(crate) fn canonical_bytes(
        &self,
        dictionary: RoleDictionary,
    ) -> Result<Vec<u8>, CanonicalEncodingError> {
        let canonical = self.canonicalized(dictionary)?;
        let mut bytes = Vec::new();
        push_bytes(&mut bytes, b"UI_DNA2_STATIC_IR_V1")?;
        push_u32(&mut bytes, canonical.schema_version.raw());
        push_u32(&mut bytes, canonical.contract_version.raw());
        push_u64(&mut bytes, canonical.id.raw());
        push_u64(&mut bytes, canonical.revision.raw());
        push_u64(&mut bytes, canonical.epoch.raw());
        push_len(&mut bytes, canonical.surfaces.len())?;
        for surface in &canonical.surfaces {
            push_u64(&mut bytes, surface.id.raw());
            push_u64(&mut bytes, surface.root.raw());
            push_u64(&mut bytes, surface.key.raw());
            push_optional_source(&mut bytes, surface.source);
        }
        push_len(&mut bytes, canonical.nodes.len())?;
        for node in &canonical.nodes {
            push_u64(&mut bytes, node.id.raw());
            push_str(&mut bytes, node.role.as_str())?;
            push_u64(&mut bytes, node.key.raw());
            push_optional_source(&mut bytes, node.source);
            push_len(&mut bytes, node.children.len())?;
            for child in &node.children {
                push_u32(&mut bytes, child.order.raw());
                push_u64(&mut bytes, child.child.raw());
            }
            push_optional_u64(&mut bytes, node.accessibility_ref.map(StaticNodeId::raw));
        }
        Ok(bytes)
    }

    pub(crate) fn from_legacy_ui_ir(
        context: LegacyUiIrAdapterContext<'_>,
        ir: &UiIr,
    ) -> Result<Self, StaticUiIrDiagnostics> {
        validate_legacy_parent_evidence(ir)?;
        let mut document = Self::new(context.document_id, context.revision, context.epoch);
        let mut root: Option<StaticNodeId> = None;
        let mut root_count = 0_u32;

        for node in ir.nodes() {
            let static_id = StaticNodeId::new(node.id().raw()).map_err(|_| {
                StaticUiIrDiagnostics::single(StaticUiIrDiagnostic::new(
                    None,
                    StaticUiIrDiagnosticKind::InvalidLegacyMapping,
                    node.id().raw(),
                    0,
                ))
            })?;
            let role = match node.kind() {
                UiIrNodeKind::Root => {
                    root_count += 1;
                    root = Some(static_id);
                    RoleId::new("root")
                }
                UiIrNodeKind::Element => RoleId::new("evidence_panel"),
                UiIrNodeKind::Text => RoleId::new("text"),
                UiIrNodeKind::Fragment => RoleId::new("fragment"),
                UiIrNodeKind::Property | UiIrNodeKind::Action | UiIrNodeKind::EffectBoundary => {
                    return Err(StaticUiIrDiagnostics::single(StaticUiIrDiagnostic::new(
                        None,
                        StaticUiIrDiagnosticKind::InvalidLegacyMapping,
                        node.id().raw(),
                        0,
                    )));
                }
            };
            let Some(source_ast_node_id) = node.source_ast_node_id() else {
                return Err(StaticUiIrDiagnostics::single(StaticUiIrDiagnostic::new(
                    None,
                    StaticUiIrDiagnosticKind::InvalidLegacyMapping,
                    node.id().raw(),
                    4,
                )));
            };
            let Some(source_ref) = context.source_ref(source_ast_node_id) else {
                return Err(StaticUiIrDiagnostics::single(StaticUiIrDiagnostic::new(
                    None,
                    StaticUiIrDiagnosticKind::InvalidLegacyMapping,
                    node.id().raw(),
                    source_ast_node_id.raw(),
                )));
            };
            let key = CollectionKey::new(node.id().raw()).map_err(|_| {
                StaticUiIrDiagnostics::single(StaticUiIrDiagnostic::new(
                    None,
                    StaticUiIrDiagnosticKind::InvalidLegacyMapping,
                    node.id().raw(),
                    1,
                ))
            })?;
            let mut static_node = StaticUiNode::new(static_id, role, key, Some(source_ref));
            for (order, child) in node.children().iter().enumerate() {
                let order = u32::try_from(order).map_err(|_| {
                    StaticUiIrDiagnostics::single(StaticUiIrDiagnostic::new(
                        None,
                        StaticUiIrDiagnosticKind::InvalidLegacyMapping,
                        node.id().raw(),
                        2,
                    ))
                })?;
                let child = StaticNodeId::new(child.raw()).map_err(|_| {
                    StaticUiIrDiagnostics::single(StaticUiIrDiagnostic::new(
                        None,
                        StaticUiIrDiagnosticKind::InvalidLegacyMapping,
                        node.id().raw(),
                        3,
                    ))
                })?;
                static_node.push_child(StaticUiChild::new(child, ChildOrder::new(order)));
            }
            document.push_node(static_node);
        }

        if root_count != 1 {
            return Err(StaticUiIrDiagnostics::single(StaticUiIrDiagnostic::new(
                None,
                StaticUiIrDiagnosticKind::InvalidLegacyMapping,
                context.document_id.raw(),
                u64::from(root_count),
            )));
        };
        let Some(root) = root else {
            return Err(StaticUiIrDiagnostics::single(StaticUiIrDiagnostic::new(
                None,
                StaticUiIrDiagnosticKind::MissingRoot,
                context.document_id.raw(),
                0,
            )));
        };
        document.push_surface(StaticUiSurface::new(
            context.surface_id,
            root,
            context.surface_key,
            context.surface_source,
        ));
        document.canonicalized(RoleDictionary::current())
    }
}

fn validate_legacy_parent_evidence(ir: &UiIr) -> Result<(), StaticUiIrDiagnostics> {
    let roots: Vec<usize> = ir
        .nodes()
        .iter()
        .enumerate()
        .filter_map(|(index, node)| (node.kind() == UiIrNodeKind::Root).then_some(index))
        .collect();
    if roots.len() != 1 {
        return Err(StaticUiIrDiagnostics::single(StaticUiIrDiagnostic::new(
            None,
            StaticUiIrDiagnosticKind::InvalidLegacyMapping,
            0,
            u64::try_from(roots.len()).unwrap_or(u64::MAX),
        )));
    }

    let root_index = roots[0];
    let mut expected_parent: Vec<Option<crate::model::UiIrNodeId>> =
        alloc::vec![None; ir.nodes().len()];
    for parent in ir.nodes() {
        for child in parent.children() {
            let Some(child_index) = ir.nodes().iter().position(|node| node.id() == *child) else {
                return Err(StaticUiIrDiagnostics::single(StaticUiIrDiagnostic::new(
                    None,
                    StaticUiIrDiagnosticKind::InvalidLegacyMapping,
                    parent.id().raw(),
                    child.raw(),
                )));
            };
            if let Some(previous) = expected_parent[child_index] {
                return Err(StaticUiIrDiagnostics::single(
                    StaticUiIrDiagnostic::legacy_parent(
                        StaticUiIrDiagnosticKind::LegacyMultipleParents,
                        child.raw(),
                        Some(previous.raw()),
                        Some(parent.id().raw()),
                    ),
                ));
            }
            expected_parent[child_index] = Some(parent.id());
        }
    }

    for (index, node) in ir.nodes().iter().enumerate() {
        let actual = node.parent();
        if index == root_index {
            if let Some(actual) = actual {
                return Err(StaticUiIrDiagnostics::single(
                    StaticUiIrDiagnostic::legacy_parent(
                        StaticUiIrDiagnosticKind::RootHasLegacyParent,
                        node.id().raw(),
                        None,
                        Some(actual.raw()),
                    ),
                ));
            }
            if let Some(expected) = expected_parent[index] {
                return Err(StaticUiIrDiagnostics::single(
                    StaticUiIrDiagnostic::legacy_parent(
                        StaticUiIrDiagnosticKind::LegacyParentMismatch,
                        node.id().raw(),
                        Some(expected.raw()),
                        None,
                    ),
                ));
            }
            continue;
        }

        let Some(expected) = expected_parent[index] else {
            return Err(StaticUiIrDiagnostics::single(
                StaticUiIrDiagnostic::legacy_parent(
                    StaticUiIrDiagnosticKind::MissingLegacyParent,
                    node.id().raw(),
                    None,
                    actual.map(crate::model::UiIrNodeId::raw),
                ),
            ));
        };
        let Some(actual) = actual else {
            return Err(StaticUiIrDiagnostics::single(
                StaticUiIrDiagnostic::legacy_parent(
                    StaticUiIrDiagnosticKind::MissingLegacyParent,
                    node.id().raw(),
                    Some(expected.raw()),
                    None,
                ),
            ));
        };
        if !ir.nodes().iter().any(|candidate| candidate.id() == actual) {
            return Err(StaticUiIrDiagnostics::single(
                StaticUiIrDiagnostic::legacy_parent(
                    StaticUiIrDiagnosticKind::DanglingLegacyParent,
                    node.id().raw(),
                    Some(expected.raw()),
                    Some(actual.raw()),
                ),
            ));
        }
        if actual != expected {
            return Err(StaticUiIrDiagnostics::single(
                StaticUiIrDiagnostic::legacy_parent(
                    StaticUiIrDiagnosticKind::LegacyParentMismatch,
                    node.id().raw(),
                    Some(expected.raw()),
                    Some(actual.raw()),
                ),
            ));
        }
    }

    Ok(())
}

pub(crate) struct LegacyUiIrAdapterContext<'a> {
    document_id: StaticDocumentId,
    surface_id: StaticSurfaceId,
    surface_key: CollectionKey,
    surface_source: Option<SourceRef>,
    revision: Revision,
    epoch: Epoch,
    source_refs: &'a [(UiAstNodeId, SourceRef)],
}

impl<'a> LegacyUiIrAdapterContext<'a> {
    pub(crate) const fn new(
        document_id: StaticDocumentId,
        surface_id: StaticSurfaceId,
        surface_key: CollectionKey,
        surface_source: Option<SourceRef>,
        revision: Revision,
        epoch: Epoch,
        source_refs: &'a [(UiAstNodeId, SourceRef)],
    ) -> Self {
        Self {
            document_id,
            surface_id,
            surface_key,
            surface_source,
            revision,
            epoch,
            source_refs,
        }
    }

    fn source_ref(&self, id: UiAstNodeId) -> Option<SourceRef> {
        self.source_refs
            .iter()
            .find_map(|(candidate, source)| (*candidate == id).then_some(*source))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct StaticUiSurface {
    id: StaticSurfaceId,
    root: StaticNodeId,
    key: CollectionKey,
    source: Option<SourceRef>,
}

impl StaticUiSurface {
    pub(crate) const fn new(
        id: StaticSurfaceId,
        root: StaticNodeId,
        key: CollectionKey,
        source: Option<SourceRef>,
    ) -> Self {
        Self {
            id,
            root,
            key,
            source,
        }
    }

    pub(crate) const fn id(&self) -> StaticSurfaceId {
        self.id
    }

    pub(crate) const fn root(&self) -> StaticNodeId {
        self.root
    }

    pub(crate) const fn key(&self) -> CollectionKey {
        self.key
    }

    pub(crate) const fn source(&self) -> Option<SourceRef> {
        self.source
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct StaticUiNode {
    id: StaticNodeId,
    role: RoleId,
    key: CollectionKey,
    source: Option<SourceRef>,
    children: Vec<StaticUiChild>,
    accessibility_ref: Option<StaticNodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct StaticUiChild {
    child: StaticNodeId,
    order: ChildOrder,
}
impl StaticUiChild {
    pub(crate) const fn new(child: StaticNodeId, order: ChildOrder) -> Self {
        Self { child, order }
    }

    pub(crate) const fn child(self) -> StaticNodeId {
        self.child
    }

    pub(crate) const fn order(self) -> ChildOrder {
        self.order
    }
}

impl StaticUiNode {
    pub(crate) fn new(
        id: StaticNodeId,
        role: RoleId,
        key: CollectionKey,
        source: Option<SourceRef>,
    ) -> Self {
        Self {
            id,
            role,
            key,
            source,
            children: Vec::new(),
            accessibility_ref: None,
        }
    }

    pub(crate) const fn id(&self) -> StaticNodeId {
        self.id
    }

    pub(crate) const fn role(&self) -> RoleId {
        self.role
    }

    pub(crate) const fn key(&self) -> CollectionKey {
        self.key
    }

    pub(crate) const fn source(&self) -> Option<SourceRef> {
        self.source
    }

    pub(crate) fn children(&self) -> &[StaticUiChild] {
        &self.children
    }

    pub(crate) fn push_child(&mut self, child: StaticUiChild) {
        self.children.push(child);
    }

    pub(crate) fn set_accessibility_ref(&mut self, target: StaticNodeId) {
        self.accessibility_ref = Some(target);
    }

    fn normalize(&mut self) {
        self.children.sort_by_key(|edge| edge.order);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum StaticUiIrDiagnosticKind {
    DuplicateSurfaceId,
    DuplicateSurfaceKey,
    DuplicateNodeId,
    DuplicateNodeKey,
    MissingRoot,
    MissingChild,
    DuplicateChild,
    DuplicateChildOrder,
    DuplicateSurfaceRoot,
    RootUsedAsChild,
    MultipleParents,
    Cycle,
    UnreachableNode,
    SharedAcrossSurfaces,
    InvalidAccessibilityReference,
    UnknownRole,
    InvalidLegacyMapping,
    RootHasLegacyParent,
    MissingLegacyParent,
    LegacyParentMismatch,
    DanglingLegacyParent,
    LegacyMultipleParents,
}

impl StaticUiIrDiagnosticKind {
    const fn code(self) -> DiagnosticCode {
        match self {
            Self::DuplicateSurfaceId => DiagnosticCode::new("SIR_DUP_SURFACE_ID"),
            Self::DuplicateSurfaceKey => DiagnosticCode::new("SIR_DUP_SURFACE_KEY"),
            Self::DuplicateNodeId => DiagnosticCode::new("SIR_DUP_NODE_ID"),
            Self::DuplicateNodeKey => DiagnosticCode::new("SIR_DUP_NODE_KEY"),
            Self::MissingRoot => DiagnosticCode::new("SIR_MISSING_ROOT"),
            Self::MissingChild => DiagnosticCode::new("SIR_MISSING_CHILD"),
            Self::DuplicateChild => DiagnosticCode::new("SIR_DUP_CHILD"),
            Self::DuplicateChildOrder => DiagnosticCode::new("SIR_DUP_CHILD_ORDER"),
            Self::DuplicateSurfaceRoot => DiagnosticCode::new("SIR_DUP_SURFACE_ROOT"),
            Self::RootUsedAsChild => DiagnosticCode::new("SIR_ROOT_USED_AS_CHILD"),
            Self::MultipleParents => DiagnosticCode::new("SIR_MULTIPLE_PARENTS"),
            Self::Cycle => DiagnosticCode::new("SIR_CYCLE"),
            Self::UnreachableNode => DiagnosticCode::new("SIR_UNREACHABLE_NODE"),
            Self::SharedAcrossSurfaces => DiagnosticCode::new("SIR_SHARED_ACROSS_SURFACES"),
            Self::InvalidAccessibilityReference => DiagnosticCode::new("SIR_BAD_A11Y_REF"),
            Self::UnknownRole => DiagnosticCode::new("SIR_UNKNOWN_ROLE"),
            Self::InvalidLegacyMapping => DiagnosticCode::new("SIR_INVALID_LEGACY"),
            Self::RootHasLegacyParent => DiagnosticCode::new("SIR_LEGACY_ROOT_PARENT"),
            Self::MissingLegacyParent => DiagnosticCode::new("SIR_LEGACY_MISSING_PARENT"),
            Self::LegacyParentMismatch => DiagnosticCode::new("SIR_LEGACY_PARENT_MISMATCH"),
            Self::DanglingLegacyParent => DiagnosticCode::new("SIR_LEGACY_DANGLING_PARENT"),
            Self::LegacyMultipleParents => DiagnosticCode::new("SIR_LEGACY_MULTI_PARENT"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct LegacyParentEvidence {
    node: u64,
    expected: Option<u64>,
    actual: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct StaticUiIrDiagnostic {
    coordinate: DiagnosticCoordinate,
    kind: StaticUiIrDiagnosticKind,
    legacy_parent: Option<LegacyParentEvidence>,
}

impl StaticUiIrDiagnostic {
    fn new(
        source: Option<SourceRef>,
        kind: StaticUiIrDiagnosticKind,
        domain: u64,
        secondary: u64,
    ) -> Self {
        Self {
            coordinate: DiagnosticCoordinate::new(source, kind.code(), domain, secondary),
            kind,
            legacy_parent: None,
        }
    }

    fn legacy_parent(
        kind: StaticUiIrDiagnosticKind,
        node: u64,
        expected: Option<u64>,
        actual: Option<u64>,
    ) -> Self {
        Self {
            coordinate: DiagnosticCoordinate::new(
                None,
                kind.code(),
                node,
                actual.or(expected).unwrap_or(0),
            ),
            kind,
            legacy_parent: Some(LegacyParentEvidence {
                node,
                expected,
                actual,
            }),
        }
    }

    pub(crate) const fn kind(self) -> StaticUiIrDiagnosticKind {
        self.kind
    }

    pub(crate) const fn coordinate(self) -> DiagnosticCoordinate {
        self.coordinate
    }
}

impl Ord for StaticUiIrDiagnostic {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (self.coordinate, self.kind, self.legacy_parent).cmp(&(
            other.coordinate,
            other.kind,
            other.legacy_parent,
        ))
    }
}

impl PartialOrd for StaticUiIrDiagnostic {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct StaticUiIrDiagnostics {
    diagnostics: Vec<StaticUiIrDiagnostic>,
}

impl StaticUiIrDiagnostics {
    fn single(diagnostic: StaticUiIrDiagnostic) -> Self {
        Self::new(Vec::from([diagnostic]))
    }

    fn new(mut diagnostics: Vec<StaticUiIrDiagnostic>) -> Self {
        diagnostics.sort();
        diagnostics.dedup();
        Self { diagnostics }
    }

    pub(crate) fn diagnostics(&self) -> &[StaticUiIrDiagnostic] {
        &self.diagnostics
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

fn validate_static_ir(
    document: &StaticUiDocument,
    dictionary: RoleDictionary,
) -> StaticUiIrDiagnostics {
    let mut diagnostics = Vec::new();

    for (index, surface) in document.surfaces.iter().enumerate() {
        if document.surfaces[..index]
            .iter()
            .any(|previous| previous.id == surface.id)
        {
            diagnostics.push(StaticUiIrDiagnostic::new(
                surface.source,
                StaticUiIrDiagnosticKind::DuplicateSurfaceId,
                surface.id.raw(),
                0,
            ));
        }
        if document.surfaces[..index]
            .iter()
            .any(|previous| previous.key == surface.key)
        {
            diagnostics.push(StaticUiIrDiagnostic::new(
                surface.source,
                StaticUiIrDiagnosticKind::DuplicateSurfaceKey,
                surface.key.raw(),
                0,
            ));
        }
        if !document.nodes.iter().any(|node| node.id == surface.root) {
            diagnostics.push(StaticUiIrDiagnostic::new(
                surface.source,
                StaticUiIrDiagnosticKind::MissingRoot,
                surface.root.raw(),
                0,
            ));
        }
    }

    for (index, node) in document.nodes.iter().enumerate() {
        if document.nodes[..index]
            .iter()
            .any(|previous| previous.id == node.id)
        {
            diagnostics.push(StaticUiIrDiagnostic::new(
                node.source,
                StaticUiIrDiagnosticKind::DuplicateNodeId,
                node.id.raw(),
                0,
            ));
        }
        if document.nodes[..index]
            .iter()
            .any(|previous| previous.key == node.key)
        {
            diagnostics.push(StaticUiIrDiagnostic::new(
                node.source,
                StaticUiIrDiagnosticKind::DuplicateNodeKey,
                node.key.raw(),
                0,
            ));
        }
        if dictionary.validate(node.role).is_err() {
            diagnostics.push(StaticUiIrDiagnostic::new(
                node.source,
                StaticUiIrDiagnosticKind::UnknownRole,
                node.key.raw(),
                0,
            ));
        }
        if let Some(target) = node.accessibility_ref {
            if target == node.id
                || !document
                    .nodes
                    .iter()
                    .any(|candidate| candidate.id == target)
            {
                diagnostics.push(StaticUiIrDiagnostic::new(
                    node.source,
                    StaticUiIrDiagnosticKind::InvalidAccessibilityReference,
                    node.id.raw(),
                    target.raw(),
                ));
            }
        }
        for (child_index, child) in node.children.iter().enumerate() {
            if !document
                .nodes
                .iter()
                .any(|candidate| candidate.id == child.child)
            {
                diagnostics.push(StaticUiIrDiagnostic::new(
                    node.source,
                    StaticUiIrDiagnosticKind::MissingChild,
                    node.id.raw(),
                    child.child.raw(),
                ));
            }
            if node.children[..child_index]
                .iter()
                .any(|previous| previous.child == child.child)
            {
                diagnostics.push(StaticUiIrDiagnostic::new(
                    node.source,
                    StaticUiIrDiagnosticKind::DuplicateChild,
                    node.id.raw(),
                    child.child.raw(),
                ));
            }
            if node.children[..child_index]
                .iter()
                .any(|previous| previous.order == child.order)
            {
                diagnostics.push(StaticUiIrDiagnostic::new(
                    node.source,
                    StaticUiIrDiagnosticKind::DuplicateChildOrder,
                    node.id.raw(),
                    u64::from(child.order.raw()),
                ));
            }
        }
    }

    validate_static_forest(document, &mut diagnostics);

    StaticUiIrDiagnostics::new(diagnostics)
}

fn validate_static_forest(
    document: &StaticUiDocument,
    diagnostics: &mut Vec<StaticUiIrDiagnostic>,
) {
    if document.surfaces.is_empty() {
        diagnostics.push(StaticUiIrDiagnostic::new(
            None,
            StaticUiIrDiagnosticKind::MissingRoot,
            document.id.raw(),
            0,
        ));
        return;
    }

    let mut root_indices = Vec::new();
    for surface in &document.surfaces {
        let Some(root_index) = static_node_index(document, surface.root) else {
            continue;
        };
        if root_indices.contains(&root_index) {
            diagnostics.push(StaticUiIrDiagnostic::new(
                surface.source,
                StaticUiIrDiagnosticKind::DuplicateSurfaceRoot,
                surface.root.raw(),
                surface.id.raw(),
            ));
        }
        root_indices.push(root_index);
    }

    let mut owner: Vec<Option<usize>> = alloc::vec![None; document.nodes.len()];
    let mut parent_count: Vec<u32> = alloc::vec![0; document.nodes.len()];
    let mut visiting: Vec<bool> = alloc::vec![false; document.nodes.len()];
    let mut complete: Vec<bool> = alloc::vec![false; document.nodes.len()];

    for (surface_index, surface) in document.surfaces.iter().enumerate() {
        if let Some(root_index) = static_node_index(document, surface.root) {
            walk_static_forest(
                document,
                surface_index,
                root_index,
                &mut owner,
                &mut parent_count,
                &mut visiting,
                &mut complete,
                diagnostics,
            );
        }
    }

    for (index, node) in document.nodes.iter().enumerate() {
        let is_root = root_indices.contains(&index);
        if is_root && parent_count[index] != 0 {
            diagnostics.push(StaticUiIrDiagnostic::new(
                node.source,
                StaticUiIrDiagnosticKind::RootUsedAsChild,
                node.id.raw(),
                0,
            ));
        }
        if !is_root && parent_count[index] == 0 {
            diagnostics.push(StaticUiIrDiagnostic::new(
                node.source,
                StaticUiIrDiagnosticKind::UnreachableNode,
                node.id.raw(),
                0,
            ));
        }
        if !is_root && parent_count[index] > 1 {
            diagnostics.push(StaticUiIrDiagnostic::new(
                node.source,
                StaticUiIrDiagnosticKind::MultipleParents,
                node.id.raw(),
                u64::from(parent_count[index]),
            ));
        }
    }
}

struct StaticTraversalFrame {
    node_index: usize,
    parent: Option<usize>,
    next_child: usize,
    entered: bool,
}

#[allow(clippy::too_many_arguments)]
fn walk_static_forest(
    document: &StaticUiDocument,
    surface_index: usize,
    root_index: usize,
    owner: &mut [Option<usize>],
    parent_count: &mut [u32],
    visiting: &mut [bool],
    complete: &mut [bool],
    diagnostics: &mut Vec<StaticUiIrDiagnostic>,
) {
    let mut stack = Vec::from([StaticTraversalFrame {
        node_index: root_index,
        parent: None,
        next_child: 0,
        entered: false,
    }]);

    while let Some(mut frame) = stack.pop() {
        let node_index = frame.node_index;
        if !frame.entered {
            frame.entered = true;
            let node = &document.nodes[node_index];
            if frame.parent.is_some() {
                parent_count[node_index] = parent_count[node_index].saturating_add(1);
            }

            match owner[node_index] {
                Some(previous) if previous != surface_index => {
                    diagnostics.push(StaticUiIrDiagnostic::new(
                        node.source,
                        StaticUiIrDiagnosticKind::SharedAcrossSurfaces,
                        node.id.raw(),
                        surface_index as u64,
                    ));
                }
                None => owner[node_index] = Some(surface_index),
                _ => {}
            }

            if visiting[node_index] {
                diagnostics.push(StaticUiIrDiagnostic::new(
                    node.source,
                    StaticUiIrDiagnosticKind::Cycle,
                    node.id.raw(),
                    0,
                ));
                continue;
            }
            if complete[node_index] {
                continue;
            }
            visiting[node_index] = true;
        }

        let next_child = frame.next_child;
        let next = document.nodes[node_index].children.get(next_child).copied();
        if let Some(child) = next {
            frame.next_child += 1;
            stack.push(frame);
            if let Some(child_index) = static_node_index(document, child.child) {
                stack.push(StaticTraversalFrame {
                    node_index: child_index,
                    parent: Some(node_index),
                    next_child: 0,
                    entered: false,
                });
            }
        } else {
            visiting[node_index] = false;
            complete[node_index] = true;
        }
    }
}

fn static_node_index(document: &StaticUiDocument, id: StaticNodeId) -> Option<usize> {
    document.nodes.iter().position(|node| node.id == id)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum CanonicalEncodingError {
    InvalidDocument(StaticUiIrDiagnostics),
    LengthOverflow,
}

impl From<StaticUiIrDiagnostics> for CanonicalEncodingError {
    fn from(value: StaticUiIrDiagnostics) -> Self {
        Self::InvalidDocument(value)
    }
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn checked_len(value: usize) -> Result<u32, CanonicalEncodingError> {
    u32::try_from(value).map_err(|_| CanonicalEncodingError::LengthOverflow)
}

fn push_len(bytes: &mut Vec<u8>, value: usize) -> Result<(), CanonicalEncodingError> {
    push_u32(bytes, checked_len(value)?);
    Ok(())
}

fn push_bytes(bytes: &mut Vec<u8>, value: &[u8]) -> Result<(), CanonicalEncodingError> {
    push_len(bytes, value.len())?;
    bytes.extend_from_slice(value);
    Ok(())
}

fn push_str(bytes: &mut Vec<u8>, value: &str) -> Result<(), CanonicalEncodingError> {
    push_bytes(bytes, value.as_bytes())
}

fn push_optional_u64(bytes: &mut Vec<u8>, value: Option<u64>) {
    match value {
        Some(value) => {
            bytes.push(1);
            push_u64(bytes, value);
        }
        None => bytes.push(0),
    }
}

fn push_optional_source(bytes: &mut Vec<u8>, source: Option<SourceRef>) {
    match source {
        Some(source) => {
            bytes.push(1);
            push_u64(bytes, source.source().raw());
            push_u32(bytes, source.span().start());
            push_u32(bytes, source.span().end());
        }
        None => bytes.push(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract_primitives::{SourceId, SourceSpan};
    use crate::model::{UiAstNodeId, UiIrNode, UiIrNodeId};

    fn doc_id(raw: u64) -> StaticDocumentId {
        StaticDocumentId::new(raw).expect("document id")
    }

    fn node_id(raw: u64) -> StaticNodeId {
        StaticNodeId::new(raw).expect("node id")
    }

    fn surface_id(raw: u64) -> StaticSurfaceId {
        StaticSurfaceId::new(raw).expect("surface id")
    }

    fn key(raw: u64) -> CollectionKey {
        CollectionKey::new(raw).expect("collection key")
    }

    fn source_ref(start: u32, end: u32) -> SourceRef {
        SourceRef::new(
            SourceId::new(1).expect("source id"),
            SourceSpan::new(start, end).expect("source span"),
        )
    }

    fn legacy_context<'a>(
        source_refs: &'a [(UiAstNodeId, SourceRef)],
    ) -> LegacyUiIrAdapterContext<'a> {
        LegacyUiIrAdapterContext::new(
            doc_id(1),
            surface_id(1),
            key(1),
            Some(source_ref(0, 1)),
            Revision::new(0),
            Epoch::new(0),
            source_refs,
        )
    }

    fn legacy_node(
        id: u64,
        kind: UiIrNodeKind,
        source_ast_id: u64,
        parent: Option<u64>,
        children: &[u64],
    ) -> UiIrNode {
        let mut node = match parent {
            Some(parent) => {
                UiIrNode::with_parent(UiIrNodeId::new(id), kind, UiIrNodeId::new(parent))
            }
            None => UiIrNode::new(UiIrNodeId::new(id), kind),
        };
        node.set_source_ast_node_id(Some(UiAstNodeId::new(source_ast_id)));
        for child in children {
            node.push_child(UiIrNodeId::new(*child));
        }
        node
    }

    fn minimal_document() -> StaticUiDocument {
        let mut document = StaticUiDocument::new(doc_id(1), Revision::new(0), Epoch::new(0));
        document.push_node(StaticUiNode::new(
            node_id(10),
            RoleId::new("root"),
            key(10),
            Some(source_ref(0, 1)),
        ));
        document.push_surface(StaticUiSurface::new(
            surface_id(1),
            node_id(10),
            key(1),
            Some(source_ref(0, 1)),
        ));
        document
    }

    fn document_with_ordered_children(order_a: u32, order_b: u32) -> StaticUiDocument {
        let mut document = StaticUiDocument::new(doc_id(1), Revision::new(0), Epoch::new(0));
        let mut root = StaticUiNode::new(
            node_id(10),
            RoleId::new("root"),
            key(10),
            Some(source_ref(0, 1)),
        );
        root.push_child(StaticUiChild::new(node_id(20), ChildOrder::new(order_a)));
        root.push_child(StaticUiChild::new(node_id(30), ChildOrder::new(order_b)));
        document.push_node(root);
        document.push_node(StaticUiNode::new(
            node_id(20),
            RoleId::new("text"),
            key(20),
            Some(source_ref(2, 3)),
        ));
        document.push_node(StaticUiNode::new(
            node_id(30),
            RoleId::new("text"),
            key(30),
            Some(source_ref(4, 5)),
        ));
        document.push_surface(StaticUiSurface::new(
            surface_id(1),
            node_id(10),
            key(1),
            Some(source_ref(0, 1)),
        ));
        document
    }

    #[test]
    fn static_ir_minimal_document_validates() {
        assert!(minimal_document()
            .validate(RoleDictionary::current())
            .is_ok());
    }

    #[test]
    fn static_ir_rejects_duplicate_ids_and_keys() {
        let mut document = minimal_document();
        document.push_node(StaticUiNode::new(
            node_id(10),
            RoleId::new("text"),
            key(10),
            Some(source_ref(2, 3)),
        ));

        let diagnostics = document
            .validate(RoleDictionary::current())
            .expect_err("duplicate");

        assert_eq!(
            diagnostics.diagnostics()[0].kind(),
            StaticUiIrDiagnosticKind::DuplicateNodeId
        );
        assert_eq!(
            diagnostics.diagnostics()[1].kind(),
            StaticUiIrDiagnosticKind::DuplicateNodeKey
        );
    }

    #[test]
    fn static_ir_rejects_dangling_references() {
        let mut document = StaticUiDocument::new(doc_id(1), Revision::new(0), Epoch::new(0));
        document.push_surface(StaticUiSurface::new(
            surface_id(1),
            node_id(99),
            key(1),
            Some(source_ref(0, 1)),
        ));

        let diagnostics = document
            .validate(RoleDictionary::current())
            .expect_err("missing root");

        assert_eq!(
            diagnostics.diagnostics()[0].kind(),
            StaticUiIrDiagnosticKind::MissingRoot
        );
    }

    #[test]
    fn static_ir_canonical_bytes_are_insertion_order_independent() {
        let mut first = minimal_document();
        first
            .nodes
            .iter_mut()
            .find(|node| node.id() == node_id(10))
            .expect("root")
            .push_child(StaticUiChild::new(node_id(20), ChildOrder::new(0)));
        first.push_node(StaticUiNode::new(
            node_id(20),
            RoleId::new("text"),
            key(20),
            Some(source_ref(2, 3)),
        ));

        let mut second = StaticUiDocument::new(doc_id(1), Revision::new(0), Epoch::new(0));
        second.push_node(StaticUiNode::new(
            node_id(20),
            RoleId::new("text"),
            key(20),
            Some(source_ref(2, 3)),
        ));
        second.push_node(StaticUiNode::new(
            node_id(10),
            RoleId::new("root"),
            key(10),
            Some(source_ref(0, 1)),
        ));
        second
            .nodes
            .iter_mut()
            .find(|node| node.id() == node_id(10))
            .expect("root")
            .push_child(StaticUiChild::new(node_id(20), ChildOrder::new(0)));
        second.push_surface(StaticUiSurface::new(
            surface_id(1),
            node_id(10),
            key(1),
            Some(source_ref(0, 1)),
        ));

        assert_eq!(
            first
                .canonical_bytes(RoleDictionary::current())
                .expect("first"),
            second
                .canonical_bytes(RoleDictionary::current())
                .expect("second")
        );
    }

    #[test]
    fn static_ir_storage_permutation_preserves_semantic_order_bytes() {
        let first = document_with_ordered_children(0, 1);
        let mut second = StaticUiDocument::new(doc_id(1), Revision::new(0), Epoch::new(0));
        second.push_node(StaticUiNode::new(
            node_id(30),
            RoleId::new("text"),
            key(30),
            Some(source_ref(4, 5)),
        ));
        let mut root = StaticUiNode::new(
            node_id(10),
            RoleId::new("root"),
            key(10),
            Some(source_ref(0, 1)),
        );
        root.push_child(StaticUiChild::new(node_id(30), ChildOrder::new(1)));
        root.push_child(StaticUiChild::new(node_id(20), ChildOrder::new(0)));
        second.push_node(root);
        second.push_node(StaticUiNode::new(
            node_id(20),
            RoleId::new("text"),
            key(20),
            Some(source_ref(2, 3)),
        ));
        second.push_surface(StaticUiSurface::new(
            surface_id(1),
            node_id(10),
            key(1),
            Some(source_ref(0, 1)),
        ));

        assert_eq!(
            first
                .canonical_bytes(RoleDictionary::current())
                .expect("first bytes"),
            second
                .canonical_bytes(RoleDictionary::current())
                .expect("second bytes")
        );
    }

    #[test]
    fn static_ir_semantic_child_order_changes_bytes() {
        let first = document_with_ordered_children(0, 1);
        let second = document_with_ordered_children(1, 0);

        assert_ne!(
            first
                .canonical_bytes(RoleDictionary::current())
                .expect("first bytes"),
            second
                .canonical_bytes(RoleDictionary::current())
                .expect("second bytes")
        );
    }

    #[test]
    fn static_ir_rejects_ordered_forest_violations() {
        let duplicate_order = document_with_ordered_children(0, 0);
        assert!(duplicate_order
            .validate(RoleDictionary::current())
            .expect_err("duplicate order")
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.kind() == StaticUiIrDiagnosticKind::DuplicateChildOrder));

        let mut cycle = minimal_document();
        cycle
            .nodes
            .iter_mut()
            .find(|node| node.id() == node_id(10))
            .expect("root")
            .push_child(StaticUiChild::new(node_id(10), ChildOrder::new(0)));
        assert!(cycle
            .validate(RoleDictionary::current())
            .expect_err("cycle")
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.kind() == StaticUiIrDiagnosticKind::Cycle));

        let mut multi_parent = document_with_ordered_children(0, 1);
        multi_parent
            .nodes
            .iter_mut()
            .find(|node| node.id() == node_id(20))
            .expect("child")
            .push_child(StaticUiChild::new(node_id(30), ChildOrder::new(0)));
        assert!(multi_parent
            .validate(RoleDictionary::current())
            .expect_err("multiple parents")
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.kind() == StaticUiIrDiagnosticKind::MultipleParents));

        let mut orphan = minimal_document();
        orphan.push_node(StaticUiNode::new(
            node_id(40),
            RoleId::new("text"),
            key(40),
            Some(source_ref(6, 7)),
        ));
        assert!(orphan
            .validate(RoleDictionary::current())
            .expect_err("orphan")
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.kind() == StaticUiIrDiagnosticKind::UnreachableNode));

        let mut duplicate_child = document_with_ordered_children(0, 1);
        duplicate_child
            .nodes
            .iter_mut()
            .find(|node| node.id() == node_id(10))
            .expect("root")
            .push_child(StaticUiChild::new(node_id(20), ChildOrder::new(2)));
        assert!(duplicate_child
            .validate(RoleDictionary::current())
            .expect_err("duplicate child")
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.kind() == StaticUiIrDiagnosticKind::DuplicateChild));

        let mut dangling_child = minimal_document();
        dangling_child
            .nodes
            .iter_mut()
            .find(|node| node.id() == node_id(10))
            .expect("root")
            .push_child(StaticUiChild::new(node_id(99), ChildOrder::new(0)));
        assert!(dangling_child
            .validate(RoleDictionary::current())
            .expect_err("dangling child")
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.kind() == StaticUiIrDiagnosticKind::MissingChild));

        let mut shared = StaticUiDocument::new(doc_id(1), Revision::new(0), Epoch::new(0));
        let mut first_root = StaticUiNode::new(
            node_id(10),
            RoleId::new("root"),
            key(10),
            Some(source_ref(0, 1)),
        );
        first_root.push_child(StaticUiChild::new(node_id(30), ChildOrder::new(0)));
        let mut second_root = StaticUiNode::new(
            node_id(20),
            RoleId::new("root"),
            key(20),
            Some(source_ref(2, 3)),
        );
        second_root.push_child(StaticUiChild::new(node_id(30), ChildOrder::new(0)));
        shared.push_node(first_root);
        shared.push_node(second_root);
        shared.push_node(StaticUiNode::new(
            node_id(30),
            RoleId::new("text"),
            key(30),
            Some(source_ref(4, 5)),
        ));
        shared.push_surface(StaticUiSurface::new(
            surface_id(1),
            node_id(10),
            key(1),
            Some(source_ref(0, 1)),
        ));
        shared.push_surface(StaticUiSurface::new(
            surface_id(2),
            node_id(20),
            key(2),
            Some(source_ref(2, 3)),
        ));
        let shared_diagnostics = shared
            .validate(RoleDictionary::current())
            .expect_err("shared node");
        assert!(shared_diagnostics
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.kind() == StaticUiIrDiagnosticKind::SharedAcrossSurfaces));

        let mut multi_node_cycle =
            StaticUiDocument::new(doc_id(1), Revision::new(0), Epoch::new(0));
        let mut root = StaticUiNode::new(
            node_id(10),
            RoleId::new("root"),
            key(10),
            Some(source_ref(0, 1)),
        );
        root.push_child(StaticUiChild::new(node_id(20), ChildOrder::new(0)));
        let mut child = StaticUiNode::new(
            node_id(20),
            RoleId::new("text"),
            key(20),
            Some(source_ref(2, 3)),
        );
        child.push_child(StaticUiChild::new(node_id(10), ChildOrder::new(0)));
        multi_node_cycle.push_node(root);
        multi_node_cycle.push_node(child);
        multi_node_cycle.push_surface(StaticUiSurface::new(
            surface_id(1),
            node_id(10),
            key(1),
            Some(source_ref(0, 1)),
        ));
        assert!(multi_node_cycle
            .validate(RoleDictionary::current())
            .expect_err("multi-node cycle")
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.kind() == StaticUiIrDiagnosticKind::Cycle));
    }

    #[test]
    fn static_ir_rejects_invalid_accessibility_reference() {
        let mut document = minimal_document();
        document
            .nodes
            .iter_mut()
            .find(|node| node.id() == node_id(10))
            .expect("root")
            .set_accessibility_ref(node_id(99));

        assert!(document
            .validate(RoleDictionary::current())
            .expect_err("bad accessibility ref")
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.kind()
                == StaticUiIrDiagnosticKind::InvalidAccessibilityReference));
    }

    #[test]
    fn static_ir_adapts_legacy_ui_ir_without_widening_authority() {
        let mut ir = UiIr::new();
        let mut root = UiIrNode::new(UiIrNodeId::new(1), UiIrNodeKind::Root);
        root.set_source_ast_node_id(Some(UiAstNodeId::new(101)));
        root.push_child(UiIrNodeId::new(2));
        ir.push_node(root);
        let mut child =
            UiIrNode::with_parent(UiIrNodeId::new(2), UiIrNodeKind::Text, UiIrNodeId::new(1));
        child.set_source_ast_node_id(Some(UiAstNodeId::new(102)));
        ir.push_node(child);
        let source_refs = [
            (UiAstNodeId::new(101), source_ref(0, 1)),
            (UiAstNodeId::new(102), source_ref(2, 3)),
        ];

        let document = StaticUiDocument::from_legacy_ui_ir(legacy_context(&source_refs), &ir)
            .expect("legacy adapter");

        assert_eq!(document.nodes().len(), 2);
        assert_eq!(document.surfaces()[0].root(), node_id(1));
    }

    #[test]
    fn static_ir_legacy_adapter_rejects_missing_source_mapping() {
        let mut ir = UiIr::new();
        let mut root = UiIrNode::new(UiIrNodeId::new(1), UiIrNodeKind::Root);
        root.set_source_ast_node_id(Some(UiAstNodeId::new(101)));
        ir.push_node(root);

        let diagnostics = StaticUiDocument::from_legacy_ui_ir(legacy_context(&[]), &ir)
            .expect_err("missing source mapping");

        assert_eq!(
            diagnostics.diagnostics()[0].kind(),
            StaticUiIrDiagnosticKind::InvalidLegacyMapping
        );
    }

    #[test]
    fn static_ir_legacy_adapter_rejects_multiple_roots() {
        let mut ir = UiIr::new();
        let mut first = UiIrNode::new(UiIrNodeId::new(1), UiIrNodeKind::Root);
        first.set_source_ast_node_id(Some(UiAstNodeId::new(101)));
        let mut second = UiIrNode::new(UiIrNodeId::new(2), UiIrNodeKind::Root);
        second.set_source_ast_node_id(Some(UiAstNodeId::new(102)));
        ir.push_node(first);
        ir.push_node(second);
        let source_refs = [
            (UiAstNodeId::new(101), source_ref(0, 1)),
            (UiAstNodeId::new(102), source_ref(2, 3)),
        ];

        let diagnostics = StaticUiDocument::from_legacy_ui_ir(legacy_context(&source_refs), &ir)
            .expect_err("multiple roots");

        assert_eq!(
            diagnostics.diagnostics()[0].kind(),
            StaticUiIrDiagnosticKind::InvalidLegacyMapping
        );
    }

    #[test]
    fn static_ir_checked_encoding_length_boundaries() {
        assert_eq!(checked_len(u32::MAX as usize), Ok(u32::MAX));
        if usize::BITS > u32::BITS {
            assert_eq!(
                checked_len((u32::MAX as usize) + 1),
                Err(CanonicalEncodingError::LengthOverflow)
            );
        }
    }

    #[test]
    fn static_ir_rejects_legacy_action_as_v2_static_ir() {
        let mut ir = UiIr::new();
        let mut action = UiIrNode::new(UiIrNodeId::new(1), UiIrNodeKind::Action);
        action.set_source_ast_node_id(Some(UiAstNodeId::new(101)));
        ir.push_node(action);
        let source_refs = [(UiAstNodeId::new(101), source_ref(0, 1))];

        let diagnostics = StaticUiDocument::from_legacy_ui_ir(legacy_context(&source_refs), &ir)
            .expect_err("legacy action rejected");

        assert_eq!(
            diagnostics.diagnostics()[0].kind(),
            StaticUiIrDiagnosticKind::InvalidLegacyMapping
        );
    }

    #[test]
    fn static_ir_validates_deep_forest_without_recursive_traversal() {
        const DEPTH: u64 = 512;
        let mut document = StaticUiDocument::new(doc_id(1), Revision::new(0), Epoch::new(0));
        for raw in 1..=DEPTH {
            let mut node = StaticUiNode::new(
                node_id(raw),
                if raw == 1 {
                    RoleId::new("root")
                } else {
                    RoleId::new("text")
                },
                key(raw),
                Some(source_ref(raw as u32, raw as u32 + 1)),
            );
            if raw < DEPTH {
                node.push_child(StaticUiChild::new(node_id(raw + 1), ChildOrder::new(0)));
            }
            document.push_node(node);
        }
        document.push_surface(StaticUiSurface::new(
            surface_id(1),
            node_id(1),
            key(1),
            Some(source_ref(0, 1)),
        ));

        assert!(document.validate(RoleDictionary::current()).is_ok());
    }

    #[test]
    fn static_ir_reports_deep_cycle_deterministically() {
        const DEPTH: u64 = 512;
        let mut document = StaticUiDocument::new(doc_id(1), Revision::new(0), Epoch::new(0));
        for raw in 1..=DEPTH {
            let mut node = StaticUiNode::new(
                node_id(raw),
                if raw == 1 {
                    RoleId::new("root")
                } else {
                    RoleId::new("text")
                },
                key(raw),
                Some(source_ref(raw as u32, raw as u32 + 1)),
            );
            let child = (raw < DEPTH)
                .then_some(raw + 1)
                .or((raw == DEPTH).then_some(1));
            if let Some(child) = child {
                node.push_child(StaticUiChild::new(node_id(child), ChildOrder::new(0)));
            }
            document.push_node(node);
        }
        document.push_surface(StaticUiSurface::new(
            surface_id(1),
            node_id(1),
            key(1),
            Some(source_ref(0, 1)),
        ));

        let first = document
            .validate(RoleDictionary::current())
            .expect_err("cycle");
        let second = document
            .validate(RoleDictionary::current())
            .expect_err("cycle");
        assert_eq!(first, second);
        assert!(first
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.kind() == StaticUiIrDiagnosticKind::Cycle));
    }

    #[test]
    fn static_ir_legacy_adapter_validates_parent_evidence() {
        let source_refs = [
            (UiAstNodeId::new(101), source_ref(0, 1)),
            (UiAstNodeId::new(102), source_ref(2, 3)),
            (UiAstNodeId::new(103), source_ref(4, 5)),
        ];

        let mut valid = UiIr::new();
        valid.push_node(legacy_node(1, UiIrNodeKind::Root, 101, None, &[2]));
        valid.push_node(legacy_node(2, UiIrNodeKind::Text, 102, Some(1), &[]));
        assert!(StaticUiDocument::from_legacy_ui_ir(legacy_context(&source_refs), &valid).is_ok());

        let mut root_with_parent = UiIr::new();
        root_with_parent.push_node(legacy_node(1, UiIrNodeKind::Root, 101, Some(2), &[2]));
        root_with_parent.push_node(legacy_node(2, UiIrNodeKind::Text, 102, Some(1), &[]));
        assert_eq!(
            StaticUiDocument::from_legacy_ui_ir(legacy_context(&source_refs), &root_with_parent)
                .expect_err("root parent")
                .diagnostics()[0]
                .kind(),
            StaticUiIrDiagnosticKind::RootHasLegacyParent
        );

        let mut missing_parent = UiIr::new();
        missing_parent.push_node(legacy_node(1, UiIrNodeKind::Root, 101, None, &[2]));
        missing_parent.push_node(legacy_node(2, UiIrNodeKind::Text, 102, None, &[]));
        assert_eq!(
            StaticUiDocument::from_legacy_ui_ir(legacy_context(&source_refs), &missing_parent)
                .expect_err("missing parent")
                .diagnostics()[0]
                .kind(),
            StaticUiIrDiagnosticKind::MissingLegacyParent
        );

        let mut wrong_parent = UiIr::new();
        wrong_parent.push_node(legacy_node(1, UiIrNodeKind::Root, 101, None, &[2, 3]));
        wrong_parent.push_node(legacy_node(2, UiIrNodeKind::Text, 102, Some(3), &[]));
        wrong_parent.push_node(legacy_node(3, UiIrNodeKind::Text, 103, Some(1), &[]));
        assert_eq!(
            StaticUiDocument::from_legacy_ui_ir(legacy_context(&source_refs), &wrong_parent)
                .expect_err("wrong parent")
                .diagnostics()[0]
                .kind(),
            StaticUiIrDiagnosticKind::LegacyParentMismatch
        );

        let mut dangling_parent = UiIr::new();
        dangling_parent.push_node(legacy_node(1, UiIrNodeKind::Root, 101, None, &[2]));
        dangling_parent.push_node(legacy_node(2, UiIrNodeKind::Text, 102, Some(99), &[]));
        assert_eq!(
            StaticUiDocument::from_legacy_ui_ir(legacy_context(&source_refs), &dangling_parent)
                .expect_err("dangling parent")
                .diagnostics()[0]
                .kind(),
            StaticUiIrDiagnosticKind::DanglingLegacyParent
        );
    }

    #[test]
    fn static_ir_legacy_adapter_rejects_child_derived_multiple_parents() {
        let source_refs = [
            (UiAstNodeId::new(101), source_ref(0, 1)),
            (UiAstNodeId::new(102), source_ref(2, 3)),
            (UiAstNodeId::new(103), source_ref(4, 5)),
            (UiAstNodeId::new(104), source_ref(6, 7)),
        ];
        let mut ir = UiIr::new();
        let mut root = UiIrNode::new(UiIrNodeId::new(1), UiIrNodeKind::Root);
        root.set_source_ast_node_id(Some(UiAstNodeId::new(101)));
        root.push_child(UiIrNodeId::new(2));
        root.push_child(UiIrNodeId::new(3));
        ir.push_node(root);
        let mut first =
            UiIrNode::with_parent(UiIrNodeId::new(2), UiIrNodeKind::Text, UiIrNodeId::new(1));
        first.set_source_ast_node_id(Some(UiAstNodeId::new(102)));
        first.push_child(UiIrNodeId::new(4));
        ir.push_node(first);
        let mut second =
            UiIrNode::with_parent(UiIrNodeId::new(3), UiIrNodeKind::Text, UiIrNodeId::new(1));
        second.set_source_ast_node_id(Some(UiAstNodeId::new(103)));
        second.push_child(UiIrNodeId::new(4));
        ir.push_node(second);
        let mut child =
            UiIrNode::with_parent(UiIrNodeId::new(4), UiIrNodeKind::Text, UiIrNodeId::new(2));
        child.set_source_ast_node_id(Some(UiAstNodeId::new(104)));
        ir.push_node(child);

        assert_eq!(
            StaticUiDocument::from_legacy_ui_ir(legacy_context(&source_refs), &ir)
                .expect_err("multiple parents")
                .diagnostics()[0]
                .kind(),
            StaticUiIrDiagnosticKind::LegacyMultipleParents
        );
    }
}
