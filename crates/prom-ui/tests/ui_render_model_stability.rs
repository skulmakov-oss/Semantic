use prom_ui::lowering::{lower_ast_to_ir, UiLoweringConfig};
use prom_ui::model::{UiIrNodeId, UiNode, UiNodeId, UiNodeKind, UiTree, UiTreeId};
use prom_ui::projection::{project_ir_to_projection, UiProjectedNodeId};
use prom_ui::renderer::{
    render_projection_to_model, UiRenderMarker, UiRenderModel, UiRenderNodeId, UiRenderNodeKind,
};
use prom_ui::tree_bridge::tree_to_ast;

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderNodeSignature {
    id: UiRenderNodeId,
    kind: UiRenderNodeKind,
    source_projection_node: UiProjectedNodeId,
    source_ir_node: Option<UiIrNodeId>,
    markers: Vec<UiRenderMarker>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderModelSignature {
    id: prom_ui::renderer::UiRenderModelId,
    source_projection: prom_ui::projection::UiProjectionArtifactId,
    source_ir_root: Option<UiIrNodeId>,
    nodes: Vec<RenderNodeSignature>,
}

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

fn build_slot_tree() -> UiTree {
    let mut tree = UiTree::new(UiTreeId::new(400));

    let root_id = UiNodeId::new(1);
    let slot_id = UiNodeId::new(2);

    let mut root = UiNode::new(root_id, UiNodeKind::Root);
    root.push_child(slot_id);

    let slot = UiNode::with_parent(slot_id, UiNodeKind::Slot, root_id);

    tree.push_node(root);
    tree.push_node(slot);
    tree
}

fn run_pipeline(tree: &UiTree) -> UiRenderModel {
    let ast = tree_to_ast(tree).expect("tree should bridge to ast");
    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("ast should lower to ir");
    let projection = project_ir_to_projection(&ir).expect("ir should project");
    render_projection_to_model(&projection).expect("projection should render")
}

fn render_signature(model: &UiRenderModel) -> RenderModelSignature {
    RenderModelSignature {
        id: model.id(),
        source_projection: model.source_projection(),
        source_ir_root: model.source_ir_root(),
        nodes: model
            .nodes()
            .iter()
            .map(|node| RenderNodeSignature {
                id: node.id(),
                kind: node.kind(),
                source_projection_node: node.source_projection_node(),
                source_ir_node: node.source_ir_node(),
                markers: node.markers().to_vec(),
            })
            .collect(),
    }
}

#[test]
fn render_model_is_stable_for_repeated_element_text_pipeline_runs() {
    let tree = build_element_text_tree();

    let first = run_pipeline(&tree);
    let second = run_pipeline(&tree);

    assert_eq!(render_signature(&first), render_signature(&second));
    assert_eq!(first.nodes().len(), 3);
    assert_eq!(second.nodes().len(), 3);
}

#[test]
fn render_model_is_stable_for_independently_rebuilt_identical_trees() {
    let tree_a = build_element_text_tree();
    let tree_b = build_element_text_tree();

    let render_a = render_signature(&run_pipeline(&tree_a));
    let render_b = render_signature(&run_pipeline(&tree_b));

    assert_eq!(render_a, render_b);
}

#[test]
fn render_model_preserves_stable_sibling_order() {
    let tree = build_sibling_element_text_tree();
    let signature = render_signature(&run_pipeline(&tree));

    assert_eq!(signature.nodes.len(), 5);
    assert_eq!(
        signature
            .nodes
            .iter()
            .map(|node| node.id.raw())
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5]
    );
    assert_eq!(
        signature
            .nodes
            .iter()
            .map(|node| node.kind)
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
        signature
            .nodes
            .iter()
            .map(|node| node.source_ir_node.map(|id| id.raw()))
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2), Some(3), Some(4), Some(5)]
    );
}

#[test]
fn render_model_preserves_stable_nested_order() {
    let tree = build_nested_element_text_tree();
    let signature = render_signature(&run_pipeline(&tree));

    assert_eq!(signature.nodes.len(), 4);
    assert_eq!(
        signature
            .nodes
            .iter()
            .map(|node| node.kind)
            .collect::<Vec<_>>(),
        vec![
            UiRenderNodeKind::Root,
            UiRenderNodeKind::Element,
            UiRenderNodeKind::Element,
            UiRenderNodeKind::Text,
        ]
    );
    assert_eq!(
        signature
            .nodes
            .iter()
            .map(|node| node.source_projection_node.raw())
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 4]
    );
    assert_eq!(
        signature
            .nodes
            .iter()
            .map(|node| node.source_ir_node.map(|id| id.raw()))
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2), Some(3), Some(4)]
    );
}

#[test]
fn render_model_preserves_source_references_across_runs() {
    let tree = build_sibling_element_text_tree();

    let first = run_pipeline(&tree);
    let second = run_pipeline(&tree);

    let first_sig = render_signature(&first);
    let second_sig = render_signature(&second);

    assert_eq!(first_sig.id, second_sig.id);
    assert_eq!(first_sig.source_projection, second_sig.source_projection);
    assert_eq!(first_sig.source_ir_root, second_sig.source_ir_root);
    assert_eq!(first_sig.nodes, second_sig.nodes);
}

#[test]
fn render_model_preserves_marker_state_across_runs() {
    let tree = build_element_text_tree();

    let first = render_signature(&run_pipeline(&tree));
    let second = render_signature(&run_pipeline(&tree));

    assert_eq!(first.nodes.len(), second.nodes.len());
    assert!(first
        .nodes
        .iter()
        .zip(second.nodes.iter())
        .all(|(left, right)| left.markers == right.markers));
    assert!(first.nodes.iter().all(|node| node.markers.is_empty()));
}

#[test]
fn render_model_does_not_mutate_input_tree() {
    let tree = build_nested_element_text_tree();
    let original = tree.clone();

    let _first = run_pipeline(&tree);
    let _second = run_pipeline(&tree);

    assert_eq!(tree, original);
    assert_eq!(tree.len(), 4);
    assert_eq!(tree.nodes()[0].id(), UiNodeId::new(1));
    assert_eq!(tree.nodes()[1].children(), &[UiNodeId::new(3)]);
    assert_eq!(tree.nodes()[2].parent(), Some(UiNodeId::new(2)));
    assert_eq!(tree.nodes()[3].parent(), Some(UiNodeId::new(3)));
}

#[test]
fn render_model_stability_does_not_create_authority_markers() {
    let element_text = render_signature(&run_pipeline(&build_element_text_tree()));
    let sibling = render_signature(&run_pipeline(&build_sibling_element_text_tree()));
    let nested = render_signature(&run_pipeline(&build_nested_element_text_tree()));
    let slot = render_signature(&run_pipeline(&build_slot_tree()));

    for signature in [element_text, sibling, nested, slot] {
        assert!(signature.nodes.iter().all(|node| matches!(
            node.kind,
            UiRenderNodeKind::Root
                | UiRenderNodeKind::Element
                | UiRenderNodeKind::Text
                | UiRenderNodeKind::Fragment
        )));
        assert!(signature.nodes.iter().all(|node| node.markers.is_empty()));
        assert!(signature.nodes.iter().all(|node| {
            !node.markers.iter().any(|marker| {
                matches!(
                    marker,
                    UiRenderMarker::Property
                        | UiRenderMarker::Action
                        | UiRenderMarker::EffectBoundary
                )
            })
        }));
    }
}

#[test]
fn render_model_stability_for_slot_fixture_is_deterministic() {
    let tree = build_slot_tree();

    let first = run_pipeline(&tree);
    let second = run_pipeline(&tree);

    let first_sig = render_signature(&first);
    let second_sig = render_signature(&second);

    assert_eq!(first_sig, second_sig);
    assert_eq!(first_sig.nodes.len(), 2);
    assert_eq!(first_sig.nodes[1].kind, UiRenderNodeKind::Fragment);
    assert!(first_sig.nodes.iter().all(|node| node.markers.is_empty()));
}
