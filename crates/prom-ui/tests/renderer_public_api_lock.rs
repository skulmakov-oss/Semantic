use prom_ui::model::UiIrNodeId;
use prom_ui::projection::{
    UiProjectedNode, UiProjectedNodeId, UiProjectedNodeKind, UiProjectionArtifact,
    UiProjectionArtifactId,
};
use prom_ui::renderer::{
    render_projection_to_model, UiRenderError, UiRenderMarker, UiRenderModel, UiRenderModelId,
    UiRenderNode, UiRenderNodeId, UiRenderNodeKind,
};

// ============================================================================
// PART 1: PUBLIC SIGNATURE LOCKS
// ============================================================================

#[test]
fn renderer_entrypoint_signature_is_locked() {
    let _: fn(&UiProjectionArtifact) -> Result<UiRenderModel, UiRenderError> =
        render_projection_to_model;
}

#[test]
fn renderer_public_types_are_locked() {
    fn assert_model(_: &UiRenderModel) {}
    fn assert_node(_: &UiRenderNode) {}
    fn assert_model_id(_: UiRenderModelId) {}
    fn assert_node_id(_: UiRenderNodeId) {}
    fn assert_node_kind(_: UiRenderNodeKind) {}
    fn assert_marker(_: UiRenderMarker) {}
    fn assert_error(_: UiRenderError) {}

    let _ = (
        assert_model,
        assert_node,
        assert_model_id,
        assert_node_id,
        assert_node_kind,
        assert_marker,
        assert_error,
    );
}

#[test]
fn render_model_accessor_surface_is_locked() {
    let _: fn(&UiRenderModel) -> UiRenderModelId = UiRenderModel::id;
    let _: fn(&UiRenderModel) -> UiProjectionArtifactId = UiRenderModel::source_projection;
    let _: fn(&UiRenderModel) -> Option<UiIrNodeId> = UiRenderModel::source_ir_root;
    let _: fn(&UiRenderModel) -> &[UiRenderNode] = UiRenderModel::nodes;
}

#[test]
fn render_node_accessor_surface_is_locked() {
    let _: fn(&UiRenderNode) -> UiRenderNodeId = UiRenderNode::id;
    let _: fn(&UiRenderNode) -> UiProjectedNodeId = UiRenderNode::source_projection_node;
    let _: fn(&UiRenderNode) -> Option<UiIrNodeId> = UiRenderNode::source_ir_node;
    let _: fn(&UiRenderNode) -> UiRenderNodeKind = UiRenderNode::kind;
    let _: fn(&UiRenderNode) -> &[UiRenderMarker] = UiRenderNode::markers;
}

// ============================================================================
// PART 2: BEHAVIORAL SMOKE TESTS
// ============================================================================

#[test]
fn test_render_model_builds_from_valid_projection() {
    let mut artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(42));
    artifact.set_source_ir_root(UiIrNodeId::new(10));

    let root = UiProjectedNode::new(UiProjectedNodeId::new(1), UiProjectedNodeKind::Root);
    artifact.push_node(root);

    let element = UiProjectedNode::new(UiProjectedNodeId::new(2), UiProjectedNodeKind::Element);
    artifact.push_node(element);

    let model = render_projection_to_model(&artifact).expect("should build render model");

    assert_eq!(model.id(), UiRenderModelId::new(42));
    assert_eq!(model.source_projection(), UiProjectionArtifactId::new(42));
    assert_eq!(model.source_ir_root(), Some(UiIrNodeId::new(10)));
    assert_eq!(model.nodes().len(), 2);
}

#[test]
fn test_render_nodes_preserve_projected_node_ids() {
    let mut artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));
    artifact.push_node(UiProjectedNode::new(
        UiProjectedNodeId::new(10),
        UiProjectedNodeKind::Root,
    ));

    let model = render_projection_to_model(&artifact).unwrap();
    let nodes = model.nodes();

    assert_eq!(nodes[0].id(), UiRenderNodeId::new(10));
    assert_eq!(
        nodes[0].source_projection_node(),
        UiProjectedNodeId::new(10)
    );
}

#[test]
fn test_deterministic_identity() {
    let mut artifact1 = UiProjectionArtifact::new(UiProjectionArtifactId::new(100));
    artifact1.push_node(UiProjectedNode::new(
        UiProjectedNodeId::new(1),
        UiProjectedNodeKind::Root,
    ));

    let mut artifact2 = UiProjectionArtifact::new(UiProjectionArtifactId::new(100));
    artifact2.push_node(UiProjectedNode::new(
        UiProjectedNodeId::new(1),
        UiProjectedNodeKind::Root,
    ));

    let model1 = render_projection_to_model(&artifact1).unwrap();
    let model2 = render_projection_to_model(&artifact2).unwrap();

    assert_eq!(model1.id(), model2.id());
    assert_eq!(model1.nodes()[0].id(), model2.nodes()[0].id());
}

#[test]
fn test_inert_marker_api_remains_non_executable() {
    // Asserting that markers are inert (just enums).
    // They expose no `execute` or `handle` methods.
    let marker = UiRenderMarker::Property;
    match marker {
        UiRenderMarker::Property
        | UiRenderMarker::Action
        | UiRenderMarker::EffectBoundary
        | UiRenderMarker::Trace => {}
    }
}
