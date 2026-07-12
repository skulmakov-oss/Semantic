//! UI DNA v2 Projection Source contracts.
//!
//! This is a programmatically constructible source-facing AST and validation
//! surface. It is not a textual `.proj.sm` parser and it is not an alias for
//! the legacy `UiAst` substrate.
#![allow(dead_code)]

use alloc::{boxed::Box, vec::Vec};

use crate::contract_primitives::{
    ChildOrder, CollectionKey, DiagnosticCode, DiagnosticCoordinate, Epoch, ProjectionSourceNodeId,
    ProjectionSourceSurfaceId, Revision, SourceId, SourceRef,
};
use crate::role_dictionary::{RoleDictionary, RoleId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ProjectionSourceTokenKind {
    Surface,
    Node,
    Role,
    Key,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ProjectionSourceRoleName(Box<str>);

impl ProjectionSourceRoleName {
    pub(crate) fn new(raw: &str) -> Self {
        Self(raw.into())
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ProjectionSourceToken {
    kind: ProjectionSourceTokenKind,
    source: SourceRef,
}

impl ProjectionSourceToken {
    pub(crate) const fn new(kind: ProjectionSourceTokenKind, source: SourceRef) -> Self {
        Self { kind, source }
    }

    pub(crate) const fn kind(self) -> ProjectionSourceTokenKind {
        self.kind
    }

    pub(crate) const fn source(self) -> SourceRef {
        self.source
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ProjectionSourceSurface {
    id: ProjectionSourceSurfaceId,
    root: ProjectionSourceNodeId,
    key: CollectionKey,
    source: SourceRef,
}

impl ProjectionSourceSurface {
    pub(crate) const fn new(
        id: ProjectionSourceSurfaceId,
        root: ProjectionSourceNodeId,
        key: CollectionKey,
        source: SourceRef,
    ) -> Self {
        Self {
            id,
            root,
            key,
            source,
        }
    }

    pub(crate) const fn id(&self) -> ProjectionSourceSurfaceId {
        self.id
    }

    pub(crate) const fn root(&self) -> ProjectionSourceNodeId {
        self.root
    }

    pub(crate) const fn key(&self) -> CollectionKey {
        self.key
    }

    pub(crate) const fn source(&self) -> SourceRef {
        self.source
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ProjectionSourceNode {
    id: ProjectionSourceNodeId,
    role: ProjectionSourceRoleName,
    key: CollectionKey,
    source: SourceRef,
    children: Vec<ProjectionSourceChild>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ProjectionSourceChild {
    child: ProjectionSourceNodeId,
    order: ChildOrder,
}

impl ProjectionSourceChild {
    pub(crate) const fn new(child: ProjectionSourceNodeId, order: ChildOrder) -> Self {
        Self { child, order }
    }

    pub(crate) const fn child(self) -> ProjectionSourceNodeId {
        self.child
    }

    pub(crate) const fn order(self) -> ChildOrder {
        self.order
    }
}

impl ProjectionSourceNode {
    pub(crate) fn new(
        id: ProjectionSourceNodeId,
        role: ProjectionSourceRoleName,
        key: CollectionKey,
        source: SourceRef,
    ) -> Self {
        Self {
            id,
            role,
            key,
            source,
            children: Vec::new(),
        }
    }

    pub(crate) const fn id(&self) -> ProjectionSourceNodeId {
        self.id
    }

    pub(crate) fn role(&self) -> &ProjectionSourceRoleName {
        &self.role
    }

    pub(crate) fn resolved_role(
        &self,
        dictionary: RoleDictionary,
    ) -> Result<RoleId, ProjectionSourceDiagnostic> {
        dictionary.resolve_name(self.role.as_str()).ok_or_else(|| {
            ProjectionSourceDiagnostic::new(
                Some(self.source),
                ProjectionSourceDiagnosticKind::UnknownRole,
                self.key.raw(),
                0,
            )
        })
    }

    pub(crate) const fn key(&self) -> CollectionKey {
        self.key
    }

    pub(crate) const fn source(&self) -> SourceRef {
        self.source
    }

    pub(crate) fn children(&self) -> &[ProjectionSourceChild] {
        &self.children
    }

    pub(crate) fn push_child(&mut self, child: ProjectionSourceChild) {
        self.children.push(child);
    }

    fn normalize(&mut self) {
        self.children.sort_by_key(|edge| edge.order);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ProjectionSourceDocument {
    source_id: SourceId,
    revision: Revision,
    epoch: Epoch,
    surfaces: Vec<ProjectionSourceSurface>,
    nodes: Vec<ProjectionSourceNode>,
}

impl ProjectionSourceDocument {
    pub(crate) fn new(source_id: SourceId, revision: Revision, epoch: Epoch) -> Self {
        Self {
            source_id,
            revision,
            epoch,
            surfaces: Vec::new(),
            nodes: Vec::new(),
        }
    }

    pub(crate) const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub(crate) const fn revision(&self) -> Revision {
        self.revision
    }

    pub(crate) const fn epoch(&self) -> Epoch {
        self.epoch
    }

    pub(crate) fn surfaces(&self) -> &[ProjectionSourceSurface] {
        &self.surfaces
    }

    pub(crate) fn nodes(&self) -> &[ProjectionSourceNode] {
        &self.nodes
    }

    pub(crate) fn push_surface(&mut self, surface: ProjectionSourceSurface) {
        self.surfaces.push(surface);
    }

    pub(crate) fn push_node(&mut self, node: ProjectionSourceNode) {
        self.nodes.push(node);
    }

    pub(crate) fn validate(
        &self,
        dictionary: RoleDictionary,
    ) -> Result<(), ProjectionSourceDiagnostics> {
        let diagnostics = validate_source(self, dictionary);
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }

    pub(crate) fn normalized(
        &self,
        dictionary: RoleDictionary,
    ) -> Result<Self, ProjectionSourceDiagnostics> {
        self.validate(dictionary)?;
        let mut normalized = self.clone();
        normalized
            .surfaces
            .sort_by_key(|surface| (surface.key, surface.id));
        normalized.nodes.sort_by_key(|node| (node.key, node.id));
        for node in &mut normalized.nodes {
            node.normalize();
        }
        Ok(normalized)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ProjectionSourceDiagnosticKind {
    DuplicateSurfaceId,
    DuplicateSurfaceKey,
    DuplicateNodeId,
    DuplicateNodeKey,
    MissingRootNode,
    MissingChild,
    DuplicateChild,
    DuplicateChildOrder,
    DuplicateSurfaceRoot,
    RootUsedAsChild,
    MultipleParents,
    Cycle,
    UnreachableNode,
    SharedAcrossSurfaces,
    UnknownRole,
}

impl ProjectionSourceDiagnosticKind {
    const fn code(self) -> DiagnosticCode {
        match self {
            Self::DuplicateSurfaceId => DiagnosticCode::new("PS_DUP_SURFACE_ID"),
            Self::DuplicateSurfaceKey => DiagnosticCode::new("PS_DUP_SURFACE_KEY"),
            Self::DuplicateNodeId => DiagnosticCode::new("PS_DUP_NODE_ID"),
            Self::DuplicateNodeKey => DiagnosticCode::new("PS_DUP_NODE_KEY"),
            Self::MissingRootNode => DiagnosticCode::new("PS_MISSING_ROOT"),
            Self::MissingChild => DiagnosticCode::new("PS_MISSING_CHILD"),
            Self::DuplicateChild => DiagnosticCode::new("PS_DUP_CHILD"),
            Self::DuplicateChildOrder => DiagnosticCode::new("PS_DUP_CHILD_ORDER"),
            Self::DuplicateSurfaceRoot => DiagnosticCode::new("PS_DUP_SURFACE_ROOT"),
            Self::RootUsedAsChild => DiagnosticCode::new("PS_ROOT_USED_AS_CHILD"),
            Self::MultipleParents => DiagnosticCode::new("PS_MULTIPLE_PARENTS"),
            Self::Cycle => DiagnosticCode::new("PS_CYCLE"),
            Self::UnreachableNode => DiagnosticCode::new("PS_UNREACHABLE_NODE"),
            Self::SharedAcrossSurfaces => DiagnosticCode::new("PS_SHARED_ACROSS_SURFACES"),
            Self::UnknownRole => DiagnosticCode::new("PS_UNKNOWN_ROLE"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ProjectionSourceDiagnostic {
    coordinate: DiagnosticCoordinate,
    kind: ProjectionSourceDiagnosticKind,
}

impl ProjectionSourceDiagnostic {
    fn new(
        source: Option<SourceRef>,
        kind: ProjectionSourceDiagnosticKind,
        domain: u64,
        secondary: u64,
    ) -> Self {
        Self {
            coordinate: DiagnosticCoordinate::new(source, kind.code(), domain, secondary),
            kind,
        }
    }

    pub(crate) const fn kind(self) -> ProjectionSourceDiagnosticKind {
        self.kind
    }

    pub(crate) const fn coordinate(self) -> DiagnosticCoordinate {
        self.coordinate
    }
}

impl Ord for ProjectionSourceDiagnostic {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (self.coordinate, self.kind).cmp(&(other.coordinate, other.kind))
    }
}

impl PartialOrd for ProjectionSourceDiagnostic {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ProjectionSourceDiagnostics {
    diagnostics: Vec<ProjectionSourceDiagnostic>,
}

impl ProjectionSourceDiagnostics {
    pub(crate) fn single(diagnostic: ProjectionSourceDiagnostic) -> Self {
        Self::new(Vec::from([diagnostic]))
    }

    fn new(mut diagnostics: Vec<ProjectionSourceDiagnostic>) -> Self {
        diagnostics.sort();
        diagnostics.dedup();
        Self { diagnostics }
    }

    pub(crate) fn diagnostics(&self) -> &[ProjectionSourceDiagnostic] {
        &self.diagnostics
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

fn validate_source(
    document: &ProjectionSourceDocument,
    dictionary: RoleDictionary,
) -> ProjectionSourceDiagnostics {
    let mut diagnostics = Vec::new();

    for (index, surface) in document.surfaces.iter().enumerate() {
        if document.surfaces[..index]
            .iter()
            .any(|previous| previous.id == surface.id)
        {
            diagnostics.push(ProjectionSourceDiagnostic::new(
                Some(surface.source),
                ProjectionSourceDiagnosticKind::DuplicateSurfaceId,
                surface.id.raw(),
                0,
            ));
        }
        if document.surfaces[..index]
            .iter()
            .any(|previous| previous.key == surface.key)
        {
            diagnostics.push(ProjectionSourceDiagnostic::new(
                Some(surface.source),
                ProjectionSourceDiagnosticKind::DuplicateSurfaceKey,
                surface.key.raw(),
                0,
            ));
        }
        if !document.nodes.iter().any(|node| node.id == surface.root) {
            diagnostics.push(ProjectionSourceDiagnostic::new(
                Some(surface.source),
                ProjectionSourceDiagnosticKind::MissingRootNode,
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
            diagnostics.push(ProjectionSourceDiagnostic::new(
                Some(node.source),
                ProjectionSourceDiagnosticKind::DuplicateNodeId,
                node.id.raw(),
                0,
            ));
        }
        if document.nodes[..index]
            .iter()
            .any(|previous| previous.key == node.key)
        {
            diagnostics.push(ProjectionSourceDiagnostic::new(
                Some(node.source),
                ProjectionSourceDiagnosticKind::DuplicateNodeKey,
                node.key.raw(),
                0,
            ));
        }
        if let Err(diagnostic) = node.resolved_role(dictionary) {
            diagnostics.push(diagnostic);
        }
        for (child_index, child) in node.children.iter().enumerate() {
            if !document
                .nodes
                .iter()
                .any(|candidate| candidate.id == child.child)
            {
                diagnostics.push(ProjectionSourceDiagnostic::new(
                    Some(node.source),
                    ProjectionSourceDiagnosticKind::MissingChild,
                    node.id.raw(),
                    child.child.raw(),
                ));
            }
            if node.children[..child_index]
                .iter()
                .any(|previous| previous.child == child.child)
            {
                diagnostics.push(ProjectionSourceDiagnostic::new(
                    Some(node.source),
                    ProjectionSourceDiagnosticKind::DuplicateChild,
                    node.id.raw(),
                    child.child.raw(),
                ));
            }
            if node.children[..child_index]
                .iter()
                .any(|previous| previous.order == child.order)
            {
                diagnostics.push(ProjectionSourceDiagnostic::new(
                    Some(node.source),
                    ProjectionSourceDiagnosticKind::DuplicateChildOrder,
                    node.id.raw(),
                    u64::from(child.order.raw()),
                ));
            }
        }
    }

    validate_source_forest(document, &mut diagnostics);

    ProjectionSourceDiagnostics::new(diagnostics)
}

fn validate_source_forest(
    document: &ProjectionSourceDocument,
    diagnostics: &mut Vec<ProjectionSourceDiagnostic>,
) {
    if document.surfaces.is_empty() {
        diagnostics.push(ProjectionSourceDiagnostic::new(
            None,
            ProjectionSourceDiagnosticKind::MissingRootNode,
            0,
            0,
        ));
        return;
    }

    let mut root_indices = Vec::new();
    let mut owner: Vec<Option<usize>> = alloc::vec![None; document.nodes.len()];
    let mut parent_count: Vec<u32> = alloc::vec![0; document.nodes.len()];
    let mut visiting: Vec<bool> = alloc::vec![false; document.nodes.len()];
    let mut complete: Vec<bool> = alloc::vec![false; document.nodes.len()];

    for (surface_index, surface) in document.surfaces.iter().enumerate() {
        let Some(root_index) = node_index(document, surface.root) else {
            continue;
        };
        if root_indices.contains(&root_index) {
            diagnostics.push(ProjectionSourceDiagnostic::new(
                Some(surface.source),
                ProjectionSourceDiagnosticKind::DuplicateSurfaceRoot,
                surface.root.raw(),
                surface.id.raw(),
            ));
        }
        root_indices.push(root_index);
        walk_source_forest(
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

    for (index, node) in document.nodes.iter().enumerate() {
        let is_root = root_indices.contains(&index);
        if is_root && parent_count[index] != 0 {
            diagnostics.push(ProjectionSourceDiagnostic::new(
                Some(node.source),
                ProjectionSourceDiagnosticKind::RootUsedAsChild,
                node.id.raw(),
                0,
            ));
        }
        if !is_root && parent_count[index] > 1 {
            diagnostics.push(ProjectionSourceDiagnostic::new(
                Some(node.source),
                ProjectionSourceDiagnosticKind::MultipleParents,
                node.id.raw(),
                u64::from(parent_count[index]),
            ));
        }
        if !is_root && parent_count[index] == 0 {
            diagnostics.push(ProjectionSourceDiagnostic::new(
                Some(node.source),
                ProjectionSourceDiagnosticKind::UnreachableNode,
                node.id.raw(),
                0,
            ));
        }
    }
}

struct SourceTraversalFrame {
    node_index: usize,
    parent: Option<usize>,
    next_child: usize,
    entered: bool,
}

#[allow(clippy::too_many_arguments)]
fn walk_source_forest(
    document: &ProjectionSourceDocument,
    surface_index: usize,
    root_index: usize,
    owner: &mut [Option<usize>],
    parent_count: &mut [u32],
    visiting: &mut [bool],
    complete: &mut [bool],
    diagnostics: &mut Vec<ProjectionSourceDiagnostic>,
) {
    let mut stack = Vec::from([SourceTraversalFrame {
        node_index: root_index,
        parent: None,
        next_child: 0,
        entered: false,
    }]);

    while let Some(mut frame) = stack.pop() {
        let current_index = frame.node_index;
        if !frame.entered {
            frame.entered = true;
            let node = &document.nodes[current_index];
            if let Some(parent) = frame.parent {
                parent_count[current_index] = parent_count[current_index].saturating_add(1);
                if parent == current_index {
                    diagnostics.push(ProjectionSourceDiagnostic::new(
                        Some(node.source),
                        ProjectionSourceDiagnosticKind::Cycle,
                        node.id.raw(),
                        node.id.raw(),
                    ));
                    continue;
                }
            }

            match owner[current_index] {
                Some(previous) if previous != surface_index => {
                    diagnostics.push(ProjectionSourceDiagnostic::new(
                        Some(node.source),
                        ProjectionSourceDiagnosticKind::SharedAcrossSurfaces,
                        node.id.raw(),
                        surface_index as u64,
                    ));
                }
                None => owner[current_index] = Some(surface_index),
                _ => {}
            }

            if visiting[current_index] {
                diagnostics.push(ProjectionSourceDiagnostic::new(
                    Some(node.source),
                    ProjectionSourceDiagnosticKind::Cycle,
                    node.id.raw(),
                    0,
                ));
                continue;
            }
            if complete[current_index] {
                continue;
            }
            visiting[current_index] = true;
        }

        let next_child = frame.next_child;
        let next = document.nodes[current_index]
            .children
            .get(next_child)
            .copied();
        if let Some(child) = next {
            frame.next_child += 1;
            stack.push(frame);
            if let Some(child_index) = node_index(document, child.child) {
                stack.push(SourceTraversalFrame {
                    node_index: child_index,
                    parent: Some(current_index),
                    next_child: 0,
                    entered: false,
                });
            }
        } else {
            visiting[current_index] = false;
            complete[current_index] = true;
        }
    }
}

fn node_index(document: &ProjectionSourceDocument, id: ProjectionSourceNodeId) -> Option<usize> {
    document.nodes.iter().position(|node| node.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract_primitives::{ChildOrder, CollectionKey, SourceSpan};
    use crate::model::UiAst;
    use alloc::string::String;

    fn source_ref(start: u32, end: u32) -> SourceRef {
        SourceRef::new(
            SourceId::new(1).expect("source id"),
            SourceSpan::new(start, end).expect("source span"),
        )
    }

    fn node_id(raw: u64) -> ProjectionSourceNodeId {
        ProjectionSourceNodeId::new(raw).expect("node id")
    }

    fn surface_id(raw: u64) -> ProjectionSourceSurfaceId {
        ProjectionSourceSurfaceId::new(raw).expect("surface id")
    }

    fn key(raw: u64) -> CollectionKey {
        CollectionKey::new(raw).expect("collection key")
    }

    fn minimal_source() -> ProjectionSourceDocument {
        let mut document = ProjectionSourceDocument::new(
            SourceId::new(1).expect("source id"),
            Revision::new(0),
            Epoch::new(0),
        );
        document.push_node(ProjectionSourceNode::new(
            node_id(10),
            ProjectionSourceRoleName::new("root"),
            key(10),
            source_ref(0, 4),
        ));
        document.push_surface(ProjectionSourceSurface::new(
            surface_id(1),
            node_id(10),
            key(1),
            source_ref(0, 4),
        ));
        document
    }

    fn source_with_ordered_children(order_a: u32, order_b: u32) -> ProjectionSourceDocument {
        let mut document = ProjectionSourceDocument::new(
            SourceId::new(1).expect("source id"),
            Revision::new(0),
            Epoch::new(0),
        );
        let mut root = ProjectionSourceNode::new(
            node_id(10),
            ProjectionSourceRoleName::new("root"),
            key(10),
            source_ref(0, 4),
        );
        root.push_child(ProjectionSourceChild::new(
            node_id(20),
            ChildOrder::new(order_a),
        ));
        root.push_child(ProjectionSourceChild::new(
            node_id(30),
            ChildOrder::new(order_b),
        ));
        document.push_node(root);
        document.push_node(ProjectionSourceNode::new(
            node_id(20),
            ProjectionSourceRoleName::new("text"),
            key(20),
            source_ref(10, 11),
        ));
        document.push_node(ProjectionSourceNode::new(
            node_id(30),
            ProjectionSourceRoleName::new("text"),
            key(30),
            source_ref(12, 13),
        ));
        document.push_surface(ProjectionSourceSurface::new(
            surface_id(1),
            node_id(10),
            key(1),
            source_ref(0, 4),
        ));
        document
    }

    #[test]
    fn projection_source_minimal_ast_validates() {
        assert!(minimal_source().validate(RoleDictionary::current()).is_ok());
    }

    #[test]
    fn projection_source_role_name_owns_input_text() {
        let role = {
            let temporary = String::from("renderer_button");
            ProjectionSourceRoleName::new(&temporary)
        };

        assert_eq!(role.as_str(), "renderer_button");
    }

    #[test]
    fn projection_source_role_name_compares_by_text() {
        assert_eq!(
            ProjectionSourceRoleName::new("root"),
            ProjectionSourceRoleName::new("root")
        );
    }

    #[test]
    fn projection_source_rejects_unknown_role() {
        let mut document = minimal_source();
        document.push_node(ProjectionSourceNode::new(
            node_id(11),
            ProjectionSourceRoleName::new("renderer_button"),
            key(11),
            source_ref(5, 9),
        ));

        let diagnostics = document
            .validate(RoleDictionary::current())
            .expect_err("unknown role");

        assert_eq!(
            diagnostics.diagnostics()[0].kind(),
            ProjectionSourceDiagnosticKind::UnknownRole
        );
        let coordinate = diagnostics.diagnostics()[0].coordinate();
        assert_eq!(coordinate.code(), DiagnosticCode::new("PS_UNKNOWN_ROLE"));
        assert_eq!(coordinate.source().expect("source").span().start(), 5);
        assert_eq!(coordinate.source().expect("source").span().end(), 9);
        assert_eq!(coordinate.domain(), 11);
        assert_eq!(coordinate.secondary(), 0);
        assert_eq!(
            document.validate(RoleDictionary::current()),
            Err(diagnostics)
        );
    }

    #[test]
    fn projection_source_normalization_is_insertion_order_independent() {
        let mut first = minimal_source();
        first
            .nodes
            .iter_mut()
            .find(|node| node.id() == node_id(10))
            .expect("root")
            .push_child(ProjectionSourceChild::new(node_id(20), ChildOrder::new(0)));
        first.push_node(ProjectionSourceNode::new(
            node_id(20),
            ProjectionSourceRoleName::new("text"),
            key(20),
            source_ref(10, 11),
        ));

        let mut second = ProjectionSourceDocument::new(
            SourceId::new(1).expect("source id"),
            Revision::new(0),
            Epoch::new(0),
        );
        second.push_node(ProjectionSourceNode::new(
            node_id(20),
            ProjectionSourceRoleName::new("text"),
            key(20),
            source_ref(10, 11),
        ));
        second.push_node(ProjectionSourceNode::new(
            node_id(10),
            ProjectionSourceRoleName::new("root"),
            key(10),
            source_ref(0, 4),
        ));
        second
            .nodes
            .iter_mut()
            .find(|node| node.id() == node_id(10))
            .expect("root")
            .push_child(ProjectionSourceChild::new(node_id(20), ChildOrder::new(0)));
        second.push_surface(ProjectionSourceSurface::new(
            surface_id(1),
            node_id(10),
            key(1),
            source_ref(0, 4),
        ));

        assert_eq!(
            first.normalized(RoleDictionary::current()).expect("first"),
            second
                .normalized(RoleDictionary::current())
                .expect("second")
        );
    }

    #[test]
    fn projection_source_storage_permutation_preserves_explicit_child_order() {
        let first = source_with_ordered_children(0, 1);
        let mut second = ProjectionSourceDocument::new(
            SourceId::new(1).expect("source id"),
            Revision::new(0),
            Epoch::new(0),
        );
        second.push_node(ProjectionSourceNode::new(
            node_id(30),
            ProjectionSourceRoleName::new("text"),
            key(30),
            source_ref(12, 13),
        ));
        let mut root = ProjectionSourceNode::new(
            node_id(10),
            ProjectionSourceRoleName::new("root"),
            key(10),
            source_ref(0, 4),
        );
        root.push_child(ProjectionSourceChild::new(node_id(30), ChildOrder::new(1)));
        root.push_child(ProjectionSourceChild::new(node_id(20), ChildOrder::new(0)));
        second.push_node(root);
        second.push_node(ProjectionSourceNode::new(
            node_id(20),
            ProjectionSourceRoleName::new("text"),
            key(20),
            source_ref(10, 11),
        ));
        second.push_surface(ProjectionSourceSurface::new(
            surface_id(1),
            node_id(10),
            key(1),
            source_ref(0, 4),
        ));

        assert_eq!(
            first.normalized(RoleDictionary::current()).expect("first"),
            second
                .normalized(RoleDictionary::current())
                .expect("second")
        );
    }

    #[test]
    fn projection_source_semantic_child_order_changes_normalized_source() {
        let first = source_with_ordered_children(0, 1);
        let second = source_with_ordered_children(1, 0);

        assert_ne!(
            first.normalized(RoleDictionary::current()).expect("first"),
            second
                .normalized(RoleDictionary::current())
                .expect("second")
        );
    }

    #[test]
    fn projection_source_rejects_duplicate_child_and_order() {
        let mut document = source_with_ordered_children(0, 0);
        document
            .nodes
            .iter_mut()
            .find(|node| node.id() == node_id(10))
            .expect("root")
            .push_child(ProjectionSourceChild::new(node_id(20), ChildOrder::new(2)));

        let diagnostics = document
            .validate(RoleDictionary::current())
            .expect_err("duplicates");

        assert!(diagnostics
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.kind() == ProjectionSourceDiagnosticKind::DuplicateChild));
        assert!(diagnostics
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.kind()
                == ProjectionSourceDiagnosticKind::DuplicateChildOrder));
    }

    #[test]
    fn projection_source_rejects_ordered_forest_violations() {
        let mut cycle = minimal_source();
        cycle
            .nodes
            .iter_mut()
            .find(|node| node.id() == node_id(10))
            .expect("root")
            .push_child(ProjectionSourceChild::new(node_id(10), ChildOrder::new(0)));
        assert!(cycle
            .validate(RoleDictionary::current())
            .expect_err("cycle")
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.kind() == ProjectionSourceDiagnosticKind::Cycle));

        let mut orphan = minimal_source();
        orphan.push_node(ProjectionSourceNode::new(
            node_id(40),
            ProjectionSourceRoleName::new("text"),
            key(40),
            source_ref(20, 21),
        ));
        assert!(
            orphan
                .validate(RoleDictionary::current())
                .expect_err("orphan")
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.kind()
                    == ProjectionSourceDiagnosticKind::UnreachableNode)
        );
    }

    #[test]
    fn projection_source_diagnostics_are_deterministically_ordered() {
        let mut document = minimal_source();
        document.push_surface(ProjectionSourceSurface::new(
            surface_id(1),
            node_id(99),
            key(1),
            source_ref(20, 21),
        ));

        let diagnostics = document
            .validate(RoleDictionary::current())
            .expect_err("duplicates");

        assert_eq!(
            diagnostics.diagnostics()[0].kind(),
            ProjectionSourceDiagnosticKind::DuplicateSurfaceId
        );
        assert_eq!(
            diagnostics.diagnostics()[1].kind(),
            ProjectionSourceDiagnosticKind::DuplicateSurfaceKey
        );
    }

    #[test]
    fn projection_source_is_not_legacy_ui_ast_alias() {
        let source = minimal_source();
        let legacy = UiAst::new();

        assert_eq!(source.nodes().len(), 1);
        assert!(legacy.nodes().is_empty());
    }

    #[test]
    fn projection_source_token_preserves_source_reference() {
        let token = ProjectionSourceToken::new(ProjectionSourceTokenKind::Role, source_ref(1, 2));

        assert_eq!(token.kind(), ProjectionSourceTokenKind::Role);
        assert_eq!(token.source().span().start(), 1);
    }

    #[test]
    fn projection_source_validates_deep_forest_without_recursive_traversal() {
        const DEPTH: u64 = 512;
        let mut document = ProjectionSourceDocument::new(
            SourceId::new(1).expect("source id"),
            Revision::new(0),
            Epoch::new(0),
        );
        for raw in 1..=DEPTH {
            let mut node = ProjectionSourceNode::new(
                node_id(raw),
                if raw == 1 {
                    ProjectionSourceRoleName::new("root")
                } else {
                    ProjectionSourceRoleName::new("text")
                },
                key(raw),
                source_ref(raw as u32, raw as u32 + 1),
            );
            if raw < DEPTH {
                node.push_child(ProjectionSourceChild::new(
                    node_id(raw + 1),
                    ChildOrder::new(0),
                ));
            }
            document.push_node(node);
        }
        document.push_surface(ProjectionSourceSurface::new(
            surface_id(1),
            node_id(1),
            key(1),
            source_ref(0, 1),
        ));

        assert!(document.validate(RoleDictionary::current()).is_ok());
    }

    #[test]
    fn projection_source_reports_deep_cycle_deterministically() {
        const DEPTH: u64 = 512;
        let mut document = ProjectionSourceDocument::new(
            SourceId::new(1).expect("source id"),
            Revision::new(0),
            Epoch::new(0),
        );
        for raw in 1..=DEPTH {
            let mut node = ProjectionSourceNode::new(
                node_id(raw),
                if raw == 1 {
                    ProjectionSourceRoleName::new("root")
                } else {
                    ProjectionSourceRoleName::new("text")
                },
                key(raw),
                source_ref(raw as u32, raw as u32 + 1),
            );
            let child = (raw < DEPTH)
                .then_some(raw + 1)
                .or((raw == DEPTH).then_some(1));
            if let Some(child) = child {
                node.push_child(ProjectionSourceChild::new(
                    node_id(child),
                    ChildOrder::new(0),
                ));
            }
            document.push_node(node);
        }
        document.push_surface(ProjectionSourceSurface::new(
            surface_id(1),
            node_id(1),
            key(1),
            source_ref(0, 1),
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
            .any(|diagnostic| diagnostic.kind() == ProjectionSourceDiagnosticKind::Cycle));
    }
}
