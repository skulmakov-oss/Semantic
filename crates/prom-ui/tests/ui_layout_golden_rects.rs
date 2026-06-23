use std::vec::Vec;

use prom_ui::projection::project_ir_to_projection;
use prom_ui::{
    lower_ast_to_ir, render_projection_to_model, solve_minimal_block_layout, tree_to_ast,
    UiLoweringConfig, UiMinimalBlockLayoutConfig, UiNode, UiNodeId, UiNodeKind, UiRenderModel,
    UiRenderNodeKind, UiTree, UiTreeId,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct GoldenRectEntry {
    render_node_id: u64,
    source_projection_node_id: u64,
    source_ir_node_id: Option<u64>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

type GoldenRectSignature = Vec<GoldenRectEntry>;
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

fn golden_config() -> UiMinimalBlockLayoutConfig {
    UiMinimalBlockLayoutConfig::new(10, 20, 300, 40, 5)
}

fn zero_size_config() -> UiMinimalBlockLayoutConfig {
    UiMinimalBlockLayoutConfig::new(-5, -10, 0, 0, 0)
}

fn negative_origin_config() -> UiMinimalBlockLayoutConfig {
    UiMinimalBlockLayoutConfig::new(-100, -200, 50, 10, 2)
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

fn golden_rect_signature(
    render_model: &UiRenderModel,
    layout_model: &prom_ui::UiLayoutRectModel,
) -> GoldenRectSignature {
    assert_eq!(render_model.nodes().len(), layout_model.entries().len());

    render_model
        .nodes()
        .iter()
        .zip(layout_model.entries().iter())
        .map(|(render_node, entry)| GoldenRectEntry {
            render_node_id: render_node.id().raw(),
            source_projection_node_id: render_node.source_projection_node().raw(),
            source_ir_node_id: render_node.source_ir_node().map(|id| id.raw()),
            x: entry.rect().x(),
            y: entry.rect().y(),
            width: entry.rect().width(),
            height: entry.rect().height(),
        })
        .collect()
}

fn render_and_layout(
    tree: &UiTree,
    config: UiMinimalBlockLayoutConfig,
) -> (UiRenderModel, prom_ui::UiLayoutRectModel) {
    let render_model = render_model_from_tree(tree);
    let layout_model = solve_minimal_block_layout(&render_model, config);
    (render_model, layout_model)
}

#[test]
fn golden_rects_match_element_text_fixture() {
    let (render_model, layout_model) =
        render_and_layout(&build_element_text_tree(), golden_config());

    assert_eq!(
        golden_rect_signature(&render_model, &layout_model),
        vec![
            GoldenRectEntry {
                render_node_id: 1,
                source_projection_node_id: 1,
                source_ir_node_id: Some(1),
                x: 10,
                y: 20,
                width: 300,
                height: 40,
            },
            GoldenRectEntry {
                render_node_id: 2,
                source_projection_node_id: 2,
                source_ir_node_id: Some(2),
                x: 10,
                y: 65,
                width: 300,
                height: 40,
            },
            GoldenRectEntry {
                render_node_id: 3,
                source_projection_node_id: 3,
                source_ir_node_id: Some(3),
                x: 10,
                y: 110,
                width: 300,
                height: 40,
            },
        ]
    );
}

#[test]
fn golden_rects_match_sibling_fixture() {
    let (render_model, layout_model) =
        render_and_layout(&build_sibling_element_text_tree(), golden_config());

    assert_eq!(
        golden_rect_signature(&render_model, &layout_model),
        vec![
            GoldenRectEntry {
                render_node_id: 1,
                source_projection_node_id: 1,
                source_ir_node_id: Some(1),
                x: 10,
                y: 20,
                width: 300,
                height: 40,
            },
            GoldenRectEntry {
                render_node_id: 2,
                source_projection_node_id: 2,
                source_ir_node_id: Some(2),
                x: 10,
                y: 65,
                width: 300,
                height: 40,
            },
            GoldenRectEntry {
                render_node_id: 3,
                source_projection_node_id: 3,
                source_ir_node_id: Some(3),
                x: 10,
                y: 110,
                width: 300,
                height: 40,
            },
            GoldenRectEntry {
                render_node_id: 4,
                source_projection_node_id: 4,
                source_ir_node_id: Some(4),
                x: 10,
                y: 155,
                width: 300,
                height: 40,
            },
            GoldenRectEntry {
                render_node_id: 5,
                source_projection_node_id: 5,
                source_ir_node_id: Some(5),
                x: 10,
                y: 200,
                width: 300,
                height: 40,
            },
        ]
    );
}

#[test]
fn golden_rects_match_nested_fixture() {
    let (render_model, layout_model) =
        render_and_layout(&build_nested_element_text_tree(), golden_config());

    assert_eq!(
        golden_rect_signature(&render_model, &layout_model),
        vec![
            GoldenRectEntry {
                render_node_id: 1,
                source_projection_node_id: 1,
                source_ir_node_id: Some(1),
                x: 10,
                y: 20,
                width: 300,
                height: 40,
            },
            GoldenRectEntry {
                render_node_id: 2,
                source_projection_node_id: 2,
                source_ir_node_id: Some(2),
                x: 10,
                y: 65,
                width: 300,
                height: 40,
            },
            GoldenRectEntry {
                render_node_id: 3,
                source_projection_node_id: 3,
                source_ir_node_id: Some(3),
                x: 10,
                y: 110,
                width: 300,
                height: 40,
            },
            GoldenRectEntry {
                render_node_id: 4,
                source_projection_node_id: 4,
                source_ir_node_id: Some(4),
                x: 10,
                y: 155,
                width: 300,
                height: 40,
            },
        ]
    );
}

#[test]
fn golden_rects_are_stable_across_repeated_runs() {
    let tree = build_element_text_tree();

    let first_render = render_model_from_tree(&tree);
    let first_layout = solve_minimal_block_layout(&first_render, golden_config());
    let second_render = render_model_from_tree(&tree);
    let second_layout = solve_minimal_block_layout(&second_render, golden_config());

    assert_eq!(
        golden_rect_signature(&first_render, &first_layout),
        golden_rect_signature(&second_render, &second_layout)
    );
}

#[test]
fn golden_rects_are_stable_across_rebuilt_identical_trees() {
    let first = render_and_layout(&build_element_text_tree(), golden_config());
    let second = render_and_layout(&build_element_text_tree(), golden_config());

    assert_eq!(
        golden_rect_signature(&first.0, &first.1),
        golden_rect_signature(&second.0, &second.1)
    );
}

#[test]
fn golden_rects_preserve_zero_sized_explicit_evidence() {
    let (render_model, layout_model) =
        render_and_layout(&build_element_text_tree(), zero_size_config());

    let signature = golden_rect_signature(&render_model, &layout_model);

    assert_eq!(signature.len(), 3);
    for entry in signature {
        assert_eq!(entry.x, -5);
        assert_eq!(entry.y, -10);
        assert_eq!(entry.width, 0);
        assert_eq!(entry.height, 0);
    }
}

#[test]
fn golden_rects_preserve_negative_origins() {
    let (render_model, layout_model) =
        render_and_layout(&build_element_text_tree(), negative_origin_config());

    assert_eq!(
        golden_rect_signature(&render_model, &layout_model),
        vec![
            GoldenRectEntry {
                render_node_id: 1,
                source_projection_node_id: 1,
                source_ir_node_id: Some(1),
                x: -100,
                y: -200,
                width: 50,
                height: 10,
            },
            GoldenRectEntry {
                render_node_id: 2,
                source_projection_node_id: 2,
                source_ir_node_id: Some(2),
                x: -100,
                y: -188,
                width: 50,
                height: 10,
            },
            GoldenRectEntry {
                render_node_id: 3,
                source_projection_node_id: 3,
                source_ir_node_id: Some(3),
                x: -100,
                y: -176,
                width: 50,
                height: 10,
            },
        ]
    );
}

#[test]
fn golden_rects_preserve_render_projection_ir_evidence() {
    let (render_model, layout_model) =
        render_and_layout(&build_nested_element_text_tree(), golden_config());

    for (render_node, entry) in render_model.nodes().iter().zip(layout_model.entries()) {
        assert_eq!(entry.render_node_id().raw(), render_node.id().raw());
        assert_eq!(
            entry.source_projection_node_id().raw(),
            render_node.source_projection_node().raw()
        );
        assert_eq!(
            entry.source_ir_node_id().map(|id| id.raw()),
            render_node.source_ir_node().map(|id| id.raw())
        );
    }
}

#[test]
fn golden_rect_solver_does_not_mutate_render_model() {
    let render_model = render_model_from_tree(&build_element_text_tree());
    let before = render_signature(&render_model);

    let _layout_model = solve_minimal_block_layout(&render_model, golden_config());

    assert_eq!(before, render_signature(&render_model));
}

#[test]
fn golden_rects_do_not_introduce_backend_runtime_or_authority() {
    let (render_model, layout_model) =
        render_and_layout(&build_element_text_tree(), golden_config());

    assert_eq!(layout_model.len(), render_model.nodes().len());
    assert!(layout_model.entries().iter().all(|entry| {
        entry.render_node_id().raw() > 0
            && entry.source_projection_node_id().raw() > 0
            && entry.rect().width() > 0
            && entry.rect().height() > 0
    }));
}
