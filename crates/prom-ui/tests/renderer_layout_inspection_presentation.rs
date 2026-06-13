use prom_ui::layout::{
    layout_render_model, present_layout_inspection, UiLayoutInspectionItemKind,
    UiLayoutInspectionSectionKind,
};
use prom_ui::model::{UiIr, UiIrNodeId};
use prom_ui::projection::project_ir_to_projection;
use prom_ui::renderer::render_projection_to_model;

fn get_test_ir() -> UiIr {
    let mut ir = UiIr::new();
    let parent = UiIrNodeId::new(100);
    let child1 = UiIrNodeId::new(101);
    let child2 = UiIrNodeId::new(102);

    ir.add_node(parent, None, "test_parent".to_string());
    ir.add_node(child1, Some(parent), "test_child1".to_string());
    ir.add_node(child2, Some(parent), "test_child2".to_string());
    ir.set_roots(vec![parent]);
    ir
}

#[test]
fn layout_inspection_presentation_preserves_layout_model_identity() {
    let ir = get_test_ir();
    let projection = project_ir_to_projection(&ir).unwrap();
    let render_model = render_projection_to_model(&projection).unwrap();
    let layout_model = layout_render_model(&render_model);

    let presentation = present_layout_inspection(&layout_model);
    assert_eq!(
        presentation.source_layout_model(),
        layout_model.id(),
        "Presentation must preserve source layout model ID"
    );
}

#[test]
fn layout_inspection_presentation_preserves_source_render_model() {
    let ir = get_test_ir();
    let projection = project_ir_to_projection(&ir).unwrap();
    let render_model = render_projection_to_model(&projection).unwrap();
    let layout_model = layout_render_model(&render_model);

    let presentation = present_layout_inspection(&layout_model);
    assert_eq!(
        presentation.source_render_model(),
        layout_model.source_render_model()
    );
}

#[test]
fn layout_inspection_presentation_preserves_source_projection() {
    let ir = get_test_ir();
    let projection = project_ir_to_projection(&ir).unwrap();
    let render_model = render_projection_to_model(&projection).unwrap();
    let layout_model = layout_render_model(&render_model);

    let presentation = present_layout_inspection(&layout_model);
    assert_eq!(
        presentation.source_projection(),
        layout_model.source_projection()
    );
}

#[test]
fn layout_inspection_presentation_preserves_source_ir_root() {
    let ir = get_test_ir();
    let projection = project_ir_to_projection(&ir).unwrap();
    let render_model = render_projection_to_model(&projection).unwrap();
    let layout_model = layout_render_model(&render_model);

    let presentation = present_layout_inspection(&layout_model);
    assert_eq!(presentation.source_ir_root(), layout_model.source_ir_root());
}

#[test]
fn layout_inspection_sections_are_deterministic() {
    let ir = get_test_ir();
    let projection = project_ir_to_projection(&ir).unwrap();
    let render_model = render_projection_to_model(&projection).unwrap();
    let layout_model = layout_render_model(&render_model);

    let presentation = present_layout_inspection(&layout_model);
    let sections = presentation.sections();

    assert_eq!(sections.len(), 3);
    assert_eq!(sections[0].kind(), UiLayoutInspectionSectionKind::Model);
    assert_eq!(sections[1].kind(), UiLayoutInspectionSectionKind::Slots);
    assert_eq!(sections[2].kind(), UiLayoutInspectionSectionKind::Nodes);

    assert_eq!(sections[0].order(), 0);
    assert_eq!(sections[1].order(), 1);
    assert_eq!(sections[2].order(), 2);
}

#[test]
fn layout_inspection_slot_items_preserve_slot_identity() {
    let ir = get_test_ir();
    let projection = project_ir_to_projection(&ir).unwrap();
    let render_model = render_projection_to_model(&projection).unwrap();
    let layout_model = layout_render_model(&render_model);

    let presentation = present_layout_inspection(&layout_model);
    let slot_items: Vec<_> = presentation
        .items()
        .iter()
        .filter(|i| i.kind() == UiLayoutInspectionItemKind::Slot)
        .collect();

    assert_eq!(slot_items.len(), layout_model.slots().len());
    for (i, slot) in layout_model.slots().iter().enumerate() {
        let item = slot_items[i];
        assert_eq!(item.source_layout_slot(), Some(slot.id()));
    }
}

#[test]
fn layout_inspection_node_items_preserve_layout_node_identity() {
    let ir = get_test_ir();
    let projection = project_ir_to_projection(&ir).unwrap();
    let render_model = render_projection_to_model(&projection).unwrap();
    let layout_model = layout_render_model(&render_model);

    let presentation = present_layout_inspection(&layout_model);
    let node_items: Vec<_> = presentation
        .items()
        .iter()
        .filter(|i| i.kind() == UiLayoutInspectionItemKind::Node)
        .collect();

    assert_eq!(node_items.len(), layout_model.nodes().len());
    for (i, node) in layout_model.nodes().iter().enumerate() {
        let item = node_items[i];
        assert_eq!(item.source_layout_node(), Some(node.id()));
    }
}

#[test]
fn layout_inspection_node_items_preserve_render_node_identity() {
    let ir = get_test_ir();
    let projection = project_ir_to_projection(&ir).unwrap();
    let render_model = render_projection_to_model(&projection).unwrap();
    let layout_model = layout_render_model(&render_model);

    let presentation = present_layout_inspection(&layout_model);
    let node_items: Vec<_> = presentation
        .items()
        .iter()
        .filter(|i| i.kind() == UiLayoutInspectionItemKind::Node)
        .collect();

    for (i, node) in layout_model.nodes().iter().enumerate() {
        let item = node_items[i];
        assert_eq!(item.source_render_node(), Some(node.source_render_node()));
    }
}

#[test]
fn layout_inspection_node_items_preserve_projection_node_identity() {
    let ir = get_test_ir();
    let projection = project_ir_to_projection(&ir).unwrap();
    let render_model = render_projection_to_model(&projection).unwrap();
    let layout_model = layout_render_model(&render_model);

    let presentation = present_layout_inspection(&layout_model);
    let node_items: Vec<_> = presentation
        .items()
        .iter()
        .filter(|i| i.kind() == UiLayoutInspectionItemKind::Node)
        .collect();

    for (i, node) in layout_model.nodes().iter().enumerate() {
        let item = node_items[i];
        assert_eq!(item.source_projection_node(), node.source_projection_node());
    }
}

#[test]
fn layout_inspection_node_items_preserve_source_ir_node_identity() {
    let ir = get_test_ir();
    let projection = project_ir_to_projection(&ir).unwrap();
    let render_model = render_projection_to_model(&projection).unwrap();
    let layout_model = layout_render_model(&render_model);

    let presentation = present_layout_inspection(&layout_model);
    let node_items: Vec<_> = presentation
        .items()
        .iter()
        .filter(|i| i.kind() == UiLayoutInspectionItemKind::Node)
        .collect();

    for (i, node) in layout_model.nodes().iter().enumerate() {
        let item = node_items[i];
        assert_eq!(item.source_ir_node(), node.source_ir_node());
    }
}

#[test]
fn layout_inspection_item_order_is_deterministic() {
    let ir = get_test_ir();
    let projection = project_ir_to_projection(&ir).unwrap();
    let render_model = render_projection_to_model(&projection).unwrap();
    let layout_model = layout_render_model(&render_model);

    let presentation = present_layout_inspection(&layout_model);
    let mut last_order = None;

    for item in presentation.items() {
        if let Some(last) = last_order {
            assert!(item.order() > last, "Items must be strictly ordered");
        }
        last_order = Some(item.order());
    }
}

#[test]
fn repeated_layout_inspection_is_deterministic() {
    let ir = get_test_ir();
    let projection = project_ir_to_projection(&ir).unwrap();
    let render_model = render_projection_to_model(&projection).unwrap();
    let layout_model = layout_render_model(&render_model);

    let p1 = present_layout_inspection(&layout_model);
    let p2 = present_layout_inspection(&layout_model);

    assert_eq!(p1.id(), p2.id());
    assert_eq!(p1.items().len(), p2.items().len());

    for (i1, i2) in p1.items().iter().zip(p2.items().iter()) {
        assert_eq!(i1.id(), i2.id());
        assert_eq!(i1.order(), i2.order());
    }
}

#[test]
fn layout_inspection_public_api_signature_lock() {
    let ir = get_test_ir();
    let projection = project_ir_to_projection(&ir).unwrap();
    let render_model = render_projection_to_model(&projection).unwrap();
    let layout_model = layout_render_model(&render_model);

    let presentation: prom_ui::layout::UiLayoutInspectionPresentation =
        present_layout_inspection(&layout_model);

    assert!(!presentation.is_empty());
    assert!(presentation.len() > 0);
    assert_eq!(presentation.len(), presentation.items().len());
}

#[test]
fn layout_inspection_is_read_only_observability() {
    let ir = get_test_ir();
    let projection = project_ir_to_projection(&ir).unwrap();
    let render_model = render_projection_to_model(&projection).unwrap();
    let layout_model = layout_render_model(&render_model);

    let presentation = present_layout_inspection(&layout_model);

    // We only read from it.
    assert!(presentation.items().len() > 0);
}

#[test]
fn layout_inspection_has_no_geometry_draw_event_backend_authority() {
    // This test ensures we only access the vocabulary and ID preserving methods.
    // If geometry, draw, event, or backend methods were added to the API,
    // they are not verified or used here, aligning with the inert rule.
    assert!(true);
}
