use prom_ui::layout::{build_layout_geometry, layout_render_model};
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

#[test]
fn geometry_model_can_be_built_from_existing_layout_model_fixture() {
    let layout_model = create_test_layout_model();

    let geometry_model = build_layout_geometry(&layout_model);
    assert_eq!(geometry_model.source_layout_model(), layout_model.id());
    assert_eq!(
        geometry_model.source_render_model(),
        layout_model.source_render_model()
    );
    assert_eq!(
        geometry_model.source_projection(),
        layout_model.source_projection()
    );
    assert_eq!(
        geometry_model.source_ir_root(),
        layout_model.source_ir_root()
    );
    assert_eq!(geometry_model.nodes().len(), layout_model.nodes().len());
}

#[test]
fn geometry_model_id_is_deterministic() {
    let layout_model = create_test_layout_model();

    let geometry_model_1 = build_layout_geometry(&layout_model);
    let geometry_model_2 = build_layout_geometry(&layout_model);

    assert_eq!(geometry_model_1.id(), geometry_model_2.id());
    assert_eq!(geometry_model_1.id().raw(), layout_model.id().raw());
}

#[test]
fn geometry_node_ids_are_deterministic() {
    let layout_model = create_test_layout_model();

    let geometry_model_1 = build_layout_geometry(&layout_model);
    let geometry_model_2 = build_layout_geometry(&layout_model);

    let ids_1: Vec<_> = geometry_model_1
        .nodes()
        .iter()
        .map(|node| node.id())
        .collect();
    let ids_2: Vec<_> = geometry_model_2
        .nodes()
        .iter()
        .map(|node| node.id())
        .collect();

    assert_eq!(ids_1, ids_2);
}

#[test]
fn geometry_node_count_order_is_deterministic() {
    let layout_model = create_test_layout_model();

    let geometry_model = build_layout_geometry(&layout_model);

    assert_eq!(geometry_model.nodes().len(), layout_model.nodes().len());
    for (index, node) in geometry_model.nodes().iter().enumerate() {
        assert_eq!(node.order(), index);
        assert_eq!(node.source_layout_node(), layout_model.nodes()[index].id());
    }
}

#[test]
fn geometry_rect_metadata_is_inert_default_unresolved() {
    let layout_model = create_test_layout_model();
    let geometry_model = build_layout_geometry(&layout_model);

    for node in geometry_model.nodes() {
        let rect = node.rect();
        assert_eq!(rect, prom_ui::layout::UiRect::default());
        assert_eq!(rect.origin.x, 0);
        assert_eq!(rect.origin.y, 0);
        assert_eq!(rect.size.width, 0);
        assert_eq!(rect.size.height, 0);
    }
}

#[test]
fn source_layout_model_reference_is_preserved() {
    let layout_model = create_test_layout_model();

    let geometry_model = build_layout_geometry(&layout_model);
    assert_eq!(geometry_model.source_layout_model(), layout_model.id());
    assert_eq!(
        geometry_model.source_render_model(),
        layout_model.source_render_model()
    );
}

#[test]
fn source_layout_node_references_are_preserved_where_exposed() {
    let layout_model = create_test_layout_model();

    let geometry_model = build_layout_geometry(&layout_model);
    for (geometry_node, layout_node) in geometry_model.nodes().iter().zip(layout_model.nodes()) {
        assert_eq!(geometry_node.source_layout_node(), layout_node.id());
        assert_eq!(geometry_node.source_layout_slot(), layout_node.slot());
        assert_eq!(
            geometry_node.source_render_node(),
            layout_node.source_render_node()
        );
        assert_eq!(
            geometry_node.source_projection_node(),
            layout_node.source_projection_node()
        );
        assert_eq!(geometry_node.source_ir_node(), layout_node.source_ir_node());
    }
}

#[test]
fn no_input_mutation() {
    let layout_model = create_test_layout_model();
    let expected = layout_model.clone();

    let _geometry_model = build_layout_geometry(&layout_model);

    assert_eq!(layout_model, expected);
}

#[test]
fn geometry_seed_does_not_expose_draw_event_backend_runtime_capability_proof_debugger_authority() {
    let layout_model = create_test_layout_model();
    let geometry_model = build_layout_geometry(&layout_model);

    assert!(!geometry_model.is_empty());
    assert_eq!(
        geometry_model.nodes()[0].rect(),
        prom_ui::layout::UiRect::default()
    );
}

#[test]
fn geometry_seed_entrypoint_signature_is_locked() {
    let layout_model = create_test_layout_model();
    let f: fn(&prom_ui::layout::UiLayoutModel) -> prom_ui::layout::UiLayoutGeometryModel =
        build_layout_geometry;
    let geometry_model = f(&layout_model);

    assert_eq!(geometry_model.source_layout_model(), layout_model.id());
}
