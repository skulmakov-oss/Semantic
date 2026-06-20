use std::vec::Vec;

use prom_ui::projection::project_ir_to_projection;
use prom_ui::{
    lower_ast_to_ir, render_projection_to_model, tree_to_ast, UiIrNodeId, UiLayoutRect,
    UiLayoutRectEntry, UiLayoutRectModel, UiLoweringConfig, UiNode, UiNodeId, UiNodeKind,
    UiProjectedNodeId, UiRenderModel, UiRenderNodeId, UiRenderNodeKind, UiTree, UiTreeId,
};

type LayoutInputSignature = Vec<(u64, u64, Option<u64>, i32, i32, u32, u32)>;

fn build_element_text_tree() -> UiTree {
    let mut tree = UiTree::new(UiTreeId::new(100));

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
    let mut tree = UiTree::new(UiTreeId::new(200));

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
    let mut tree = UiTree::new(UiTreeId::new(300));

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

// Slot fixture omitted because this contract PR must not change source or pipeline behavior.

fn render_model_from_tree(tree: &UiTree) -> UiRenderModel {
    let ast = tree_to_ast(tree).expect("tree should bridge to ast");
    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("ast should lower to ir");
    let projection = project_ir_to_projection(&ir).expect("ir should project");
    render_projection_to_model(&projection).expect("projection should render")
}

fn rect_for_index(index: usize) -> UiLayoutRect {
    let index = index as i32;
    UiLayoutRect::new(-(index + 1), -(index * 2), index as u32, index as u32)
}

fn layout_input_from_render_model_explicitly(model: &UiRenderModel) -> UiLayoutRectModel {
    let mut layout = UiLayoutRectModel::new();

    for (index, node) in model.nodes().iter().enumerate() {
        layout.push_entry(UiLayoutRectEntry::new(
            node.id(),
            node.source_projection_node(),
            node.source_ir_node(),
            rect_for_index(index),
        ));
    }

    layout
}

fn layout_input_signature(model: &UiLayoutRectModel) -> LayoutInputSignature {
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

fn render_model_signature(model: &UiRenderModel) -> LayoutInputSignature {
    model
        .nodes()
        .iter()
        .enumerate()
        .map(|(index, node)| {
            (
                node.id().raw(),
                node.source_projection_node().raw(),
                node.source_ir_node().map(|id| id.raw()),
                rect_for_index(index).x(),
                rect_for_index(index).y(),
                rect_for_index(index).width(),
                rect_for_index(index).height(),
            )
        })
        .collect()
}

fn build_manual_layout_input_with_missing_ir() -> UiLayoutRectModel {
    let mut model = UiLayoutRectModel::new();

    model.push_entry(UiLayoutRectEntry::new(
        UiRenderNodeId::new(1),
        UiProjectedNodeId::new(1),
        Some(UiIrNodeId::new(1)),
        rect_for_index(0),
    ));
    model.push_entry(UiLayoutRectEntry::new(
        UiRenderNodeId::new(2),
        UiProjectedNodeId::new(2),
        None,
        rect_for_index(1),
    ));

    model
}

#[test]
fn layout_input_contract_can_cover_render_model_nodes() {
    let render_model = render_model_from_tree(&build_element_text_tree());
    let layout_model = layout_input_from_render_model_explicitly(&render_model);

    assert_eq!(render_model.nodes().len(), layout_model.len());
    assert_eq!(layout_model.len(), 3);

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
fn layout_input_contract_preserves_render_order() {
    let render_model = render_model_from_tree(&build_sibling_element_text_tree());
    let layout_model = layout_input_from_render_model_explicitly(&render_model);

    assert_eq!(
        render_model
            .nodes()
            .iter()
            .map(|node| node.kind())
            .collect::<Vec<_>>(),
        vec![
            UiRenderNodeKind::Root,
            UiRenderNodeKind::Element,
            UiRenderNodeKind::Text,
            UiRenderNodeKind::Element,
            UiRenderNodeKind::Text,
        ]
    );
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
    assert_eq!(
        layout_input_signature(&layout_model),
        render_model_signature(&render_model)
    );
}

#[test]
fn layout_input_contract_preserves_source_evidence() {
    let render_model = render_model_from_tree(&build_nested_element_text_tree());
    let layout_model = layout_input_from_render_model_explicitly(&render_model);

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
fn layout_input_contract_supports_missing_ir_evidence_explicitly() {
    let model = build_manual_layout_input_with_missing_ir();

    assert_eq!(model.len(), 2);
    assert_eq!(
        model.entries()[0].source_ir_node_id(),
        Some(UiIrNodeId::new(1))
    );
    assert_eq!(model.entries()[1].source_ir_node_id(), None);
}

#[test]
fn layout_input_contract_uses_integer_only_geometry() {
    let rect = UiLayoutRect::new(-7, 9, 0, 12);
    let entry = UiLayoutRectEntry::new(
        UiRenderNodeId::new(7),
        UiProjectedNodeId::new(8),
        None,
        rect,
    );

    assert_eq!(entry.rect().x(), -7);
    assert_eq!(entry.rect().y(), 9);
    assert_eq!(entry.rect().width(), 0);
    assert_eq!(entry.rect().height(), 12);
}

#[test]
fn layout_input_contract_distinguishes_zero_rect_from_absence() {
    let mut model = UiLayoutRectModel::new();
    let zero_rect = UiLayoutRect::new(0, 0, 0, 0);

    assert!(model.is_empty());
    assert_eq!(model.len(), 0);

    model.push_entry(UiLayoutRectEntry::new(
        UiRenderNodeId::new(1),
        UiProjectedNodeId::new(1),
        Some(UiIrNodeId::new(1)),
        zero_rect,
    ));

    assert_eq!(model.len(), 1);
    assert_eq!(model.entries()[0].rect(), zero_rect);
    assert_eq!(model.entries()[0].rect().width(), 0);
    assert_eq!(model.entries()[0].rect().height(), 0);
}

#[test]
fn layout_input_contract_is_stable_for_repeated_render_inputs() {
    let tree = build_element_text_tree();

    let first = layout_input_signature(&layout_input_from_render_model_explicitly(
        &render_model_from_tree(&tree),
    ));
    let second = layout_input_signature(&layout_input_from_render_model_explicitly(
        &render_model_from_tree(&tree),
    ));

    assert_eq!(first, second);
}

#[test]
fn layout_input_contract_is_stable_for_rebuilt_identical_trees() {
    let first = layout_input_signature(&layout_input_from_render_model_explicitly(
        &render_model_from_tree(&build_element_text_tree()),
    ));
    let second = layout_input_signature(&layout_input_from_render_model_explicitly(
        &render_model_from_tree(&build_element_text_tree()),
    ));

    assert_eq!(first, second);
}

#[test]
fn layout_input_contract_preserves_sibling_order() {
    let layout_model = layout_input_from_render_model_explicitly(&render_model_from_tree(
        &build_sibling_element_text_tree(),
    ));

    assert_eq!(layout_model.len(), 5);
    assert_eq!(
        layout_model
            .entries()
            .iter()
            .map(|entry| entry.render_node_id().raw())
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5]
    );
    assert_eq!(
        layout_model
            .entries()
            .iter()
            .map(|entry| entry.rect())
            .collect::<Vec<_>>(),
        vec![
            rect_for_index(0),
            rect_for_index(1),
            rect_for_index(2),
            rect_for_index(3),
            rect_for_index(4),
        ]
    );
}

#[test]
fn layout_input_contract_preserves_nested_order() {
    let render_model = render_model_from_tree(&build_nested_element_text_tree());
    let layout_model = layout_input_from_render_model_explicitly(&render_model);

    assert_eq!(layout_model.len(), 4);
    assert_eq!(
        layout_model
            .entries()
            .iter()
            .map(|entry| entry.render_node_id().raw())
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 4]
    );
    assert_eq!(
        layout_model
            .entries()
            .iter()
            .map(|entry| entry.source_projection_node_id().raw())
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 4]
    );
    assert_eq!(
        layout_model
            .entries()
            .iter()
            .map(|entry| entry.source_ir_node_id().map(|id| id.raw()))
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2), Some(3), Some(4)]
    );
}

#[test]
fn layout_input_contract_does_not_introduce_solver_or_authority() {
    let layout_model = layout_input_from_render_model_explicitly(&render_model_from_tree(
        &build_element_text_tree(),
    ));

    assert_eq!(layout_model.len(), 3);
    assert_eq!(layout_model.entries().len(), 3);
    assert_eq!(
        layout_model
            .entries()
            .iter()
            .map(|entry| entry.rect().width())
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(
        layout_model
            .entries()
            .iter()
            .map(|entry| entry.rect().height())
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
}
