use prom_ui::model::{UiNode, UiNodeId, UiNodeKind, UiNodeResolution, UiTree, UiTreeId};

#[test]
fn ui_node_resolution_defaults_to_known() {
    let node = UiNode::new(UiNodeId::new(1), UiNodeKind::Element);
    assert_eq!(node.resolution(), UiNodeResolution::Known);
    assert_eq!(node.kind(), UiNodeKind::Element);
}

#[test]
fn ui_node_unknown_resolution_is_preserved() {
    let node = UiNode::with_resolution(
        UiNodeId::new(2),
        UiNodeKind::Text,
        UiNodeResolution::Unknown,
    );
    assert_eq!(node.resolution(), UiNodeResolution::Unknown);
}

#[test]
fn ui_node_conflict_resolution_is_preserved() {
    let node = UiNode::with_resolution(
        UiNodeId::new(3),
        UiNodeKind::Slot,
        UiNodeResolution::Conflict,
    );
    assert_eq!(node.resolution(), UiNodeResolution::Conflict);
}

#[test]
fn ui_node_kind_remains_structural_when_resolution_is_unknown() {
    let node = UiNode::with_resolution(
        UiNodeId::new(4),
        UiNodeKind::Fragment,
        UiNodeResolution::Unknown,
    );
    assert_eq!(node.kind(), UiNodeKind::Fragment);
    assert_eq!(node.resolution(), UiNodeResolution::Unknown);
    // Not squashed into a boolean representation.
    assert_ne!(format!("{:?}", node.resolution()), "true");
    assert_ne!(format!("{:?}", node.resolution()), "false");
}

#[test]
fn ui_node_kind_remains_structural_when_resolution_is_conflict() {
    let node = UiNode::with_resolution(
        UiNodeId::new(5),
        UiNodeKind::Root,
        UiNodeResolution::Conflict,
    );
    assert_eq!(node.kind(), UiNodeKind::Root);
    assert_eq!(node.resolution(), UiNodeResolution::Conflict);
}

#[test]
fn ui_tree_preserves_node_order_with_resolution_metadata() {
    let mut tree = UiTree::new(UiTreeId::new(10));

    let root = UiNode::new(UiNodeId::new(100), UiNodeKind::Root);
    let mut known_child =
        UiNode::with_parent(UiNodeId::new(101), UiNodeKind::Element, UiNodeId::new(100));
    known_child.push_child(UiNodeId::new(102));

    let unknown_child = UiNode::with_resolution(
        UiNodeId::new(102),
        UiNodeKind::Text,
        UiNodeResolution::Unknown,
    );

    let conflict_child = UiNode::with_resolution(
        UiNodeId::new(103),
        UiNodeKind::Element,
        UiNodeResolution::Conflict,
    );

    tree.push_node(root);
    tree.push_node(known_child);
    tree.push_node(unknown_child);
    tree.push_node(conflict_child);

    assert_eq!(tree.len(), 4);
    assert_eq!(tree.nodes()[0].id(), UiNodeId::new(100));
    assert_eq!(tree.nodes()[1].id(), UiNodeId::new(101));
    assert_eq!(tree.nodes()[2].id(), UiNodeId::new(102));
    assert_eq!(tree.nodes()[3].id(), UiNodeId::new(103));

    assert_eq!(tree.nodes()[1].resolution(), UiNodeResolution::Known);
    assert_eq!(tree.nodes()[2].resolution(), UiNodeResolution::Unknown);
    assert_eq!(tree.nodes()[3].resolution(), UiNodeResolution::Conflict);
}

#[test]
fn ui_model_seed_does_not_require_renderer_layout_or_backend() {
    let tree = UiTree::new(UiTreeId::new(999));
    let node = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);

    assert_eq!(tree.id().raw(), 999);
    assert_eq!(node.kind(), UiNodeKind::Root);
    // Tree and Node instantiate without backend injection, renderer context, or layout constraints.
}

#[test]
fn ui_model_seed_does_not_introduce_workbench_or_studio_authority() {
    let node = UiNode::with_resolution(
        UiNodeId::new(42),
        UiNodeKind::Element,
        UiNodeResolution::Known,
    );
    // Model relies solely on local primitive vocabulary.
    assert_eq!(node.id().raw(), 42);
    assert_eq!(node.resolution(), UiNodeResolution::Known);
}
