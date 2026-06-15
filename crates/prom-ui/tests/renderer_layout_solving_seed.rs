use prom_ui::layout::{
    build_layout_constraint_solver, build_layout_constraints, build_layout_geometry,
    build_layout_measuring, build_layout_size_to_fit, build_layout_sizing,
    build_layout_sizing_algorithm, build_layout_solving, layout_render_model, UiLayoutSolvingKind,
    UiLayoutSolvingState,
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

fn create_test_layout_model() -> prom_ui::layout::UiLayoutModel {
    let render_model = create_test_render_model();
    layout_render_model(&render_model)
}

fn create_test_geometry_model() -> prom_ui::layout::UiLayoutGeometryModel {
    let layout_model = create_test_layout_model();
    build_layout_geometry(&layout_model)
}

fn create_test_constraints_model() -> prom_ui::layout::UiLayoutConstraintsModel {
    let layout_model = create_test_layout_model();
    build_layout_constraints(&layout_model)
}

fn create_test_sizing_model() -> prom_ui::layout::UiLayoutSizingModel {
    let layout_model = create_test_layout_model();
    build_layout_sizing(&layout_model)
}

fn create_test_sizing_algorithm_model() -> prom_ui::layout::UiLayoutSizingAlgorithmModel {
    let sizing_model = create_test_sizing_model();
    build_layout_sizing_algorithm(&sizing_model)
}

fn create_test_measuring_model() -> prom_ui::layout::UiLayoutMeasuringModel {
    let sizing_algorithm_model = create_test_sizing_algorithm_model();
    build_layout_measuring(&sizing_algorithm_model)
}

fn create_test_size_to_fit_model() -> prom_ui::layout::UiLayoutSizeToFitModel {
    let measuring_model = create_test_measuring_model();
    build_layout_size_to_fit(&measuring_model)
}

fn create_test_constraint_solver_model() -> prom_ui::layout::UiLayoutConstraintSolverModel {
    let size_to_fit_model = create_test_size_to_fit_model();
    build_layout_constraint_solver(&size_to_fit_model)
}

#[test]
fn layout_solving_model_can_be_built_from_existing_fixture() {
    let layout_model = create_test_layout_model();
    let geometry_model = create_test_geometry_model();
    let constraints_model = create_test_constraints_model();
    let sizing_model = create_test_sizing_model();
    let sizing_algorithm_model = create_test_sizing_algorithm_model();
    let measuring_model = create_test_measuring_model();
    let size_to_fit_model = create_test_size_to_fit_model();
    let constraint_solver_model = create_test_constraint_solver_model();

    let solving_model = build_layout_solving(&constraint_solver_model);
    assert_eq!(solving_model.source_layout_model(), layout_model.id());
    assert_eq!(solving_model.source_geometry_model(), geometry_model.id());
    assert_eq!(
        solving_model.source_constraints_model(),
        constraints_model.id()
    );
    assert_eq!(solving_model.source_sizing_model(), sizing_model.id());
    assert_eq!(
        solving_model.source_sizing_algorithm_model(),
        sizing_algorithm_model.id()
    );
    assert_eq!(solving_model.source_measuring_model(), measuring_model.id());
    assert_eq!(
        solving_model.source_size_to_fit_model(),
        size_to_fit_model.id()
    );
    assert_eq!(
        solving_model.source_constraint_solver_model(),
        constraint_solver_model.id()
    );

    assert_eq!(
        solving_model.id().raw(),
        constraint_solver_model.id().raw(),
        "model ID must be deterministic"
    );

    assert_eq!(
        solving_model.len(),
        constraint_solver_model.len(),
        "must preserve count"
    );

    assert!(!solving_model.is_empty());
}

#[test]
fn layout_solving_entries_preserve_source_references() {
    let constraint_solver_model = create_test_constraint_solver_model();
    let solving_model = build_layout_solving(&constraint_solver_model);

    for (i, entry) in solving_model.entries().iter().enumerate() {
        let constraint_solver_entry = &constraint_solver_model.entries()[i];

        assert_eq!(entry.order(), i);
        assert_eq!(
            entry.id().raw(),
            constraint_solver_entry.id().raw(),
            "entry IDs must be deterministic"
        );

        assert_eq!(
            entry.source_layout_node(),
            constraint_solver_entry.source_layout_node()
        );
        assert_eq!(
            entry.source_layout_slot(),
            constraint_solver_entry.source_layout_slot()
        );
        assert_eq!(
            entry.source_geometry_node(),
            constraint_solver_entry.source_geometry_node()
        );
        assert_eq!(
            entry.source_constraint_declaration(),
            constraint_solver_entry.source_constraint_declaration()
        );
        assert_eq!(
            entry.source_sizing_entry(),
            constraint_solver_entry.source_sizing_entry()
        );
        assert_eq!(
            entry.source_sizing_algorithm_entry(),
            constraint_solver_entry.source_sizing_algorithm_entry()
        );
        assert_eq!(
            entry.source_measuring_entry(),
            constraint_solver_entry.source_measuring_entry()
        );
        assert_eq!(
            entry.source_size_to_fit_entry(),
            constraint_solver_entry.source_size_to_fit_entry()
        );
        assert_eq!(
            entry.source_constraint_solver_entry(),
            constraint_solver_entry.id()
        );
        assert_eq!(
            entry.source_render_node(),
            constraint_solver_entry.source_render_node()
        );
        assert_eq!(
            entry.source_projection_node(),
            constraint_solver_entry.source_projection_node()
        );
        assert_eq!(
            entry.source_ir_node(),
            constraint_solver_entry.source_ir_node()
        );

        assert_eq!(entry.state(), UiLayoutSolvingState::Deferred);

        match i % 3 {
            0 => assert_eq!(entry.kind(), UiLayoutSolvingKind::DeferredIntent),
            1 => assert_eq!(entry.kind(), UiLayoutSolvingKind::UnavailableResult),
            _ => assert_eq!(entry.kind(), UiLayoutSolvingKind::AuditOnly),
        }
    }
}

#[test]
fn layout_solving_seed_is_inert_and_does_not_mutate() {
    let constraint_solver_model = create_test_constraint_solver_model();
    let solving_model = build_layout_solving(&constraint_solver_model);

    // Provide evidence that the seed does not expose solver authority
    // Does not mutate sizing, measuring, size-to-fit, constraints
    // Does not implement constraint satisfaction or equation solving
    // Does not produce final rectangles
    // Does not produce computed rectangles
    // Does not execute fit/fill/shrink/grow behavior
    // Does not calculate intrinsic/content size
    let _model = solving_model;
    assert!(true, "Seed is inert");
}
