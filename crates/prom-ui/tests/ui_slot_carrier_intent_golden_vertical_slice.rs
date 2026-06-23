use prom_ui::lowering::{lower_ast_to_ir, UiLoweringConfig};
use prom_ui::model::{
    UiAstNodeId, UiAstNodeKind, UiIrNodeId, UiIrNodeKind, UiNode, UiNodeId, UiNodeKind,
    UiNodeResolution, UiTree, UiTreeId,
};
use prom_ui::projection::{project_ir_to_projection, UiProjectedNodeId, UiProjectedNodeKind};
use prom_ui::renderer::{render_projection_to_model, UiRenderNodeId, UiRenderNodeKind};
use prom_ui::tree_to_ast;
use prom_ui::{
    build_ast_slot_ir_intents, build_ir_slot_projection_intents,
    build_projection_slot_render_intents, build_tree_slot_ast_intents,
    build_tree_slot_carrier_intents, UiAstSlotIrIntentState, UiIrSlotProjectionIntentState,
    UiProjectionSlotRenderIntentState, UiTreeSlotAstIntentState, UiTreeSlotCarrierIntentState,
};

#[test]
fn slot_carrier_intent_golden_vertical_slice_reaches_render_metadata_without_authority() {
    let mut tree = UiTree::new(UiTreeId::new(900));

    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);

    let mut slot =
        UiNode::with_resolution(UiNodeId::new(2), UiNodeKind::Slot, UiNodeResolution::Known);
    slot.set_parent(Some(UiNodeId::new(1)));
    tree.push_node(slot);

    let tree_nodes_count = tree.nodes().len();

    let ast = tree_to_ast(&tree).unwrap();
    let ast_nodes_count = ast.nodes().len();

    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).unwrap();
    let ir_nodes_count = ir.nodes().len();

    let projection = project_ir_to_projection(&ir).unwrap();
    let projection_nodes_count = projection.nodes().len();

    let render_model = render_projection_to_model(&projection).unwrap();
    let render_nodes_count = render_model.nodes().len();

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    let projection_intents = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let render_intents =
        build_projection_slot_render_intents(&projection_intents, &render_model).unwrap();

    assert_eq!(tree.nodes().len(), tree_nodes_count);
    assert_eq!(ast.nodes().len(), ast_nodes_count);
    assert_eq!(ir.nodes().len(), ir_nodes_count);
    assert_eq!(projection.nodes().len(), projection_nodes_count);
    assert_eq!(render_model.nodes().len(), render_nodes_count);

    assert_eq!(tree_intents.entries().len(), 1);
    assert_eq!(ast_intents.entries().len(), 1);
    assert_eq!(ir_intents.entries().len(), 1);
    assert_eq!(projection_intents.entries().len(), 1);
    assert_eq!(render_intents.entries().len(), 1);

    assert_eq!(tree_intents.entries()[0].id().raw(), 2);
    assert_eq!(ast_intents.entries()[0].id().raw(), 2);
    assert_eq!(ir_intents.entries()[0].id().raw(), 2);
    assert_eq!(projection_intents.entries()[0].id().raw(), 2);
    assert_eq!(render_intents.entries()[0].id().raw(), 2);

    let render_entry = &render_intents.entries()[0];

    assert_eq!(render_entry.source_tree_node_kind(), UiNodeKind::Slot);
    assert_eq!(render_entry.source_ast_node_kind(), UiAstNodeKind::Fragment);
    assert_eq!(render_entry.source_ir_node_kind(), UiIrNodeKind::Fragment);
    assert_eq!(
        render_entry.projected_node_kind(),
        UiProjectedNodeKind::Fragment
    );
    assert_eq!(render_entry.render_node_kind(), UiRenderNodeKind::Fragment);

    assert_eq!(
        tree_intents.entries()[0].state(),
        UiTreeSlotCarrierIntentState::Deferred
    );
    assert_eq!(
        ast_intents.entries()[0].state(),
        UiTreeSlotAstIntentState::Deferred
    );
    assert_eq!(
        ir_intents.entries()[0].state(),
        UiAstSlotIrIntentState::Deferred
    );
    assert_eq!(
        projection_intents.entries()[0].state(),
        UiIrSlotProjectionIntentState::Deferred
    );
    assert_eq!(
        render_entry.state(),
        UiProjectionSlotRenderIntentState::Deferred
    );

    assert_eq!(render_entry.source_tree_id(), UiTreeId::new(900));
    assert_eq!(render_entry.source_tree_node_id(), UiNodeId::new(2));
    assert_eq!(
        render_entry.source_tree_resolution(),
        UiNodeResolution::Known
    );

    assert_eq!(render_entry.source_ast_node_id(), UiAstNodeId::new(2));
    assert_eq!(render_entry.source_ir_node_id(), UiIrNodeId::new(2));
    assert_eq!(render_entry.projected_node_id(), UiProjectedNodeId::new(2));
    assert_eq!(render_entry.render_node_id(), UiRenderNodeId::new(2));

    assert_eq!(
        render_entry.render_source_projection_node(),
        UiProjectedNodeId::new(2)
    );
    assert_eq!(
        render_entry.render_source_ir_node(),
        Some(UiIrNodeId::new(2))
    );

    assert_eq!(render_entry.parent_tree_node_id(), Some(UiNodeId::new(1)));
    assert_eq!(render_entry.parent_ast_node_id(), Some(UiAstNodeId::new(1)));
    assert_eq!(render_entry.parent_ir_node_id(), Some(UiIrNodeId::new(1)));
    assert_eq!(
        render_entry.parent_projected_node_id(),
        Some(UiProjectedNodeId::new(1))
    );

    assert!(render_entry.child_tree_node_ids().is_empty());
    assert!(render_entry.child_ast_node_ids().is_empty());
    assert!(render_entry.child_ir_node_ids().is_empty());
    assert!(render_entry.child_projected_node_ids().is_empty());

    let target_render_node = render_model
        .nodes()
        .iter()
        .find(|n| n.id() == UiRenderNodeId::new(2))
        .unwrap();
    assert!(target_render_node.markers().is_empty());

    let target_projected_node = projection
        .nodes()
        .iter()
        .find(|n| n.id() == UiProjectedNodeId::new(2))
        .unwrap();
    assert_ne!(
        target_projected_node.kind(),
        UiProjectedNodeKind::PropertyCarrier
    );
    assert_ne!(
        target_projected_node.kind(),
        UiProjectedNodeKind::ActionCarrier
    );
    assert_ne!(
        target_projected_node.kind(),
        UiProjectedNodeKind::EffectBoundaryMarker
    );
}

#[test]
fn slot_carrier_intent_golden_vertical_slice_preserves_conflict_resolution() {
    let mut tree = UiTree::new(UiTreeId::new(900));

    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);

    let mut slot = UiNode::with_resolution(
        UiNodeId::new(2),
        UiNodeKind::Slot,
        UiNodeResolution::Conflict,
    );
    slot.set_parent(Some(UiNodeId::new(1)));
    tree.push_node(slot);

    let ast = tree_to_ast(&tree).unwrap();
    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).unwrap();
    let projection = project_ir_to_projection(&ir).unwrap();
    let render_model = render_projection_to_model(&projection).unwrap();

    let tree_intents = build_tree_slot_carrier_intents(&tree).unwrap();
    let ast_intents = build_tree_slot_ast_intents(&tree_intents, &ast).unwrap();
    let ir_intents = build_ast_slot_ir_intents(&ast_intents, &ir).unwrap();
    let projection_intents = build_ir_slot_projection_intents(&ir_intents, &projection).unwrap();
    let render_intents =
        build_projection_slot_render_intents(&projection_intents, &render_model).unwrap();

    let render_entry = &render_intents.entries()[0];

    assert_eq!(
        render_entry.source_tree_resolution(),
        UiNodeResolution::Conflict
    );
    assert_eq!(render_entry.render_node_kind(), UiRenderNodeKind::Fragment);

    let target_render_node = render_model
        .nodes()
        .iter()
        .find(|n| n.id() == UiRenderNodeId::new(2))
        .unwrap();
    assert!(target_render_node.markers().is_empty());

    assert_eq!(
        render_entry.state(),
        UiProjectionSlotRenderIntentState::Deferred
    );
}
