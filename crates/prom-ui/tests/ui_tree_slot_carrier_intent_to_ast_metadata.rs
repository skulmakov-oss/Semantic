use prom_ui::model::{
    UiAst, UiAstNode, UiAstNodeId, UiAstNodeKind, UiNode, UiNodeId, UiNodeKind, UiNodeResolution,
    UiTree, UiTreeId,
};
use prom_ui::tree_slot_ast_intent::{
    build_tree_slot_ast_intents, UiTreeSlotAstIntentDiagnosticKind, UiTreeSlotAstIntentKind,
    UiTreeSlotAstIntentState,
};
use prom_ui::tree_slot_intent::build_tree_slot_carrier_intents;

#[test]
fn empty_tree_slot_intents_build_empty_ast_intent_model() {
    let tree = UiTree::new(UiTreeId::new(1));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let ast = UiAst::new();
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    assert!(ast_intents.entries().is_empty());
}

#[test]
fn single_tree_slot_intent_links_to_ast_fragment() {
    let mut tree = UiTree::new(UiTreeId::new(2));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let mut slot =
        UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    slot.set_parent(Some(UiNodeId::new(1)));
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_root = UiAstNode::new(UiAstNodeId::new(1), UiAstNodeKind::Root);
    let ast_fragment = UiAstNode::with_parent(
        UiAstNodeId::new(2),
        UiAstNodeKind::Fragment,
        UiAstNodeId::new(1),
    );
    ast.push_node(ast_root);
    ast.push_node(ast_fragment);

    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    assert_eq!(ast_intents.entries().len(), 1);
}

#[test]
fn multiple_tree_slot_intents_preserve_order() {
    let mut tree = UiTree::new(UiTreeId::new(3));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let mut slot1 =
        UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    let mut slot2 =
        UiNode::with_resolution(UiNodeId::new(3), UiNodeKind::Slot, UiNodeResolution::Known);
    slot1.set_parent(Some(UiNodeId::new(1)));
    slot2.set_parent(Some(UiNodeId::new(1)));
    root.push_child(UiNodeId::new(2));
    root.push_child(UiNodeId::new(3));
    tree.push_node(root);
    tree.push_node(slot1);
    tree.push_node(slot2);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_root = UiAstNode::new(UiAstNodeId::new(1), UiAstNodeKind::Root);
    let ast_frag1 = UiAstNode::with_parent(
        UiAstNodeId::new(2),
        UiAstNodeKind::Fragment,
        UiAstNodeId::new(1),
    );
    let ast_frag2 = UiAstNode::with_parent(
        UiAstNodeId::new(3),
        UiAstNodeKind::Fragment,
        UiAstNodeId::new(1),
    );
    ast.push_node(ast_root);
    ast.push_node(ast_frag1);
    ast.push_node(ast_frag2);

    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    assert_eq!(ast_intents.entries().len(), 2);
    assert_eq!(ast_intents.entries()[0].source_tree_node_id().raw(), 2);
    assert_eq!(ast_intents.entries()[1].source_tree_node_id().raw(), 3);
}

#[test]
fn ast_intent_entry_id_is_deterministic() {
    let mut tree = UiTree::new(UiTreeId::new(4));
    let slot = UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_fragment = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast.push_node(ast_fragment);

    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    assert_eq!(ast_intents.entries()[0].id().raw(), 2);
}

#[test]
fn ast_intent_preserves_source_tree_intent_entry_id() {
    let mut tree = UiTree::new(UiTreeId::new(5));
    let slot = UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_fragment = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast.push_node(ast_fragment);

    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    assert_eq!(
        ast_intents.entries()[0].source_tree_intent_entry_id(),
        tree_intents.entries()[0].id()
    );
}

#[test]
fn ast_intent_preserves_source_tree_id() {
    let mut tree = UiTree::new(UiTreeId::new(6));
    let slot = UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_fragment = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast.push_node(ast_fragment);

    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    assert_eq!(ast_intents.entries()[0].source_tree_id(), UiTreeId::new(6));
}

#[test]
fn ast_intent_preserves_source_tree_node_id() {
    let mut tree = UiTree::new(UiTreeId::new(7));
    let slot = UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_fragment = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast.push_node(ast_fragment);

    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    assert_eq!(
        ast_intents.entries()[0].source_tree_node_id(),
        UiNodeId::new(2)
    );
}

#[test]
fn ast_intent_preserves_source_tree_node_kind() {
    let mut tree = UiTree::new(UiTreeId::new(8));
    let slot = UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_fragment = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast.push_node(ast_fragment);

    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    assert_eq!(
        ast_intents.entries()[0].source_tree_node_kind(),
        UiNodeKind::Slot
    );
}

#[test]
fn ast_intent_preserves_source_tree_resolution() {
    let mut tree = UiTree::new(UiTreeId::new(9));
    let slot = UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_fragment = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast.push_node(ast_fragment);

    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    assert_eq!(
        ast_intents.entries()[0].source_tree_resolution(),
        UiNodeResolution::Known
    );
}

#[test]
fn ast_intent_preserves_ast_node_id() {
    let mut tree = UiTree::new(UiTreeId::new(10));
    let slot = UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_fragment = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast.push_node(ast_fragment);

    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    assert_eq!(ast_intents.entries()[0].ast_node_id(), UiAstNodeId::new(2));
}

#[test]
fn ast_intent_preserves_ast_node_kind_as_fragment() {
    let mut tree = UiTree::new(UiTreeId::new(11));
    let slot = UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_fragment = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast.push_node(ast_fragment);

    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    assert_eq!(
        ast_intents.entries()[0].ast_node_kind(),
        UiAstNodeKind::Fragment
    );
}

#[test]
fn ast_intent_preserves_parent_tree_handle() {
    let mut tree = UiTree::new(UiTreeId::new(12));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let mut slot =
        UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    slot.set_parent(Some(UiNodeId::new(1)));
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_root = UiAstNode::new(UiAstNodeId::new(1), UiAstNodeKind::Root);
    let ast_fragment = UiAstNode::with_parent(
        UiAstNodeId::new(2),
        UiAstNodeKind::Fragment,
        UiAstNodeId::new(1),
    );
    ast.push_node(ast_root);
    ast.push_node(ast_fragment);

    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    assert_eq!(
        ast_intents.entries()[0].parent_tree_node_id(),
        Some(UiNodeId::new(1))
    );
}

#[test]
fn ast_intent_preserves_child_tree_handles() {
    let mut tree = UiTree::new(UiTreeId::new(13));
    let mut slot =
        UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    let mut text =
        UiNode::with_resolution(UiNodeId::new(3), UiNodeKind::Text, UiNodeResolution::Known);
    slot.push_child(UiNodeId::new(3));
    text.set_parent(Some(UiNodeId::new(2)));
    tree.push_node(slot);
    tree.push_node(text);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let mut ast_fragment = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast_fragment.push_child(UiAstNodeId::new(3));
    ast.push_node(ast_fragment);

    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    assert_eq!(
        ast_intents.entries()[0].child_tree_node_ids(),
        &[UiNodeId::new(3)]
    );
}

#[test]
fn ast_intent_preserves_parent_ast_handle() {
    let mut tree = UiTree::new(UiTreeId::new(14));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let mut slot =
        UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    slot.set_parent(Some(UiNodeId::new(1)));
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_root = UiAstNode::new(UiAstNodeId::new(1), UiAstNodeKind::Root);
    let ast_fragment = UiAstNode::with_parent(
        UiAstNodeId::new(2),
        UiAstNodeKind::Fragment,
        UiAstNodeId::new(1),
    );
    ast.push_node(ast_root);
    ast.push_node(ast_fragment);

    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    assert_eq!(
        ast_intents.entries()[0].parent_ast_node_id(),
        Some(UiAstNodeId::new(1))
    );
}

#[test]
fn ast_intent_preserves_child_ast_handles() {
    let mut tree = UiTree::new(UiTreeId::new(15));
    let mut slot =
        UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    let mut text =
        UiNode::with_resolution(UiNodeId::new(3), UiNodeKind::Text, UiNodeResolution::Known);
    slot.push_child(UiNodeId::new(3));
    text.set_parent(Some(UiNodeId::new(2)));
    tree.push_node(slot);
    tree.push_node(text);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let mut ast_fragment = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast_fragment.push_child(UiAstNodeId::new(3));
    ast.push_node(ast_fragment);

    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    assert_eq!(
        ast_intents.entries()[0].child_ast_node_ids(),
        &[UiAstNodeId::new(3)]
    );
}

#[test]
fn unknown_resolution_is_preserved() {
    let mut tree = UiTree::new(UiTreeId::new(16));
    let slot = UiNode::with_resolution(
        UiNodeId::new(2),
        UiNodeKind::Slot,
        UiNodeResolution::Unknown,
    );
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_fragment = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast.push_node(ast_fragment);

    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    assert_eq!(
        ast_intents.entries()[0].source_tree_resolution(),
        UiNodeResolution::Unknown
    );
}

#[test]
fn conflict_resolution_is_preserved() {
    let mut tree = UiTree::new(UiTreeId::new(17));
    let slot = UiNode::with_resolution(
        UiNodeId::new(2),
        UiNodeKind::Slot,
        UiNodeResolution::Conflict,
    );
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_fragment = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast.push_node(ast_fragment);

    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    assert_eq!(
        ast_intents.entries()[0].source_tree_resolution(),
        UiNodeResolution::Conflict
    );
}

#[test]
fn missing_ast_node_returns_diagnostic() {
    let mut tree = UiTree::new(UiTreeId::new(18));
    let slot = UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let ast = UiAst::new(); // empty ast

    let err = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap_err();
    assert_eq!(err.diagnostics().len(), 1);
    match err.diagnostics()[0].kind() {
        UiTreeSlotAstIntentDiagnosticKind::MissingAstNode {
            source_tree_node_id,
        } => {
            assert_eq!(*source_tree_node_id, UiNodeId::new(2));
        }
        _ => panic!("Expected MissingAstNode"),
    }
}

#[test]
fn non_fragment_ast_node_returns_diagnostic() {
    let mut tree = UiTree::new(UiTreeId::new(19));
    let slot = UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_element = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Element);
    ast.push_node(ast_element);

    let err = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap_err();
    assert_eq!(err.diagnostics().len(), 1);
    match err.diagnostics()[0].kind() {
        UiTreeSlotAstIntentDiagnosticKind::UnexpectedAstNodeKind {
            source_tree_node_id,
            ast_node_id,
            actual_kind,
        } => {
            assert_eq!(*source_tree_node_id, UiNodeId::new(2));
            assert_eq!(*ast_node_id, UiAstNodeId::new(2));
            assert_eq!(*actual_kind, UiAstNodeKind::Element);
        }
        _ => panic!("Expected UnexpectedAstNodeKind"),
    }
}

#[test]
fn diagnostic_preserves_source_tree_node_id() {
    let mut tree = UiTree::new(UiTreeId::new(20));
    let slot = UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let ast = UiAst::new();

    let err = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap_err();
    match err.diagnostics()[0].kind() {
        UiTreeSlotAstIntentDiagnosticKind::MissingAstNode {
            source_tree_node_id,
        } => {
            assert_eq!(*source_tree_node_id, UiNodeId::new(2));
        }
        _ => panic!("Expected MissingAstNode"),
    }
}

#[test]
fn non_fragment_diagnostic_preserves_ast_node_id_and_kind() {
    let mut tree = UiTree::new(UiTreeId::new(21));
    let slot = UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_text = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Text);
    ast.push_node(ast_text);

    let err = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap_err();
    match err.diagnostics()[0].kind() {
        UiTreeSlotAstIntentDiagnosticKind::UnexpectedAstNodeKind {
            source_tree_node_id,
            ast_node_id,
            actual_kind,
        } => {
            assert_eq!(*source_tree_node_id, UiNodeId::new(2));
            assert_eq!(*ast_node_id, UiAstNodeId::new(2));
            assert_eq!(*actual_kind, UiAstNodeKind::Text);
        }
        _ => panic!("Expected UnexpectedAstNodeKind"),
    }
}

#[test]
fn mismatched_input_returns_no_partial_model() {
    let mut tree = UiTree::new(UiTreeId::new(22));
    let slot1 =
        UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    let slot2 =
        UiNode::with_resolution(UiNodeId::new(3), UiNodeKind::Slot, UiNodeResolution::Known);
    tree.push_node(slot1);
    tree.push_node(slot2);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_fragment = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast.push_node(ast_fragment);

    let result = build_tree_slot_ast_intents(&tree_intents, &ast);
    assert!(result.is_err());
}

#[test]
fn builder_does_not_mutate_inputs() {
    let mut tree = UiTree::new(UiTreeId::new(23));
    let slot = UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_fragment = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast.push_node(ast_fragment);

    let len_before = ast.len();
    let _ = build_tree_slot_ast_intents(&tree_intents, &ast);
    assert_eq!(ast.len(), len_before);
}

#[test]
fn builder_does_not_call_tree_to_ast_or_lowering_pipeline() {
    let mut tree = UiTree::new(UiTreeId::new(23));
    let slot = UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_fragment = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast.push_node(ast_fragment);

    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    assert_eq!(ast_intents.entries().len(), 1);
    let entry = &ast_intents.entries()[0];
    assert_eq!(entry.ast_node_kind(), UiAstNodeKind::Fragment);
    assert_eq!(
        entry.kind(),
        UiTreeSlotAstIntentKind::AstFragmentLinkedToTreeSlotIntent
    );
    assert_eq!(entry.state(), UiTreeSlotAstIntentState::Deferred);
    assert_eq!(entry.source_tree_node_kind(), UiNodeKind::Slot);
}

#[test]
fn metadata_does_not_create_attribute_binding_or_action_semantics() {
    let mut tree = UiTree::new(UiTreeId::new(24));
    let slot = UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_fragment = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast.push_node(ast_fragment);

    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let entry = &ast_intents.entries()[0];
    assert_eq!(
        entry.kind(),
        UiTreeSlotAstIntentKind::AstFragmentLinkedToTreeSlotIntent
    );
}

#[test]
fn metadata_does_not_create_carrier_or_effect_boundary() {
    let mut tree = UiTree::new(UiTreeId::new(25));
    let slot = UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_fragment = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast.push_node(ast_fragment);

    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let entry = &ast_intents.entries()[0];
    assert_eq!(entry.state(), UiTreeSlotAstIntentState::Deferred);
}

#[test]
fn slot_ast_intent_is_inert_and_non_authoritative() {
    let mut tree = UiTree::new(UiTreeId::new(26));
    let slot = UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    tree.push_node(slot);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_fragment = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast.push_node(ast_fragment);

    let intents_len_before = tree_intents.entries().len();
    let ast_len_before = ast.len();
    let ast_node_id_before = ast.nodes()[0].id();

    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();

    assert_eq!(ast_intents.entries().len(), 1);
    let entry = &ast_intents.entries()[0];
    assert_eq!(entry.state(), UiTreeSlotAstIntentState::Deferred);
    assert_eq!(entry.source_tree_resolution(), UiNodeResolution::Known);
    assert_eq!(entry.ast_node_kind(), UiAstNodeKind::Fragment);

    assert_eq!(tree_intents.entries().len(), intents_len_before);
    assert_eq!(ast.len(), ast_len_before);
    assert_eq!(ast.nodes()[0].id(), ast_node_id_before);
}
