use prom_ui::model::UiIrNodeId;
use prom_ui::projection::{
    UiProjectedNode, UiProjectedNodeId, UiProjectedNodeKind, UiProjectionArtifact,
    UiProjectionArtifactId,
};
use prom_ui::renderer::{
    present_render_trace, render_projection_to_model, UiRenderModel, UiRenderTraceLink,
    UiRenderTraceLinkKind, UiRenderTracePresentation,
};

fn create_test_render_model() -> UiRenderModel {
    let mut artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(42));
    artifact.set_source_ir_root(UiIrNodeId::new(10));

    let root = UiProjectedNode::new(UiProjectedNodeId::new(1), UiProjectedNodeKind::Root);
    artifact.push_node(root);

    let element = UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(2),
        UiProjectedNodeKind::Element,
        UiIrNodeId::new(20),
    );
    artifact.push_node(element);

    render_projection_to_model(&artifact).expect("should build render model")
}

#[test]
fn trace_presentation_entrypoint_signature_is_locked() {
    let _: fn(&UiRenderModel) -> UiRenderTracePresentation = present_render_trace;
}

#[test]
fn trace_presentation_public_types_are_locked() {
    fn assert_presentation(_: &UiRenderTracePresentation) {}
    fn assert_link(_: &UiRenderTraceLink) {}
    fn assert_kind(_: UiRenderTraceLinkKind) {}

    let _ = (assert_presentation, assert_link, assert_kind);
}

#[test]
fn test_trace_presentation_builds_from_render_model_read_only() {
    let model = create_test_render_model();
    let presentation = present_render_trace(&model);

    assert_eq!(presentation.source_render_model(), model.id());
    assert_eq!(presentation.source_projection(), model.source_projection());

    // Model should be unmutated (implied by &UiRenderModel argument)
}

#[test]
fn test_deterministic_presentation_id() {
    let model = create_test_render_model();

    let presentation1 = present_render_trace(&model);
    let presentation2 = present_render_trace(&model);

    assert_eq!(presentation1.id(), presentation2.id());
}

#[test]
fn test_deterministic_trace_link_ids() {
    let model = create_test_render_model();

    let presentation1 = present_render_trace(&model);
    let presentation2 = present_render_trace(&model);

    assert_eq!(presentation1.links().len(), presentation2.links().len());

    for (link1, link2) in presentation1
        .links()
        .iter()
        .zip(presentation2.links().iter())
    {
        assert_eq!(link1.id(), link2.id());
        assert_eq!(link1.kind(), link2.kind());
    }
}

#[test]
fn test_source_link_preservation() {
    let model = create_test_render_model();
    let presentation = present_render_trace(&model);

    // There should be a model to projection link
    let model_link = presentation
        .links()
        .iter()
        .find(|l| l.kind() == UiRenderTraceLinkKind::RenderModelToProjection)
        .unwrap();
    assert_eq!(model_link.source_ir_node(), model.source_ir_root());

    // For each node in the model, there should be trace links
    for node in model.nodes() {
        let node_proj_link = presentation
            .links()
            .iter()
            .find(|l| {
                l.kind() == UiRenderTraceLinkKind::RenderNodeToProjectionNode
                    && l.source_render_node() == Some(node.id())
            })
            .expect("render node must have projection trace link");

        assert_eq!(
            node_proj_link.source_projection_node(),
            Some(node.source_projection_node())
        );

        if let Some(ir_node) = node.source_ir_node() {
            let node_ir_link = presentation
                .links()
                .iter()
                .find(|l| {
                    l.kind() == UiRenderTraceLinkKind::RenderNodeToIrNode
                        && l.source_render_node() == Some(node.id())
                })
                .expect("render node must have ir trace link");

            assert_eq!(node_ir_link.source_ir_node(), Some(ir_node));
        }
    }
}
