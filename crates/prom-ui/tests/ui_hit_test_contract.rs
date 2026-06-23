#![cfg_attr(not(feature = "std"), no_std)]

use prom_ui::renderer::UiRenderNodeId;
use prom_ui::{
    DefaultHitTester, UiHitTester, UiLayoutRect, UiLayoutRectEntry, UiLayoutRectModel,
    UiProjectedNodeId,
};

#[test]
fn hit_test_finds_first_matching_node_by_reverse_z_order() {
    let mut layout = UiLayoutRectModel::new();
    let node1 = UiProjectedNodeId::new(1);
    let node2 = UiProjectedNodeId::new(2);
    let render_node1 = UiRenderNodeId::new(1);
    let render_node2 = UiRenderNodeId::new(2);

    // Node 1 is a large background (0, 0, 100, 100)
    layout.push_entry(UiLayoutRectEntry::new(
        render_node1,
        node1,
        None,
        UiLayoutRect::new(0, 0, 100, 100),
    ));

    // Node 2 is a smaller foreground node on top of it (10, 10, 50, 50)
    layout.push_entry(UiLayoutRectEntry::new(
        render_node2,
        node2,
        None,
        UiLayoutRect::new(10, 10, 50, 50),
    ));

    let tester = DefaultHitTester::new();

    // Inside the small node 2 should return node2
    assert_eq!(tester.find_target_at(&layout, 20.0, 20.0), Some(node2));

    // Outside node 2 but inside node 1 should return node1
    assert_eq!(tester.find_target_at(&layout, 80.0, 80.0), Some(node1));

    // Outside both should return None
    assert_eq!(tester.find_target_at(&layout, 200.0, 200.0), None);

    // Edge of node 2 (inclusive of x/y)
    assert_eq!(tester.find_target_at(&layout, 10.0, 10.0), Some(node2));

    // Bottom edge (exclusive) should fall through to node 1
    assert_eq!(tester.find_target_at(&layout, 60.0, 60.0), Some(node1));
}

#[test]
fn hit_test_returns_none_for_empty_layout() {
    let layout = UiLayoutRectModel::new();
    let tester = DefaultHitTester::new();
    assert_eq!(tester.find_target_at(&layout, 50.0, 50.0), None);
}

#[test]
fn hit_test_returns_none_for_negative_coordinates() {
    let mut layout = UiLayoutRectModel::new();
    let node1 = UiProjectedNodeId::new(1);
    let render_node1 = UiRenderNodeId::new(1);
    layout.push_entry(UiLayoutRectEntry::new(
        render_node1,
        node1,
        None,
        UiLayoutRect::new(0, 0, 100, 100),
    ));

    let tester = DefaultHitTester::new();
    assert_eq!(tester.find_target_at(&layout, -10.0, 50.0), None);
}
