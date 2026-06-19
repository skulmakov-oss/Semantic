use prom_ui::model::{UiNode, UiNodeId, UiNodeKind, UiNodeResolution, UiTree, UiTreeId};
use prom_ui::tree_slot_intent::{
    build_tree_slot_carrier_intents, UiTreeSlotCarrierIntentDiagnosticKind,
    UiTreeSlotCarrierIntentKind, UiTreeSlotCarrierIntentState,
};

#[test]
fn empty_tree_builds_empty_slot_intent_model() {
    let tree = UiTree::new(UiTreeId::new(1));
    let model = build_tree_slot_carrier_intents(&tree).unwrap();
    assert!(model.entries().is_empty());
}

#[test]
fn tree_without_slots_builds_empty_slot_intent_model() {
    let mut tree = UiTree::new(UiTreeId::new(2));
    let root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    tree.push_node(root);
    let model = build_tree_slot_carrier_intents(&tree).unwrap();
    assert!(model.entries().is_empty());
}

#[test]
fn single_slot_produces_one_intent_entry() {
    let mut tree = UiTree::new(UiTreeId::new(3));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let slot = UiNode::with_parent(UiNodeId::new(2), UiNodeKind::Slot, UiNodeId::new(1));
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);
    tree.push_node(slot);

    let model = build_tree_slot_carrier_intents(&tree).unwrap();
    assert_eq!(model.entries().len(), 1);
}

#[test]
fn multiple_slots_preserve_tree_order() {
    let mut tree = UiTree::new(UiTreeId::new(4));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let slot1 = UiNode::with_parent(UiNodeId::new(2), UiNodeKind::Slot, UiNodeId::new(1));
    let slot2 = UiNode::with_parent(UiNodeId::new(3), UiNodeKind::Slot, UiNodeId::new(1));
    root.push_child(UiNodeId::new(2));
    root.push_child(UiNodeId::new(3));
    tree.push_node(root);
    tree.push_node(slot1);
    tree.push_node(slot2);

    let model = build_tree_slot_carrier_intents(&tree).unwrap();
    assert_eq!(model.entries().len(), 2);
    assert_eq!(model.entries()[0].source_node_id(), UiNodeId::new(2));
    assert_eq!(model.entries()[1].source_node_id(), UiNodeId::new(3));
}

#[test]
fn slot_intent_entry_id_is_deterministic() {
    let mut tree = UiTree::new(UiTreeId::new(5));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let slot = UiNode::with_parent(UiNodeId::new(2), UiNodeKind::Slot, UiNodeId::new(1));
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);
    tree.push_node(slot);

    let model1 = build_tree_slot_carrier_intents(&tree).unwrap();
    let model2 = build_tree_slot_carrier_intents(&tree).unwrap();
    assert_eq!(model1.entries()[0].id(), model2.entries()[0].id());
    assert_eq!(model1.entries()[0].id().raw(), 2);
}

#[test]
fn slot_intent_preserves_source_tree_id() {
    let mut tree = UiTree::new(UiTreeId::new(6));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let slot = UiNode::with_parent(UiNodeId::new(2), UiNodeKind::Slot, UiNodeId::new(1));
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);
    tree.push_node(slot);

    let model = build_tree_slot_carrier_intents(&tree).unwrap();
    assert_eq!(model.entries()[0].source_tree_id(), UiTreeId::new(6));
}

#[test]
fn slot_intent_preserves_source_node_id() {
    let mut tree = UiTree::new(UiTreeId::new(7));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let slot = UiNode::with_parent(UiNodeId::new(2), UiNodeKind::Slot, UiNodeId::new(1));
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);
    tree.push_node(slot);

    let model = build_tree_slot_carrier_intents(&tree).unwrap();
    assert_eq!(model.entries()[0].source_node_id(), UiNodeId::new(2));
}

#[test]
fn slot_intent_preserves_source_node_kind() {
    let mut tree = UiTree::new(UiTreeId::new(8));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let slot = UiNode::with_parent(UiNodeId::new(2), UiNodeKind::Slot, UiNodeId::new(1));
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);
    tree.push_node(slot);

    let model = build_tree_slot_carrier_intents(&tree).unwrap();
    assert_eq!(model.entries()[0].source_node_kind(), UiNodeKind::Slot);
}

#[test]
fn slot_intent_preserves_parent_handle() {
    let mut tree = UiTree::new(UiTreeId::new(9));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let slot = UiNode::with_parent(UiNodeId::new(2), UiNodeKind::Slot, UiNodeId::new(1));
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);
    tree.push_node(slot);

    let model = build_tree_slot_carrier_intents(&tree).unwrap();
    assert_eq!(model.entries()[0].parent(), Some(UiNodeId::new(1)));
}

#[test]
fn slot_intent_preserves_child_handles() {
    let mut tree = UiTree::new(UiTreeId::new(10));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let mut slot = UiNode::with_parent(UiNodeId::new(2), UiNodeKind::Slot, UiNodeId::new(1));
    let element = UiNode::with_parent(UiNodeId::new(3), UiNodeKind::Element, UiNodeId::new(2));
    slot.push_child(UiNodeId::new(3));
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);
    tree.push_node(slot);
    tree.push_node(element);

    let model = build_tree_slot_carrier_intents(&tree).unwrap();
    assert_eq!(model.entries()[0].children(), &[UiNodeId::new(3)]);
}

#[test]
fn slot_intent_preserves_known_resolution() {
    let mut tree = UiTree::new(UiTreeId::new(11));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let mut slot =
        UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    slot.set_parent(Some(UiNodeId::new(1)));
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);
    tree.push_node(slot);

    let model = build_tree_slot_carrier_intents(&tree).unwrap();
    assert_eq!(
        model.entries()[0].source_resolution(),
        UiNodeResolution::Known
    );
}

#[test]
fn unknown_slot_resolution_does_not_block_intent_metadata() {
    let mut tree = UiTree::new(UiTreeId::new(12));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let mut slot = UiNode::with_resolution(
        UiNodeId::new(2),
        UiNodeKind::Slot,
        UiNodeResolution::Unknown,
    );
    slot.set_parent(Some(UiNodeId::new(1)));
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);
    tree.push_node(slot);

    let model = build_tree_slot_carrier_intents(&tree).unwrap();
    assert_eq!(
        model.entries()[0].source_resolution(),
        UiNodeResolution::Unknown
    );
}

#[test]
fn conflict_slot_resolution_does_not_block_intent_metadata() {
    let mut tree = UiTree::new(UiTreeId::new(13));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let mut slot = UiNode::with_resolution(
        UiNodeId::new(2),
        UiNodeKind::Slot,
        UiNodeResolution::Conflict,
    );
    slot.set_parent(Some(UiNodeId::new(1)));
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);
    tree.push_node(slot);

    let model = build_tree_slot_carrier_intents(&tree).unwrap();
    assert_eq!(
        model.entries()[0].source_resolution(),
        UiNodeResolution::Conflict
    );
}

#[test]
fn invalid_tree_returns_intent_diagnostic() {
    let mut tree = UiTree::new(UiTreeId::new(14));
    let root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    // Tree missing child
    let mut root2 = UiNode::new(UiNodeId::new(2), UiNodeKind::Root);
    root2.push_child(UiNodeId::new(99));
    tree.push_node(root);
    tree.push_node(root2);

    let err = build_tree_slot_carrier_intents(&tree).unwrap_err();
    assert_eq!(
        err.diagnostics()[0].kind(),
        UiTreeSlotCarrierIntentDiagnosticKind::TreeValidationFailed
    );
}

#[test]
fn invalid_tree_returns_no_partial_intent_model() {
    let mut tree = UiTree::new(UiTreeId::new(15));
    let root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    // Tree missing child
    let mut root2 = UiNode::new(UiNodeId::new(2), UiNodeKind::Root);
    root2.push_child(UiNodeId::new(99));
    tree.push_node(root);
    tree.push_node(root2);

    let result = build_tree_slot_carrier_intents(&tree);
    assert!(result.is_err());
}

#[test]
fn slot_intent_builder_does_not_mutate_input_tree() {
    let mut tree = UiTree::new(UiTreeId::new(16));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let slot = UiNode::with_parent(UiNodeId::new(2), UiNodeKind::Slot, UiNodeId::new(1));
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);
    tree.push_node(slot);

    let _ = build_tree_slot_carrier_intents(&tree);
    assert_eq!(tree.len(), 2);
    assert_eq!(tree.nodes()[0].id(), UiNodeId::new(1));
}

#[test]
fn non_slot_nodes_are_ignored() {
    let mut tree = UiTree::new(UiTreeId::new(17));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let mut element = UiNode::with_parent(UiNodeId::new(2), UiNodeKind::Element, UiNodeId::new(1));
    let text = UiNode::with_parent(UiNodeId::new(3), UiNodeKind::Text, UiNodeId::new(2));
    root.push_child(UiNodeId::new(2));
    element.push_child(UiNodeId::new(3));
    tree.push_node(root);
    tree.push_node(element);
    tree.push_node(text);

    let model = build_tree_slot_carrier_intents(&tree).unwrap();
    assert!(model.entries().is_empty());
}

#[test]
fn slot_intent_does_not_create_attribute_binding_or_action_semantics() {
    let mut tree = UiTree::new(UiTreeId::new(18));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let slot = UiNode::with_parent(UiNodeId::new(2), UiNodeKind::Slot, UiNodeId::new(1));
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);
    tree.push_node(slot);

    let model = build_tree_slot_carrier_intents(&tree).unwrap();
    for entry in model.entries() {
        assert_eq!(
            entry.kind(),
            UiTreeSlotCarrierIntentKind::StructuralSlotBoundary
        );
        assert_eq!(entry.state(), UiTreeSlotCarrierIntentState::Deferred);
        // Does not produce UiAstNodeKind::Attribute/Binding/Action
    }
}

#[test]
fn slot_intent_does_not_create_carrier_or_effect_boundary() {
    let mut tree = UiTree::new(UiTreeId::new(19));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let mut slot =
        UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    slot.set_parent(Some(UiNodeId::new(1)));
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);
    tree.push_node(slot);

    let model = build_tree_slot_carrier_intents(&tree).unwrap();

    assert_eq!(model.entries().len(), 1);
    let entry = &model.entries()[0];
    assert_eq!(
        entry.kind(),
        UiTreeSlotCarrierIntentKind::StructuralSlotBoundary
    );
    assert_eq!(entry.state(), UiTreeSlotCarrierIntentState::Deferred);
    assert_eq!(entry.source_node_kind(), UiNodeKind::Slot);
}

#[test]
fn slot_intent_is_inert_and_non_authoritative() {
    let mut tree = UiTree::new(UiTreeId::new(20));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let mut slot =
        UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    slot.set_parent(Some(UiNodeId::new(1)));
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);
    tree.push_node(slot);

    let tree_len_before = tree.len();
    let node_id_0_before = tree.nodes()[0].id();
    let node_id_1_before = tree.nodes()[1].id();

    let model = build_tree_slot_carrier_intents(&tree).unwrap();

    assert_eq!(model.entries().len(), 1);
    assert_eq!(
        model.entries()[0].state(),
        UiTreeSlotCarrierIntentState::Deferred
    );
    assert_eq!(
        model.entries()[0].source_resolution(),
        UiNodeResolution::Known
    );

    assert_eq!(tree.len(), tree_len_before);
    assert_eq!(tree.nodes()[0].id(), node_id_0_before);
    assert_eq!(tree.nodes()[1].id(), node_id_1_before);
}
