use prom_ui::model::UiIrNodeId;
use prom_ui::projection::{
    UiProjectedNode, UiProjectedNodeId, UiProjectedNodeKind, UiProjectionArtifact,
    UiProjectionArtifactId, UiProjectionTraceRef,
};
use prom_ui::renderer::{
    present_render_diagnostics, render_projection_to_model, UiRenderDiagnosticItem,
    UiRenderDiagnosticKind, UiRenderDiagnosticSeverity, UiRenderDiagnosticsPresentation,
    UiRenderDiagnosticsPresentationId, UiRenderModel,
};

// --- Test 1 & 4 & 5 ---
#[test]
fn test_diagnostics_presentation_builds_from_render_model_read_only() {
    let mut artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(42));
    artifact.set_source_ir_root(UiIrNodeId::new(10));

    let root = UiProjectedNode::new(UiProjectedNodeId::new(1), UiProjectedNodeKind::Root);
    artifact.push_node(root);

    let prop_carrier = UiProjectedNode::new(
        UiProjectedNodeId::new(2),
        UiProjectedNodeKind::PropertyCarrier,
    );
    artifact.push_node(prop_carrier);

    let action_carrier = UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(3),
        UiProjectedNodeKind::ActionCarrier,
        UiIrNodeId::new(12),
    );
    artifact.push_node(action_carrier);

    let mut traced_node =
        UiProjectedNode::new(UiProjectedNodeId::new(4), UiProjectedNodeKind::Element);
    traced_node.set_trace(UiProjectionTraceRef::new(400));
    artifact.push_node(traced_node);

    let effect_boundary = UiProjectedNode::new(
        UiProjectedNodeId::new(5),
        UiProjectedNodeKind::EffectBoundaryMarker,
    );
    artifact.push_node(effect_boundary);

    let model = render_projection_to_model(&artifact).expect("should build render model");

    let presentation = present_render_diagnostics(&model);

    assert_eq!(
        presentation.id(),
        UiRenderDiagnosticsPresentationId::new(42)
    );
    assert_eq!(presentation.source_render_model(), model.id());
    assert_eq!(presentation.source_projection(), model.source_projection());

    let items = presentation.items();
    // We expect 4 items:
    // 1 for property carrier
    // 1 for action carrier
    // 1 for trace
    // 1 for effect boundary
    assert_eq!(items.len(), 4);

    // Property Carrier
    assert_eq!(items[0].kind(), UiRenderDiagnosticKind::PropertyMarker);
    assert_eq!(items[0].severity(), UiRenderDiagnosticSeverity::Info);
    assert_eq!(items[0].source_render_node().unwrap().raw(), 2);

    // Action Carrier
    assert_eq!(items[1].kind(), UiRenderDiagnosticKind::ActionMarker);
    assert_eq!(items[1].severity(), UiRenderDiagnosticSeverity::Warning);
    assert_eq!(items[1].source_render_node().unwrap().raw(), 3);
    assert_eq!(items[1].source_ir_node(), Some(UiIrNodeId::new(12)));

    // Trace
    assert_eq!(items[2].kind(), UiRenderDiagnosticKind::TraceMarker);
    assert_eq!(items[2].severity(), UiRenderDiagnosticSeverity::Info);
    assert_eq!(items[2].source_render_node().unwrap().raw(), 4);

    // Effect Boundary
    assert_eq!(
        items[3].kind(),
        UiRenderDiagnosticKind::EffectBoundaryMarker
    );
    assert_eq!(items[3].severity(), UiRenderDiagnosticSeverity::Warning);
    assert_eq!(items[3].source_render_node().unwrap().raw(), 5);
}

// --- Test 2 ---
#[test]
fn test_deterministic_presentation_id() {
    let mut artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(100));
    artifact.push_node(UiProjectedNode::new(
        UiProjectedNodeId::new(1),
        UiProjectedNodeKind::Root,
    ));

    let model = render_projection_to_model(&artifact).unwrap();

    let presentation1 = present_render_diagnostics(&model);
    let presentation2 = present_render_diagnostics(&model);

    assert_eq!(presentation1.id(), presentation2.id());
}

// --- Test 3 ---
#[test]
fn test_deterministic_diagnostic_item_ids() {
    let mut artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(100));
    artifact.push_node(UiProjectedNode::new(
        UiProjectedNodeId::new(1),
        UiProjectedNodeKind::PropertyCarrier,
    ));
    artifact.push_node(UiProjectedNode::new(
        UiProjectedNodeId::new(2),
        UiProjectedNodeKind::ActionCarrier,
    ));

    let model = render_projection_to_model(&artifact).unwrap();

    let presentation1 = present_render_diagnostics(&model);
    let presentation2 = present_render_diagnostics(&model);

    assert_eq!(presentation1.items().len(), 2);
    assert_eq!(presentation1.items()[0].id(), presentation2.items()[0].id());
    assert_eq!(presentation1.items()[1].id(), presentation2.items()[1].id());
}

// --- Test 7 ---
#[test]
fn diagnostics_presentation_entrypoint_signature_is_locked() {
    let _: fn(&UiRenderModel) -> UiRenderDiagnosticsPresentation = present_render_diagnostics;
}

#[test]
fn diagnostics_presentation_public_types_are_locked() {
    fn assert_presentation(_: &UiRenderDiagnosticsPresentation) {}
    fn assert_item(_: &UiRenderDiagnosticItem) {}
    fn assert_kind(_: UiRenderDiagnosticKind) {}
    fn assert_severity(_: UiRenderDiagnosticSeverity) {}

    let _ = (
        assert_presentation,
        assert_item,
        assert_kind,
        assert_severity,
    );
}

// Ensure no backend/capability code is introduced.
// The renderer diagnostics presentation is an inert local display structure.
// It is explicitly forbidden from implementing semantic truth, dispatching events,
// rendering backend UI, admitting capabilities, or integrating with Workbench/Studio.
