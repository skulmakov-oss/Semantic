use prom_ui::projection::project_ir_to_projection;
use prom_ui::{
    lower_ast_to_ir, render_projection_to_model, tree_to_ast, UiAstNodeKind, UiIrNodeId,
    UiIrNodeKind, UiLoweringConfig, UiNode, UiNodeId, UiNodeKind, UiNodeResolution,
    UiProjectedNodeKind, UiRenderMarker, UiRenderModel, UiRenderNodeKind, UiTree, UiTreeId,
};

fn foundation_tree() -> UiTree {
    let mut tree = UiTree::new(UiTreeId::new(1));

    let mut root =
        UiNode::with_resolution(UiNodeId::new(1), UiNodeKind::Root, UiNodeResolution::Known);
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);

    let mut element = UiNode::with_resolution(
        UiNodeId::new(2),
        UiNodeKind::Element,
        UiNodeResolution::Unknown,
    );
    element.set_parent(Some(UiNodeId::new(1)));
    element.push_child(UiNodeId::new(3));
    tree.push_node(element);

    let mut text = UiNode::with_resolution(
        UiNodeId::new(3),
        UiNodeKind::Text,
        UiNodeResolution::Conflict,
    );
    text.set_parent(Some(UiNodeId::new(2)));
    tree.push_node(text);

    tree
}

fn render_signature(
    model: &UiRenderModel,
) -> Vec<(u64, UiRenderNodeKind, u64, Option<u64>, Vec<UiRenderMarker>)> {
    model
        .nodes()
        .iter()
        .map(|node| {
            (
                node.id().raw(),
                node.kind(),
                node.source_projection_node().raw(),
                node.source_ir_node().map(|id| id.raw()),
                node.markers().to_vec(),
            )
        })
        .collect()
}

#[test]
fn foundation_public_api_surface_is_externally_usable() {
    let tree = foundation_tree();
    let ast = tree_to_ast(&tree).expect("tree_to_ast should succeed");
    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("lower_ast_to_ir should succeed");
    let projection =
        project_ir_to_projection(&ir).expect("project_ir_to_projection should succeed");
    let render =
        render_projection_to_model(&projection).expect("render_projection_to_model should succeed");

    assert_eq!(tree.len(), 3);
    assert_eq!(ast.len(), 3);
    assert_eq!(ir.len(), 3);
    assert_eq!(projection.len(), 3);
    assert_eq!(render.nodes().len(), 3);
}

#[test]
fn foundation_public_node_kinds_are_matchable() {
    let tree = foundation_tree();
    let ast = tree_to_ast(&tree).expect("tree_to_ast should succeed");
    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("lower_ast_to_ir should succeed");
    let projection =
        project_ir_to_projection(&ir).expect("project_ir_to_projection should succeed");
    let render =
        render_projection_to_model(&projection).expect("render_projection_to_model should succeed");

    assert!(matches!(tree.nodes()[0].kind(), UiNodeKind::Root));
    assert!(matches!(tree.nodes()[1].kind(), UiNodeKind::Element));
    assert!(matches!(tree.nodes()[2].kind(), UiNodeKind::Text));

    assert!(matches!(ast.nodes()[0].kind(), UiAstNodeKind::Root));
    assert!(matches!(ast.nodes()[1].kind(), UiAstNodeKind::Element));
    assert!(matches!(ast.nodes()[2].kind(), UiAstNodeKind::Text));

    assert!(matches!(ir.nodes()[0].kind(), UiIrNodeKind::Root));
    assert!(matches!(ir.nodes()[1].kind(), UiIrNodeKind::Element));
    assert!(matches!(ir.nodes()[2].kind(), UiIrNodeKind::Text));

    assert!(matches!(
        projection.nodes()[0].kind(),
        UiProjectedNodeKind::Root
    ));
    assert!(matches!(
        projection.nodes()[1].kind(),
        UiProjectedNodeKind::Element
    ));
    assert!(matches!(
        projection.nodes()[2].kind(),
        UiProjectedNodeKind::Text
    ));

    assert!(matches!(render.nodes()[0].kind(), UiRenderNodeKind::Root));
    assert!(matches!(
        render.nodes()[1].kind(),
        UiRenderNodeKind::Element
    ));
    assert!(matches!(render.nodes()[2].kind(), UiRenderNodeKind::Text));
}

#[test]
fn foundation_public_node_resolutions_are_matchable() {
    let tree = foundation_tree();

    assert_eq!(tree.nodes()[0].resolution(), UiNodeResolution::Known);
    assert_eq!(tree.nodes()[1].resolution(), UiNodeResolution::Unknown);
    assert_eq!(tree.nodes()[2].resolution(), UiNodeResolution::Conflict);
}

#[test]
fn foundation_public_ids_are_constructible_and_comparable() {
    let tree_id = UiTreeId::new(1);
    let node_id = UiNodeId::new(2);
    let ast_node_id = prom_ui::UiAstNodeId::new(3);
    let ir_node_id = UiIrNodeId::new(4);
    let projected_node_id = prom_ui::UiProjectedNodeId::new(5);
    let render_node_id = prom_ui::UiRenderNodeId::new(6);

    assert_eq!(tree_id, UiTreeId::new(1));
    assert_eq!(node_id, UiNodeId::new(2));
    assert_eq!(ast_node_id, prom_ui::UiAstNodeId::new(3));
    assert_eq!(ir_node_id, UiIrNodeId::new(4));
    assert_eq!(projected_node_id, prom_ui::UiProjectedNodeId::new(5));
    assert_eq!(render_node_id, prom_ui::UiRenderNodeId::new(6));
}

#[test]
fn foundation_public_ir_nodes_expose_ast_source_evidence() {
    let tree = foundation_tree();
    let ast = tree_to_ast(&tree).expect("tree_to_ast should succeed");
    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("lower_ast_to_ir should succeed");

    assert_eq!(
        ir.nodes()[0].source_ast_node_id(),
        Some(ast.nodes()[0].id())
    );
    assert_eq!(
        ir.nodes()[1].source_ast_node_id(),
        Some(ast.nodes()[1].id())
    );
    assert_eq!(
        ir.nodes()[2].source_ast_node_id(),
        Some(ast.nodes()[2].id())
    );
}

#[test]
fn foundation_public_accessors_expose_render_source_evidence() {
    let tree = foundation_tree();
    let ast = tree_to_ast(&tree).expect("tree_to_ast should succeed");
    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("lower_ast_to_ir should succeed");
    let projection =
        project_ir_to_projection(&ir).expect("project_ir_to_projection should succeed");
    let render =
        render_projection_to_model(&projection).expect("render_projection_to_model should succeed");

    assert_eq!(render.id().raw(), projection.id().raw());
    assert_eq!(render.nodes()[0].id().raw(), 1);
    assert_eq!(render.nodes()[0].kind(), UiRenderNodeKind::Root);
    assert_eq!(
        render.nodes()[0].source_projection_node(),
        projection.nodes()[0].id()
    );
    assert_eq!(render.nodes()[0].source_ir_node(), Some(ir.nodes()[0].id()));
    assert!(render.nodes()[0].markers().is_empty());

    assert_eq!(
        render.nodes()[1].source_projection_node(),
        projection.nodes()[1].id()
    );
    assert_eq!(render.nodes()[1].source_ir_node(), Some(ir.nodes()[1].id()));
    assert!(render.nodes()[1].markers().is_empty());

    assert_eq!(
        render.nodes()[2].source_projection_node(),
        projection.nodes()[2].id()
    );
    assert_eq!(render.nodes()[2].source_ir_node(), Some(ir.nodes()[2].id()));
    assert!(render.nodes()[2].markers().is_empty());
}

#[test]
fn foundation_public_api_is_repeatable() {
    let tree = foundation_tree();

    let render_one = {
        let ast = tree_to_ast(&tree).expect("tree_to_ast should succeed");
        let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("lower_ast_to_ir should succeed");
        let projection =
            project_ir_to_projection(&ir).expect("project_ir_to_projection should succeed");
        render_projection_to_model(&projection).expect("render_projection_to_model should succeed")
    };

    let render_two = {
        let ast = tree_to_ast(&tree).expect("tree_to_ast should succeed");
        let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("lower_ast_to_ir should succeed");
        let projection =
            project_ir_to_projection(&ir).expect("project_ir_to_projection should succeed");
        render_projection_to_model(&projection).expect("render_projection_to_model should succeed")
    };

    assert_eq!(render_signature(&render_one), render_signature(&render_two));
}

#[test]
fn foundation_public_api_stays_inert() {
    let tree = foundation_tree();
    let ast = tree_to_ast(&tree).expect("tree_to_ast should succeed");
    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("lower_ast_to_ir should succeed");
    let projection =
        project_ir_to_projection(&ir).expect("project_ir_to_projection should succeed");
    let render =
        render_projection_to_model(&projection).expect("render_projection_to_model should succeed");

    assert!(render.nodes().iter().all(|node| matches!(
        node.kind(),
        UiRenderNodeKind::Root | UiRenderNodeKind::Element | UiRenderNodeKind::Text
    )));
    assert!(render.nodes().iter().all(|node| node.markers().is_empty()));
    assert!(render_signature(&render)
        .iter()
        .all(|(_, _, _, _, markers)| markers.is_empty()));
}
