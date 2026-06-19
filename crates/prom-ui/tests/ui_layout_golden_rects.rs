use prom_ui::layout::{
    build_layout_geometry, layout_render_model, solve_minimal_blocks, UiLayoutGeometryModel, UiRect,
};
use prom_ui::lowering::{lower_ast_to_ir, UiLoweringConfig};
use prom_ui::model::{UiNode, UiNodeId, UiNodeKind, UiNodeResolution, UiTree, UiTreeId};
use prom_ui::projection::project_ir_to_projection;
use prom_ui::renderer::{render_projection_to_model, UiRenderNodeKind};
use prom_ui::tree_to_ast;

#[test]
fn layout_golden_rects_minimal_block() {
    let mut tree = UiTree::new(UiTreeId::new(300));

    // Root (1)
    let mut root = UiNode::new(UiNodeId::new(1), UiNodeKind::Root);
    root.push_child(UiNodeId::new(2));
    tree.push_node(root);

    // Element (2)
    let mut element = UiNode::new(UiNodeId::new(2), UiNodeKind::Element);
    element.set_parent(Some(UiNodeId::new(1)));
    element.push_child(UiNodeId::new(3));
    tree.push_node(element);

    // Text inside Element (3)
    let mut text = UiNode::new(UiNodeId::new(3), UiNodeKind::Text);
    text.set_parent(Some(UiNodeId::new(2)));
    tree.push_node(text);

    // Pipeline
    let ast = tree_to_ast(&tree).unwrap();
    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig::default()).unwrap();
    let projection = project_ir_to_projection(&ir).unwrap();
    let render_model = render_projection_to_model(&projection).unwrap();

    let layout_model = layout_render_model(&render_model);

    // INPUT: Geometry with zeroes
    let input = build_layout_geometry(&layout_model);

    // ACT: Run the minimal block solver
    let output = solve_minimal_blocks(&input, &render_model);

    assert_eq!(output.nodes().len(), 3);

    // The order of nodes in the projection (and thus layout) is deterministic:
    // Typically: Root -> Element -> Text

    // Root
    let root_geom = &output.nodes()[0];
    let root_render = render_model
        .nodes()
        .iter()
        .find(|n| n.id() == root_geom.source_render_node())
        .unwrap();
    assert_eq!(root_render.kind(), UiRenderNodeKind::Root);
    assert_eq!(
        root_geom.rect(),
        UiRect::from_xywh(0, 0, 0, 0),
        "Root should have zero dimensions"
    );

    // Element
    let element_geom = &output.nodes()[1];
    let element_render = render_model
        .nodes()
        .iter()
        .find(|n| n.id() == element_geom.source_render_node())
        .unwrap();
    assert_eq!(element_render.kind(), UiRenderNodeKind::Element);
    assert_eq!(
        element_geom.rect(),
        UiRect::from_xywh(0, 0, 100, 50),
        "Element should have 100x50 and start at y=0"
    );

    // Text
    let text_geom = &output.nodes()[2];
    let text_render = render_model
        .nodes()
        .iter()
        .find(|n| n.id() == text_geom.source_render_node())
        .unwrap();
    assert_eq!(text_render.kind(), UiRenderNodeKind::Text);
    assert_eq!(
        text_geom.rect(),
        UiRect::from_xywh(0, 50, 80, 20),
        "Text should have 80x20 and start at y=50"
    );
}
