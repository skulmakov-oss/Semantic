use prom_ui::model::UiIrNodeId;
use prom_ui::projection::{
    UiProjectedNode, UiProjectedNodeId, UiProjectedNodeKind, UiProjectionArtifact,
    UiProjectionArtifactId, UiProjectionTraceRef,
};
use prom_ui::renderer::{
    render_projection_to_model, UiRenderError, UiRenderMarker, UiRenderModelId, UiRenderNodeId,
    UiRenderNodeKind,
};

#[test]
fn test_render_model_builds_from_projected_valid_ir() {
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
fn test_deterministic_model_ids() {
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
    assert_eq!(model1.id(), UiRenderModelId::new(100));
}

#[test]
fn test_render_node_ids_preserve_projection_node_ids() {
    let mut artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));
    artifact.push_node(UiProjectedNode::new(
        UiProjectedNodeId::new(10),
        UiProjectedNodeKind::Root,
    ));
    artifact.push_node(UiProjectedNode::new(
        UiProjectedNodeId::new(20),
        UiProjectedNodeKind::Element,
    ));

    let model = render_projection_to_model(&artifact).unwrap();
    let nodes = model.nodes();

    assert_eq!(nodes[0].id(), UiRenderNodeId::new(10));
    assert_eq!(
        nodes[0].source_projection_node(),
        UiProjectedNodeId::new(10)
    );

    assert_eq!(nodes[1].id(), UiRenderNodeId::new(20));
    assert_eq!(
        nodes[1].source_projection_node(),
        UiProjectedNodeId::new(20)
    );
}

#[test]
fn test_source_trace_preservation() {
    let mut artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));
    artifact.set_source_ir_root(UiIrNodeId::new(50));

    let mut node = UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(1),
        UiProjectedNodeKind::Element,
        UiIrNodeId::new(51),
    );
    artifact.push_node(node);

    let model = render_projection_to_model(&artifact).unwrap();
    assert_eq!(model.source_ir_root(), Some(UiIrNodeId::new(50)));
    assert_eq!(model.nodes()[0].source_ir_node(), Some(UiIrNodeId::new(51)));
}

#[test]
fn test_inert_property_action_effect_markers() {
    let mut artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));

    artifact.push_node(UiProjectedNode::new(
        UiProjectedNodeId::new(1),
        UiProjectedNodeKind::PropertyCarrier,
    ));
    artifact.push_node(UiProjectedNode::new(
        UiProjectedNodeId::new(2),
        UiProjectedNodeKind::ActionCarrier,
    ));
    artifact.push_node(UiProjectedNode::new(
        UiProjectedNodeId::new(3),
        UiProjectedNodeKind::EffectBoundaryMarker,
    ));

    let mut traced_node =
        UiProjectedNode::new(UiProjectedNodeId::new(4), UiProjectedNodeKind::Element);
    traced_node.set_trace(UiProjectionTraceRef::new(400));
    artifact.push_node(traced_node);

    let model = render_projection_to_model(&artifact).unwrap();
    let nodes = model.nodes();

    assert_eq!(nodes[0].kind(), UiRenderNodeKind::Fragment);
    assert!(nodes[0].markers().contains(&UiRenderMarker::Property));

    assert_eq!(nodes[1].kind(), UiRenderNodeKind::Fragment);
    assert!(nodes[1].markers().contains(&UiRenderMarker::Action));

    assert_eq!(nodes[2].kind(), UiRenderNodeKind::Fragment);
    assert!(nodes[2].markers().contains(&UiRenderMarker::EffectBoundary));

    assert_eq!(nodes[3].kind(), UiRenderNodeKind::Element);
    assert!(nodes[3].markers().contains(&UiRenderMarker::Trace));
}

#[test]
fn test_empty_projection_returns_error() {
    let artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));
    let result = render_projection_to_model(&artifact);
    assert_eq!(result, Err(UiRenderError::EmptyProjection));
}
