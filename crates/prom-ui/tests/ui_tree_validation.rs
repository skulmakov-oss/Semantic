use prom_ui::model::{UiNode, UiNodeId, UiNodeKind, UiNodeResolution, UiTree, UiTreeId};
use prom_ui::validation::{validate_tree, UiTreeValidationConfig, UiTreeValidationDiagnosticKind};

fn tree_with_nodes(nodes: impl IntoIterator<Item = UiNode>) -> UiTree {
    let mut tree = UiTree::new(UiTreeId::new(1));
    for node in nodes {
        tree.push_node(node);
    }
    tree
}

#[test]
fn empty_tree_is_valid() {
    let tree = UiTree::new(UiTreeId::new(2));
    let result = validate_tree(&tree, &UiTreeValidationConfig::default());
    assert_eq!(result, Ok(()));
}

#[test]
fn single_root_tree_is_valid() {
    let tree = tree_with_nodes([UiNode::new(UiNodeId::new(1), UiNodeKind::Root)]);
    let result = validate_tree(&tree, &UiTreeValidationConfig::default());
    assert_eq!(result, Ok(()));
}

#[test]
fn zero_root_non_empty_tree_is_valid_in_first_seed() {
    let tree = tree_with_nodes([UiNode::new(UiNodeId::new(2), UiNodeKind::Element)]);
    let result = validate_tree(&tree, &UiTreeValidationConfig::default());
    assert_eq!(result, Ok(()));
}

#[test]
fn multiple_roots_returns_diagnostic() {
    let tree = tree_with_nodes([
        UiNode::new(UiNodeId::new(1), UiNodeKind::Root),
        UiNode::new(UiNodeId::new(2), UiNodeKind::Root),
    ]);

    let err = validate_tree(&tree, &UiTreeValidationConfig::default()).expect_err("multiple roots");
    assert!(err
        .diagnostics()
        .iter()
        .any(|d| matches!(d.kind(), UiTreeValidationDiagnosticKind::MultipleRoots)));
}

#[test]
fn duplicate_node_id_returns_diagnostic() {
    let tree = tree_with_nodes([
        UiNode::new(UiNodeId::new(3), UiNodeKind::Root),
        UiNode::new(UiNodeId::new(3), UiNodeKind::Element),
    ]);

    let err = validate_tree(&tree, &UiTreeValidationConfig::default()).expect_err("duplicate id");
    assert!(err.diagnostics().iter().any(|d| matches!(
        d.kind(),
        UiTreeValidationDiagnosticKind::DuplicateNodeId(id) if id == UiNodeId::new(3)
    )));
}

#[test]
fn missing_parent_target_returns_diagnostic() {
    let tree = tree_with_nodes([UiNode::with_parent(
        UiNodeId::new(4),
        UiNodeKind::Element,
        UiNodeId::new(99),
    )]);

    let err = validate_tree(&tree, &UiTreeValidationConfig::default()).expect_err("missing parent");
    assert!(err.diagnostics().iter().any(|d| matches!(
        d.kind(),
        UiTreeValidationDiagnosticKind::MissingParentTarget { node_id, parent_id }
        if node_id == UiNodeId::new(4) && parent_id == UiNodeId::new(99)
    )));
}

#[test]
fn missing_child_target_returns_diagnostic() {
    let mut node = UiNode::new(UiNodeId::new(5), UiNodeKind::Root);
    node.push_child(UiNodeId::new(100));
    let tree = tree_with_nodes([node]);

    let err = validate_tree(&tree, &UiTreeValidationConfig::default()).expect_err("missing child");
    assert!(err.diagnostics().iter().any(|d| matches!(
        d.kind(),
        UiTreeValidationDiagnosticKind::MissingChildTarget { node_id, child_id }
        if node_id == UiNodeId::new(5) && child_id == UiNodeId::new(100)
    )));
}

#[test]
fn consistent_parent_child_relationship_is_valid() {
    let mut parent = UiNode::new(UiNodeId::new(6), UiNodeKind::Root);
    parent.push_child(UiNodeId::new(7));
    let child = UiNode::with_parent(UiNodeId::new(7), UiNodeKind::Element, UiNodeId::new(6));
    let tree = tree_with_nodes([parent, child]);

    let result = validate_tree(&tree, &UiTreeValidationConfig::default());
    assert_eq!(result, Ok(()));
}

#[test]
fn parent_without_matching_child_returns_diagnostic() {
    let parent = UiNode::new(UiNodeId::new(8), UiNodeKind::Root);
    let child = UiNode::with_parent(UiNodeId::new(9), UiNodeKind::Element, UiNodeId::new(8));
    let tree = tree_with_nodes([parent, child]);

    let err = validate_tree(&tree, &UiTreeValidationConfig::default())
        .expect_err("inconsistent relationship");
    assert!(err.diagnostics().iter().any(|d| matches!(
        d.kind(),
        UiTreeValidationDiagnosticKind::InconsistentParentChild { parent_id, child_id }
        if parent_id == UiNodeId::new(8) && child_id == UiNodeId::new(9)
    )));
}

#[test]
fn child_without_matching_parent_returns_diagnostic() {
    let mut parent = UiNode::new(UiNodeId::new(10), UiNodeKind::Root);
    parent.push_child(UiNodeId::new(11));
    let child = UiNode::new(UiNodeId::new(11), UiNodeKind::Element);
    let tree = tree_with_nodes([parent, child]);

    let err = validate_tree(&tree, &UiTreeValidationConfig::default())
        .expect_err("inconsistent relationship");
    assert!(err.diagnostics().iter().any(|d| matches!(
        d.kind(),
        UiTreeValidationDiagnosticKind::InconsistentParentChild { parent_id, child_id }
        if parent_id == UiNodeId::new(10) && child_id == UiNodeId::new(11)
    )));
}

#[test]
fn self_parent_returns_diagnostic() {
    let tree = tree_with_nodes([UiNode::with_parent(
        UiNodeId::new(12),
        UiNodeKind::Element,
        UiNodeId::new(12),
    )]);

    let err = validate_tree(&tree, &UiTreeValidationConfig::default()).expect_err("self parent");
    assert!(err.diagnostics().iter().any(|d| matches!(
        d.kind(),
        UiTreeValidationDiagnosticKind::SelfParent(id) if id == UiNodeId::new(12)
    )));
}

#[test]
fn self_child_returns_diagnostic() {
    let mut node = UiNode::new(UiNodeId::new(13), UiNodeKind::Root);
    node.push_child(UiNodeId::new(13));
    let tree = tree_with_nodes([node]);

    let err = validate_tree(&tree, &UiTreeValidationConfig::default()).expect_err("self child");
    assert!(err.diagnostics().iter().any(|d| matches!(
        d.kind(),
        UiTreeValidationDiagnosticKind::SelfChild(id) if id == UiNodeId::new(13)
    )));
}

#[test]
fn unknown_resolution_does_not_invalidate_tree() {
    let tree = tree_with_nodes([UiNode::with_resolution(
        UiNodeId::new(14),
        UiNodeKind::Element,
        UiNodeResolution::Unknown,
    )]);

    let result = validate_tree(&tree, &UiTreeValidationConfig::default());
    assert_eq!(result, Ok(()));
}

#[test]
fn conflict_resolution_does_not_invalidate_tree() {
    let tree = tree_with_nodes([UiNode::with_resolution(
        UiNodeId::new(15),
        UiNodeKind::Element,
        UiNodeResolution::Conflict,
    )]);

    let result = validate_tree(&tree, &UiTreeValidationConfig::default());
    assert_eq!(result, Ok(()));
}

#[test]
fn validation_does_not_mutate_input_tree() {
    let tree = tree_with_nodes([UiNode::new(UiNodeId::new(16), UiNodeKind::Element)]);

    let _ = validate_tree(&tree, &UiTreeValidationConfig::default());

    assert_eq!(tree.len(), 1);
    assert_eq!(tree.nodes()[0].id(), UiNodeId::new(16));
}

#[test]
fn diagnostics_preserve_node_ids() {
    let tree = tree_with_nodes([UiNode::with_parent(
        UiNodeId::new(17),
        UiNodeKind::Element,
        UiNodeId::new(18),
    )]);

    let err = validate_tree(&tree, &UiTreeValidationConfig::default()).expect_err("missing parent");
    assert_eq!(err.diagnostics()[0].node_id(), Some(UiNodeId::new(17)));
}

#[test]
fn diagnostic_order_is_deterministic() {
    let mut node = UiNode::new(UiNodeId::new(19), UiNodeKind::Root);
    node.push_child(UiNodeId::new(20)); // missing child
    let mut node2 = UiNode::new(
        UiNodeId::new(19), // duplicate id
        UiNodeKind::Element,
    );
    node2.push_child(UiNodeId::new(21)); // missing child

    let tree = tree_with_nodes([node, node2]);

    let err =
        validate_tree(&tree, &UiTreeValidationConfig::default()).expect_err("multiple errors");

    // Order should follow traversal:
    // Node 19 (first): MissingChildTarget (20)
    // Node 19 (second): DuplicateNodeId (19), MissingChildTarget (21)

    let mut iter = err.diagnostics().iter();
    assert!(matches!(
        iter.next().unwrap().kind(),
        UiTreeValidationDiagnosticKind::MissingChildTarget { child_id, .. } if child_id == UiNodeId::new(20)
    ));
    assert!(matches!(
        iter.next().unwrap().kind(),
        UiTreeValidationDiagnosticKind::DuplicateNodeId(id) if id == UiNodeId::new(19)
    ));
    assert!(matches!(
        iter.next().unwrap().kind(),
        UiTreeValidationDiagnosticKind::MissingChildTarget { child_id, .. } if child_id == UiNodeId::new(21)
    ));
}

#[test]
fn tree_validation_is_inert_and_non_authoritative() {
    let _config = UiTreeValidationConfig;
    let tree = tree_with_nodes([UiNode::new(UiNodeId::new(22), UiNodeKind::Root)]);

    let result = validate_tree(&tree, &UiTreeValidationConfig::default());
    assert_eq!(result, Ok(()));
}
