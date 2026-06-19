use prom_ui::ast_slot_ir_intent::{build_ast_slot_ir_intents, UiAstSlotIrIntentModel};
use prom_ui::ir_slot_projection_intent::{
    build_ir_slot_projection_intents, UiIrSlotProjectionIntentDiagnosticKind,
    UiIrSlotProjectionIntentKind, UiIrSlotProjectionIntentState,
};
use prom_ui::model::{
    UiAst, UiAstNode, UiAstNodeId, UiAstNodeKind, UiIr, UiIrNode, UiIrNodeId, UiIrNodeKind, UiNode,
    UiNodeId, UiNodeKind, UiNodeResolution, UiTree, UiTreeId,
};
use prom_ui::projection::{
    UiProjectedNode, UiProjectedNodeId, UiProjectedNodeKind, UiProjectionArtifact,
    UiProjectionArtifactId,
};
use prom_ui::tree_slot_ast_intent::build_tree_slot_ast_intents;
use prom_ui::tree_slot_intent::build_tree_slot_carrier_intents;

fn build_baseline_artifact() -> (
    UiAstSlotIrIntentModel,
    UiProjectionArtifact,
    UiProjectedNodeId,
    UiProjectedNodeId,
) {
    let mut tree = UiTree::new(UiTreeId::new(10));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let mut slot =
        UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    slot.set_parent(Some(UiNodeId::new(1)));
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);
    tree.push_node(slot);

    let mut ast = UiAst::new();
    let mut ast_root = UiAstNode::new(UiAstNodeId::new(3), UiAstNodeKind::Root);
    let mut ast_frag = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast_frag.set_parent(Some(UiAstNodeId::new(3)));
    ast_root.push_child(UiAstNodeId::new(2));
    ast.push_node(ast_root);
    ast.push_node(ast_frag);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();

    let mut ir = UiIr::new();
    let mut ir_root = UiIrNode::new(UiIrNodeId::new(1), UiIrNodeKind::Root);
    let mut ir_frag = UiIrNode::new(UiIrNodeId::new(2), UiIrNodeKind::Fragment);
    ir_frag.set_parent(Some(UiIrNodeId::new(1)));
    ir_root.push_child(UiIrNodeId::new(2));
    ir.push_node(ir_root);
    ir.push_node(ir_frag);

    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();

    let mut projection = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));
    let mut proj_root = UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(7),
        UiProjectedNodeKind::Root,
        UiIrNodeId::new(1),
    );
    let mut proj_frag = UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(8),
        UiProjectedNodeKind::Fragment,
        UiIrNodeId::new(2),
    );
    proj_frag.set_parent(UiProjectedNodeId::new(7));
    proj_root.push_child(UiProjectedNodeId::new(8));

    projection.push_node(proj_root);
    projection.push_node(proj_frag);

    (
        ir_intents,
        projection,
        UiProjectedNodeId::new(7),
        UiProjectedNodeId::new(8),
    )
}

#[test]
fn empty_ir_slot_intents_build_empty_projection_intent_model() {
    let mut tree = UiTree::new(UiTreeId::new(1));
    tree.push_node(UiNode::new(UiNodeId::new(1), UiNodeKind::Root));
    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();

    let mut ast = UiAst::new();
    ast.push_node(UiAstNode::new(UiAstNodeId::new(1), UiAstNodeKind::Root));
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();

    let mut ir = UiIr::new();
    ir.push_node(UiIrNode::new(UiIrNodeId::new(1), UiIrNodeKind::Root));
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();

    let projection = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));

    let result = build_ir_slot_projection_intents(&ir_intents, &projection);
    let model = result.unwrap();
    assert_eq!(model.entries().len(), 0);
}

#[test]
fn single_ir_slot_intent_links_to_projected_fragment() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();

    assert_eq!(model.entries().len(), 1);
    let entry = &model.entries()[0];
    assert_eq!(entry.projected_node_kind(), UiProjectedNodeKind::Fragment);
    assert_eq!(
        entry.kind(),
        UiIrSlotProjectionIntentKind::ProjectedFragmentLinkedToIrSlotIntent
    );
}

#[test]
fn multiple_ir_slot_intents_preserve_order() {
    let mut tree = UiTree::new(UiTreeId::new(10));
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

    let mut ast = UiAst::new();
    let mut ast_root = UiAstNode::new(UiAstNodeId::new(1), UiAstNodeKind::Root);
    let mut ast_frag1 = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    let mut ast_frag2 = UiAstNode::new(UiAstNodeId::new(3), UiAstNodeKind::Fragment);
    ast_frag1.set_parent(Some(UiAstNodeId::new(1)));
    ast_frag2.set_parent(Some(UiAstNodeId::new(1)));
    ast_root.push_child(UiAstNodeId::new(2));
    ast_root.push_child(UiAstNodeId::new(3));
    ast.push_node(ast_root);
    ast.push_node(ast_frag1);
    ast.push_node(ast_frag2);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();

    let mut ir = UiIr::new();
    let mut ir_root = UiIrNode::new(UiIrNodeId::new(1), UiIrNodeKind::Root);
    let mut ir_frag1 = UiIrNode::new(UiIrNodeId::new(2), UiIrNodeKind::Fragment);
    let mut ir_frag2 = UiIrNode::new(UiIrNodeId::new(3), UiIrNodeKind::Fragment);
    ir_frag1.set_parent(Some(UiIrNodeId::new(1)));
    ir_frag2.set_parent(Some(UiIrNodeId::new(1)));
    ir_root.push_child(UiIrNodeId::new(2));
    ir_root.push_child(UiIrNodeId::new(3));
    ir.push_node(ir_root);
    ir.push_node(ir_frag1);
    ir.push_node(ir_frag2);

    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();

    let mut projection = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));
    let mut proj_root = UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(31),
        UiProjectedNodeKind::Root,
        UiIrNodeId::new(1),
    );

    let mut proj_frag1 = UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(32),
        UiProjectedNodeKind::Fragment,
        UiIrNodeId::new(2),
    );
    proj_frag1.set_parent(UiProjectedNodeId::new(31));

    let mut proj_frag2 = UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(33),
        UiProjectedNodeKind::Fragment,
        UiIrNodeId::new(3),
    );
    proj_frag2.set_parent(UiProjectedNodeId::new(31));

    proj_root.push_child(UiProjectedNodeId::new(32));
    proj_root.push_child(UiProjectedNodeId::new(33));

    projection.push_node(proj_root);
    projection.push_node(proj_frag1);
    projection.push_node(proj_frag2);

    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();

    assert_eq!(model.entries().len(), 2);
    assert_eq!(model.entries()[0].source_tree_node_id(), UiNodeId::new(2));
    assert_eq!(model.entries()[1].source_tree_node_id(), UiNodeId::new(3));
}

#[test]
fn projection_intent_entry_id_is_deterministic() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.id().raw(), 2);
}

#[test]
fn projection_intent_preserves_source_ir_intent_entry_id() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let source_id = ir_intents.entries()[0].id();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.source_ir_intent_entry_id(), source_id);
}

#[test]
fn projection_intent_preserves_source_ast_intent_entry_id() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let source_id = ir_intents.entries()[0].source_ast_intent_entry_id();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.source_ast_intent_entry_id(), source_id);
}

#[test]
fn projection_intent_preserves_source_tree_intent_entry_id() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let source_id = ir_intents.entries()[0].source_tree_intent_entry_id();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.source_tree_intent_entry_id(), source_id);
}

#[test]
fn projection_intent_preserves_source_tree_id() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.source_tree_id(), UiTreeId::new(10));
}

#[test]
fn projection_intent_preserves_source_tree_node_id() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.source_tree_node_id(), UiNodeId::new(2));
}

#[test]
fn projection_intent_preserves_source_tree_node_kind() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.source_tree_node_kind(), UiNodeKind::Slot);
}

#[test]
fn projection_intent_preserves_source_tree_resolution() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.source_tree_resolution(), UiNodeResolution::Known);
}

#[test]
fn projection_intent_preserves_source_ast_node_id() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.source_ast_node_id(), UiAstNodeId::new(2));
}

#[test]
fn projection_intent_preserves_source_ast_node_kind_as_fragment() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.source_ast_node_kind(), UiAstNodeKind::Fragment);
}

#[test]
fn projection_intent_preserves_source_ir_node_id() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.source_ir_node_id(), UiIrNodeId::new(2));
}

#[test]
fn projection_intent_preserves_source_ir_node_kind_as_fragment() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.source_ir_node_kind(), UiIrNodeKind::Fragment);
}

#[test]
fn projection_intent_preserves_projected_node_id() {
    let (ir_intents, projection, _, proj_frag_id) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.projected_node_id(), proj_frag_id);
}

#[test]
fn projection_intent_preserves_projected_node_kind_as_fragment() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.projected_node_kind(), UiProjectedNodeKind::Fragment);
}

#[test]
fn projection_intent_preserves_parent_tree_handle() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.parent_tree_node_id(), Some(UiNodeId::new(1)));
}

#[test]
fn projection_intent_preserves_child_tree_handles() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.child_tree_node_ids().len(), 0);
}

#[test]
fn projection_intent_preserves_parent_ast_handle() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.parent_ast_node_id(), Some(UiAstNodeId::new(1)));
}

#[test]
fn projection_intent_preserves_child_ast_handles() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.child_ast_node_ids().len(), 0);
}

#[test]
fn projection_intent_preserves_parent_ir_handle() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.parent_ir_node_id(), Some(UiIrNodeId::new(1)));
}

#[test]
fn projection_intent_preserves_child_ir_handles() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.child_ir_node_ids().len(), 0);
}

#[test]
fn projection_intent_preserves_parent_projected_handle() {
    let (ir_intents, projection, proj_root_id, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.parent_projected_node_id(), Some(proj_root_id));
}

#[test]
fn projection_intent_preserves_child_projected_handles() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let entry = &model.entries()[0];
    assert_eq!(entry.child_projected_node_ids().len(), 0);
}

#[test]
fn unknown_resolution_is_preserved() {
    let mut tree = UiTree::new(UiTreeId::new(10));
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

    let mut ast = UiAst::new();
    let mut ast_root = UiAstNode::new(UiAstNodeId::new(3), UiAstNodeKind::Root);
    let mut ast_frag = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast_frag.set_parent(Some(UiAstNodeId::new(3)));
    ast_root.push_child(UiAstNodeId::new(2));
    ast.push_node(ast_root);
    ast.push_node(ast_frag);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();

    let mut ir = UiIr::new();
    let mut ir_root = UiIrNode::new(UiIrNodeId::new(1), UiIrNodeKind::Root);
    let mut ir_frag = UiIrNode::new(UiIrNodeId::new(2), UiIrNodeKind::Fragment);
    ir_frag.set_parent(Some(UiIrNodeId::new(1)));
    ir_root.push_child(UiIrNodeId::new(2));
    ir.push_node(ir_root);
    ir.push_node(ir_frag);

    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();

    let mut projection = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));
    let mut proj_root = UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(7),
        UiProjectedNodeKind::Root,
        UiIrNodeId::new(1),
    );
    let mut proj_frag = UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(8),
        UiProjectedNodeKind::Fragment,
        UiIrNodeId::new(2),
    );
    proj_frag.set_parent(UiProjectedNodeId::new(7));
    proj_root.push_child(UiProjectedNodeId::new(8));

    projection.push_node(proj_root);
    projection.push_node(proj_frag);

    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();

    assert_eq!(
        model.entries()[0].source_tree_resolution(),
        UiNodeResolution::Unknown
    );
}

#[test]
fn conflict_resolution_is_preserved() {
    let mut tree = UiTree::new(UiTreeId::new(10));
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

    let mut ast = UiAst::new();
    let mut ast_root = UiAstNode::new(UiAstNodeId::new(3), UiAstNodeKind::Root);
    let mut ast_frag = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast_frag.set_parent(Some(UiAstNodeId::new(3)));
    ast_root.push_child(UiAstNodeId::new(2));
    ast.push_node(ast_root);
    ast.push_node(ast_frag);

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();

    let mut ir = UiIr::new();
    let mut ir_root = UiIrNode::new(UiIrNodeId::new(1), UiIrNodeKind::Root);
    let mut ir_frag = UiIrNode::new(UiIrNodeId::new(2), UiIrNodeKind::Fragment);
    ir_frag.set_parent(Some(UiIrNodeId::new(1)));
    ir_root.push_child(UiIrNodeId::new(2));
    ir.push_node(ir_root);
    ir.push_node(ir_frag);

    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();

    let mut projection = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));
    let mut proj_root = UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(7),
        UiProjectedNodeKind::Root,
        UiIrNodeId::new(1),
    );
    let mut proj_frag = UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(8),
        UiProjectedNodeKind::Fragment,
        UiIrNodeId::new(2),
    );
    proj_frag.set_parent(UiProjectedNodeId::new(7));
    proj_root.push_child(UiProjectedNodeId::new(8));

    projection.push_node(proj_root);
    projection.push_node(proj_frag);

    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();

    assert_eq!(
        model.entries()[0].source_tree_resolution(),
        UiNodeResolution::Conflict
    );
}

#[test]
fn missing_projected_node_returns_diagnostic() {
    let (ir_intents, mut projection, _, _) = build_baseline_artifact();

    // Create a new projection with only the root, missing the fragment
    let mut new_projection = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));
    for node in projection.nodes() {
        if node.kind() == UiProjectedNodeKind::Root {
            new_projection.push_node(node.clone());
        }
    }

    let result = build_ir_slot_projection_intents(&ir_intents, &new_projection);
    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    let mut iter = diagnostics.iter();
    let diag = iter.next().unwrap();
    match diag.kind() {
        UiIrSlotProjectionIntentDiagnosticKind::MissingProjectedNode { source_ir_node_id } => {
            assert_eq!(*source_ir_node_id, UiIrNodeId::new(2));
        }
        _ => panic!("Expected MissingProjectedNode"),
    }
}

#[test]
fn non_fragment_projected_node_returns_diagnostic() {
    let (ir_intents, mut projection, _, _) = build_baseline_artifact();

    let mut new_projection = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));
    for node in projection.nodes() {
        if node.kind() == UiProjectedNodeKind::Fragment {
            let mut new_bad = UiProjectedNode::with_source_ir_node(
                node.id(),
                UiProjectedNodeKind::PropertyCarrier,
                node.source_ir_node_id().unwrap(),
            );
            if let Some(p) = node.parent() {
                new_bad.set_parent(p);
            }
            new_projection.push_node(new_bad);
        } else {
            new_projection.push_node(node.clone());
        }
    }

    let result = build_ir_slot_projection_intents(&ir_intents, &new_projection);
    assert!(result.is_err());
}

#[test]
fn diagnostic_preserves_source_ir_node_id() {
    let (ir_intents, _, _, _) = build_baseline_artifact();
    let new_projection = UiProjectionArtifact::new(UiProjectionArtifactId::new(1)); // missing node
    let result = build_ir_slot_projection_intents(&ir_intents, &new_projection);
    let diagnostics = result.unwrap_err();
    let diag = diagnostics.iter().next().unwrap();
    match diag.kind() {
        UiIrSlotProjectionIntentDiagnosticKind::MissingProjectedNode { source_ir_node_id } => {
            assert_eq!(*source_ir_node_id, UiIrNodeId::new(2));
        }
        _ => panic!("Expected MissingProjectedNode"),
    }
}

#[test]
fn non_fragment_diagnostic_preserves_projected_node_id_and_kind() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let mut new_projection = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));
    for node in projection.nodes() {
        if node.kind() == UiProjectedNodeKind::Fragment {
            let new_bad = UiProjectedNode::with_source_ir_node(
                node.id(),
                UiProjectedNodeKind::PropertyCarrier,
                node.source_ir_node_id().unwrap(),
            );
            new_projection.push_node(new_bad);
        } else {
            new_projection.push_node(node.clone());
        }
    }

    let result = build_ir_slot_projection_intents(&ir_intents, &new_projection);
    let diagnostics = result.unwrap_err();
    let diag = diagnostics.iter().next().unwrap();
    match diag.kind() {
        UiIrSlotProjectionIntentDiagnosticKind::UnexpectedProjectedNodeKind {
            source_ir_node_id,
            projected_node_id,
            actual_kind,
        } => {
            assert_eq!(*source_ir_node_id, UiIrNodeId::new(2));
            assert_eq!(*projected_node_id, UiProjectedNodeId::new(8));
            assert_eq!(*actual_kind, UiProjectedNodeKind::PropertyCarrier);
        }
        _ => panic!("Expected UnexpectedProjectedNodeKind"),
    }
}

#[test]
fn mismatched_input_returns_no_partial_model() {
    let (ir_intents, _, _, _) = build_baseline_artifact();
    let new_projection = UiProjectionArtifact::new(UiProjectionArtifactId::new(1)); // missing node
    let result = build_ir_slot_projection_intents(&ir_intents, &new_projection);
    assert!(result.is_err()); // It doesn't return a partial model, it returns Err
}

#[test]
fn builder_does_not_mutate_inputs() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let intents_clone = ir_intents.clone();
    let projection_clone = projection.clone();

    let _ = build_ir_slot_projection_intents(&ir_intents, &projection);

    assert_eq!(ir_intents.entries().len(), intents_clone.entries().len());
    assert_eq!(projection.nodes().len(), projection_clone.nodes().len());
}

#[test]
fn builder_does_not_call_projection_or_render_pipeline() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();

    let projection_len_before = projection.nodes().len();
    let projected_node_id_before = projection.nodes()[0].id();

    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();

    assert_eq!(model.entries().len(), 1);
    let entry = &model.entries()[0];

    assert_eq!(
        entry.kind(),
        UiIrSlotProjectionIntentKind::ProjectedFragmentLinkedToIrSlotIntent
    );
    assert_eq!(entry.state(), UiIrSlotProjectionIntentState::Deferred);
    assert_eq!(entry.projected_node_kind(), UiProjectedNodeKind::Fragment);
    assert_eq!(entry.source_ir_node_kind(), UiIrNodeKind::Fragment);

    assert_eq!(projection.nodes().len(), projection_len_before);
    assert_eq!(projection.nodes()[0].id(), projected_node_id_before);
}

#[test]
fn metadata_does_not_create_projection_carriers() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    for entry in model.entries() {
        assert_eq!(entry.projected_node_kind(), UiProjectedNodeKind::Fragment);
        assert_ne!(
            entry.projected_node_kind(),
            UiProjectedNodeKind::PropertyCarrier
        );
        assert_ne!(
            entry.projected_node_kind(),
            UiProjectedNodeKind::ActionCarrier
        );
        assert_ne!(
            entry.projected_node_kind(),
            UiProjectedNodeKind::EffectBoundaryMarker
        );
    }
}

#[test]
fn slot_projection_intent_is_inert_and_non_authoritative() {
    let (ir_intents, projection, _, _) = build_baseline_artifact();
    let model = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    for entry in model.entries() {
        assert_eq!(entry.state(), UiIrSlotProjectionIntentState::Deferred);
    }
}
