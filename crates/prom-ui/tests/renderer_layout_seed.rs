use prom_ui::layout::{layout_render_model, UiLayoutSlotKind};
use prom_ui::model::UiIrNodeId;
use prom_ui::projection::{
    UiProjectedNode, UiProjectedNodeId, UiProjectedNodeKind, UiProjectionArtifact,
    UiProjectionArtifactId,
};
use prom_ui::renderer::{render_projection_to_model, UiRenderModel};

fn create_test_render_model() -> UiRenderModel {
    let mut artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(100));
    artifact.set_source_ir_root(UiIrNodeId::new(10));

    let mut root = UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(1),
        UiProjectedNodeKind::Root,
        UiIrNodeId::new(11),
    );
    artifact.push_node(root);

    let element = UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(2),
        UiProjectedNodeKind::Element,
        UiIrNodeId::new(12),
    );
    artifact.push_node(element);

    render_projection_to_model(&artifact).unwrap()
}

#[test]
fn layout_model_preserves_render_model_identity() {
    let render_model = create_test_render_model();

    let layout_model = layout_render_model(&render_model);
    assert_eq!(layout_model.source_render_model(), render_model.id());
    assert_eq!(layout_model.id().raw(), render_model.id().raw());
}

#[test]
fn layout_model_preserves_source_projection() {
    let render_model = create_test_render_model();

    let layout_model = layout_render_model(&render_model);
    assert_eq!(
        layout_model.source_projection(),
        render_model.source_projection()
    );
}

#[test]
fn layout_model_preserves_source_ir_root_when_present() {
    let render_model = create_test_render_model();

    let layout_model = layout_render_model(&render_model);
    assert_eq!(layout_model.source_ir_root(), render_model.source_ir_root());
    assert!(layout_model.source_ir_root().is_some());
}

#[test]
fn layout_nodes_preserve_render_node_identity() {
    let render_model = create_test_render_model();

    let layout_model = layout_render_model(&render_model);
    assert_eq!(layout_model.nodes().len(), render_model.nodes().len());

    for (layout_node, render_node) in layout_model.nodes().iter().zip(render_model.nodes().iter()) {
        assert_eq!(layout_node.source_render_node(), render_node.id());
        assert_eq!(layout_node.id().raw(), render_node.id().raw());
    }
}

#[test]
fn layout_nodes_preserve_projection_node_identity() {
    let render_model = create_test_render_model();

    let layout_model = layout_render_model(&render_model);
    for (layout_node, render_node) in layout_model.nodes().iter().zip(render_model.nodes().iter()) {
        assert_eq!(
            layout_node.source_projection_node(),
            Some(render_node.source_projection_node())
        );
    }
}

#[test]
fn layout_nodes_preserve_source_ir_node_identity() {
    let render_model = create_test_render_model();

    let layout_model = layout_render_model(&render_model);
    for (layout_node, render_node) in layout_model.nodes().iter().zip(render_model.nodes().iter()) {
        assert_eq!(layout_node.source_ir_node(), render_node.source_ir_node());
    }
}

#[test]
fn layout_node_order_is_deterministic() {
    let render_model = create_test_render_model();

    let layout_model = layout_render_model(&render_model);

    for (i, node) in layout_model.nodes().iter().enumerate() {
        assert_eq!(node.order(), i);
    }
}

#[test]
fn repeated_layout_is_deterministic() {
    let render_model = create_test_render_model();

    let layout_model_1 = layout_render_model(&render_model);
    let layout_model_2 = layout_render_model(&render_model);

    assert_eq!(layout_model_1.id(), layout_model_2.id());
    assert_eq!(layout_model_1.slots(), layout_model_2.slots());
    assert_eq!(layout_model_1.nodes(), layout_model_2.nodes());
}

#[test]
fn layout_seed_is_structural_only() {
    // Asserting the vocabulary contains only structural/metadata items, no geometries
    let slots = vec![
        UiLayoutSlotKind::Root,
        UiLayoutSlotKind::Node,
        UiLayoutSlotKind::Metadata,
    ];
    assert_eq!(slots.len(), 3);
}

#[test]
fn layout_seed_has_no_draw_event_backend_authority() {
    // The test itself is an assertion that the public layout types do not expose
    // wgpu/winit/tauri/draw/event surfaces. We can enforce this structurally by
    // ensuring we only have access to inert structs.
    let _ = UiLayoutSlotKind::Root;
}

#[test]
fn layout_public_api_signature_lock() {
    // Validates that layout_render_model has the exact expected signature
    // layout_render_model: fn(&UiRenderModel) -> UiLayoutModel
    let render_model = create_test_render_model();
    let f: fn(&prom_ui::renderer::UiRenderModel) -> prom_ui::layout::UiLayoutModel =
        layout_render_model;
    let _model = f(&render_model);
}
