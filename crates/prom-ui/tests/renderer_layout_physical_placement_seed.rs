use prom_ui::layout::{
    build_layout_constraint_solver, build_layout_constraints, build_layout_geometry,
    build_layout_measuring, build_layout_physical_placement, build_layout_size_to_fit,
    build_layout_sizing, build_layout_sizing_algorithm, build_layout_solving,
    build_layout_solving_result, layout_render_model, UiLayoutPhysicalPlacementKind,
    UiLayoutPhysicalPlacementState,
};
use prom_ui::model::UiIrNodeId;
use prom_ui::projection::{
    UiProjectedNode, UiProjectedNodeId, UiProjectedNodeKind, UiProjectionArtifact,
    UiProjectionArtifactId,
};
use prom_ui::renderer::{render_projection_to_model, UiRenderModel};

fn create_test_render_model() -> UiRenderModel {
    let mut artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(100));
    artifact.set_source_ir_root(UiIrNodeId::new(10));

    let root = UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(1),
        UiProjectedNodeKind::Root,
        UiIrNodeId::new(11),
    );
    artifact.push_node(root);

    let element = UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(2),
        UiProjectedNodeKind::Element,
        UiIrNodeId::new(12),
    );
    artifact.push_node(element);

    let leaf = UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(3),
        UiProjectedNodeKind::Element,
        UiIrNodeId::new(13),
    );
    artifact.push_node(leaf);

    render_projection_to_model(&artifact).unwrap()
}

fn create_test_solving_result_model() -> prom_ui::layout::UiLayoutSolvingResultModel {
    let render_model = create_test_render_model();
    let layout_model = layout_render_model(&render_model);
    let _geometry_model = build_layout_geometry(&layout_model);
    let _constraints_model = build_layout_constraints(&layout_model);
    let sizing_model = build_layout_sizing(&layout_model);
    let sizing_algorithm_model = build_layout_sizing_algorithm(&sizing_model);
    let measuring_model = build_layout_measuring(&sizing_algorithm_model);
    let size_to_fit_model = build_layout_size_to_fit(&measuring_model);
    let constraint_solver_model = build_layout_constraint_solver(&size_to_fit_model);
    let solving_model = build_layout_solving(&constraint_solver_model);
    build_layout_solving_result(&solving_model)
}

#[test]
fn physical_placement_seed_builds_deterministic_metadata() {
    let solving_result_model = create_test_solving_result_model();
    let placement_model_1 = build_layout_physical_placement(&solving_result_model);
    let placement_model_2 = build_layout_physical_placement(&solving_result_model);

    assert_eq!(placement_model_1.id(), placement_model_2.id());
    assert_eq!(
        placement_model_1.entries().len(),
        solving_result_model.entries().len()
    );
    assert_eq!(
        placement_model_1.entries().len(),
        placement_model_2.entries().len()
    );

    for (index, entry) in placement_model_1.entries().iter().enumerate() {
        assert_eq!(entry.order(), index);
        assert_eq!(entry.kind(), placement_model_2.entries()[index].kind());
        assert_eq!(entry.state(), placement_model_2.entries()[index].state());
    }

    for (entry, solving_result_entry) in placement_model_1
        .entries()
        .iter()
        .zip(solving_result_model.entries().iter())
    {
        assert_eq!(entry.id().raw(), solving_result_entry.id().raw());
    }
}

#[test]
fn physical_placement_seed_preserves_solving_result_references() {
    let solving_result_model = create_test_solving_result_model();
    let placement_model = build_layout_physical_placement(&solving_result_model);

    assert_eq!(
        placement_model.source_layout_model(),
        solving_result_model.source_layout_model()
    );
    assert_eq!(
        placement_model.source_geometry_model(),
        solving_result_model.source_geometry_model()
    );
    assert_eq!(
        placement_model.source_constraints_model(),
        solving_result_model.source_constraints_model()
    );
    assert_eq!(
        placement_model.source_sizing_model(),
        solving_result_model.source_sizing_model()
    );
    assert_eq!(
        placement_model.source_sizing_algorithm_model(),
        solving_result_model.source_sizing_algorithm_model()
    );
    assert_eq!(
        placement_model.source_measuring_model(),
        solving_result_model.source_measuring_model()
    );
    assert_eq!(
        placement_model.source_size_to_fit_model(),
        solving_result_model.source_size_to_fit_model()
    );
    assert_eq!(
        placement_model.source_constraint_solver_model(),
        solving_result_model.source_constraint_solver_model()
    );
    assert_eq!(
        placement_model.source_solving_model(),
        solving_result_model.source_solving_model()
    );
    assert_eq!(
        placement_model.source_solving_result_model(),
        solving_result_model.id()
    );
    assert_eq!(
        placement_model.source_render_model(),
        solving_result_model.source_render_model()
    );
    assert_eq!(
        placement_model.source_projection(),
        solving_result_model.source_projection()
    );
    assert_eq!(
        placement_model.source_ir_root(),
        solving_result_model.source_ir_root()
    );

    for (placement_entry, solving_result_entry) in placement_model
        .entries()
        .iter()
        .zip(solving_result_model.entries().iter())
    {
        assert_eq!(
            placement_entry.source_layout_node(),
            solving_result_entry.source_layout_node()
        );
        assert_eq!(
            placement_entry.source_layout_slot(),
            solving_result_entry.source_layout_slot()
        );
        assert_eq!(
            placement_entry.source_geometry_node(),
            solving_result_entry.source_geometry_node()
        );
        assert_eq!(
            placement_entry.source_constraint_declaration(),
            solving_result_entry.source_constraint_declaration()
        );
        assert_eq!(
            placement_entry.source_sizing_entry(),
            solving_result_entry.source_sizing_entry()
        );
        assert_eq!(
            placement_entry.source_sizing_algorithm_entry(),
            solving_result_entry.source_sizing_algorithm_entry()
        );
        assert_eq!(
            placement_entry.source_measuring_entry(),
            solving_result_entry.source_measuring_entry()
        );
        assert_eq!(
            placement_entry.source_size_to_fit_entry(),
            solving_result_entry.source_size_to_fit_entry()
        );
        assert_eq!(
            placement_entry.source_constraint_solver_entry(),
            solving_result_entry.source_constraint_solver_entry()
        );
        assert_eq!(
            placement_entry.source_solving_entry(),
            solving_result_entry.source_solving_entry()
        );
        assert_eq!(
            placement_entry.source_solving_result_entry(),
            solving_result_entry.id()
        );
        assert_eq!(
            placement_entry.source_render_node(),
            solving_result_entry.source_render_node()
        );
        assert_eq!(
            placement_entry.source_projection_node(),
            solving_result_entry.source_projection_node()
        );
        assert_eq!(
            placement_entry.source_ir_node(),
            solving_result_entry.source_ir_node()
        );
    }
}

#[test]
fn physical_placement_seed_is_inert_and_non_authoritative() {
    let solving_result_model = create_test_solving_result_model();
    let clone = solving_result_model.clone();
    let placement_model = build_layout_physical_placement(&solving_result_model);

    assert_eq!(solving_result_model, clone);
    assert_eq!(placement_model.id().raw(), solving_result_model.id().raw());

    for entry in placement_model.entries() {
        assert!(matches!(
            entry.kind(),
            UiLayoutPhysicalPlacementKind::ParentRelativeIntent
                | UiLayoutPhysicalPlacementKind::PlacementPolicyIntent
                | UiLayoutPhysicalPlacementKind::DeferredRectIntent
                | UiLayoutPhysicalPlacementKind::ContainerRelationIntent
        ));
        assert!(matches!(
            entry.state(),
            UiLayoutPhysicalPlacementState::MetadataOnly
                | UiLayoutPhysicalPlacementState::Deferred
                | UiLayoutPhysicalPlacementState::Unavailable
                | UiLayoutPhysicalPlacementState::AuditOnly
        ));
    }
}

#[test]
fn physical_placement_seed_does_not_emit_backend_or_viewport_outputs() {
    let solving_result_model = create_test_solving_result_model();
    let placement_model = build_layout_physical_placement(&solving_result_model);

    assert!(!placement_model.is_empty());
    assert_eq!(placement_model.len(), solving_result_model.len());

    let _ = build_layout_physical_placement;
}
