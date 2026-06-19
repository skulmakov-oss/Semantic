use prom_ui::ast_slot_ir_intent::{
    build_ast_slot_ir_intents, UiAstSlotIrIntentDiagnosticKind, UiAstSlotIrIntentKind,
    UiAstSlotIrIntentState,
};
use prom_ui::model::{
    UiAst, UiAstNode, UiAstNodeId, UiAstNodeKind, UiIr, UiIrNode, UiIrNodeId, UiIrNodeKind, UiNode,
    UiNodeId, UiNodeKind, UiNodeResolution, UiTree, UiTreeId,
};
use prom_ui::tree_slot_ast_intent::build_tree_slot_ast_intents;
use prom_ui::tree_slot_intent::build_tree_slot_carrier_intents;

#[test]
fn empty_ast_slot_intents_build_empty_ir_intent_model() {
    let tree = UiTree::new(UiTreeId::new(1));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let ast = UiAst::new();
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let ir = UiIr::new();
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(ir_intents.entries().len(), 0);
}

#[test]
fn single_ast_slot_intent_links_to_ir_fragment() {
    let mut tree = UiTree::new(UiTreeId::new(2));
    let slot = UiNode::with_resolution(UiNodeId::new(3), UiNodeKind::Slot, UiNodeResolution::Known);
    tree.push_node(slot);
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    let ast_fragment = UiAstNode::new(UiAstNodeId::new(3), UiAstNodeKind::Fragment);
    ast.push_node(ast_fragment);
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();

    let mut ir = UiIr::new();
    let ir_fragment = UiIrNode::new(UiIrNodeId::new(3), UiIrNodeKind::Fragment);
    ir.push_node(ir_fragment);

    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(ir_intents.entries().len(), 1);
    let entry = &ir_intents.entries()[0];
    assert_eq!(entry.ir_node_id(), UiIrNodeId::new(3));
    assert_eq!(entry.ir_node_kind(), UiIrNodeKind::Fragment);
    assert_eq!(
        entry.kind(),
        UiAstSlotIrIntentKind::IrFragmentLinkedToAstSlotIntent
    );
    assert_eq!(entry.state(), UiAstSlotIrIntentState::Deferred);
}

#[test]
fn multiple_ast_slot_intents_preserve_order() {
    let mut tree = UiTree::new(UiTreeId::new(3));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(10),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(20),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(10),
        UiAstNodeKind::Fragment,
    ));
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(20),
        UiAstNodeKind::Fragment,
    ));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();

    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(10), UiIrNodeKind::Fragment));
    ir.push_node(UiIrNode::new(UiIrNodeId::new(20), UiIrNodeKind::Fragment));

    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(ir_intents.entries().len(), 2);
    assert_eq!(ir_intents.entries()[0].ir_node_id(), UiIrNodeId::new(10));
    assert_eq!(ir_intents.entries()[1].ir_node_id(), UiIrNodeId::new(20));
}

#[test]
fn ir_intent_entry_id_is_deterministic() {
    let mut tree = UiTree::new(UiTreeId::new(4));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(5),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(UiAstNodeId::new(5), UiAstNodeKind::Fragment));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();

    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(5), UiIrNodeKind::Fragment));
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();

    let entry = &ir_intents.entries()[0];
    assert_eq!(entry.id().raw(), 5);
}

#[test]
fn ir_intent_preserves_source_ast_intent_entry_id() {
    let mut tree = UiTree::new(UiTreeId::new(4));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(5),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(UiAstNodeId::new(5), UiAstNodeKind::Fragment));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();

    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(5), UiIrNodeKind::Fragment));
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();

    let entry = &ir_intents.entries()[0];
    assert_eq!(
        entry.source_ast_intent_entry_id(),
        ast_intents.entries()[0].id()
    );
}

#[test]
fn ir_intent_preserves_source_tree_intent_entry_id() {
    let mut tree = UiTree::new(UiTreeId::new(4));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(5),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(UiAstNodeId::new(5), UiAstNodeKind::Fragment));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();

    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(5), UiIrNodeKind::Fragment));
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();

    let entry = &ir_intents.entries()[0];
    assert_eq!(
        entry.source_tree_intent_entry_id(),
        tree_intents.entries()[0].id()
    );
}

#[test]
fn ir_intent_preserves_source_tree_id() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(5),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(UiAstNodeId::new(5), UiAstNodeKind::Fragment));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(5), UiIrNodeKind::Fragment));
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(ir_intents.entries()[0].source_tree_id(), UiTreeId::new(42));
}

#[test]
fn ir_intent_preserves_source_tree_node_id() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(50),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(50),
        UiAstNodeKind::Fragment,
    ));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment));
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(
        ir_intents.entries()[0].source_tree_node_id(),
        UiNodeId::new(50)
    );
}

#[test]
fn ir_intent_preserves_source_tree_node_kind() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(50),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(50),
        UiAstNodeKind::Fragment,
    ));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment));
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(
        ir_intents.entries()[0].source_tree_node_kind(),
        UiNodeKind::Slot
    );
}

#[test]
fn ir_intent_preserves_source_tree_resolution() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(50),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(50),
        UiAstNodeKind::Fragment,
    ));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment));
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(
        ir_intents.entries()[0].source_tree_resolution(),
        UiNodeResolution::Known
    );
}

#[test]
fn ir_intent_preserves_source_ast_node_id() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(50),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(50),
        UiAstNodeKind::Fragment,
    ));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment));
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(
        ir_intents.entries()[0].source_ast_node_id(),
        UiAstNodeId::new(50)
    );
}

#[test]
fn ir_intent_preserves_source_ast_node_kind_as_fragment() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(50),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(50),
        UiAstNodeKind::Fragment,
    ));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment));
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(
        ir_intents.entries()[0].source_ast_node_kind(),
        UiAstNodeKind::Fragment
    );
}

#[test]
fn ir_intent_preserves_ir_node_id() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(50),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(50),
        UiAstNodeKind::Fragment,
    ));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment));
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(ir_intents.entries()[0].ir_node_id(), UiIrNodeId::new(50));
}

#[test]
fn ir_intent_preserves_ir_node_kind_as_fragment() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(50),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(50),
        UiAstNodeKind::Fragment,
    ));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment));
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(
        ir_intents.entries()[0].ir_node_kind(),
        UiIrNodeKind::Fragment
    );
}

#[test]
fn ir_intent_preserves_parent_tree_handle() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let mut slot =
        UiNode::with_resolution(UiNodeId::new(50), UiNodeKind::Slot, UiNodeResolution::Known);
    slot.set_parent(Some(UiNodeId::new(1)));
    root.push_child(UiNodeId::new(50));
    tree.push_node(root);
    tree.push_node(slot);
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    let mut ast_root = UiAstNode::new(UiAstNodeId::new(1), UiAstNodeKind::Root);
    let mut ast_slot = UiAstNode::new(UiAstNodeId::new(50), UiAstNodeKind::Fragment);
    ast_slot.set_parent(Some(UiAstNodeId::new(1)));
    ast_root.push_child(UiAstNodeId::new(50));
    ast.push_node(ast_root);
    ast.push_node(ast_slot);
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    let mut ir_root = UiIrNode::new(UiIrNodeId::new(1), UiIrNodeKind::Root);
    let mut ir_slot = UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment);
    ir_slot.set_parent(Some(UiIrNodeId::new(1)));
    ir_root.push_child(UiIrNodeId::new(50));
    ir.push_node(ir_root);
    ir.push_node(ir_slot);
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(
        ir_intents.entries()[0].parent_tree_node_id(),
        Some(UiNodeId::new(1))
    );
}

#[test]
fn ir_intent_preserves_child_tree_handles() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    let mut slot =
        UiNode::with_resolution(UiNodeId::new(50), UiNodeKind::Slot, UiNodeResolution::Known);
    let mut child = UiNode::new(UiNodeId::new(2), UiNodeKind::Element);
    child.set_parent(Some(UiNodeId::new(50)));
    slot.push_child(UiNodeId::new(2));
    tree.push_node(slot);
    tree.push_node(child);
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    let mut ast_slot = UiAstNode::new(UiAstNodeId::new(50), UiAstNodeKind::Fragment);
    let mut ast_child = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Element);
    ast_child.set_parent(Some(UiAstNodeId::new(50)));
    ast_slot.push_child(UiAstNodeId::new(2));
    ast.push_node(ast_slot);
    ast.push_node(ast_child);
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    let mut ir_slot = UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment);
    let mut ir_child = UiIrNode::new(UiIrNodeId::new(2), UiIrNodeKind::Element);
    ir_child.set_parent(Some(UiIrNodeId::new(50)));
    ir_slot.push_child(UiIrNodeId::new(2));
    ir.push_node(ir_slot);
    ir.push_node(ir_child);
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(
        ir_intents.entries()[0].child_tree_node_ids(),
        &[UiNodeId::new(2)]
    );
}

#[test]
fn ir_intent_preserves_parent_ast_handle() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let mut slot =
        UiNode::with_resolution(UiNodeId::new(50), UiNodeKind::Slot, UiNodeResolution::Known);
    slot.set_parent(Some(UiNodeId::new(1)));
    root.push_child(UiNodeId::new(50));
    tree.push_node(root);
    tree.push_node(slot);
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    let mut ast_root = UiAstNode::new(UiAstNodeId::new(1), UiAstNodeKind::Root);
    let mut ast_slot = UiAstNode::new(UiAstNodeId::new(50), UiAstNodeKind::Fragment);
    ast_slot.set_parent(Some(UiAstNodeId::new(1)));
    ast_root.push_child(UiAstNodeId::new(50));
    ast.push_node(ast_root);
    ast.push_node(ast_slot);
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    let mut ir_root = UiIrNode::new(UiIrNodeId::new(1), UiIrNodeKind::Root);
    let mut ir_slot = UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment);
    ir_slot.set_parent(Some(UiIrNodeId::new(1)));
    ir_root.push_child(UiIrNodeId::new(50));
    ir.push_node(ir_root);
    ir.push_node(ir_slot);
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(
        ir_intents.entries()[0].parent_ast_node_id(),
        Some(UiAstNodeId::new(1))
    );
}

#[test]
fn ir_intent_preserves_child_ast_handles() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    let mut slot =
        UiNode::with_resolution(UiNodeId::new(50), UiNodeKind::Slot, UiNodeResolution::Known);
    let mut child = UiNode::new(UiNodeId::new(2), UiNodeKind::Element);
    child.set_parent(Some(UiNodeId::new(50)));
    slot.push_child(UiNodeId::new(2));
    tree.push_node(slot);
    tree.push_node(child);
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    let mut ast_slot = UiAstNode::new(UiAstNodeId::new(50), UiAstNodeKind::Fragment);
    let mut ast_child = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Element);
    ast_child.set_parent(Some(UiAstNodeId::new(50)));
    ast_slot.push_child(UiAstNodeId::new(2));
    ast.push_node(ast_slot);
    ast.push_node(ast_child);
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    let mut ir_slot = UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment);
    let mut ir_child = UiIrNode::new(UiIrNodeId::new(2), UiIrNodeKind::Element);
    ir_child.set_parent(Some(UiIrNodeId::new(50)));
    ir_slot.push_child(UiIrNodeId::new(2));
    ir.push_node(ir_slot);
    ir.push_node(ir_child);
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(
        ir_intents.entries()[0].child_ast_node_ids(),
        &[UiAstNodeId::new(2)]
    );
}

#[test]
fn ir_intent_preserves_parent_ir_handle() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let mut slot =
        UiNode::with_resolution(UiNodeId::new(50), UiNodeKind::Slot, UiNodeResolution::Known);
    slot.set_parent(Some(UiNodeId::new(1)));
    root.push_child(UiNodeId::new(50));
    tree.push_node(root);
    tree.push_node(slot);
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    let mut ast_root = UiAstNode::new(UiAstNodeId::new(1), UiAstNodeKind::Root);
    let mut ast_slot = UiAstNode::new(UiAstNodeId::new(50), UiAstNodeKind::Fragment);
    ast_slot.set_parent(Some(UiAstNodeId::new(1)));
    ast_root.push_child(UiAstNodeId::new(50));
    ast.push_node(ast_root);
    ast.push_node(ast_slot);
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    let mut ir_root = UiIrNode::new(UiIrNodeId::new(1), UiIrNodeKind::Root);
    let mut ir_slot = UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment);
    ir_slot.set_parent(Some(UiIrNodeId::new(1)));
    ir_root.push_child(UiIrNodeId::new(50));
    ir.push_node(ir_root);
    ir.push_node(ir_slot);
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(
        ir_intents.entries()[0].parent_ir_node_id(),
        Some(UiIrNodeId::new(1))
    );
}

#[test]
fn ir_intent_preserves_child_ir_handles() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    let mut slot =
        UiNode::with_resolution(UiNodeId::new(50), UiNodeKind::Slot, UiNodeResolution::Known);
    let mut child = UiNode::new(UiNodeId::new(2), UiNodeKind::Element);
    child.set_parent(Some(UiNodeId::new(50)));
    slot.push_child(UiNodeId::new(2));
    tree.push_node(slot);
    tree.push_node(child);
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    let mut ast_slot = UiAstNode::new(UiAstNodeId::new(50), UiAstNodeKind::Fragment);
    let mut ast_child = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Element);
    ast_child.set_parent(Some(UiAstNodeId::new(50)));
    ast_slot.push_child(UiAstNodeId::new(2));
    ast.push_node(ast_slot);
    ast.push_node(ast_child);
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    let mut ir_slot = UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment);
    let mut ir_child = UiIrNode::new(UiIrNodeId::new(2), UiIrNodeKind::Element);
    ir_child.set_parent(Some(UiIrNodeId::new(50)));
    ir_slot.push_child(UiIrNodeId::new(2));
    ir.push_node(ir_slot);
    ir.push_node(ir_child);
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(
        ir_intents.entries()[0].child_ir_node_ids(),
        &[UiIrNodeId::new(2)]
    );
}

#[test]
fn unknown_resolution_is_preserved() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(50),
        UiNodeKind::Slot,
        UiNodeResolution::Unknown,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(50),
        UiAstNodeKind::Fragment,
    ));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment));
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(
        ir_intents.entries()[0].source_tree_resolution(),
        UiNodeResolution::Unknown
    );
}

#[test]
fn conflict_resolution_is_preserved() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(50),
        UiNodeKind::Slot,
        UiNodeResolution::Conflict,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(50),
        UiAstNodeKind::Fragment,
    ));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment));
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(
        ir_intents.entries()[0].source_tree_resolution(),
        UiNodeResolution::Conflict
    );
}

#[test]
fn missing_ir_node_returns_diagnostic() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(50),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(50),
        UiAstNodeKind::Fragment,
    ));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let ir = UiIr::new();
    let err = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap_err();
    assert_eq!(err.len(), 1);
    match err.diagnostics()[0].kind() {
        UiAstSlotIrIntentDiagnosticKind::MissingIrNode { source_ast_node_id } => {
            assert_eq!(*source_ast_node_id, UiAstNodeId::new(50));
        }
        _ => panic!("expected missing ir node"),
    }
}

#[test]
fn non_fragment_ir_node_returns_diagnostic() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(50),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(50),
        UiAstNodeKind::Fragment,
    ));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Element));
    let err = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap_err();
    assert_eq!(err.len(), 1);
    match err.diagnostics()[0].kind() {
        UiAstSlotIrIntentDiagnosticKind::UnexpectedIrNodeKind {
            source_ast_node_id,
            ir_node_id,
            actual_kind,
        } => {
            assert_eq!(*source_ast_node_id, UiAstNodeId::new(50));
            assert_eq!(*ir_node_id, UiIrNodeId::new(50));
            assert_eq!(*actual_kind, UiIrNodeKind::Element);
        }
        _ => panic!("expected unexpected ir node kind"),
    }
}

#[test]
fn diagnostic_preserves_source_ast_node_id() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(50),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(50),
        UiAstNodeKind::Fragment,
    ));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let ir = UiIr::new();
    let err = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap_err();
    match err.diagnostics()[0].kind() {
        UiAstSlotIrIntentDiagnosticKind::MissingIrNode { source_ast_node_id } => {
            assert_eq!(*source_ast_node_id, UiAstNodeId::new(50));
        }
        _ => panic!("unexpected"),
    }
}

#[test]
fn non_fragment_diagnostic_preserves_ir_node_id_and_kind() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(50),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(50),
        UiAstNodeKind::Fragment,
    ));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Text));
    let err = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap_err();
    match err.diagnostics()[0].kind() {
        UiAstSlotIrIntentDiagnosticKind::UnexpectedIrNodeKind {
            ir_node_id,
            actual_kind,
            ..
        } => {
            assert_eq!(*ir_node_id, UiIrNodeId::new(50));
            assert_eq!(*actual_kind, UiIrNodeKind::Text);
        }
        _ => panic!("unexpected"),
    }
}

#[test]
fn mismatched_input_returns_no_partial_model() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(50),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(51),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(50),
        UiAstNodeKind::Fragment,
    ));
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(51),
        UiAstNodeKind::Fragment,
    ));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment));
    // missing 51
    let err = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap_err();
    assert_eq!(err.len(), 1);
}

#[test]
fn builder_does_not_mutate_inputs() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(50),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(50),
        UiAstNodeKind::Fragment,
    ));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment));

    let len_before = ir.nodes().len();
    let _ = build_ast_slot_ir_intents(&ast_intents, &ir);
    assert_eq!(ir.nodes().len(), len_before);
}

#[test]
fn builder_does_not_call_lowering_or_render_pipeline() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(50),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(50),
        UiAstNodeKind::Fragment,
    ));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment));

    let intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(intents.entries().len(), 1);
}

#[test]
fn metadata_does_not_create_property_action_or_effect_boundary_semantics() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(50),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(50),
        UiAstNodeKind::Fragment,
    ));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment));

    let intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(
        intents.entries()[0].kind(),
        UiAstSlotIrIntentKind::IrFragmentLinkedToAstSlotIntent
    );
}

#[test]
fn metadata_does_not_create_carrier_or_effect_boundary() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(50),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(50),
        UiAstNodeKind::Fragment,
    ));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment));

    let intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(
        intents.entries()[0].state(),
        UiAstSlotIrIntentState::Deferred
    );
}

#[test]
fn slot_ir_intent_is_inert_and_non_authoritative() {
    let mut tree = UiTree::new(UiTreeId::new(42));
    tree.push_node(UiNode::with_resolution(
        UiNodeId::new(50),
        UiNodeKind::Slot,
        UiNodeResolution::Known,
    ));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(
        UiAstNodeId::new(50),
        UiAstNodeKind::Fragment,
    ));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(50), UiIrNodeKind::Fragment));

    let len_before = ir.nodes().len();
    let intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    assert_eq!(intents.entries().len(), 1);
    assert_eq!(ir.nodes().len(), len_before);
}
