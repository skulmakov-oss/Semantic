use std::vec::Vec;

use prom_ui::projection::project_ir_to_projection;
use prom_ui::{
    lower_ast_to_ir, render_projection_to_model, solve_minimal_block_layout, tree_to_ast,
    UiLoweringConfig, UiMinimalBlockLayoutConfig, UiNode, UiNodeId, UiNodeKind, UiRenderModel,
    UiRenderNodeKind, UiTree, UiTreeId,
};

type LayoutSignature = Vec<(u64, u64, Option<u64>, i32, i32, u32, u32)>;
type RenderSignature = Vec<(u64, UiRenderNodeKind, u64, Option<u64>, usize)>;

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

fn layout_signature(model: &prom_ui::UiLayoutRectModel) -> LayoutSignature {
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

fn render_signature(model: &UiRenderModel) -> RenderSignature {
    model
        .nodes()
        .iter()
        .map(|node| {
            (
                node.id().raw(),
                node.kind(),
                node.source_projection_node().raw(),
                node.source_ir_node().map(|id| id.raw()),
                node.markers().len(),
            )
        })
        .collect()
}

fn solve_with_config(
    tree: &UiTree,
    config: UiMinimalBlockLayoutConfig,
) -> prom_ui::UiLayoutRectModel {
    let render_model = render_model_from_tree(tree);
    solve_minimal_block_layout(&render_model, config)
}

fn render_model_and_layout(
    tree: &UiTree,
    config: UiMinimalBlockLayoutConfig,
) -> (UiRenderModel, prom_ui::UiLayoutRectModel) {
    let render_model = render_model_from_tree(tree);
    let layout_model = solve_minimal_block_layout(&render_model, config);
    (render_model, layout_model)
}

#[test]
fn minimal_block_layout_config_preserves_integer_values() {
    let config = UiMinimalBlockLayoutConfig::new(-13, 27, 300, 40, 5);

    assert_eq!(config.origin_x(), -13);
    assert_eq!(config.origin_y(), 27);
    assert_eq!(config.width(), 300);
    assert_eq!(config.row_height(), 40);
    assert_eq!(config.row_gap(), 5);
}

#[test]
fn minimal_block_layout_emits_one_rect_per_render_node() {
    let config = UiMinimalBlockLayoutConfig::new(10, 20, 300, 40, 5);
    let render_model = render_model_from_tree(&build_element_text_tree());
    let layout_model = solve_minimal_block_layout(&render_model, config);

    assert_eq!(layout_model.len(), render_model.nodes().len());
    assert_eq!(layout_model.len(), 3);

    for (render_node, entry) in render_model.nodes().iter().zip(layout_model.entries()) {
        assert_eq!(entry.render_node_id(), render_node.id());
    }
}

#[test]
fn minimal_block_layout_preserves_render_order() {
    let config = UiMinimalBlockLayoutConfig::new(10, 20, 300, 40, 5);
    let render_model = render_model_from_tree(&build_sibling_element_text_tree());
    let layout_model = solve_minimal_block_layout(&render_model, config);

    let render_ids = render_model
        .nodes()
        .iter()
        .map(|node| node.id().raw())
        .collect::<Vec<_>>();
    let layout_ids = layout_model
        .entries()
        .iter()
        .map(|entry| entry.render_node_id().raw())
        .collect::<Vec<_>>();

    assert_eq!(render_ids, layout_ids);
}

#[test]
fn minimal_block_layout_computes_expected_integer_rects() {
    let config = UiMinimalBlockLayoutConfig::new(10, 20, 300, 40, 5);
    let render_model = render_model_from_tree(&build_sibling_element_text_tree());
    let layout_model = solve_minimal_block_layout(&render_model, config);

    let expected_y = [20, 65, 110, 155, 200];

    for (index, entry) in layout_model.entries().iter().enumerate() {
        assert_eq!(entry.rect().x(), 10);
        assert_eq!(entry.rect().y(), expected_y[index]);
        assert_eq!(entry.rect().width(), 300);
        assert_eq!(entry.rect().height(), 40);
    }
}

#[test]
fn minimal_block_layout_preserves_render_projection_ir_evidence() {
    let config = UiMinimalBlockLayoutConfig::new(4, -8, 33, 12, 1);
    let render_model = render_model_from_tree(&build_nested_element_text_tree());
    let layout_model = solve_minimal_block_layout(&render_model, config);

    for (render_node, entry) in render_model.nodes().iter().zip(layout_model.entries()) {
        assert_eq!(entry.render_node_id(), render_node.id());
        assert_eq!(
            entry.source_projection_node_id(),
            render_node.source_projection_node()
        );
        assert_eq!(entry.source_ir_node_id(), render_node.source_ir_node());
    }
}

#[test]
fn minimal_block_layout_is_deterministic_for_repeated_runs() {
    let config = UiMinimalBlockLayoutConfig::new(10, 20, 300, 40, 5);
    let tree = build_element_text_tree();

    let first_render = render_model_from_tree(&tree);
    let first_layout = solve_minimal_block_layout(&first_render, config);
    let second_render = render_model_from_tree(&tree);
    let second_layout = solve_minimal_block_layout(&second_render, config);

    assert_eq!(
        render_signature(&first_render),
        render_signature(&second_render)
    );
    assert_eq!(
        layout_signature(&first_layout),
        layout_signature(&second_layout)
    );
}

#[test]
fn minimal_block_layout_is_deterministic_for_rebuilt_identical_trees() {
    let config = UiMinimalBlockLayoutConfig::new(10, 20, 300, 40, 5);

    let first = solve_with_config(&build_element_text_tree(), config);
    let second = solve_with_config(&build_element_text_tree(), config);

    assert_eq!(layout_signature(&first), layout_signature(&second));
}

#[test]
fn minimal_block_layout_preserves_sibling_fixture_sequence() {
    let config = UiMinimalBlockLayoutConfig::new(10, 20, 300, 40, 5);
    let (render_model, layout_model) =
        render_model_and_layout(&build_sibling_element_text_tree(), config);

    assert_eq!(
        render_model
            .nodes()
            .iter()
            .map(|node| node.id().raw())
            .collect::<Vec<_>>(),
        layout_model
            .entries()
            .iter()
            .map(|entry| entry.render_node_id().raw())
            .collect::<Vec<_>>()
    );
    assert_eq!(layout_model.entries()[0].rect().y(), 20);
    assert_eq!(layout_model.entries()[1].rect().y(), 65);
    assert_eq!(layout_model.entries()[2].rect().y(), 110);
    assert_eq!(layout_model.entries()[3].rect().y(), 155);
    assert_eq!(layout_model.entries()[4].rect().y(), 200);
}

#[test]
fn minimal_block_layout_preserves_nested_order() {
    let config = UiMinimalBlockLayoutConfig::new(10, 20, 300, 40, 5);
    let (render_model, layout_model) =
        render_model_and_layout(&build_nested_element_text_tree(), config);

    assert_eq!(
        render_model
            .nodes()
            .iter()
            .map(|node| node.id().raw())
            .collect::<Vec<_>>(),
        layout_model
            .entries()
            .iter()
            .map(|entry| entry.render_node_id().raw())
            .collect::<Vec<_>>()
    );
    assert_eq!(layout_model.entries()[0].rect().y(), 20);
    assert_eq!(layout_model.entries()[1].rect().y(), 65);
    assert_eq!(layout_model.entries()[2].rect().y(), 110);
    assert_eq!(layout_model.entries()[3].rect().y(), 155);
}

#[test]
fn minimal_block_layout_does_not_mutate_render_model() {
    let config = UiMinimalBlockLayoutConfig::new(10, 20, 300, 40, 5);
    let render_model = render_model_from_tree(&build_element_text_tree());
    let before = render_signature(&render_model);

    let layout_model = solve_minimal_block_layout(&render_model, config);

    assert_eq!(before, render_signature(&render_model));
    assert_eq!(layout_model.len(), render_model.nodes().len());
}

#[test]
fn minimal_block_layout_allows_zero_width_or_height_as_explicit_evidence() {
    let config = UiMinimalBlockLayoutConfig::new(0, 0, 0, 0, 0);
    let render_model = render_model_from_tree(&build_element_text_tree());
    let layout_model = solve_minimal_block_layout(&render_model, config);

    assert_eq!(layout_model.len(), render_model.nodes().len());
    for entry in layout_model.entries() {
        assert_eq!(entry.rect().width(), 0);
        assert_eq!(entry.rect().height(), 0);
    }
}

#[test]
fn minimal_block_layout_does_not_introduce_backend_runtime_or_authority() {
    let config = UiMinimalBlockLayoutConfig::new(10, 20, 300, 40, 5);
    let render_model = render_model_from_tree(&build_element_text_tree());
    let layout_model = solve_minimal_block_layout(&render_model, config);

    assert_eq!(layout_model.len(), 3);
    assert!(layout_model.entries().iter().all(|entry| {
        entry.rect().width() == 300
            && entry.rect().height() == 40
            && entry.source_projection_node_id().raw() > 0
            && entry.render_node_id().raw() > 0
    }));
}
