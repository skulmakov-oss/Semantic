use core::convert::Infallible;

use prom_ui::projection::project_ir_to_projection;
use prom_ui::{
    lower_ast_to_ir, render_projection_to_model, solve_minimal_block_layout, tree_to_ast,
    UiLayoutRect, UiLayoutRectModel, UiLoweringConfig, UiMinimalBlockLayoutConfig, UiNode,
    UiNodeId, UiNodeKind, UiRenderModel, UiRenderNodeKind, UiTree, UiTreeId,
};
use prom_ui_backend_native::{UiBackendFrame, UiBackendFrameEntry, UiFrameSink};

type RenderSignature = Vec<(u64, u64, Option<u64>, UiRenderNodeKind, u64, Option<u64>)>;
type LayoutSignature = Vec<(u64, u64, Option<u64>, i32, i32, u32, u32)>;
type FrameSignature = Vec<(u64, u64, Option<u64>, i32, i32, u32, u32, UiRenderNodeKind)>;

fn build_element_text_tree() -> UiTree {
    let mut tree = UiTree::new(UiTreeId::new(1));

    let root_id = UiNodeId::new(1);
    let element_id = UiNodeId::new(2);
    let text_id = UiNodeId::new(3);

    let mut root = UiNode::new(root_id, UiNodeKind::Root);
    root.push_child(element_id);

    let mut element = UiNode::with_parent(element_id, UiNodeKind::Element, root_id);
    element.push_child(text_id);

    let text = UiNode::with_parent(text_id, UiNodeKind::Text, element_id);

    tree.push_node(root);
    tree.push_node(element);
    tree.push_node(text);
    tree
}

fn build_sibling_element_text_tree() -> UiTree {
    let mut tree = UiTree::new(UiTreeId::new(2));

    let root_id = UiNodeId::new(1);
    let left_element_id = UiNodeId::new(2);
    let left_text_id = UiNodeId::new(3);
    let right_element_id = UiNodeId::new(4);
    let right_text_id = UiNodeId::new(5);

    let mut root = UiNode::new(root_id, UiNodeKind::Root);
    root.push_child(left_element_id);
    root.push_child(right_element_id);

    let mut left_element = UiNode::with_parent(left_element_id, UiNodeKind::Element, root_id);
    left_element.push_child(left_text_id);

    let left_text = UiNode::with_parent(left_text_id, UiNodeKind::Text, left_element_id);

    let mut right_element = UiNode::with_parent(right_element_id, UiNodeKind::Element, root_id);
    right_element.push_child(right_text_id);

    let right_text = UiNode::with_parent(right_text_id, UiNodeKind::Text, right_element_id);

    tree.push_node(root);
    tree.push_node(left_element);
    tree.push_node(left_text);
    tree.push_node(right_element);
    tree.push_node(right_text);
    tree
}

fn build_nested_element_text_tree() -> UiTree {
    let mut tree = UiTree::new(UiTreeId::new(3));

    let root_id = UiNodeId::new(1);
    let outer_element_id = UiNodeId::new(2);
    let inner_element_id = UiNodeId::new(3);
    let text_id = UiNodeId::new(4);

    let mut root = UiNode::new(root_id, UiNodeKind::Root);
    root.push_child(outer_element_id);

    let mut outer_element = UiNode::with_parent(outer_element_id, UiNodeKind::Element, root_id);
    outer_element.push_child(inner_element_id);

    let mut inner_element =
        UiNode::with_parent(inner_element_id, UiNodeKind::Element, outer_element_id);
    inner_element.push_child(text_id);

    let text = UiNode::with_parent(text_id, UiNodeKind::Text, inner_element_id);

    tree.push_node(root);
    tree.push_node(outer_element);
    tree.push_node(inner_element);
    tree.push_node(text);
    tree
}

fn render_model_from_tree(tree: &UiTree) -> UiRenderModel {
    let ast = tree_to_ast(tree).expect("tree should bridge to ast");
    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("ast should lower to ir");
    let projection = project_ir_to_projection(&ir).expect("ir should project");
    render_projection_to_model(&projection).expect("projection should render")
}

fn layout_model_from_render_model(render_model: &UiRenderModel) -> UiLayoutRectModel {
    solve_minimal_block_layout(
        render_model,
        UiMinimalBlockLayoutConfig::new(10, 20, 300, 40, 5),
    )
}

fn render_signature(model: &UiRenderModel) -> RenderSignature {
    model
        .nodes()
        .iter()
        .map(|node| {
            (
                node.id().raw(),
                node.source_projection_node().raw(),
                node.source_ir_node().map(|id| id.raw()),
                node.kind(),
                node.markers().len() as u64,
                None,
            )
        })
        .collect()
}

fn layout_signature(model: &UiLayoutRectModel) -> LayoutSignature {
    model
        .entries()
        .iter()
        .map(|entry| {
            (
                entry.render_node_id().raw(),
                entry.source_projection_node_id().raw(),
                entry.source_ir_node_id().map(|id| id.raw()),
                entry.rect().x(),
                entry.rect().y(),
                entry.rect().width(),
                entry.rect().height(),
            )
        })
        .collect()
}

fn frame_signature(frame: &UiBackendFrame) -> FrameSignature {
    frame
        .entries()
        .iter()
        .map(|entry| {
            (
                entry.render_node_id().raw(),
                entry.source_projection_node_id().raw(),
                entry.source_ir_node_id().map(|id| id.raw()),
                entry.rect().x(),
                entry.rect().y(),
                entry.rect().width(),
                entry.rect().height(),
                entry.kind(),
            )
        })
        .collect()
}

fn build_backend_frame_from_render_and_layout(
    render_model: &UiRenderModel,
    layout_model: &UiLayoutRectModel,
) -> UiBackendFrame {
    let entries = render_model
        .nodes()
        .iter()
        .zip(layout_model.entries().iter())
        .map(|(render_node, layout_entry)| {
            UiBackendFrameEntry::new(
                render_node.id(),
                render_node.source_projection_node(),
                render_node.source_ir_node(),
                layout_entry.rect(),
                render_node.kind(),
            )
        })
        .collect::<Vec<_>>();

    UiBackendFrame::new(entries)
}

fn build_frame_bundle(tree: &UiTree) -> (UiRenderModel, UiLayoutRectModel, UiBackendFrame) {
    let render_model = render_model_from_tree(tree);
    let layout_model = layout_model_from_render_model(&render_model);
    let frame = build_backend_frame_from_render_and_layout(&render_model, &layout_model);
    (render_model, layout_model, frame)
}

#[derive(Default)]
struct RecordingSink {
    frames: Vec<UiBackendFrame>,
}

impl UiFrameSink for RecordingSink {
    type Error = Infallible;

    fn submit_frame(&mut self, frame: &UiBackendFrame) -> Result<(), Self::Error> {
        self.frames.push(frame.clone());
        Ok(())
    }
}

#[test]
fn offscreen_static_frame_can_be_built_from_element_text_fixture() {
    let (_, _, frame) = build_frame_bundle(&build_element_text_tree());

    assert_eq!(frame.len(), 3);
    assert_eq!(
        frame_signature(&frame),
        vec![
            (1, 1, Some(1), 10, 20, 300, 40, UiRenderNodeKind::Root),
            (2, 2, Some(2), 10, 65, 300, 40, UiRenderNodeKind::Element),
            (3, 3, Some(3), 10, 110, 300, 40, UiRenderNodeKind::Text),
        ]
    );
}

#[test]
fn offscreen_static_frame_preserves_render_order() {
    let (render_model, layout_model, frame) =
        build_frame_bundle(&build_sibling_element_text_tree());

    let expected: Vec<_> = render_model
        .nodes()
        .iter()
        .map(|node| node.id().raw())
        .collect();
    let actual: Vec<_> = frame
        .entries()
        .iter()
        .map(|entry| entry.render_node_id().raw())
        .collect();

    assert_eq!(expected, actual);
    assert_eq!(layout_model.len(), frame.len());
}

#[test]
fn offscreen_static_frame_preserves_projection_and_ir_source_evidence() {
    let (render_model, _, frame) = build_frame_bundle(&build_nested_element_text_tree());

    for (render_node, entry) in render_model.nodes().iter().zip(frame.entries()) {
        assert_eq!(entry.render_node_id(), render_node.id());
        assert_eq!(
            entry.source_projection_node_id(),
            render_node.source_projection_node()
        );
        assert_eq!(entry.source_ir_node_id(), render_node.source_ir_node());
    }
}

#[test]
fn offscreen_static_frame_preserves_layout_rect_evidence() {
    let (_, layout_model, frame) = build_frame_bundle(&build_element_text_tree());

    for (layout_entry, frame_entry) in layout_model.entries().iter().zip(frame.entries()) {
        assert_eq!(frame_entry.rect(), layout_entry.rect());
    }
}

#[test]
fn offscreen_static_frame_can_be_submitted_to_fake_sink() {
    let (_, _, frame) = build_frame_bundle(&build_element_text_tree());
    let mut sink = RecordingSink::default();

    sink.submit_frame(&frame).expect("fake sink accepts frame");

    assert_eq!(sink.frames.len(), 1);
    assert_eq!(sink.frames[0], frame);
}

#[test]
fn offscreen_static_frame_sink_submission_does_not_mutate_frame() {
    let (_, _, frame) = build_frame_bundle(&build_element_text_tree());
    let mut sink = RecordingSink::default();
    let before = frame_signature(&frame);

    sink.submit_frame(&frame).expect("fake sink accepts frame");

    let after = frame_signature(&frame);
    assert_eq!(before, after);
    assert_eq!(sink.frames[0], frame);
}

#[test]
fn offscreen_static_frame_is_deterministic_across_repeated_runs() {
    let first = build_frame_bundle(&build_element_text_tree()).2;
    let second = build_frame_bundle(&build_element_text_tree()).2;

    assert_eq!(frame_signature(&first), frame_signature(&second));
}

#[test]
fn offscreen_static_frame_is_deterministic_for_rebuilt_identical_trees() {
    let first = build_frame_bundle(&build_element_text_tree()).2;
    let second = build_frame_bundle(&build_element_text_tree()).2;

    assert_eq!(frame_signature(&first), frame_signature(&second));
}

#[test]
fn offscreen_static_frame_preserves_sibling_fixture_sequence() {
    let (render_model, layout_model, frame) =
        build_frame_bundle(&build_sibling_element_text_tree());

    assert_eq!(frame.len(), 5);
    assert_eq!(render_signature(&render_model).len(), 5);
    assert_eq!(layout_signature(&layout_model).len(), 5);
    assert_eq!(
        frame
            .entries()
            .iter()
            .map(|entry| entry.rect().y())
            .collect::<Vec<_>>(),
        vec![20, 65, 110, 155, 200]
    );
}

#[test]
fn offscreen_static_frame_preserves_nested_fixture_sequence() {
    let (render_model, layout_model, frame) = build_frame_bundle(&build_nested_element_text_tree());

    assert_eq!(frame.len(), 4);
    assert_eq!(render_signature(&render_model).len(), 4);
    assert_eq!(layout_signature(&layout_model).len(), 4);
    assert_eq!(
        frame
            .entries()
            .iter()
            .map(|entry| entry.rect().y())
            .collect::<Vec<_>>(),
        vec![20, 65, 110, 155]
    );
}

#[test]
fn offscreen_static_frame_construction_does_not_mutate_render_or_layout_models() {
    let (render_model, layout_model, frame) = build_frame_bundle(&build_element_text_tree());
    let render_before = render_signature(&render_model);
    let layout_before = layout_signature(&layout_model);

    let rebuilt_frame = build_backend_frame_from_render_and_layout(&render_model, &layout_model);

    assert_eq!(render_before, render_signature(&render_model));
    assert_eq!(layout_before, layout_signature(&layout_model));
    assert_eq!(frame_signature(&frame), frame_signature(&rebuilt_frame));
}

#[test]
fn offscreen_static_frame_boundary_exposes_no_drawing_window_or_runtime_execution() {
    let (_, _, frame) = build_frame_bundle(&build_element_text_tree());
    let entry = frame.entries()[0].clone();

    assert_eq!(frame.entries().len(), 3);
    assert_eq!(entry.kind(), UiRenderNodeKind::Root);
    assert_eq!(entry.rect(), UiLayoutRect::new(10, 20, 300, 40));
}
