use prom_ui::ast_slot_ir_intent::build_ast_slot_ir_intents;
use prom_ui::ir_slot_projection_intent::build_ir_slot_projection_intents;
use prom_ui::model::{
    UiAst, UiAstNode, UiAstNodeId, UiAstNodeKind, UiIr, UiIrNode, UiIrNodeId, UiIrNodeKind, UiNode,
    UiNodeId, UiNodeKind, UiNodeResolution, UiTree, UiTreeId,
};
use prom_ui::projection::{
    UiProjectedNode, UiProjectedNodeId, UiProjectedNodeKind, UiProjectionArtifact,
    UiProjectionArtifactId,
};
use prom_ui::projection_slot_render_intent::{
    build_projection_slot_render_intents, UiProjectionSlotRenderIntentDiagnosticKind,
    UiProjectionSlotRenderIntentKind, UiProjectionSlotRenderIntentState,
};
use prom_ui::renderer::{
    render_projection_to_model, UiRenderModel, UiRenderNodeId, UiRenderNodeKind,
};
use prom_ui::tree_slot_ast_intent::build_tree_slot_ast_intents;
use prom_ui::tree_slot_intent::build_tree_slot_carrier_intents;

fn build_baseline_artifact() -> (
    prom_ui::ir_slot_projection_intent::UiIrSlotProjectionIntentModel,
    UiRenderModel,
    UiProjectionArtifact,
    UiIr,
    UiAst,
    UiTree,
) {
    let mut tree = UiTree::new(UiTreeId::new(10));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let mut frag =
        UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    frag.set_parent(Some(UiNodeId::new(1)));
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);
    tree.push_node(frag);

    let mut ast = UiAst::new();
    let mut ast_root = UiAstNode::new(UiAstNodeId::new(1), UiAstNodeKind::Root);
    let mut ast_frag = UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Fragment);
    ast_frag.set_parent(Some(UiAstNodeId::new(1)));
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

    let proj_intents = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let render_model = render_projection_to_model(&projection).unwrap();

    (proj_intents, render_model, projection, ir, ast, tree)
}

#[test]
fn single_projection_intent_links_to_render_fragment() {
    let (proj_intents, render_model, _, _, _, _) = build_baseline_artifact();
    let model = build_projection_slot_render_intents(&proj_intents, &render_model).unwrap();

    assert_eq!(model.entries().len(), 1);
    let entry = &model.entries()[0];

    assert_eq!(entry.render_node_id(), UiRenderNodeId::new(8));
    assert_eq!(entry.render_node_kind(), UiRenderNodeKind::Fragment);
    assert_eq!(entry.projected_node_id(), UiProjectedNodeId::new(8));
    assert_eq!(entry.projected_node_kind(), UiProjectedNodeKind::Fragment);
}

#[test]
fn intent_entry_id_is_deterministic() {
    let (proj_intents, render_model, _, _, _, _) = build_baseline_artifact();
    let model1 = build_projection_slot_render_intents(&proj_intents, &render_model).unwrap();
    let model2 = build_projection_slot_render_intents(&proj_intents, &render_model).unwrap();

    assert_eq!(model1.entries()[0].id(), model2.entries()[0].id());
    // entry id is derived from projected_node_id which is 8
    assert_eq!(model1.entries()[0].id().raw(), 8);
}

#[test]
fn missing_render_node_returns_diagnostic() {
    let (proj_intents, _render_model, projection, _, _, _) = build_baseline_artifact();

    // create a modified render model missing the fragment
    // since UiRenderModel has private fields, we use render_projection_to_model with modified projection
    let mut bad_proj = UiProjectionArtifact::new(UiProjectionArtifactId::new(2));
    for p in projection.nodes() {
        if p.kind() == UiProjectedNodeKind::Root {
            bad_proj.push_node(p.clone());
        }
    }

    let bad_render = render_projection_to_model(&bad_proj).unwrap();

    let result = build_projection_slot_render_intents(&proj_intents, &bad_render);
    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    let diag = diagnostics.iter().next().unwrap();

    match diag.kind() {
        UiProjectionSlotRenderIntentDiagnosticKind::MissingRenderNode {
            source_projection_node_id,
        } => {
            assert_eq!(*source_projection_node_id, UiProjectedNodeId::new(8));
        }
        _ => panic!("Expected MissingRenderNode"),
    }
}

#[test]
fn non_fragment_render_node_returns_diagnostic() {
    let (proj_intents, _, projection, _, _, _) = build_baseline_artifact();

    let mut bad_proj = UiProjectionArtifact::new(UiProjectionArtifactId::new(2));
    for p in projection.nodes() {
        if p.kind() == UiProjectedNodeKind::Fragment {
            // changing the projection node kind to ActionCarrier which will map to UiRenderNodeKind::Fragment but with an Action Marker
            let bad_node = UiProjectedNode::with_source_ir_node(
                p.id(),
                UiProjectedNodeKind::ActionCarrier,
                p.source_ir_node_id().unwrap(),
            );
            bad_proj.push_node(bad_node);
        } else {
            bad_proj.push_node(p.clone());
        }
    }

    let bad_render = render_projection_to_model(&bad_proj).unwrap();

    let result = build_projection_slot_render_intents(&proj_intents, &bad_render);
    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    let diag = diagnostics.iter().next().unwrap();

    match diag.kind() {
        UiProjectionSlotRenderIntentDiagnosticKind::UnexpectedRenderMarkers {
            source_projection_node_id,
            ..
        } => {
            assert_eq!(*source_projection_node_id, UiProjectedNodeId::new(8));
        }
        _ => panic!("Expected UnexpectedRenderMarkers"),
    }
}

#[test]
fn builder_does_not_mutate_inputs() {
    let (proj_intents, render_model, _, _, _, _) = build_baseline_artifact();

    let proj_intents_clone = proj_intents.clone();
    let render_model_clone = render_model.clone();

    let _ = build_projection_slot_render_intents(&proj_intents, &render_model);

    assert_eq!(
        proj_intents.entries().len(),
        proj_intents_clone.entries().len()
    );
    assert_eq!(render_model.nodes().len(), render_model_clone.nodes().len());
}

#[test]
fn render_intent_preserves_tree_source_references() {
    let (proj_intents, render_model, _, _, _, _) = build_baseline_artifact();
    let model = build_projection_slot_render_intents(&proj_intents, &render_model).unwrap();
    let entry = &model.entries()[0];

    assert_eq!(entry.source_tree_node_id(), UiNodeId::new(2));
    assert_eq!(entry.source_tree_node_kind(), UiNodeKind::Slot);
    assert_eq!(entry.source_tree_id(), UiTreeId::new(10));
    assert_eq!(entry.source_tree_resolution(), UiNodeResolution::Known);
    assert_eq!(
        entry.source_tree_intent_entry_id(),
        proj_intents.entries()[0].source_tree_intent_entry_id()
    );
}

#[test]
fn render_intent_preserves_ast_source_references() {
    let (proj_intents, render_model, _, _, _, _) = build_baseline_artifact();
    let model = build_projection_slot_render_intents(&proj_intents, &render_model).unwrap();
    let entry = &model.entries()[0];

    assert_eq!(entry.source_ast_node_id(), UiAstNodeId::new(2));
    assert_eq!(entry.source_ast_node_kind(), UiAstNodeKind::Fragment);
    assert_eq!(
        entry.source_ast_intent_entry_id(),
        proj_intents.entries()[0].source_ast_intent_entry_id()
    );
}

#[test]
fn render_intent_preserves_ir_source_references() {
    let (proj_intents, render_model, _, _, _, _) = build_baseline_artifact();
    let model = build_projection_slot_render_intents(&proj_intents, &render_model).unwrap();
    let entry = &model.entries()[0];

    assert_eq!(entry.source_ir_node_id(), UiIrNodeId::new(2));
    assert_eq!(entry.source_ir_node_kind(), UiIrNodeKind::Fragment);
    assert_eq!(
        entry.source_ir_intent_entry_id(),
        proj_intents.entries()[0].source_ir_intent_entry_id()
    );
}

#[test]
fn render_intent_preserves_projection_source_references() {
    let (proj_intents, render_model, _, _, _, _) = build_baseline_artifact();
    let model = build_projection_slot_render_intents(&proj_intents, &render_model).unwrap();
    let entry = &model.entries()[0];

    assert_eq!(entry.projected_node_id(), UiProjectedNodeId::new(8));
    assert_eq!(entry.projected_node_kind(), UiProjectedNodeKind::Fragment);
    assert_eq!(
        entry.source_projection_intent_entry_id(),
        proj_intents.entries()[0].id()
    );
}

#[test]
fn render_intent_preserves_render_source_projection_node() {
    let (proj_intents, render_model, _, _, _, _) = build_baseline_artifact();
    let model = build_projection_slot_render_intents(&proj_intents, &render_model).unwrap();
    let entry = &model.entries()[0];
    let matched_node = render_model
        .nodes()
        .iter()
        .find(|n| n.id() == entry.render_node_id())
        .unwrap();
    let expected_proj_id = matched_node.source_projection_node();
    assert_eq!(entry.render_source_projection_node(), expected_proj_id);
}

#[test]
fn render_intent_preserves_render_source_ir_node() {
    let (proj_intents, render_model, _, _, _, _) = build_baseline_artifact();
    let model = build_projection_slot_render_intents(&proj_intents, &render_model).unwrap();
    let entry = &model.entries()[0];
    let matched_node = render_model
        .nodes()
        .iter()
        .find(|n| n.id() == entry.render_node_id())
        .unwrap();
    let expected_ir_id = matched_node.source_ir_node();
    assert_eq!(entry.render_source_ir_node(), expected_ir_id);
}

#[test]
fn intent_entry_state_is_deferred() {
    let (proj_intents, render_model, _, _, _, _) = build_baseline_artifact();
    let model = build_projection_slot_render_intents(&proj_intents, &render_model).unwrap();
    let entry = &model.entries()[0];

    assert_eq!(entry.state(), UiProjectionSlotRenderIntentState::Deferred);
}

#[test]
fn intent_entry_kind_is_render_node_linked() {
    let (proj_intents, render_model, _, _, _, _) = build_baseline_artifact();
    let model = build_projection_slot_render_intents(&proj_intents, &render_model).unwrap();
    let entry = &model.entries()[0];

    assert_eq!(
        entry.kind(),
        UiProjectionSlotRenderIntentKind::RenderNodeLinkedToProjectionSlotIntent
    );
}

#[test]
fn slot_projection_intent_is_inert_and_non_authoritative() {
    let (proj_intents, render_model, _, _, _, _) = build_baseline_artifact();

    let proj_intents_len_before = proj_intents.entries().len();
    let render_model_len_before = render_model.nodes().len();
    let render_node_id_before = render_model.nodes()[0].id();

    let model = build_projection_slot_render_intents(&proj_intents, &render_model).unwrap();

    assert_eq!(model.entries().len(), 1);
    let entry = &model.entries()[0];

    assert_eq!(entry.state(), UiProjectionSlotRenderIntentState::Deferred);
    assert_eq!(
        entry.kind(),
        UiProjectionSlotRenderIntentKind::RenderNodeLinkedToProjectionSlotIntent
    );
    assert_eq!(entry.source_tree_node_kind(), UiNodeKind::Slot);
    assert_eq!(entry.source_ir_node_kind(), UiIrNodeKind::Fragment);
    assert_eq!(entry.projected_node_kind(), UiProjectedNodeKind::Fragment);
    assert_eq!(entry.render_node_kind(), UiRenderNodeKind::Fragment);

    let matched_node = render_model
        .nodes()
        .iter()
        .find(|n| n.id() == entry.render_node_id())
        .unwrap();
    assert!(matched_node.markers().is_empty());

    assert_eq!(proj_intents.entries().len(), proj_intents_len_before);
    assert_eq!(render_model.nodes().len(), render_model_len_before);
    assert_eq!(render_model.nodes()[0].id(), render_node_id_before);
}

#[test]
fn missing_render_node_does_not_return_partial_model() {
    let (proj_intents, _, projection, _, _, _) = build_baseline_artifact();

    let mut bad_proj = UiProjectionArtifact::new(UiProjectionArtifactId::new(2));
    for p in projection.nodes() {
        if p.kind() == UiProjectedNodeKind::Root {
            bad_proj.push_node(p.clone());
        }
    }
    let bad_render = render_projection_to_model(&bad_proj).unwrap();

    let result = build_projection_slot_render_intents(&proj_intents, &bad_render);
    assert!(result.is_err());
}

#[test]
fn multiple_projection_intents_preserve_order() {
    // We create a tree with multiple slots and verify order is preserved
    let mut tree = UiTree::new(UiTreeId::new(10));
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    let mut frag1 =
        UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    let mut frag2 =
        UiNode::with_resolution(UiNodeId::new(3), UiNodeKind::Slot, UiNodeResolution::Known);
    frag1.set_parent(Some(UiNodeId::new(1)));
    frag2.set_parent(Some(UiNodeId::new(1)));
    root.push_child(UiNodeId::new(2));
    root.push_child(UiNodeId::new(3));
    tree.push_node(root);
    tree.push_node(frag1);
    tree.push_node(frag2);

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
        UiProjectedNodeId::new(7),
        UiProjectedNodeKind::Root,
        UiIrNodeId::new(1),
    );
    let mut proj_frag1 = UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(8),
        UiProjectedNodeKind::Fragment,
        UiIrNodeId::new(2),
    );
    let mut proj_frag2 = UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(9),
        UiProjectedNodeKind::Fragment,
        UiIrNodeId::new(3),
    );
    proj_frag1.set_parent(UiProjectedNodeId::new(7));
    proj_frag2.set_parent(UiProjectedNodeId::new(7));
    proj_root.push_child(UiProjectedNodeId::new(8));
    proj_root.push_child(UiProjectedNodeId::new(9));

    projection.push_node(proj_root);
    projection.push_node(proj_frag1);
    projection.push_node(proj_frag2);

    let proj_intents = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let render_model = render_projection_to_model(&projection).unwrap();

    let model = build_projection_slot_render_intents(&proj_intents, &render_model).unwrap();

    assert_eq!(model.entries().len(), 2);
    assert_eq!(
        model.entries()[0].projected_node_id(),
        UiProjectedNodeId::new(8)
    );
    assert_eq!(
        model.entries()[1].projected_node_id(),
        UiProjectedNodeId::new(9)
    );
}

#[test]
fn diagnostic_preserves_source_projection_node_id() {
    let (proj_intents, _, projection, _, _, _) = build_baseline_artifact();
    let mut bad_proj = UiProjectionArtifact::new(UiProjectionArtifactId::new(2));
    for p in projection.nodes() {
        if p.kind() == UiProjectedNodeKind::Root {
            bad_proj.push_node(p.clone());
        }
    }
    let bad_render = render_projection_to_model(&bad_proj).unwrap();

    let result = build_projection_slot_render_intents(&proj_intents, &bad_render);
    let diagnostics = result.unwrap_err();
    let diag = diagnostics.iter().next().unwrap();
    match diag.kind() {
        UiProjectionSlotRenderIntentDiagnosticKind::MissingRenderNode {
            source_projection_node_id,
        } => {
            assert_eq!(*source_projection_node_id, UiProjectedNodeId::new(8));
        }
        _ => panic!("Expected MissingRenderNode"),
    }
}
