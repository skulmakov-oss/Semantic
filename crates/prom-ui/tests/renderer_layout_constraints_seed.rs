use prom_ui::layout::{
    build_layout_constraints, build_layout_geometry, layout_render_model, UiLayoutConstraintKind,
    UiLayoutConstraintState, UiLayoutConstraintsModel, UiLayoutModel,
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

fn create_test_layout_model() -> UiLayoutModel {
    let render_model = create_test_render_model();
    layout_render_model(&render_model)
}

fn create_test_constraints_model() -> UiLayoutConstraintsModel {
    let layout_model = create_test_layout_model();
    build_layout_constraints(&layout_model)
}

#[test]
fn constraints_model_can_be_built_from_existing_layout_model_fixture() {
    let layout_model = create_test_layout_model();
    let geometry_model = build_layout_geometry(&layout_model);

    let constraints_model = build_layout_constraints(&layout_model);
    assert_eq!(constraints_model.source_layout_model(), layout_model.id());
    assert_eq!(
        constraints_model.source_geometry_model(),
        geometry_model.id()
    );
    assert_eq!(
        constraints_model.source_render_model(),
        layout_model.source_render_model()
    );
    assert_eq!(
        constraints_model.source_projection(),
        layout_model.source_projection()
    );
    assert_eq!(
        constraints_model.source_ir_root(),
        layout_model.source_ir_root()
    );
    assert_eq!(constraints_model.len(), layout_model.nodes().len());
}

#[test]
fn constraints_model_id_is_deterministic() {
    let layout_model = create_test_layout_model();

    let constraints_model_1 = build_layout_constraints(&layout_model);
    let constraints_model_2 = build_layout_constraints(&layout_model);

    assert_eq!(constraints_model_1.id(), constraints_model_2.id());
    assert_eq!(constraints_model_1.id().raw(), layout_model.id().raw());
}

#[test]
fn constraint_declaration_ids_are_deterministic() {
    let constraints_model = create_test_constraints_model();
    let second_constraints_model = create_test_constraints_model();

    let ids_1: Vec<_> = constraints_model
        .declarations()
        .iter()
        .map(|declaration| declaration.id())
        .collect();
    let ids_2: Vec<_> = second_constraints_model
        .declarations()
        .iter()
        .map(|declaration| declaration.id())
        .collect();

    assert_eq!(ids_1, ids_2);
}

#[test]
fn constraint_declaration_count_order_is_deterministic() {
    let layout_model = create_test_layout_model();
    let constraints_model = build_layout_constraints(&layout_model);

    assert_eq!(
        constraints_model.declarations().len(),
        layout_model.nodes().len()
    );
    for (index, declaration) in constraints_model.declarations().iter().enumerate() {
        assert_eq!(declaration.order(), index);
        assert_eq!(
            declaration.source_layout_node(),
            layout_model.nodes()[index].id()
        );
    }
}

#[test]
fn constraint_kind_state_metadata_is_inert_default_unresolved() {
    let constraints_model = create_test_constraints_model();

    for declaration in constraints_model.declarations() {
        assert_eq!(declaration.kind(), UiLayoutConstraintKind::Unresolved);
        assert_eq!(declaration.state(), UiLayoutConstraintState::Unresolved);
    }
}

#[test]
fn source_layout_model_reference_is_preserved() {
    let layout_model = create_test_layout_model();

    let constraints_model = build_layout_constraints(&layout_model);
    assert_eq!(constraints_model.source_layout_model(), layout_model.id());
    assert_eq!(
        constraints_model.source_geometry_model(),
        build_layout_geometry(&layout_model).id()
    );
}

#[test]
fn source_layout_geometry_references_are_preserved_where_exposed() {
    let layout_model = create_test_layout_model();
    let geometry_model = build_layout_geometry(&layout_model);

    let constraints_model = build_layout_constraints(&layout_model);
    for ((declaration, layout_node), geometry_node) in constraints_model
        .declarations()
        .iter()
        .zip(layout_model.nodes())
        .zip(geometry_model.nodes())
    {
        assert_eq!(declaration.source_layout_node(), layout_node.id());
        assert_eq!(declaration.source_layout_slot(), layout_node.slot());
        assert_eq!(declaration.source_geometry_node(), geometry_node.id());
        assert_eq!(
            declaration.source_render_node(),
            layout_node.source_render_node()
        );
        assert_eq!(
            declaration.source_projection_node(),
            layout_node.source_projection_node()
        );
        assert_eq!(declaration.source_ir_node(), layout_node.source_ir_node());
    }
}

#[test]
fn no_input_mutation() {
    let layout_model = create_test_layout_model();
    let expected = layout_model.clone();

    let _constraints_model = build_layout_constraints(&layout_model);

    assert_eq!(layout_model, expected);
}

#[test]
fn constraints_seed_does_not_expose_solver_sizing_layout_solving_or_effect_authority() {
    let constraints_model = create_test_constraints_model();

    assert!(!constraints_model.is_empty());
    assert_eq!(
        constraints_model.declarations()[0].kind(),
        UiLayoutConstraintKind::Unresolved
    );
    assert_eq!(
        constraints_model.declarations()[0].state(),
        UiLayoutConstraintState::Unresolved
    );
}

#[test]
fn constraints_seed_does_not_expose_draw_event_backend_runtime_capability_proof_debugger_authority()
{
    let constraints_model = create_test_constraints_model();

    assert_eq!(constraints_model.declarations().len(), 2);
    assert_eq!(
        constraints_model.declarations()[0].kind(),
        UiLayoutConstraintKind::Unresolved
    );
    assert_eq!(
        constraints_model.declarations()[1].state(),
        UiLayoutConstraintState::Unresolved
    );
}

#[test]
fn constraints_seed_entrypoint_signature_is_locked() {
    let layout_model = create_test_layout_model();
    let f: fn(&prom_ui::layout::UiLayoutModel) -> prom_ui::layout::UiLayoutConstraintsModel =
        build_layout_constraints;
    let constraints_model = f(&layout_model);

    assert_eq!(constraints_model.source_layout_model(), layout_model.id());
}
