use prom_ui::layout::constraint_solver::build_layout_constraint_solver;
use prom_ui::layout::constraints::build_layout_constraints;
use prom_ui::layout::geometry::{build_layout_geometry, UiLayoutGeometryRect};
use prom_ui::layout::measuring::build_layout_measuring;
use prom_ui::layout::physical_placement::{
    build_layout_physical_placement, UiLayoutPhysicalPlacementKind, UiLayoutPhysicalPlacementState,
};
use prom_ui::layout::size_to_fit::build_layout_size_to_fit;
use prom_ui::layout::sizing::build_layout_sizing;
use prom_ui::layout::sizing_algorithm::build_layout_sizing_algorithm;
use prom_ui::layout::solving::build_layout_solving;
use prom_ui::layout::solving::build_layout_solving_result;
use prom_ui::model::UiIrNodeId;
use prom_ui::projection::{
    UiProjectedNode, UiProjectedNodeId, UiProjectedNodeKind, UiProjectionArtifact,
    UiProjectionArtifactId,
};
use prom_ui::renderer::{render_projection_to_model, UiRenderModel, UiRenderNodeId};

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

#[test]
fn layout_physical_placement_produces_deterministic_final_rectangles() {
    let render_model = create_test_render_model();

    let layout_model = prom_ui::layout::layout_render_model(&render_model);
    let _geometry_model = build_layout_geometry(&layout_model);
    let _constraints_model = build_layout_constraints(&layout_model);
    let sizing_model = build_layout_sizing(&layout_model);
    let sizing_algo_model = build_layout_sizing_algorithm(&sizing_model);
    let measuring_model = build_layout_measuring(&sizing_algo_model);
    let size_to_fit_model = build_layout_size_to_fit(&measuring_model);
    let constraint_solver_model = build_layout_constraint_solver(&size_to_fit_model);
    let solving_model = build_layout_solving(&constraint_solver_model);
    let solving_result_model = build_layout_solving_result(&solving_model);

    // Act
    let physical_placement_model = build_layout_physical_placement(&solving_result_model);

    // Assert
    assert_eq!(physical_placement_model.entries().len(), 3);

    let entry0 = &physical_placement_model.entries()[0];
    let entry1 = &physical_placement_model.entries()[1];
    let entry2 = &physical_placement_model.entries()[2];

    assert_eq!(entry0.order(), 0);
    assert_eq!(
        entry0.kind(),
        UiLayoutPhysicalPlacementKind::ParentRelativeIntent
    );
    assert_eq!(entry0.state(), UiLayoutPhysicalPlacementState::MetadataOnly);
    assert_eq!(
        entry0.final_rect(),
        UiLayoutGeometryRect::new(0, 0, 100, 50)
    );

    assert_eq!(entry1.order(), 1);
    assert_eq!(
        entry1.kind(),
        UiLayoutPhysicalPlacementKind::PlacementPolicyIntent
    );
    assert_eq!(entry1.state(), UiLayoutPhysicalPlacementState::Deferred);
    assert_eq!(
        entry1.final_rect(),
        UiLayoutGeometryRect::new(10, 20, 105, 55)
    );

    assert_eq!(entry2.order(), 2);
    assert_eq!(
        entry2.kind(),
        UiLayoutPhysicalPlacementKind::DeferredRectIntent
    );
    assert_eq!(entry2.state(), UiLayoutPhysicalPlacementState::Unavailable);
    assert_eq!(
        entry2.final_rect(),
        UiLayoutGeometryRect::new(20, 40, 110, 60)
    );

    // Ensure projection references are maintained
    assert_eq!(
        entry1.source_projection_node(),
        Some(UiProjectedNodeId::new(2))
    );
    assert_eq!(entry1.source_ir_node(), Some(UiIrNodeId::new(12)));
    assert_eq!(entry1.source_render_node(), UiRenderNodeId::new(2));
}
