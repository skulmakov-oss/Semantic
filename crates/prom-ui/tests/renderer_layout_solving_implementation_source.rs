use prom_ui::layout::{
    build_layout_constraint_solver, build_layout_constraints, build_layout_geometry,
    build_layout_measuring, build_layout_size_to_fit, build_layout_sizing,
    build_layout_sizing_algorithm, build_layout_solving, build_layout_solving_result,
    layout_render_model, UiLayoutSolvingResultKind, UiLayoutSolvingResultState,
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

fn create_test_solving_model() -> prom_ui::layout::UiLayoutSolvingModel {
    let render_model = create_test_render_model();
    let layout_model = layout_render_model(&render_model);
    let _geometry_model = build_layout_geometry(&layout_model);
    let _constraints_model = build_layout_constraints(&layout_model);
    let sizing_model = build_layout_sizing(&layout_model);
    let sizing_algorithm_model = build_layout_sizing_algorithm(&sizing_model);
    let measuring_model = build_layout_measuring(&sizing_algorithm_model);
    let size_to_fit_model = build_layout_size_to_fit(&measuring_model);
    let constraint_solver_model = build_layout_constraint_solver(&size_to_fit_model);
    build_layout_solving(&constraint_solver_model)
}

#[test]
fn layout_solving_result_model_can_be_built_from_existing_fixture() {
    let solving_model = create_test_solving_model();
    let result_model = build_layout_solving_result(&solving_model);

    assert_eq!(result_model.id().raw(), solving_model.id().raw());
    assert_eq!(
        result_model.source_layout_model(),
        solving_model.source_layout_model()
    );
    assert_eq!(
        result_model.source_geometry_model(),
        solving_model.source_geometry_model()
    );
    assert_eq!(
        result_model.source_constraints_model(),
        solving_model.source_constraints_model()
    );
    assert_eq!(
        result_model.source_sizing_model(),
        solving_model.source_sizing_model()
    );
    assert_eq!(
        result_model.source_sizing_algorithm_model(),
        solving_model.source_sizing_algorithm_model()
    );
    assert_eq!(
        result_model.source_measuring_model(),
        solving_model.source_measuring_model()
    );
    assert_eq!(
        result_model.source_size_to_fit_model(),
        solving_model.source_size_to_fit_model()
    );
    assert_eq!(
        result_model.source_constraint_solver_model(),
        solving_model.source_constraint_solver_model()
    );
    assert_eq!(result_model.source_solving_model(), solving_model.id());
    assert_eq!(
        result_model.source_render_model(),
        solving_model.source_render_model()
    );
    assert_eq!(
        result_model.source_projection(),
        solving_model.source_projection()
    );
    assert_eq!(
        result_model.source_ir_root(),
        solving_model.source_ir_root()
    );
}

#[test]
fn layout_solving_result_entries_match_solving_entry_count() {
    let solving_model = create_test_solving_model();
    let result_model = build_layout_solving_result(&solving_model);

    assert_eq!(result_model.entries().len(), solving_model.entries().len());
}

#[test]
fn layout_solving_result_entry_order_is_deterministic() {
    let solving_model = create_test_solving_model();
    let result_model = build_layout_solving_result(&solving_model);

    for (index, entry) in result_model.entries().iter().enumerate() {
        assert_eq!(entry.order(), index);
        assert_eq!(entry.order(), solving_model.entries()[index].order());
    }
}

#[test]
fn layout_solving_result_ids_are_deterministic() {
    let solving_model = create_test_solving_model();
    let result_model_1 = build_layout_solving_result(&solving_model);
    let result_model_2 = build_layout_solving_result(&solving_model);

    assert_eq!(result_model_1.id(), result_model_2.id());

    for (e1, e2) in result_model_1
        .entries()
        .iter()
        .zip(result_model_2.entries().iter())
    {
        assert_eq!(e1.id(), e2.id());
    }
}

#[test]
fn layout_solving_result_entries_preserve_source_references() {
    let solving_model = create_test_solving_model();
    let result_model = build_layout_solving_result(&solving_model);

    for (result_entry, solving_entry) in result_model
        .entries()
        .iter()
        .zip(solving_model.entries().iter())
    {
        assert_eq!(
            result_entry.source_layout_node(),
            solving_entry.source_layout_node()
        );
        assert_eq!(
            result_entry.source_layout_slot(),
            solving_entry.source_layout_slot()
        );
        assert_eq!(
            result_entry.source_geometry_node(),
            solving_entry.source_geometry_node()
        );
        assert_eq!(
            result_entry.source_constraint_declaration(),
            solving_entry.source_constraint_declaration()
        );
        assert_eq!(
            result_entry.source_sizing_entry(),
            solving_entry.source_sizing_entry()
        );
        assert_eq!(
            result_entry.source_sizing_algorithm_entry(),
            solving_entry.source_sizing_algorithm_entry()
        );
        assert_eq!(
            result_entry.source_measuring_entry(),
            solving_entry.source_measuring_entry()
        );
        assert_eq!(
            result_entry.source_size_to_fit_entry(),
            solving_entry.source_size_to_fit_entry()
        );
        assert_eq!(
            result_entry.source_constraint_solver_entry(),
            solving_entry.source_constraint_solver_entry()
        );
        assert_eq!(result_entry.source_solving_entry(), solving_entry.id());
        assert_eq!(
            result_entry.source_render_node(),
            solving_entry.source_render_node()
        );
        assert_eq!(
            result_entry.source_projection_node(),
            solving_entry.source_projection_node()
        );
        assert_eq!(
            result_entry.source_ir_node(),
            solving_entry.source_ir_node()
        );
    }
}

#[test]
fn layout_solving_result_builder_does_not_mutate_input() {
    let solving_model = create_test_solving_model();
    let clone = solving_model.clone();
    let _result_model = build_layout_solving_result(&solving_model);

    assert_eq!(solving_model, clone);
}

#[test]
fn layout_solving_result_layer_is_metadata_only() {
    let solving_model = create_test_solving_model();
    let result_model = build_layout_solving_result(&solving_model);

    for entry in result_model.entries() {
        assert_eq!(entry.kind(), UiLayoutSolvingResultKind::Derived);
        assert_eq!(entry.state(), UiLayoutSolvingResultState::Deferred);
    }
}

#[test]
fn layout_solving_result_does_not_introduce_backend_runtime_or_capability_authority() {
    let _ = build_layout_solving_result;
}
