use prom_ui::model::UiIrNodeId;
use prom_ui::projection::{
    UiProjectedNode, UiProjectedNodeId, UiProjectedNodeKind, UiProjectionArtifact,
    UiProjectionArtifactId, UiProjectionTraceRef,
};
use prom_ui::renderer::{
    present_render_markers, render_projection_to_model, UiRenderMarkerEmphasis,
    UiRenderMarkerVisualRole,
};

#[test]
fn test_marker_presentation_derives_items_deterministically() {
    let mut artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(400));

    // Add nodes with kinds that map to markers
    artifact.push_node(UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(401),
        UiProjectedNodeKind::PropertyCarrier,
        UiIrNodeId::new(1001),
    ));
    artifact.push_node(UiProjectedNode::new(
        UiProjectedNodeId::new(402),
        UiProjectedNodeKind::ActionCarrier,
    ));
    let mut trace_node = UiProjectedNode::new(
        UiProjectedNodeId::new(403),
        UiProjectedNodeKind::EffectBoundaryMarker,
    );
    trace_node.set_trace(UiProjectionTraceRef::new(403));
    artifact.push_node(trace_node);

    let model = render_projection_to_model(&artifact).expect("Should render");
    let presentation = present_render_markers(&model);

    assert_eq!(presentation.source_render_model(), model.id());
    assert_eq!(presentation.source_projection(), artifact.id());

    let items = presentation.items();
    // 1 property marker, 1 action marker, 1 effect boundary marker + 1 trace marker on the third node
    assert_eq!(items.len(), 4);

    let i0 = items[0].clone();
    assert_eq!(i0.role(), UiRenderMarkerVisualRole::PropertyIndicator);
    assert_eq!(i0.emphasis(), UiRenderMarkerEmphasis::Subtle);

    let i1 = items[1].clone();
    assert_eq!(i1.role(), UiRenderMarkerVisualRole::ActionIndicator);
    assert_eq!(i1.emphasis(), UiRenderMarkerEmphasis::Prominent);

    let i2 = items[2].clone();
    assert_eq!(i2.role(), UiRenderMarkerVisualRole::EffectBoundaryIndicator);
    assert_eq!(i2.emphasis(), UiRenderMarkerEmphasis::Prominent);

    let i3 = items[3].clone();
    assert_eq!(i3.role(), UiRenderMarkerVisualRole::TraceIndicator);
    assert_eq!(i3.emphasis(), UiRenderMarkerEmphasis::Subtle);

    // Verify determinism
    let presentation_again = present_render_markers(&model);
    assert_eq!(presentation.id(), presentation_again.id());
    for (a, b) in presentation
        .items()
        .iter()
        .zip(presentation_again.items().iter())
    {
        assert_eq!(a.id(), b.id());
    }
}
