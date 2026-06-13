use prom_ui::layout::{
    build_layout_constraints, build_layout_geometry, build_layout_sizing, layout_render_model,
    UiLayoutSizingKind, UiLayoutSizingModel, UiLayoutSizingState,
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

fn create_test_sizing_model() -> UiLayoutSizingModel {
    let layout_model = create_test_layout_model();
    build_layout_sizing(&layout_model)
}

#[test]
fn sizing_model_can_be_built_from_existing_layout_geometry_constraints_fixture() {
    let layout_model = create_test_layout_model();
    let geometry_model = create_test_geometry_model();
    let constraints_model = create_test_constraints_model();

    let sizing_model = build_layout_sizing(&layout_model);
    assert_eq!(sizing_model.source_layout_model(), layout_model.id());
    assert_eq!(sizing_model.source_geometry_model(), geometry_model.id());
    assert_eq!(
        sizing_model.source_constraints_model(),
        constraints_model.id()
    );
    assert_eq!(
        sizing_model.source_render_model(),
        layout_model.source_render_model()
    );
    assert_eq!(
        sizing_model.source_projection(),
        layout_model.source_projection()
    );
    assert_eq!(sizing_model.source_ir_root(), layout_model.source_ir_root());
    assert_eq!(
        sizing_model.entries().len(),
        constraints_model.declarations().len()
    );
}

#[test]
fn sizing_model_id_is_deterministic() {
    let layout_model = create_test_layout_model();

    let sizing_model_1 = build_layout_sizing(&layout_model);
    let sizing_model_2 = build_layout_sizing(&layout_model);

    assert_eq!(sizing_model_1.id(), sizing_model_2.id());
    assert_eq!(sizing_model_1.id().raw(), layout_model.id().raw());
}

#[test]
fn sizing_entry_ids_are_deterministic() {
    let sizing_model = create_test_sizing_model();
    let second_sizing_model = create_test_sizing_model();

    let ids_1: Vec<_> = sizing_model
        .entries()
        .iter()
        .map(|entry| entry.id())
        .collect();
    let ids_2: Vec<_> = second_sizing_model
        .entries()
        .iter()
        .map(|entry| entry.id())
        .collect();

    assert_eq!(ids_1, ids_2);
}

#[test]
fn sizing_entry_count_order_is_deterministic() {
    let layout_model = create_test_layout_model();
    let constraints_model = create_test_constraints_model();
    let sizing_model = build_layout_sizing(&layout_model);

    assert_eq!(
        sizing_model.entries().len(),
        constraints_model.declarations().len()
    );
    for (index, entry) in sizing_model.entries().iter().enumerate() {
        assert_eq!(entry.order(), index);
        assert_eq!(
            entry.source_constraint_declaration(),
            constraints_model.declarations()[index].id()
        );
    }
}

#[test]
fn sizing_kind_state_metadata_is_inert_default_unresolved() {
    let sizing_model = create_test_sizing_model();

    for entry in sizing_model.entries() {
        assert_eq!(entry.kind(), UiLayoutSizingKind::Unresolved);
        assert_eq!(entry.state(), UiLayoutSizingState::Unresolved);
    }
}

#[test]
fn source_layout_model_reference_is_preserved() {
    let layout_model = create_test_layout_model();

    let sizing_model = build_layout_sizing(&layout_model);
    assert_eq!(sizing_model.source_layout_model(), layout_model.id());
    assert_eq!(
        sizing_model.source_geometry_model(),
        build_layout_geometry(&layout_model).id()
    );
    assert_eq!(
        sizing_model.source_constraints_model(),
        build_layout_constraints(&layout_model).id()
    );
}

#[test]
fn source_layout_geometry_constraints_references_are_preserved_where_exposed() {
    let layout_model = create_test_layout_model();
    let geometry_model = build_layout_geometry(&layout_model);
    let constraints_model = build_layout_constraints(&layout_model);

    let sizing_model = build_layout_sizing(&layout_model);
    for (((entry, layout_node), geometry_node), declaration) in sizing_model
        .entries()
        .iter()
        .zip(layout_model.nodes())
        .zip(geometry_model.nodes())
        .zip(constraints_model.declarations())
    {
        assert_eq!(entry.source_layout_node(), layout_node.id());
        assert_eq!(entry.source_layout_slot(), layout_node.slot());
        assert_eq!(entry.source_geometry_node(), geometry_node.id());
        assert_eq!(entry.source_constraint_declaration(), declaration.id());
        assert_eq!(entry.source_render_node(), layout_node.source_render_node());
        assert_eq!(
            entry.source_projection_node(),
            layout_node.source_projection_node()
        );
        assert_eq!(entry.source_ir_node(), layout_node.source_ir_node());
    }
}

#[test]
fn no_input_mutation() {
    let layout_model = create_test_layout_model();
    let expected = layout_model.clone();

    let _sizing_model = build_layout_sizing(&layout_model);

    assert_eq!(layout_model, expected);
}

#[test]
fn sizing_seed_does_not_expose_sizing_algorithm_measuring_or_size_to_fit_authority() {
    let sizing_model = create_test_sizing_model();

    assert!(!sizing_model.is_empty());
    assert_eq!(
        sizing_model.entries()[0].kind(),
        UiLayoutSizingKind::Unresolved
    );
    assert_eq!(
        sizing_model.entries()[0].state(),
        UiLayoutSizingState::Unresolved
    );
}

#[test]
fn sizing_seed_does_not_expose_constraint_solver_constraint_satisfaction_or_layout_solving_authority(
) {
    let sizing_model = create_test_sizing_model();

    assert_eq!(sizing_model.entries().len(), 2);
    assert_eq!(sizing_model.entries()[0].id().raw(), 1);
    assert_eq!(sizing_model.entries()[1].id().raw(), 2);
}

#[test]
fn sizing_seed_does_not_expose_draw_event_backend_runtime_capability_proof_debugger_authority() {
    let sizing_model = create_test_sizing_model();

    assert_eq!(sizing_model.entries()[0].source_layout_node().raw(), 1);
    assert_eq!(sizing_model.entries()[1].source_layout_node().raw(), 2);
}

#[test]
fn sizing_seed_entrypoint_signature_is_locked() {
    let layout_model = create_test_layout_model();
    let f: fn(&prom_ui::layout::UiLayoutModel) -> UiLayoutSizingModel = build_layout_sizing;
    let sizing_model = f(&layout_model);

    assert_eq!(sizing_model.source_layout_model(), layout_model.id());
}
