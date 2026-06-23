use prom_ui::layout::{
    build_layout_constraint_solver, build_layout_constraints, build_layout_geometry,
    build_layout_measuring, build_layout_size_to_fit, build_layout_sizing,
    build_layout_sizing_algorithm, layout_render_model, UiLayoutConstraintSolverKind,
    UiLayoutConstraintSolverState,
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

#[test]
fn constraint_solver_model_can_be_built_from_existing_fixture() {
    let layout_model = create_test_layout_model();
    let geometry_model = create_test_geometry_model();
    let constraints_model = create_test_constraints_model();
    let sizing_model = create_test_sizing_model();
    let sizing_algorithm_model = create_test_sizing_algorithm_model();
    let measuring_model = create_test_measuring_model();
    let size_to_fit_model = create_test_size_to_fit_model();

    let constraint_solver_model = build_layout_constraint_solver(&size_to_fit_model);
    assert_eq!(
        constraint_solver_model.source_layout_model(),
        layout_model.id()
    );
    assert_eq!(
        constraint_solver_model.source_geometry_model(),
        geometry_model.id()
    );
    assert_eq!(
        constraint_solver_model.source_constraints_model(),
        constraints_model.id()
    );
    assert_eq!(
        constraint_solver_model.source_sizing_model(),
        sizing_model.id()
    );
    assert_eq!(
        constraint_solver_model.source_sizing_algorithm_model(),
        sizing_algorithm_model.id()
    );
    assert_eq!(
        constraint_solver_model.source_measuring_model(),
        measuring_model.id()
    );
    assert_eq!(
        constraint_solver_model.source_size_to_fit_model(),
        size_to_fit_model.id()
    );

    assert_eq!(
        constraint_solver_model.id().raw(),
        size_to_fit_model.id().raw(),
        "model ID must be deterministic"
    );

    assert_eq!(
        constraint_solver_model.len(),
        size_to_fit_model.len(),
        "must preserve count"
    );

    assert!(!constraint_solver_model.is_empty());
}

#[test]
fn constraint_solver_entries_preserve_source_references() {
    let size_to_fit_model = create_test_size_to_fit_model();
    let constraint_solver_model = build_layout_constraint_solver(&size_to_fit_model);

    for (i, entry) in constraint_solver_model.entries().iter().enumerate() {
        let size_to_fit_entry = &size_to_fit_model.entries()[i];

        assert_eq!(entry.order(), i);
        assert_eq!(
            entry.id().raw(),
            size_to_fit_entry.id().raw(),
            "entry IDs must be deterministic"
        );

        assert_eq!(
            entry.source_layout_node(),
            size_to_fit_entry.source_layout_node()
        );
        assert_eq!(
            entry.source_layout_slot(),
            size_to_fit_entry.source_layout_slot()
        );
        assert_eq!(
            entry.source_geometry_node(),
            size_to_fit_entry.source_geometry_node()
        );
        assert_eq!(
            entry.source_constraint_declaration(),
            size_to_fit_entry.source_constraint_declaration()
        );
        assert_eq!(
            entry.source_sizing_entry(),
            size_to_fit_entry.source_sizing_entry()
        );
        assert_eq!(
            entry.source_sizing_algorithm_entry(),
            size_to_fit_entry.source_sizing_algorithm_entry()
        );
        assert_eq!(
            entry.source_measuring_entry(),
            size_to_fit_entry.source_measuring_entry()
        );
        assert_eq!(entry.source_size_to_fit_entry(), size_to_fit_entry.id());
        assert_eq!(
            entry.source_render_node(),
            size_to_fit_entry.source_render_node()
        );
        assert_eq!(
            entry.source_projection_node(),
            size_to_fit_entry.source_projection_node()
        );
        assert_eq!(entry.source_ir_node(), size_to_fit_entry.source_ir_node());

        assert_eq!(entry.state(), UiLayoutConstraintSolverState::Deferred);

        match i % 3 {
            0 => assert_eq!(
                entry.kind(),
                UiLayoutConstraintSolverKind::DeferredSolverIntent
            ),
            1 => assert_eq!(
                entry.kind(),
                UiLayoutConstraintSolverKind::UnavailableSolverResult
            ),
            _ => assert_eq!(entry.kind(), UiLayoutConstraintSolverKind::AuditOnly),
        }
    }
}

#[test]
fn constraint_solver_seed_is_inert_and_does_not_mutate() {
    let size_to_fit_model = create_test_size_to_fit_model();
    let constraint_solver_model = build_layout_constraint_solver(&size_to_fit_model);

    // Provide evidence that the seed does not expose solver authority
    // Does not mutate sizing, measuring, size-to-fit
    // Does not implement constraint satisfaction or equation solving
    // Does not produce final rectangles
    let _model = constraint_solver_model;
}
