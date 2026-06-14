use prom_ui::layout::{
    build_layout_constraints, build_layout_geometry, build_layout_measuring, build_layout_sizing,
    build_layout_sizing_algorithm, layout_render_model, UiLayoutMeasuringKind,
    UiLayoutMeasuringModel, UiLayoutMeasuringState,
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

fn create_test_measuring_model() -> UiLayoutMeasuringModel {
    let sizing_algorithm_model = create_test_sizing_algorithm_model();
    build_layout_measuring(&sizing_algorithm_model)
}

#[test]
fn measuring_model_can_be_built_from_existing_layout_geometry_constraints_sizing_sizing_algorithm_fixture(
) {
    let layout_model = create_test_layout_model();
    let geometry_model = create_test_geometry_model();
    let constraints_model = create_test_constraints_model();
    let sizing_model = create_test_sizing_model();
    let sizing_algorithm_model = create_test_sizing_algorithm_model();

    let measuring_model = build_layout_measuring(&sizing_algorithm_model);
    assert_eq!(measuring_model.source_layout_model(), layout_model.id());
    assert_eq!(measuring_model.source_geometry_model(), geometry_model.id());
    assert_eq!(
        measuring_model.source_constraints_model(),
        constraints_model.id()
    );
    assert_eq!(measuring_model.source_sizing_model(), sizing_model.id());
    assert_eq!(
        measuring_model.source_sizing_algorithm_model(),
        sizing_algorithm_model.id()
    );
    assert_eq!(
        measuring_model.source_render_model(),
        layout_model.source_render_model()
    );
    assert_eq!(
        measuring_model.source_projection(),
        layout_model.source_projection()
    );
    assert_eq!(
        measuring_model.source_ir_root(),
        layout_model.source_ir_root()
    );
    assert_eq!(
        measuring_model.entries().len(),
        sizing_algorithm_model.entries().len()
    );
}

#[test]
fn measuring_model_id_is_deterministic() {
    let sizing_algorithm_model = create_test_sizing_algorithm_model();

    let first = build_layout_measuring(&sizing_algorithm_model);
    let second = build_layout_measuring(&sizing_algorithm_model);

    assert_eq!(first.id(), second.id());
    assert_eq!(first.id().raw(), sizing_algorithm_model.id().raw());
}

#[test]
fn measuring_entry_ids_are_deterministic() {
    let first = create_test_measuring_model();
    let second = create_test_measuring_model();

    let ids_1: Vec<_> = first.entries().iter().map(|entry| entry.id()).collect();
    let ids_2: Vec<_> = second.entries().iter().map(|entry| entry.id()).collect();

    assert_eq!(ids_1, ids_2);
}

#[test]
fn measuring_entry_count_order_is_deterministic() {
    let sizing_algorithm_model = create_test_sizing_algorithm_model();
    let measuring_model = build_layout_measuring(&sizing_algorithm_model);

    assert_eq!(
        measuring_model.entries().len(),
        sizing_algorithm_model.entries().len()
    );
    for (index, (measuring_entry, sizing_algorithm_entry)) in measuring_model
        .entries()
        .iter()
        .zip(sizing_algorithm_model.entries())
        .enumerate()
    {
        assert_eq!(measuring_entry.order(), index);
        assert_eq!(
            measuring_entry.source_sizing_algorithm_entry(),
            sizing_algorithm_entry.id()
        );
    }
}

#[test]
fn measuring_kind_state_metadata_is_inert_deferred_unavailable_audit_only() {
    let measuring_model = create_test_measuring_model();

    assert_eq!(
        measuring_model.entries()[0].kind(),
        UiLayoutMeasuringKind::DeferredRequest
    );
    assert_eq!(
        measuring_model.entries()[1].kind(),
        UiLayoutMeasuringKind::UnavailableResult
    );
    assert_eq!(
        measuring_model.entries()[2].kind(),
        UiLayoutMeasuringKind::AuditOnly
    );

    for entry in measuring_model.entries() {
        assert_eq!(entry.state(), UiLayoutMeasuringState::Deferred);
    }
}

#[test]
fn source_layout_model_reference_is_preserved() {
    let layout_model = create_test_layout_model();
    let measuring_model = build_layout_measuring(&create_test_sizing_algorithm_model());

    assert_eq!(measuring_model.source_layout_model(), layout_model.id());
    assert_eq!(
        measuring_model.source_geometry_model(),
        build_layout_geometry(&layout_model).id()
    );
    assert_eq!(
        measuring_model.source_constraints_model(),
        build_layout_constraints(&layout_model).id()
    );
    assert_eq!(
        measuring_model.source_sizing_model(),
        build_layout_sizing(&layout_model).id()
    );
    assert_eq!(
        measuring_model.source_sizing_algorithm_model(),
        build_layout_sizing_algorithm(&build_layout_sizing(&layout_model)).id()
    );
}

#[test]
fn source_layout_geometry_constraints_sizing_sizing_algorithm_references_are_preserved_where_exposed(
) {
    let layout_model = create_test_layout_model();
    let geometry_model = build_layout_geometry(&layout_model);
    let constraints_model = build_layout_constraints(&layout_model);
    let sizing_model = build_layout_sizing(&layout_model);
    let sizing_algorithm_model = build_layout_sizing_algorithm(&sizing_model);

    let measuring_model = build_layout_measuring(&sizing_algorithm_model);
    for (
        (((measuring_entry, layout_node), geometry_node), declaration),
        (sizing_entry, sizing_algorithm_entry),
    ) in measuring_model
        .entries()
        .iter()
        .zip(layout_model.nodes())
        .zip(geometry_model.nodes())
        .zip(constraints_model.declarations())
        .zip(
            sizing_model
                .entries()
                .iter()
                .zip(sizing_algorithm_model.entries()),
        )
    {
        assert_eq!(measuring_entry.source_layout_node(), layout_node.id());
        assert_eq!(measuring_entry.source_layout_slot(), layout_node.slot());
        assert_eq!(measuring_entry.source_geometry_node(), geometry_node.id());
        assert_eq!(
            measuring_entry.source_constraint_declaration(),
            declaration.id()
        );
        assert_eq!(measuring_entry.source_sizing_entry(), sizing_entry.id());
        assert_eq!(
            measuring_entry.source_sizing_algorithm_entry(),
            sizing_algorithm_entry.id()
        );
        assert_eq!(
            measuring_entry.source_render_node(),
            layout_node.source_render_node()
        );
        assert_eq!(
            measuring_entry.source_projection_node(),
            layout_node.source_projection_node()
        );
        assert_eq!(
            measuring_entry.source_ir_node(),
            layout_node.source_ir_node()
        );
    }
}

#[test]
fn no_input_mutation() {
    let layout_model = create_test_layout_model();
    let expected_layout = layout_model.clone();
    let geometry_model = build_layout_geometry(&layout_model);
    let expected_geometry = geometry_model.clone();
    let constraints_model = build_layout_constraints(&layout_model);
    let expected_constraints = constraints_model.clone();
    let sizing_model = build_layout_sizing(&layout_model);
    let expected_sizing = sizing_model.clone();
    let sizing_algorithm_model = build_layout_sizing_algorithm(&sizing_model);
    let expected_sizing_algorithm = sizing_algorithm_model.clone();

    let _measuring_model = build_layout_measuring(&sizing_algorithm_model);

    assert_eq!(layout_model, expected_layout);
    assert_eq!(geometry_model, expected_geometry);
    assert_eq!(constraints_model, expected_constraints);
    assert_eq!(sizing_model, expected_sizing);
    assert_eq!(sizing_algorithm_model, expected_sizing_algorithm);
}

#[test]
fn measuring_seed_does_not_expose_real_text_glyph_image_widget_measurement_authority() {
    let measuring_model = create_test_measuring_model();

    assert!(!measuring_model.is_empty());
    assert_eq!(
        measuring_model.entries()[0].kind(),
        UiLayoutMeasuringKind::DeferredRequest
    );
    assert_eq!(
        measuring_model.entries()[1].kind(),
        UiLayoutMeasuringKind::UnavailableResult
    );
    assert_eq!(
        measuring_model.entries()[2].kind(),
        UiLayoutMeasuringKind::AuditOnly
    );
}

#[test]
fn measuring_seed_does_not_expose_font_backend_gpu_wgpu_winit_tauri_measurement_authority() {
    let measuring_model = create_test_measuring_model();

    assert_eq!(measuring_model.entries().len(), 3);
    assert_eq!(
        measuring_model.entries()[0].state(),
        UiLayoutMeasuringState::Deferred
    );
}

#[test]
fn measuring_seed_does_not_expose_size_to_fit_authority() {
    let measuring_model = create_test_measuring_model();

    assert_eq!(measuring_model.entries()[0].source_layout_node().raw(), 1);
    assert_eq!(measuring_model.entries()[1].source_layout_node().raw(), 2);
}

#[test]
fn measuring_seed_does_not_expose_intrinsic_content_size_calculation_as_executable_behavior() {
    let measuring_model = create_test_measuring_model();

    assert_eq!(measuring_model.entries()[2].source_layout_node().raw(), 3);
    assert_eq!(
        measuring_model.entries()[0]
            .source_sizing_algorithm_entry()
            .raw(),
        1
    );
}

#[test]
fn measuring_seed_does_not_expose_constraint_solver_constraint_satisfaction_or_layout_solving_authority(
) {
    let measuring_model = create_test_measuring_model();

    assert_eq!(measuring_model.entries().len(), 3);
    assert_eq!(measuring_model.entries()[0].source_sizing_entry().raw(), 1);
    assert_eq!(measuring_model.entries()[1].source_sizing_entry().raw(), 2);
    assert_eq!(measuring_model.entries()[2].source_sizing_entry().raw(), 3);
}

#[test]
fn measuring_seed_does_not_expose_draw_event_backend_runtime_capability_proof_debugger_authority() {
    let measuring_model = create_test_measuring_model();

    assert_eq!(
        measuring_model.entries()[0]
            .source_constraint_declaration()
            .raw(),
        1
    );
    assert_eq!(
        measuring_model.entries()[1]
            .source_constraint_declaration()
            .raw(),
        2
    );
    assert_eq!(
        measuring_model.entries()[2]
            .source_constraint_declaration()
            .raw(),
        3
    );
}

#[test]
fn measuring_seed_entrypoint_signature_is_locked() {
    let sizing_algorithm_model = create_test_sizing_algorithm_model();
    let f: fn(&prom_ui::layout::UiLayoutSizingAlgorithmModel) -> UiLayoutMeasuringModel =
        build_layout_measuring;
    let measuring_model = f(&sizing_algorithm_model);

    assert_eq!(
        measuring_model.source_sizing_algorithm_model(),
        sizing_algorithm_model.id()
    );
}
