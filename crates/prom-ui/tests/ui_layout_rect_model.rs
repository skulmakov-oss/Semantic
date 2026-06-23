use prom_ui::{
    UiIrNodeId, UiLayoutRect, UiLayoutRectEntry, UiLayoutRectModel, UiProjectedNodeId,
    UiRenderNodeId,
};

fn entry(
    render_node_id: u64,
    projection_node_id: u64,
    ir_node_id: Option<u64>,
    rect: UiLayoutRect,
) -> UiLayoutRectEntry {
    UiLayoutRectEntry::new(
        UiRenderNodeId::new(render_node_id),
        UiProjectedNodeId::new(projection_node_id),
        ir_node_id.map(UiIrNodeId::new),
        rect,
    )
}

#[test]
fn layout_rect_preserves_integer_geometry() {
    let rect = UiLayoutRect::new(-12, 34, 0, 56);

    assert_eq!(rect.x(), -12);
    assert_eq!(rect.y(), 34);
    assert_eq!(rect.width(), 0);
    assert_eq!(rect.height(), 56);
}

#[test]
fn layout_rect_entry_preserves_render_projection_ir_evidence() {
    let rect = UiLayoutRect::new(7, -8, 9, 10);
    let entry = entry(11, 12, Some(13), rect);

    assert_eq!(entry.render_node_id(), UiRenderNodeId::new(11));
    assert_eq!(
        entry.source_projection_node_id(),
        UiProjectedNodeId::new(12)
    );
    assert_eq!(entry.source_ir_node_id(), Some(UiIrNodeId::new(13)));
    assert_eq!(entry.rect(), rect);
}

#[test]
fn layout_rect_entry_supports_missing_ir_evidence_explicitly() {
    let rect = UiLayoutRect::new(1, 2, 3, 4);
    let entry = entry(20, 21, None, rect);

    assert_eq!(entry.render_node_id(), UiRenderNodeId::new(20));
    assert_eq!(
        entry.source_projection_node_id(),
        UiProjectedNodeId::new(21)
    );
    assert_eq!(entry.source_ir_node_id(), None);
    assert_eq!(entry.rect(), rect);
}

#[test]
fn layout_rect_model_preserves_insertion_order() {
    let rect_a = UiLayoutRect::new(0, 0, 1, 1);
    let rect_b = UiLayoutRect::new(2, 3, 4, 5);
    let rect_c = UiLayoutRect::new(6, 7, 8, 9);

    let mut model = UiLayoutRectModel::new();
    model.push_entry(entry(1, 101, Some(201), rect_a));
    model.push_entry(entry(2, 102, Some(202), rect_b));
    model.push_entry(entry(3, 103, None, rect_c));

    let entries = model.entries();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].render_node_id(), UiRenderNodeId::new(1));
    assert_eq!(entries[1].render_node_id(), UiRenderNodeId::new(2));
    assert_eq!(entries[2].render_node_id(), UiRenderNodeId::new(3));
    assert_eq!(entries[0].rect(), rect_a);
    assert_eq!(entries[1].rect(), rect_b);
    assert_eq!(entries[2].rect(), rect_c);
}

#[test]
fn layout_rect_model_reports_len_and_empty_state() {
    let mut model = UiLayoutRectModel::new();

    assert!(model.is_empty());
    assert_eq!(model.len(), 0);
    assert!(model.entries().is_empty());

    model.push_entry(entry(9, 10, Some(11), UiLayoutRect::new(1, 1, 1, 1)));

    assert!(!model.is_empty());
    assert_eq!(model.len(), 1);
    assert_eq!(model.entries().len(), 1);
}

#[test]
fn zero_sized_rect_is_explicit_layout_evidence_not_absence() {
    let mut model = UiLayoutRectModel::new();
    let rect = UiLayoutRect::new(5, 6, 0, 0);
    model.push_entry(entry(30, 31, Some(32), rect));

    assert_eq!(model.len(), 1);
    assert_eq!(model.entries()[0].rect(), rect);
    assert_eq!(model.entries()[0].rect().width(), 0);
    assert_eq!(model.entries()[0].rect().height(), 0);
}

#[test]
fn layout_rect_model_public_api_is_externally_usable() {
    let rect = UiLayoutRect::new(-1, -2, 3, 4);
    let mut model = UiLayoutRectModel::new();
    model.push_entry(entry(40, 41, Some(42), rect));

    let stored = &model.entries()[0];
    assert_eq!(stored.render_node_id(), UiRenderNodeId::new(40));
    assert_eq!(
        stored.source_projection_node_id(),
        UiProjectedNodeId::new(41)
    );
    assert_eq!(stored.source_ir_node_id(), Some(UiIrNodeId::new(42)));
    assert_eq!(stored.rect(), rect);
}

#[test]
fn layout_rect_model_does_not_create_authority_or_backend_behavior() {
    let rect = UiLayoutRect::new(12, 13, 14, 15);
    let mut model = UiLayoutRectModel::new();
    model.push_entry(entry(50, 51, None, rect));

    let stored = &model.entries()[0];
    assert_eq!(stored.render_node_id(), UiRenderNodeId::new(50));
    assert_eq!(
        stored.source_projection_node_id(),
        UiProjectedNodeId::new(51)
    );
    assert_eq!(stored.source_ir_node_id(), None);
    assert_eq!(stored.rect(), rect);
    assert_eq!(model.len(), 1);
    assert_eq!(
        model
            .entries()
            .iter()
            .map(|entry| entry.rect())
            .collect::<Vec<_>>(),
        vec![rect]
    );
}

#[test]
fn layout_rect_model_is_cloneable_and_comparable() {
    let mut model = UiLayoutRectModel::new();
    model.push_entry(entry(60, 61, Some(62), UiLayoutRect::new(1, 2, 3, 4)));
    model.push_entry(entry(63, 64, None, UiLayoutRect::new(5, 6, 7, 8)));

    let cloned = model.clone();
    assert_eq!(cloned, model);
    assert_eq!(cloned.entries().len(), 2);
    assert_eq!(cloned.entries()[1].source_ir_node_id(), None);
}
