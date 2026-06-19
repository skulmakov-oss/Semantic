use prom_ui::lowering::{lower_ast_to_ir, UiLoweringConfig};
use prom_ui::model::{
    UiAstNodeId, UiAstNodeKind, UiIrNodeId, UiIrNodeKind, UiNode, UiNodeId, UiNodeKind, UiTree,
    UiTreeId,
};
use prom_ui::projection::{project_ir_to_projection, UiProjectedNodeId, UiProjectedNodeKind};
use prom_ui::renderer::{render_projection_to_model, UiRenderNodeId, UiRenderNodeKind};
use prom_ui::tree_to_ast;

#[test]
fn element_text_golden_vertical_slice_propagates_deterministically() {
    let mut tree = UiTree::new(UiTreeId::new(100));

    // Root (1)
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);

    // Element (2)
    let mut element = UiNode::new(UiNodeId::new(2), UiNodeKind::Element);
    element.set_parent(Some(UiNodeId::new(1)));
    element.push_child(UiNodeId::new(3));
    tree.push_node(element);

    // Text (3)
    let mut text = UiNode::new(UiNodeId::new(3), UiNodeKind::Text);
    text.set_parent(Some(UiNodeId::new(2)));
    tree.push_node(text);

    let tree_nodes_count = tree.nodes().len();

    let ast = tree_to_ast(&tree).unwrap();
    let ast_nodes_count = ast.nodes().len();

    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig::default()).unwrap();
    let ir_nodes_count = ir.nodes().len();

    let projection = project_ir_to_projection(&ir).unwrap();
    let projection_nodes_count = projection.nodes().len();

    let render_model = render_projection_to_model(&projection).unwrap();
    let render_nodes_count = render_model.nodes().len();

    assert_eq!(tree.nodes().len(), tree_nodes_count);
    assert_eq!(ast.nodes().len(), ast_nodes_count);
    assert_eq!(ir.nodes().len(), ir_nodes_count);
    assert_eq!(projection.nodes().len(), projection_nodes_count);
    assert_eq!(render_model.nodes().len(), render_nodes_count);

    // Root checks
    let render_root = render_model
        .nodes()
        .iter()
        .find(|n| n.id() == UiRenderNodeId::new(1))
        .unwrap();
    assert_eq!(render_root.kind(), UiRenderNodeKind::Root);

    // Element checks
    let render_element = render_model
        .nodes()
        .iter()
        .find(|n| n.id() == UiRenderNodeId::new(2))
        .unwrap();
    assert_eq!(render_element.kind(), UiRenderNodeKind::Element);
    assert!(render_element.markers().is_empty());
    assert!(render_element.source_ir_node().is_some());

    // Text checks
    let render_text = render_model
        .nodes()
        .iter()
        .find(|n| n.id() == UiRenderNodeId::new(3))
        .unwrap();
    assert_eq!(render_text.kind(), UiRenderNodeKind::Text);
    assert!(render_text.markers().is_empty());
    assert!(render_text.source_ir_node().is_some());

    // Source references checks for Element
    assert_eq!(render_element.source_ir_node().unwrap(), UiIrNodeId::new(2));
    let ast_element = ast
        .nodes()
        .iter()
        .find(|n| n.id() == UiAstNodeId::new(2))
        .unwrap();
    assert_eq!(ast_element.kind(), UiAstNodeKind::Element);

    let ir_element = ir
        .nodes()
        .iter()
        .find(|n| n.id() == UiIrNodeId::new(2))
        .unwrap();
    assert_eq!(ir_element.kind(), UiIrNodeKind::Element);

    let projected_element = projection
        .nodes()
        .iter()
        .find(|n| n.id() == UiProjectedNodeId::new(2))
        .unwrap();
    assert_eq!(projected_element.kind(), UiProjectedNodeKind::Element);

    // Source references checks for Text
    assert_eq!(render_text.source_ir_node().unwrap(), UiIrNodeId::new(3));
    let ast_text = ast
        .nodes()
        .iter()
        .find(|n| n.id() == UiAstNodeId::new(3))
        .unwrap();
    assert_eq!(ast_text.kind(), UiAstNodeKind::Text);

    let ir_text = ir
        .nodes()
        .iter()
        .find(|n| n.id() == UiIrNodeId::new(3))
        .unwrap();
    assert_eq!(ir_text.kind(), UiIrNodeKind::Text);

    let projected_text = projection
        .nodes()
        .iter()
        .find(|n| n.id() == UiProjectedNodeId::new(3))
        .unwrap();
    assert_eq!(projected_text.kind(), UiProjectedNodeKind::Text);
}
