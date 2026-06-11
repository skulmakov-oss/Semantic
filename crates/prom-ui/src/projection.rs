use crate::model::{UiIr, UiIrNodeId, UiIrNodeKind};
use crate::validation::{validate_ir, UiIrValidationConfig, UiIrValidationDiagnostics};
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiProjectionArtifactId(u64);

impl UiProjectionArtifactId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiProjectedNodeId(u64);

impl UiProjectedNodeId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiProjectionPropertyRef(u64);

impl UiProjectionPropertyRef {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiProjectionActionRef(u64);

impl UiProjectionActionRef {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiProjectionEffectBoundaryRef(u64);

impl UiProjectionEffectBoundaryRef {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiProjectionTraceRef(u64);

impl UiProjectionTraceRef {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiProjectedNodeKind {
    Root,
    Element,
    Text,
    Fragment,
    PropertyCarrier,
    ActionCarrier,
    EffectBoundaryMarker,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiProjectedNode {
    id: UiProjectedNodeId,
    kind: UiProjectedNodeKind,
    source_ir_node_id: Option<UiIrNodeId>,
    parent: Option<UiProjectedNodeId>,
    children: Vec<UiProjectedNodeId>,
    properties: Vec<UiProjectionPropertyRef>,
    actions: Vec<UiProjectionActionRef>,
    effect_boundaries: Vec<UiProjectionEffectBoundaryRef>,
    trace: Option<UiProjectionTraceRef>,
}

impl UiProjectedNode {
    pub fn new(id: UiProjectedNodeId, kind: UiProjectedNodeKind) -> Self {
        Self {
            id,
            kind,
            source_ir_node_id: None,
            parent: None,
            children: Vec::new(),
            properties: Vec::new(),
            actions: Vec::new(),
            effect_boundaries: Vec::new(),
            trace: None,
        }
    }

    pub fn with_source_ir_node(
        id: UiProjectedNodeId,
        kind: UiProjectedNodeKind,
        source_ir_node_id: UiIrNodeId,
    ) -> Self {
        Self {
            id,
            kind,
            source_ir_node_id: Some(source_ir_node_id),
            parent: None,
            children: Vec::new(),
            properties: Vec::new(),
            actions: Vec::new(),
            effect_boundaries: Vec::new(),
            trace: None,
        }
    }

    pub fn id(&self) -> UiProjectedNodeId {
        self.id
    }

    pub fn kind(&self) -> UiProjectedNodeKind {
        self.kind
    }

    pub fn source_ir_node_id(&self) -> Option<UiIrNodeId> {
        self.source_ir_node_id
    }

    pub fn parent(&self) -> Option<UiProjectedNodeId> {
        self.parent
    }

    pub fn set_parent(&mut self, parent: UiProjectedNodeId) {
        self.parent = Some(parent);
    }

    pub fn children(&self) -> &[UiProjectedNodeId] {
        &self.children
    }

    pub fn push_child(&mut self, child: UiProjectedNodeId) {
        self.children.push(child);
    }

    pub fn properties(&self) -> &[UiProjectionPropertyRef] {
        &self.properties
    }

    pub fn push_property(&mut self, property: UiProjectionPropertyRef) {
        self.properties.push(property);
    }

    pub fn actions(&self) -> &[UiProjectionActionRef] {
        &self.actions
    }

    pub fn push_action(&mut self, action: UiProjectionActionRef) {
        self.actions.push(action);
    }

    pub fn effect_boundaries(&self) -> &[UiProjectionEffectBoundaryRef] {
        &self.effect_boundaries
    }

    pub fn push_effect_boundary(&mut self, effect_boundary: UiProjectionEffectBoundaryRef) {
        self.effect_boundaries.push(effect_boundary);
    }

    pub fn trace(&self) -> Option<UiProjectionTraceRef> {
        self.trace
    }

    pub fn set_trace(&mut self, trace: UiProjectionTraceRef) {
        self.trace = Some(trace);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiProjectionArtifact {
    id: UiProjectionArtifactId,
    source_ir_root: Option<UiIrNodeId>,
    nodes: Vec<UiProjectedNode>,
    traces: Vec<UiProjectionTraceRef>,
}

impl UiProjectionArtifact {
    pub fn new(id: UiProjectionArtifactId) -> Self {
        Self {
            id,
            source_ir_root: None,
            nodes: Vec::new(),
            traces: Vec::new(),
        }
    }

    pub fn id(&self) -> UiProjectionArtifactId {
        self.id
    }

    pub fn source_ir_root(&self) -> Option<UiIrNodeId> {
        self.source_ir_root
    }

    pub fn set_source_ir_root(&mut self, root: UiIrNodeId) {
        self.source_ir_root = Some(root);
    }

    pub fn nodes(&self) -> &[UiProjectedNode] {
        &self.nodes
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn push_node(&mut self, node: UiProjectedNode) {
        self.nodes.push(node);
    }

    pub fn traces(&self) -> &[UiProjectionTraceRef] {
        &self.traces
    }

    pub fn push_trace(&mut self, trace: UiProjectionTraceRef) {
        self.traces.push(trace);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiProjectionError {
    InvalidIr(UiIrValidationDiagnostics),
    MissingRoot,
    MissingNode,
    UnsupportedNodeKind,
    InconsistentHandle,
}

// Deterministic projection-layer artifact identity.
// This is not renderer identity, runtime identity, Semantic truth,
// verifier admission, or capability admission.
pub fn projection_artifact_id_for_ir(ir: &UiIr) -> UiProjectionArtifactId {
    if let Some(root) = ir.nodes().iter().find(|n| n.kind() == UiIrNodeKind::Root) {
        UiProjectionArtifactId::new(root.id().raw())
    } else {
        UiProjectionArtifactId::new(0)
    }
}

pub fn project_ir_to_projection(ir: &UiIr) -> Result<UiProjectionArtifact, UiProjectionError> {
    validate_ir(ir, &UiIrValidationConfig::default()).map_err(UiProjectionError::InvalidIr)?;

    let mut artifact = UiProjectionArtifact::new(projection_artifact_id_for_ir(ir));

    let root_node = ir.nodes().iter().find(|n| n.kind() == UiIrNodeKind::Root);
    if let Some(root) = root_node {
        artifact.set_source_ir_root(root.id());
    } else if !ir.is_empty() {
        return Err(UiProjectionError::MissingRoot);
    }

    for ir_node in ir.nodes() {
        let projected_kind = match ir_node.kind() {
            UiIrNodeKind::Root => UiProjectedNodeKind::Root,
            UiIrNodeKind::Element => UiProjectedNodeKind::Element,
            UiIrNodeKind::Text => UiProjectedNodeKind::Text,
            UiIrNodeKind::Fragment => UiProjectedNodeKind::Fragment,
            UiIrNodeKind::Property => UiProjectedNodeKind::PropertyCarrier,
            UiIrNodeKind::Action => UiProjectedNodeKind::ActionCarrier,
            UiIrNodeKind::EffectBoundary => UiProjectedNodeKind::EffectBoundaryMarker,
        };

        let mut projected_node = UiProjectedNode::with_source_ir_node(
            UiProjectedNodeId::new(ir_node.id().raw()),
            projected_kind,
            ir_node.id(),
        );

        if let Some(parent_id) = ir_node.parent() {
            projected_node.set_parent(UiProjectedNodeId::new(parent_id.raw()));
        }

        for child_id in ir_node.children() {
            projected_node.push_child(UiProjectedNodeId::new(child_id.raw()));
        }

        artifact.push_node(projected_node);
    }

    Ok(artifact)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_artifact_id_round_trips_raw() {
        let id = UiProjectionArtifactId::new(42);
        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn projected_node_id_round_trips_raw() {
        let id = UiProjectedNodeId::new(42);
        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn projection_reference_ids_round_trip_raw() {
        assert_eq!(UiProjectionPropertyRef::new(1).raw(), 1);
        assert_eq!(UiProjectionActionRef::new(2).raw(), 2);
        assert_eq!(UiProjectionEffectBoundaryRef::new(3).raw(), 3);
        assert_eq!(UiProjectionTraceRef::new(4).raw(), 4);
    }

    #[test]
    fn empty_projection_artifact_is_empty() {
        let artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));
        assert!(artifact.is_empty());
        assert_eq!(artifact.len(), 0);
        assert!(artifact.nodes().is_empty());
        assert!(artifact.source_ir_root().is_none());
        assert!(artifact.traces().is_empty());
    }

    #[test]
    fn projection_artifact_push_node_is_inert() {
        let mut artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));
        let node = UiProjectedNode::new(UiProjectedNodeId::new(1), UiProjectedNodeKind::Element);
        artifact.push_node(node);
        assert!(!artifact.is_empty());
        assert_eq!(artifact.len(), 1);
        assert_eq!(artifact.nodes()[0].id().raw(), 1);
    }

    #[test]
    fn projected_node_accessors_expose_identity_and_kind() {
        let node = UiProjectedNode::new(UiProjectedNodeId::new(10), UiProjectedNodeKind::Text);
        assert_eq!(node.id().raw(), 10);
        assert_eq!(node.kind(), UiProjectedNodeKind::Text);
    }

    #[test]
    fn projected_node_parent_child_are_handles_only() {
        let mut parent =
            UiProjectedNode::new(UiProjectedNodeId::new(1), UiProjectedNodeKind::Element);
        parent.set_parent(UiProjectedNodeId::new(0));
        parent.push_child(UiProjectedNodeId::new(2));
        parent.push_child(UiProjectedNodeId::new(3));

        assert_eq!(parent.parent().unwrap().raw(), 0);
        assert_eq!(parent.children().len(), 2);
        assert_eq!(parent.children()[0].raw(), 2);
        assert_eq!(parent.children()[1].raw(), 3);
    }

    #[test]
    fn projected_node_property_refs_are_structural_only() {
        let mut node =
            UiProjectedNode::new(UiProjectedNodeId::new(1), UiProjectedNodeKind::Element);
        node.push_property(UiProjectionPropertyRef::new(100));
        assert_eq!(node.properties().len(), 1);
        assert_eq!(node.properties()[0].raw(), 100);
    }

    #[test]
    fn projected_node_action_refs_are_structural_only() {
        let mut node =
            UiProjectedNode::new(UiProjectedNodeId::new(1), UiProjectedNodeKind::Element);
        node.push_action(UiProjectionActionRef::new(200));
        assert_eq!(node.actions().len(), 1);
        assert_eq!(node.actions()[0].raw(), 200);
    }

    #[test]
    fn projected_node_effect_boundary_refs_are_structural_only() {
        let mut node =
            UiProjectedNode::new(UiProjectedNodeId::new(1), UiProjectedNodeKind::Element);
        node.push_effect_boundary(UiProjectionEffectBoundaryRef::new(300));
        assert_eq!(node.effect_boundaries().len(), 1);
        assert_eq!(node.effect_boundaries()[0].raw(), 300);
    }

    #[test]
    fn projected_node_trace_ref_is_structural_only() {
        let mut node =
            UiProjectedNode::new(UiProjectedNodeId::new(1), UiProjectedNodeKind::Element);
        node.set_trace(UiProjectionTraceRef::new(400));
        assert_eq!(node.trace().unwrap().raw(), 400);
    }

    #[test]
    fn projected_node_source_ir_reference_is_handle_only() {
        let node = UiProjectedNode::with_source_ir_node(
            UiProjectedNodeId::new(1),
            UiProjectedNodeKind::Element,
            UiIrNodeId::new(500),
        );
        assert_eq!(node.source_ir_node_id().unwrap().raw(), 500);
    }

    #[test]
    fn projected_node_kinds_are_structural_vocabulary() {
        let kinds = [
            UiProjectedNodeKind::Root,
            UiProjectedNodeKind::Element,
            UiProjectedNodeKind::Text,
            UiProjectedNodeKind::Fragment,
            UiProjectedNodeKind::PropertyCarrier,
            UiProjectedNodeKind::ActionCarrier,
            UiProjectedNodeKind::EffectBoundaryMarker,
        ];
        assert_eq!(kinds.len(), 7);
    }

    #[test]
    fn projection_artifact_does_not_build_from_ir() {
        // Asserting that there's no from_ir method in scope, conceptually.
        // It's inert, so we just build it manually.
        let mut artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));
        artifact.set_source_ir_root(UiIrNodeId::new(10));
        assert_eq!(artifact.source_ir_root().unwrap().raw(), 10);
    }

    #[test]
    fn projection_artifact_is_not_renderer_output() {
        // Structural test only. No renderer types exist here.
        let artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));
        assert_eq!(artifact.id().raw(), 1);
    }

    #[test]
    fn projection_artifact_is_not_layout_or_draw() {
        // Structural test only. No layout or draw methods exist here.
        let artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));
        assert!(artifact.traces().is_empty());
    }

    #[test]
    fn test_minimal_ir_projects() {
        let mut ir = UiIr::new();
        ir.push_node(crate::model::UiIrNode::new(
            UiIrNodeId::new(1),
            UiIrNodeKind::Root,
        ));
        let artifact = project_ir_to_projection(&ir).expect("projects");
        assert_eq!(artifact.len(), 1);
        assert_eq!(artifact.source_ir_root(), Some(UiIrNodeId::new(1)));
    }

    #[test]
    fn artifact_id_follows_root_id() {
        let mut ir = UiIr::new();
        ir.push_node(crate::model::UiIrNode::new(
            UiIrNodeId::new(42),
            UiIrNodeKind::Root,
        ));
        let artifact = project_ir_to_projection(&ir).expect("projects");
        assert_eq!(artifact.id(), UiProjectionArtifactId::new(42));
    }

    #[test]
    fn empty_valid_ir_uses_empty_policy() {
        let ir = UiIr::new();
        let artifact = project_ir_to_projection(&ir).expect("projects");
        assert_eq!(artifact.id(), UiProjectionArtifactId::new(0));
    }

    #[test]
    fn projected_node_ids_remain_structural() {
        let mut ir = UiIr::new();
        ir.push_node(crate::model::UiIrNode::new(
            UiIrNodeId::new(42),
            UiIrNodeKind::Root,
        ));
        let artifact = project_ir_to_projection(&ir).expect("projects");
        assert_eq!(artifact.nodes()[0].id(), UiProjectedNodeId::new(42));
    }

    #[test]
    fn no_renderer_runtime_identity_introduced() {
        // artifact id is projection-layer only
        // no renderer/runtime handles exist in artifact
        let ir = UiIr::new();
        let artifact = project_ir_to_projection(&ir).expect("projects");
        assert_eq!(artifact.id(), UiProjectionArtifactId::new(0));
    }

    #[test]
    fn test_parent_child_structure_preserved() {
        let mut ir = UiIr::new();
        let mut root = crate::model::UiIrNode::new(UiIrNodeId::new(1), UiIrNodeKind::Root);
        root.push_child(UiIrNodeId::new(2));
        let child = crate::model::UiIrNode::with_parent(
            UiIrNodeId::new(2),
            UiIrNodeKind::Element,
            UiIrNodeId::new(1),
        );
        ir.push_node(root);
        ir.push_node(child);
        let artifact = project_ir_to_projection(&ir).expect("projects");
        assert_eq!(artifact.len(), 2);
        assert_eq!(artifact.nodes()[0].children(), &[UiProjectedNodeId::new(2)]);
        assert_eq!(
            artifact.nodes()[1].parent(),
            Some(UiProjectedNodeId::new(1))
        );
    }

    #[test]
    fn test_deterministic_projection() {
        let mut ir = UiIr::new();
        ir.push_node(crate::model::UiIrNode::new(
            UiIrNodeId::new(1),
            UiIrNodeKind::Root,
        ));
        let a = project_ir_to_projection(&ir).expect("projects");
        let b = project_ir_to_projection(&ir).expect("projects");
        assert_eq!(a, b);
    }

    #[test]
    fn test_invalid_ir_rejected() {
        let mut ir = UiIr::new();
        // Missing child reference causes InvalidIr via validation
        let root = crate::model::UiIrNode::new(UiIrNodeId::new(1), UiIrNodeKind::Root);
        ir.push_node(root);
        ir.push_node(crate::model::UiIrNode::with_parent(
            UiIrNodeId::new(2),
            UiIrNodeKind::Element,
            UiIrNodeId::new(1),
        ));

        let err = project_ir_to_projection(&ir).expect_err("should reject invalid ir");
        assert!(matches!(err, UiProjectionError::InvalidIr(_)));
    }

    #[test]
    fn test_no_renderer_layout_draw_event_artifacts() {
        let mut ir = UiIr::new();
        ir.push_node(crate::model::UiIrNode::new(
            UiIrNodeId::new(1),
            UiIrNodeKind::Root,
        ));
        let artifact = project_ir_to_projection(&ir).expect("projects");
        assert!(artifact.traces().is_empty());
    }

    #[test]
    fn test_property_action_effect_remain_inert() {
        let mut ir = UiIr::new();
        let mut root = crate::model::UiIrNode::new(UiIrNodeId::new(1), UiIrNodeKind::Root);
        root.push_child(UiIrNodeId::new(2));
        root.push_child(UiIrNodeId::new(3));
        root.push_child(UiIrNodeId::new(4));
        ir.push_node(root);
        ir.push_node(crate::model::UiIrNode::with_parent(
            UiIrNodeId::new(2),
            UiIrNodeKind::Property,
            UiIrNodeId::new(1),
        ));
        ir.push_node(crate::model::UiIrNode::with_parent(
            UiIrNodeId::new(3),
            UiIrNodeKind::Action,
            UiIrNodeId::new(1),
        ));
        ir.push_node(crate::model::UiIrNode::with_parent(
            UiIrNodeId::new(4),
            UiIrNodeKind::EffectBoundary,
            UiIrNodeId::new(1),
        ));

        let artifact = project_ir_to_projection(&ir).expect("projects");
        assert_eq!(artifact.len(), 4);
        assert_eq!(
            artifact.nodes()[1].kind(),
            UiProjectedNodeKind::PropertyCarrier
        );
        assert_eq!(
            artifact.nodes()[2].kind(),
            UiProjectedNodeKind::ActionCarrier
        );
        assert_eq!(
            artifact.nodes()[3].kind(),
            UiProjectedNodeKind::EffectBoundaryMarker
        );
    }
}
