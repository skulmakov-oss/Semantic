use prom_ui::layout::{build_layout_geometry, layout_render_model, UiLayoutGeometryModel, UiRect};
use prom_ui::lowering::{lower_ast_to_ir, UiLoweringConfig};
use prom_ui::model::{UiNode, UiNodeId, UiNodeKind, UiNodeResolution, UiTree, UiTreeId};
use prom_ui::projection::project_ir_to_projection;
use prom_ui::renderer::render_projection_to_model;
use prom_ui::tree_to_ast;

/// Dummy layout solver acting as a boundary contract implementation.
/// Takes a zero-initialized geometry model and returns a model with computed rects.
fn solve_layout(input: &UiLayoutGeometryModel) -> UiLayoutGeometryModel {
    let mut solved_nodes = Vec::new();

    // In a real solver, we would use block layout, flex algorithms, etc.
    // For the contract, we just prove we can ingest zero-rects and emit solved rects.
    for (i, node) in input.nodes().iter().enumerate() {
        let computed_rect = UiRect::from_xywh(
            (i * 10) as i32,
            (i * 15) as i32,
            100 + i as u32 * 20,
            50 + i as u32 * 10,
        );
        solved_nodes.push(node.clone().with_rect(computed_rect));
    }

    input.clone().with_nodes(solved_nodes)
}

#[test]
fn test_solver_contract() {
    let mut tree = UiTree::new(UiTreeId::new(200));

    // Root (1)
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    root.push_child(UiNodeId::new(2));
    root.push_child(UiNodeId::new(3));
    tree.push_node(root);

    // Element (2)
    let mut element = UiNode::new(UiNodeId::new(2), UiNodeKind::Element);
    element.set_parent(Some(UiNodeId::new(1)));
    element.push_child(UiNodeId::new(4));
    tree.push_node(element);

    // Text inside Element (4)
    let mut text = UiNode::new(UiNodeId::new(4), UiNodeKind::Text);
    text.set_parent(Some(UiNodeId::new(2)));
    tree.push_node(text);

    // Slot (3)
    let mut slot =
        UiNode::with_resolution(UiNodeId::new(3), UiNodeKind::Slot, UiNodeResolution::Known);
    slot.set_parent(Some(UiNodeId::new(1)));
    tree.push_node(slot);

    // Pipeline
    let ast = tree_to_ast(&tree).unwrap();
    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig::default()).unwrap();
    let projection = project_ir_to_projection(&ir).unwrap();
    let render_model = render_projection_to_model(&projection).unwrap();

    let layout_model = layout_render_model(&render_model);

    // INPUT: Geometry with zeroes
    let input = build_layout_geometry(&layout_model);

    // Verify input contract
    for node in input.nodes() {
        assert_eq!(node.rect().width(), 0);
        assert_eq!(node.rect().height(), 0);
    }

    // ACT: Run the dummy solver
    let output = solve_layout(&input);

    // OUTPUT: Geometry with numbers
    assert_eq!(output.nodes().len(), 4);

    for (i, node) in output.nodes().iter().enumerate() {
        let rect = node.rect();
        // Check that solver replaced zeroes with computed values
        assert_eq!(rect.x(), (i * 10) as i32);
        assert_eq!(rect.y(), (i * 15) as i32);
        assert_eq!(rect.width(), 100 + i as u32 * 20);
        assert_eq!(rect.height(), 50 + i as u32 * 10);
    }
}
