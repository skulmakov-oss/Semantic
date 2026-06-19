use prom_ui::{
    lower_ast_to_ir, project_ir_to_projection, render_projection_to_model, tree_to_ast, UiAst,
    UiIr, UiLoweringConfig, UiNode, UiNodeId, UiNodeKind, UiProjectionArtifact, UiRenderModel,
    UiTree, UiTreeId,
};

#[test]
fn public_api_is_locked_and_accessible_from_root() {
    // 1. Instantiate UiTree
    let mut tree = UiTree::new(UiTreeId::new(1));
    let root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    tree.push_node(root);

    // 2. tree_to_ast -> UiAst
    let ast: UiAst = tree_to_ast(&tree).unwrap();
    assert_eq!(ast.nodes().len(), 1);

    // 3. lower_ast_to_ir -> UiIr
    let config = UiLoweringConfig::default();
    let ir: UiIr = lower_ast_to_ir(&ast, &config).unwrap();
    assert_eq!(ir.nodes().len(), 1);

    // 4. project_ir_to_projection -> UiProjectionArtifact
    let projection: UiProjectionArtifact = project_ir_to_projection(&ir).unwrap();
    assert_eq!(projection.nodes().len(), 1);

    // 5. render_projection_to_model -> UiRenderModel
    let render_model: UiRenderModel = render_projection_to_model(&projection).unwrap();
    assert_eq!(render_model.nodes().len(), 1);
}
