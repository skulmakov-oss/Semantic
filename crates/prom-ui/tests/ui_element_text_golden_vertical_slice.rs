use prom_ui::lowering::{lower_ast_to_ir, UiLoweringConfig};
use prom_ui::model::{
    UiAst, UiAstNodeKind, UiIr, UiIrNodeKind, UiNode, UiNodeId, UiNodeKind, UiTree, UiTreeId,
};
use prom_ui::projection::{project_ir_to_projection, UiProjectedNodeKind, UiProjectionArtifact};
use prom_ui::renderer::{
    render_projection_to_model, UiRenderMarker, UiRenderModel, UiRenderNodeKind,
};
use prom_ui::tree_bridge::tree_to_ast;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ElementTextPipeline {
    ast: UiAst,
    ir: UiIr,
    projection: UiProjectionArtifact,
    render_model: UiRenderModel,
}

fn build_element_text_tree() -> UiTree {
    let mut tree = UiTree::new(UiTreeId::new(900));

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

fn run_element_text_pipeline(tree: &UiTree) -> ElementTextPipeline {
    let ast = tree_to_ast(tree).expect("tree should bridge to ast");
    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("ast should lower to ir");
    let projection = project_ir_to_projection(&ir).expect("ir should project");
    let render_model =
        render_projection_to_model(&projection).expect("projection should render to model");

    ElementTextPipeline {
        ast,
        ir,
        projection,
        render_model,
    }
}

#[test]
fn element_text_vertical_slice_preserves_tree_structure() {
    let tree = build_element_text_tree();

    assert_eq!(tree.len(), 3);
    assert_eq!(tree.nodes()[0].id(), UiNodeId::new(1));
    assert_eq!(tree.nodes()[0].kind(), UiNodeKind::Root);
    assert_eq!(tree.nodes()[0].children(), &[UiNodeId::new(2)]);

    assert_eq!(tree.nodes()[1].id(), UiNodeId::new(2));
    assert_eq!(tree.nodes()[1].kind(), UiNodeKind::Element);
    assert_eq!(tree.nodes()[1].parent(), Some(UiNodeId::new(1)));
    assert_eq!(tree.nodes()[1].children(), &[UiNodeId::new(3)]);

    assert_eq!(tree.nodes()[2].id(), UiNodeId::new(3));
    assert_eq!(tree.nodes()[2].kind(), UiNodeKind::Text);
    assert_eq!(tree.nodes()[2].parent(), Some(UiNodeId::new(2)));
    assert!(tree.nodes()[2].children().is_empty());
}

#[test]
fn element_text_vertical_slice_preserves_ast_mapping() {
    let tree = build_element_text_tree();
    let ast = tree_to_ast(&tree).expect("tree should bridge to ast");

    assert_eq!(ast.len(), 3);
    assert_eq!(ast.nodes()[0].id().raw(), 1);
    assert_eq!(ast.nodes()[0].kind(), UiAstNodeKind::Root);
    assert_eq!(
        ast.nodes()[0].children(),
        &[prom_ui::model::UiAstNodeId::new(2)]
    );

    assert_eq!(ast.nodes()[1].id().raw(), 2);
    assert_eq!(ast.nodes()[1].kind(), UiAstNodeKind::Element);
    assert_eq!(
        ast.nodes()[1].parent(),
        Some(prom_ui::model::UiAstNodeId::new(1))
    );
    assert_eq!(
        ast.nodes()[1].children(),
        &[prom_ui::model::UiAstNodeId::new(3)]
    );

    assert_eq!(ast.nodes()[2].id().raw(), 3);
    assert_eq!(ast.nodes()[2].kind(), UiAstNodeKind::Text);
    assert_eq!(
        ast.nodes()[2].parent(),
        Some(prom_ui::model::UiAstNodeId::new(2))
    );
    assert!(ast.nodes()[2].children().is_empty());
}

#[test]
fn element_text_vertical_slice_preserves_ir_mapping() {
    let tree = build_element_text_tree();
    let ast = tree_to_ast(&tree).expect("tree should bridge to ast");
    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("ast should lower to ir");

    assert_eq!(ir.len(), 3);
    assert_eq!(ir.nodes()[0].id().raw(), ast.nodes()[0].id().raw());
    assert_eq!(ir.nodes()[0].kind(), UiIrNodeKind::Root);
    assert_eq!(
        ir.nodes()[0].children(),
        &[prom_ui::model::UiIrNodeId::new(2)]
    );

    assert_eq!(ir.nodes()[1].id().raw(), ast.nodes()[1].id().raw());
    assert_eq!(ir.nodes()[1].kind(), UiIrNodeKind::Element);
    assert_eq!(
        ir.nodes()[1].parent(),
        Some(prom_ui::model::UiIrNodeId::new(1))
    );
    assert_eq!(
        ir.nodes()[1].children(),
        &[prom_ui::model::UiIrNodeId::new(3)]
    );

    assert_eq!(ir.nodes()[2].id().raw(), ast.nodes()[2].id().raw());
    assert_eq!(ir.nodes()[2].kind(), UiIrNodeKind::Text);
    assert_eq!(
        ir.nodes()[2].parent(),
        Some(prom_ui::model::UiIrNodeId::new(2))
    );
    assert!(ir.nodes()[2].children().is_empty());
}

#[test]
fn element_text_vertical_slice_preserves_projection_mapping() {
    let tree = build_element_text_tree();
    let ast = tree_to_ast(&tree).expect("tree should bridge to ast");
    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("ast should lower to ir");
    let projection = project_ir_to_projection(&ir).expect("ir should project");

    assert_eq!(projection.len(), 3);
    assert_eq!(
        projection.source_ir_root(),
        Some(prom_ui::model::UiIrNodeId::new(1))
    );
    assert_eq!(projection.nodes()[0].id().raw(), 1);
    assert_eq!(projection.nodes()[0].kind(), UiProjectedNodeKind::Root);
    assert_eq!(
        projection.nodes()[0].source_ir_node_id(),
        Some(prom_ui::model::UiIrNodeId::new(1))
    );
    assert_eq!(
        projection.nodes()[0].children(),
        &[prom_ui::projection::UiProjectedNodeId::new(2)]
    );

    assert_eq!(projection.nodes()[1].id().raw(), 2);
    assert_eq!(projection.nodes()[1].kind(), UiProjectedNodeKind::Element);
    assert_eq!(
        projection.nodes()[1].source_ir_node_id(),
        Some(prom_ui::model::UiIrNodeId::new(2))
    );
    assert_eq!(
        projection.nodes()[1].parent(),
        Some(prom_ui::projection::UiProjectedNodeId::new(1))
    );
    assert_eq!(
        projection.nodes()[1].children(),
        &[prom_ui::projection::UiProjectedNodeId::new(3)]
    );

    assert_eq!(projection.nodes()[2].id().raw(), 3);
    assert_eq!(projection.nodes()[2].kind(), UiProjectedNodeKind::Text);
    assert_eq!(
        projection.nodes()[2].source_ir_node_id(),
        Some(prom_ui::model::UiIrNodeId::new(3))
    );
    assert_eq!(
        projection.nodes()[2].parent(),
        Some(prom_ui::projection::UiProjectedNodeId::new(2))
    );
    assert!(projection.nodes()[2].children().is_empty());
}

#[test]
fn element_text_vertical_slice_preserves_render_sources() {
    let pipeline = run_element_text_pipeline(&build_element_text_tree());

    assert_eq!(pipeline.render_model.nodes().len(), 3);
    assert_eq!(
        pipeline.render_model.source_ir_root(),
        Some(prom_ui::model::UiIrNodeId::new(1))
    );
    assert_eq!(pipeline.render_model.nodes()[0].id().raw(), 1);
    assert_eq!(
        pipeline.render_model.nodes()[0].kind(),
        UiRenderNodeKind::Root
    );
    assert_eq!(
        pipeline.render_model.nodes()[0].source_projection_node(),
        prom_ui::projection::UiProjectedNodeId::new(1)
    );
    assert_eq!(
        pipeline.render_model.nodes()[0].source_ir_node(),
        Some(prom_ui::model::UiIrNodeId::new(1))
    );
    assert!(pipeline.render_model.nodes()[0].markers().is_empty());

    assert_eq!(pipeline.render_model.nodes()[1].id().raw(), 2);
    assert_eq!(
        pipeline.render_model.nodes()[1].kind(),
        UiRenderNodeKind::Element
    );
    assert_eq!(
        pipeline.render_model.nodes()[1].source_projection_node(),
        prom_ui::projection::UiProjectedNodeId::new(2)
    );
    assert_eq!(
        pipeline.render_model.nodes()[1].source_ir_node(),
        Some(prom_ui::model::UiIrNodeId::new(2))
    );
    assert!(pipeline.render_model.nodes()[1].markers().is_empty());

    assert_eq!(pipeline.render_model.nodes()[2].id().raw(), 3);
    assert_eq!(
        pipeline.render_model.nodes()[2].kind(),
        UiRenderNodeKind::Text
    );
    assert_eq!(
        pipeline.render_model.nodes()[2].source_projection_node(),
        prom_ui::projection::UiProjectedNodeId::new(3)
    );
    assert_eq!(
        pipeline.render_model.nodes()[2].source_ir_node(),
        Some(prom_ui::model::UiIrNodeId::new(3))
    );
    assert!(pipeline.render_model.nodes()[2].markers().is_empty());
}

#[test]
fn element_text_vertical_slice_records_golden_id_chain() {
    let tree = build_element_text_tree();
    let first = run_element_text_pipeline(&tree);
    let second = run_element_text_pipeline(&tree);

    assert_eq!(first, second);

    let chain = [
        (
            tree.nodes()[0].id().raw(),
            first.ast.nodes()[0].id().raw(),
            first.ir.nodes()[0].id().raw(),
            first.projection.nodes()[0].id().raw(),
            first.render_model.nodes()[0].id().raw(),
        ),
        (
            tree.nodes()[1].id().raw(),
            first.ast.nodes()[1].id().raw(),
            first.ir.nodes()[1].id().raw(),
            first.projection.nodes()[1].id().raw(),
            first.render_model.nodes()[1].id().raw(),
        ),
        (
            tree.nodes()[2].id().raw(),
            first.ast.nodes()[2].id().raw(),
            first.ir.nodes()[2].id().raw(),
            first.projection.nodes()[2].id().raw(),
            first.render_model.nodes()[2].id().raw(),
        ),
    ];

    assert_eq!(chain, [(1, 1, 1, 1, 1), (2, 2, 2, 2, 2), (3, 3, 3, 3, 3)]);
}

#[test]
fn element_text_vertical_slice_does_not_create_action_or_effect_authority() {
    let pipeline = run_element_text_pipeline(&build_element_text_tree());

    for node in pipeline.render_model.nodes() {
        assert!(matches!(
            node.kind(),
            UiRenderNodeKind::Root | UiRenderNodeKind::Element | UiRenderNodeKind::Text
        ));
        assert!(node.markers().is_empty());
        assert!(!node.markers().iter().any(|marker| matches!(
            marker,
            UiRenderMarker::Property | UiRenderMarker::Action | UiRenderMarker::EffectBoundary
        )));
    }
}

#[test]
fn element_text_vertical_slice_does_not_mutate_input_tree() {
    let tree = build_element_text_tree();
    let original = tree.clone();

    let _pipeline = run_element_text_pipeline(&tree);

    assert_eq!(tree, original);
}
