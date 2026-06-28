use prom_ui::{
    lower_ast_to_ir, tree_to_ast, UiAstNodeId, UiAstNodeKind, UiIrNodeId, UiIrNodeKind,
    UiLoweringConfig, UiNode, UiNodeId, UiNodeKind, UiTree, UiTreeId,
};

fn foundation_tree() -> UiTree {
    let mut tree = UiTree::new(UiTreeId::new(1));

    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);

    let mut element = UiNode::with_parent(UiNodeId::new(2), UiNodeKind::Element, UiNodeId::new(1));
    element.push_child(UiNodeId::new(3));
    tree.push_node(element);

    tree.push_node(UiNode::with_parent(
        UiNodeId::new(3),
        UiNodeKind::Text,
        UiNodeId::new(2),
    ));

    tree
}

#[test]
fn foundation_public_ir_nodes_expose_source_ast_node_ids() {
    let tree = foundation_tree();
    let ast = tree_to_ast(&tree).expect("tree_to_ast should succeed");
    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("lower_ast_to_ir should succeed");

    assert_eq!(ast.len(), 3);
    assert_eq!(ir.len(), 3);

    let expected = [
        (
            UiIrNodeId::new(1),
            UiIrNodeKind::Root,
            Some(UiAstNodeId::new(1)),
        ),
        (
            UiIrNodeId::new(2),
            UiIrNodeKind::Element,
            Some(UiAstNodeId::new(2)),
        ),
        (
            UiIrNodeId::new(3),
            UiIrNodeKind::Text,
            Some(UiAstNodeId::new(3)),
        ),
    ];

    for (node, (expected_id, expected_kind, expected_source_ast_id)) in
        ir.nodes().iter().zip(expected)
    {
        assert_eq!(node.id(), expected_id);
        assert_eq!(node.kind(), expected_kind);
        assert_eq!(node.source_ast_node_id(), expected_source_ast_id);
    }
}

#[test]
fn foundation_public_api_surface_is_callable_through_crate_facade() {
    let tree = foundation_tree();

    let ast = tree_to_ast(&tree).expect("tree_to_ast should succeed");
    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("lower_ast_to_ir should succeed");

    assert_eq!(tree.id(), UiTreeId::new(1));
    assert_eq!(ast.nodes()[0].kind(), UiAstNodeKind::Root);
    assert_eq!(ast.nodes()[1].kind(), UiAstNodeKind::Element);
    assert_eq!(ast.nodes()[2].kind(), UiAstNodeKind::Text);
    assert_eq!(
        ir.nodes()[0].source_ast_node_id(),
        Some(UiAstNodeId::new(1))
    );
    assert_eq!(
        ir.nodes()[1].source_ast_node_id(),
        Some(UiAstNodeId::new(2))
    );
    assert_eq!(
        ir.nodes()[2].source_ast_node_id(),
        Some(UiAstNodeId::new(3))
    );
}
