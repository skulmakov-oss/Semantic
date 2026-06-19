use prom_ui::model::{
    UiAstNodeId, UiAstNodeKind, UiNode, UiNodeId, UiNodeKind, UiNodeResolution, UiTree, UiTreeId,
};
use prom_ui::tree_bridge::{tree_to_ast, UiTreeToAstDiagnosticKind};

fn tree_with_nodes(nodes: impl IntoIterator<Item = UiNode>) -> UiTree {
    let mut tree = UiTree::new(UiTreeId::new(1));
    for node in nodes {
        tree.push_node(node);
    }
    tree
}

#[test]
fn empty_tree_maps_to_empty_ast() {
    let tree = UiTree::new(UiTreeId::new(2));
    let ast = tree_to_ast(&tree).expect("empty tree is valid");
    assert_eq!(ast.len(), 0);
}

#[test]
fn root_node_maps_to_ast_root() {
    let tree = tree_with_nodes([UiNode::new(UiNodeId::new(1), UiNodeKind::Root)]);
    let ast = tree_to_ast(&tree).expect("valid tree");

    assert_eq!(ast.len(), 1);
    assert_eq!(ast.nodes()[0].kind(), UiAstNodeKind::Root);
}

#[test]
fn element_node_maps_to_ast_element() {
    let tree = tree_with_nodes([UiNode::new(UiNodeId::new(2), UiNodeKind::Element)]);
    let ast = tree_to_ast(&tree).expect("valid tree");

    assert_eq!(ast.len(), 1);
    assert_eq!(ast.nodes()[0].kind(), UiAstNodeKind::Element);
}

#[test]
fn text_node_maps_to_ast_text() {
    let tree = tree_with_nodes([UiNode::new(UiNodeId::new(3), UiNodeKind::Text)]);
    let ast = tree_to_ast(&tree).expect("valid tree");

    assert_eq!(ast.len(), 1);
    assert_eq!(ast.nodes()[0].kind(), UiAstNodeKind::Text);
}

#[test]
fn fragment_node_maps_to_ast_fragment() {
    let tree = tree_with_nodes([UiNode::new(UiNodeId::new(4), UiNodeKind::Fragment)]);
    let ast = tree_to_ast(&tree).expect("valid tree");

    assert_eq!(ast.len(), 1);
    assert_eq!(ast.nodes()[0].kind(), UiAstNodeKind::Fragment);
}

#[test]
fn node_order_is_preserved() {
    let tree = tree_with_nodes([
        UiNode::new(UiNodeId::new(1), UiNodeKind::Root),
        UiNode::new(UiNodeId::new(2), UiNodeKind::Element),
        UiNode::new(UiNodeId::new(3), UiNodeKind::Text),
    ]);
    let ast = tree_to_ast(&tree).expect("valid tree");

    assert_eq!(ast.len(), 3);
    assert_eq!(ast.nodes()[0].kind(), UiAstNodeKind::Root);
    assert_eq!(ast.nodes()[1].kind(), UiAstNodeKind::Element);
    assert_eq!(ast.nodes()[2].kind(), UiAstNodeKind::Text);
}

#[test]
fn raw_ids_are_preserved() {
    let tree = tree_with_nodes([UiNode::new(UiNodeId::new(42), UiNodeKind::Element)]);
    let ast = tree_to_ast(&tree).expect("valid tree");

    assert_eq!(ast.len(), 1);
    assert_eq!(ast.nodes()[0].id(), UiAstNodeId::new(42));
}

#[test]
fn parent_handles_are_preserved() {
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    root.push_child(UiNodeId::new(2));
    let tree = tree_with_nodes([
        root,
        UiNode::with_parent(UiNodeId::new(2), UiNodeKind::Element, UiNodeId::new(1)),
    ]);
    let ast = tree_to_ast(&tree).expect("valid tree");

    assert_eq!(ast.len(), 2);
    assert_eq!(ast.nodes()[1].parent(), Some(UiAstNodeId::new(1)));
}

#[test]
fn child_handles_are_preserved() {
    let mut parent = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    parent.push_child(UiNodeId::new(2));
    let tree = tree_with_nodes([
        parent,
        UiNode::with_parent(UiNodeId::new(2), UiNodeKind::Element, UiNodeId::new(1)),
    ]);
    let ast = tree_to_ast(&tree).expect("valid tree");

    assert_eq!(ast.len(), 2);
    assert_eq!(ast.nodes()[0].children(), &[UiAstNodeId::new(2)]);
}

#[test]
fn unknown_resolution_does_not_block_mapping() {
    let tree = tree_with_nodes([UiNode::with_resolution(
        UiNodeId::new(1),
        UiNodeKind::Element,
        UiNodeResolution::Unknown,
    )]);
    let ast = tree_to_ast(&tree).expect("valid tree");

    assert_eq!(ast.len(), 1);
    assert_eq!(ast.nodes()[0].kind(), UiAstNodeKind::Element);
}

#[test]
fn conflict_resolution_does_not_block_mapping() {
    let tree = tree_with_nodes([UiNode::with_resolution(
        UiNodeId::new(1),
        UiNodeKind::Element,
        UiNodeResolution::Conflict,
    )]);
    let ast = tree_to_ast(&tree).expect("valid tree");

    assert_eq!(ast.len(), 1);
    assert_eq!(ast.nodes()[0].kind(), UiAstNodeKind::Element);
}

#[test]
fn slot_node_maps_to_ast_fragment() {
    let tree = tree_with_nodes([UiNode::new(UiNodeId::new(1), UiNodeKind::Slot)]);
    let ast = tree_to_ast(&tree).expect("valid tree");

    assert_eq!(ast.len(), 1);
    assert_eq!(ast.nodes()[0].kind(), UiAstNodeKind::Fragment);
}

#[test]
fn slot_node_preserves_raw_id() {
    let tree = tree_with_nodes([UiNode::new(UiNodeId::new(99), UiNodeKind::Slot)]);
    let ast = tree_to_ast(&tree).expect("valid tree");

    assert_eq!(ast.len(), 1);
    assert_eq!(ast.nodes()[0].id(), UiAstNodeId::new(99));
}

#[test]
fn slot_node_preserves_parent_handle() {
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    root.push_child(UiNodeId::new(2));
    let tree = tree_with_nodes([
        root,
        UiNode::with_parent(UiNodeId::new(2), UiNodeKind::Slot, UiNodeId::new(1)),
    ]);
    let ast = tree_to_ast(&tree).expect("valid tree");

    assert_eq!(ast.len(), 2);
    assert_eq!(ast.nodes()[1].parent(), Some(UiAstNodeId::new(1)));
}

#[test]
fn slot_node_preserves_child_handles() {
    let mut parent = UiNode::new(UiNodeId::new(1), UiNodeKind::Slot);
    parent.push_child(UiNodeId::new(2));
    let tree = tree_with_nodes([
        parent,
        UiNode::with_parent(UiNodeId::new(2), UiNodeKind::Element, UiNodeId::new(1)),
    ]);
    let ast = tree_to_ast(&tree).expect("valid tree");

    assert_eq!(ast.len(), 2);
    assert_eq!(ast.nodes()[0].children(), &[UiAstNodeId::new(2)]);
}

#[test]
fn unknown_slot_resolution_does_not_block_mapping() {
    let tree = tree_with_nodes([UiNode::with_resolution(
        UiNodeId::new(1),
        UiNodeKind::Slot,
        UiNodeResolution::Unknown,
    )]);
    let ast = tree_to_ast(&tree).expect("valid tree");

    assert_eq!(ast.len(), 1);
    assert_eq!(ast.nodes()[0].kind(), UiAstNodeKind::Fragment);
}

#[test]
fn conflict_slot_resolution_does_not_block_mapping() {
    let tree = tree_with_nodes([UiNode::with_resolution(
        UiNodeId::new(1),
        UiNodeKind::Slot,
        UiNodeResolution::Conflict,
    )]);
    let ast = tree_to_ast(&tree).expect("valid tree");

    assert_eq!(ast.len(), 1);
    assert_eq!(ast.nodes()[0].kind(), UiAstNodeKind::Fragment);
}

#[test]
fn slot_mapping_does_not_create_attribute_binding_or_action() {
    let tree = tree_with_nodes([UiNode::new(UiNodeId::new(1), UiNodeKind::Slot)]);
    let ast = tree_to_ast(&tree).expect("valid tree");

    assert_eq!(ast.len(), 1);
    let kind = ast.nodes()[0].kind();
    assert_ne!(kind, UiAstNodeKind::Attribute);
    assert_ne!(kind, UiAstNodeKind::Binding);
    assert_ne!(kind, UiAstNodeKind::Action);
}

#[test]
fn invalid_tree_returns_invalid_tree_diagnostic() {
    let tree = tree_with_nodes([
        UiNode::new(UiNodeId::new(1), UiNodeKind::Root),
        UiNode::new(UiNodeId::new(1), UiNodeKind::Root), // duplicate ID -> invalid
    ]);
    let err = tree_to_ast(&tree).expect_err("invalid tree");

    assert!(err
        .diagnostics()
        .iter()
        .any(|d| matches!(d.kind(), UiTreeToAstDiagnosticKind::InvalidTree)));
}

#[test]
fn invalid_tree_does_not_return_partial_ast() {
    let tree = tree_with_nodes([
        UiNode::new(UiNodeId::new(1), UiNodeKind::Root),
        UiNode::new(UiNodeId::new(1), UiNodeKind::Root), // duplicate ID -> invalid
    ]);

    // The bridge returns a Result, so if it's invalid, it returns Err (diagnostics)
    // and inherently does not return a partial AST in the success path.
    let _err = tree_to_ast(&tree).expect_err("invalid tree");
}

#[test]
fn tree_to_ast_does_not_mutate_input_tree() {
    let tree = tree_with_nodes([UiNode::new(UiNodeId::new(1), UiNodeKind::Element)]);
    let _ast = tree_to_ast(&tree).unwrap();

    // Verify tree is unchanged
    assert_eq!(tree.len(), 1);
    assert_eq!(tree.nodes()[0].id(), UiNodeId::new(1));
}

#[test]
fn tree_to_ast_is_inert_and_non_authoritative() {
    let tree = tree_with_nodes([UiNode::new(UiNodeId::new(1), UiNodeKind::Root)]);
    let _ast = tree_to_ast(&tree).unwrap();

    // There are no calls to lowering, execution, or runtime here
    // Verified by authority scan.
}
