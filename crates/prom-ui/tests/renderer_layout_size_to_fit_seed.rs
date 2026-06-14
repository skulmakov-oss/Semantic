use prom_ui::layout::{
    build_layout_constraints, build_layout_geometry, build_layout_measuring,
    build_layout_size_to_fit, build_layout_sizing, build_layout_sizing_algorithm,
    layout_render_model, UiLayoutMeasuringModel, UiLayoutSizeToFitKind, UiLayoutSizeToFitModel,
    UiLayoutSizeToFitState,
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
fn size_to_fit_model_can_be_built_from_existing_layout_geometry_constraints_sizing_sizing_algorithm_measuring_fixture(
) {
    let layout_model = create_test_layout_model();
    let geometry_model = create_test_geometry_model();
    let constraints_model = create_test_constraints_model();
    let sizing_model = create_test_sizing_model();
    let sizing_algorithm_model = create_test_sizing_algorithm_model();
    let measuring_model = create_test_measuring_model();

    let size_to_fit_model = build_layout_size_to_fit(&measuring_model);
    assert_eq!(size_to_fit_model.source_layout_model(), layout_model.id());
    assert_eq!(
        size_to_fit_model.source_geometry_model(),
        geometry_model.id()
    );
    assert_eq!(
        size_to_fit_model.source_constraints_model(),
        constraints_model.id()
    );
    assert_eq!(size_to_fit_model.source_sizing_model(), sizing_model.id());
    assert_eq!(
        size_to_fit_model.source_sizing_algorithm_model(),
        sizing_algorithm_model.id()
    );
    assert_eq!(
        size_to_fit_model.source_measuring_model(),
        measuring_model.id()
    );
    assert_eq!(
        size_to_fit_model.source_render_model(),
        layout_model.source_render_model()
    );
    assert_eq!(
        size_to_fit_model.source_projection(),
        layout_model.source_projection()
    );
    assert_eq!(
        size_to_fit_model.source_ir_root(),
        layout_model.source_ir_root()
    );

    assert_eq!(size_to_fit_model.len(), measuring_model.len());
}

#[test]
fn size_to_fit_model_id_is_deterministic() {
    let measuring_model = create_test_measuring_model();
    let size_to_fit_model = build_layout_size_to_fit(&measuring_model);
    assert_eq!(size_to_fit_model.id().raw(), measuring_model.id().raw());
}

#[test]
fn size_to_fit_entry_ids_are_deterministic() {
    let measuring_model = create_test_measuring_model();
    let size_to_fit_model = build_layout_size_to_fit(&measuring_model);
    assert_eq!(
        size_to_fit_model.entries()[0].id().raw(),
        measuring_model.entries()[0].id().raw()
    );
    assert_eq!(
        size_to_fit_model.entries()[1].id().raw(),
        measuring_model.entries()[1].id().raw()
    );
}

#[test]
fn size_to_fit_entry_count_order_is_deterministic() {
    let measuring_model = create_test_measuring_model();
    let size_to_fit_model = build_layout_size_to_fit(&measuring_model);
    assert_eq!(size_to_fit_model.len(), 3);
    assert_eq!(size_to_fit_model.entries()[0].order(), 0);
    assert_eq!(size_to_fit_model.entries()[1].order(), 1);
    assert_eq!(size_to_fit_model.entries()[2].order(), 2);
}

#[test]
fn size_to_fit_kind_state_metadata_is_inert_deferred_unavailable_audit_only() {
    let measuring_model = create_test_measuring_model();
    let size_to_fit_model = build_layout_size_to_fit(&measuring_model);

    assert_eq!(
        size_to_fit_model.entries()[0].kind(),
        UiLayoutSizeToFitKind::DeferredIntent
    );
    assert_eq!(
        size_to_fit_model.entries()[0].state(),
        UiLayoutSizeToFitState::Deferred
    );

    assert_eq!(
        size_to_fit_model.entries()[1].kind(),
        UiLayoutSizeToFitKind::UnavailableResult
    );
    assert_eq!(
        size_to_fit_model.entries()[1].state(),
        UiLayoutSizeToFitState::Deferred
    );
}

#[test]
fn source_layout_model_reference_is_preserved() {
    let layout_model = create_test_layout_model();
    let measuring_model = create_test_measuring_model();
    let size_to_fit_model = build_layout_size_to_fit(&measuring_model);

    assert_eq!(size_to_fit_model.source_layout_model(), layout_model.id());
    assert_eq!(
        size_to_fit_model.source_render_model(),
        layout_model.source_render_model()
    );
    assert_eq!(
        size_to_fit_model.source_projection(),
        layout_model.source_projection()
    );
    assert_eq!(
        size_to_fit_model.source_ir_root(),
        layout_model.source_ir_root()
    );
}

#[test]
fn source_layout_geometry_constraints_sizing_sizing_algorithm_measuring_references_are_preserved_where_exposed(
) {
    let measuring_model = create_test_measuring_model();
    let size_to_fit_model = build_layout_size_to_fit(&measuring_model);

    for (i, entry) in size_to_fit_model.entries().iter().enumerate() {
        let measuring_entry = &measuring_model.entries()[i];

        assert_eq!(
            entry.source_layout_node(),
            measuring_entry.source_layout_node()
        );
        assert_eq!(
            entry.source_layout_slot(),
            measuring_entry.source_layout_slot()
        );
        assert_eq!(
            entry.source_geometry_node(),
            measuring_entry.source_geometry_node()
        );
        assert_eq!(
            entry.source_constraint_declaration(),
            measuring_entry.source_constraint_declaration()
        );
        assert_eq!(
            entry.source_sizing_entry(),
            measuring_entry.source_sizing_entry()
        );
        assert_eq!(
            entry.source_sizing_algorithm_entry(),
            measuring_entry.source_sizing_algorithm_entry()
        );
        assert_eq!(entry.source_measuring_entry(), measuring_entry.id());
        assert_eq!(
            entry.source_render_node(),
            measuring_entry.source_render_node()
        );
        assert_eq!(
            entry.source_projection_node(),
            measuring_entry.source_projection_node()
        );
        assert_eq!(entry.source_ir_node(), measuring_entry.source_ir_node());
    }
}

#[test]
fn no_input_mutation() {
    let measuring_model = create_test_measuring_model();
    let original = measuring_model.clone();

    let _size_to_fit_model = build_layout_size_to_fit(&measuring_model);

    assert_eq!(measuring_model, original);
}

#[test]
fn size_to_fit_seed_does_not_expose_executable_fit_fill_shrink_grow_behavior() {
    let _ = UiLayoutSizeToFitKind::DeferredIntent;
}

#[test]
fn size_to_fit_seed_does_not_expose_intrinsic_content_size_calculation_as_executable_behavior() {
    let _ = UiLayoutSizeToFitState::Deferred;
}

#[test]
fn size_to_fit_seed_does_not_expose_real_text_glyph_image_widget_measurement_authority() {
    let _ = UiLayoutSizeToFitKind::AuditOnly;
}

#[test]
fn size_to_fit_seed_does_not_expose_font_backend_gpu_wgpu_winit_tauri_measurement_authority() {
    let _ = UiLayoutSizeToFitState::Deferred;
}

#[test]
fn size_to_fit_seed_does_not_expose_constraint_solver_constraint_satisfaction_or_layout_solving_authority(
) {
    let _ = UiLayoutSizeToFitKind::UnavailableResult;
}

#[test]
fn size_to_fit_seed_does_not_expose_draw_event_backend_runtime_capability_proof_debugger_authority()
{
    let _ = UiLayoutSizeToFitState::Deferred;
}

#[test]
fn size_to_fit_seed_entrypoint_signature_is_locked() {
    let _: fn(&UiLayoutMeasuringModel) -> UiLayoutSizeToFitModel = build_layout_size_to_fit;
}
