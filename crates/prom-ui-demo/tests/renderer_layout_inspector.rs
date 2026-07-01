#[path = "../src/renderer_layout_inspector.rs"]
mod renderer_layout_inspector;

use renderer_layout_inspector::{
    demo_inspector_tree, find_node, flatten_nodes, render_inspector_text,
};

#[test]
fn demo_tree_is_deterministic() {
    assert_eq!(demo_inspector_tree(), demo_inspector_tree());
}

#[test]
fn flatten_nodes_preserves_expected_order() {
    let tree = demo_inspector_tree();
    let nodes = flatten_nodes(&tree);
    let ids: Vec<&str> = nodes.into_iter().map(|node| node.id).collect();
    assert_eq!(
        ids,
        vec!["root", "header", "sidebar", "content", "card_a", "card_b", "footer"]
    );
}

#[test]
fn find_node_returns_card_b() {
    let tree = demo_inspector_tree();
    let card_b = find_node(&tree, "card_b").expect("card_b must exist");
    assert_eq!(card_b.kind, "card");
    assert_eq!(card_b.rect.x, 588);
    assert_eq!(card_b.rect.y, 88);
    assert_eq!(card_b.rect.width, 320);
    assert_eq!(card_b.rect.height, 160);
}

#[test]
fn render_inspector_text_marks_selected_node() {
    let tree = demo_inspector_tree();
    let output = render_inspector_text(&tree, "content");

    assert!(output.contains("Renderer/Layout Inspector"));
    assert!(output.contains("> content"));
    assert!(output.contains("id: content"));
    assert!(output.contains("kind: panel"));
    assert!(output.contains("rect: x=220 y=64 w=740 h=420"));
    assert!(output.contains("children: 2"));
}
