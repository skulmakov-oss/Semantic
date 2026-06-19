use prom_ui::layout::{build_layout_geometry, layout_render_model};
use prom_ui::lowering::{lower_ast_to_ir, UiLoweringConfig};
use prom_ui::model::{UiNode, UiNodeId, UiNodeKind, UiNodeResolution, UiTree, UiTreeId};
use prom_ui::projection::project_ir_to_projection;
use prom_ui::renderer::render_projection_to_model;
use prom_ui::tree_to_ast;

#[test]
fn layout_geometry_initializes_with_zero_rects() {
    let mut tree = UiTree::new(UiTreeId::new(100));

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

    // Slot (sibling to Element) (3)
    let mut slot =
        UiNode::with_resolution(UiNodeId::new(3), UiNodeKind::Slot, UiNodeResolution::Known);
    slot.set_parent(Some(UiNodeId::new(1)));
    tree.push_node(slot);

    let ast = tree_to_ast(&tree).unwrap();
    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig::default()).unwrap();
    let projection = project_ir_to_projection(&ir).unwrap();
    let render_model = render_projection_to_model(&projection).unwrap();

    let layout_model = layout_render_model(&render_model);
    let geometry_model = build_layout_geometry(&layout_model);

    assert_eq!(layout_model.nodes().len(), 4);
    assert_eq!(geometry_model.nodes().len(), 4);

    for geometry_node in geometry_model.nodes() {
        let rect = geometry_node.rect();
        assert_eq!(rect.x(), 0);
        assert_eq!(rect.y(), 0);
        assert_eq!(rect.width(), 0);
        assert_eq!(rect.height(), 0);

        assert!(geometry_node.source_ir_node().is_some());
    }
}
