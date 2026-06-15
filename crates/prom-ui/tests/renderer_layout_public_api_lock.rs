use prom_ui::layout::{
    layout_render_model, UiLayoutModel, UiLayoutModelId, UiLayoutNode, UiLayoutNodeId,
    UiLayoutSlot, UiLayoutSlotId, UiLayoutSlotKind,
};
use prom_ui::model::UiIrNodeId;
use prom_ui::projection::{
    UiProjectedNode, UiProjectedNodeId, UiProjectedNodeKind, UiProjectionArtifact,
    UiProjectionArtifactId,
};
use prom_ui::renderer::{render_projection_to_model, UiRenderModel};

fn assert_copy<T: Copy>() {}
fn assert_clone<T: Clone>() {}
fn assert_debug<T: core::fmt::Debug>() {}
fn assert_eq_trait<T: Eq>() {}
fn assert_hash<T: std::hash::Hash>() {}
fn assert_default<T: Default>() {}

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

#[test]
fn layout_module_is_public() {
    // The fact that this compiles ensures `prom_ui::layout` is a public module.
}

#[test]
fn layout_id_new_and_raw_are_public() {
    let model_id = UiLayoutModelId::new(11);
    assert_eq!(model_id.raw(), 11);

    let node_id = UiLayoutNodeId::new(22);
    assert_eq!(node_id.raw(), 22);

    let slot_id = UiLayoutSlotId::new(33);
    assert_eq!(slot_id.raw(), 33);

    // Traits
    assert_copy::<UiLayoutModelId>();
    assert_clone::<UiLayoutModelId>();
    assert_debug::<UiLayoutModelId>();
    assert_eq_trait::<UiLayoutModelId>();
    assert_hash::<UiLayoutModelId>();
    assert_default::<UiLayoutModelId>();

    assert_copy::<UiLayoutNodeId>();
    assert_clone::<UiLayoutNodeId>();
    assert_debug::<UiLayoutNodeId>();
    assert_eq_trait::<UiLayoutNodeId>();
    assert_hash::<UiLayoutNodeId>();
    assert_default::<UiLayoutNodeId>();

    assert_copy::<UiLayoutSlotId>();
    assert_clone::<UiLayoutSlotId>();
    assert_debug::<UiLayoutSlotId>();
    assert_eq_trait::<UiLayoutSlotId>();
    assert_hash::<UiLayoutSlotId>();
    assert_default::<UiLayoutSlotId>();
}

#[test]
fn layout_slot_kind_variants_are_public() {
    let _root = UiLayoutSlotKind::Root;
    let _node = UiLayoutSlotKind::Node;
    let _metadata = UiLayoutSlotKind::Metadata;

    assert_copy::<UiLayoutSlotKind>();
    assert_clone::<UiLayoutSlotKind>();
    assert_debug::<UiLayoutSlotKind>();
    assert_eq_trait::<UiLayoutSlotKind>();
}

#[test]
fn layout_slot_accessors_are_public() {
    let render_model = create_test_render_model();
    let model = layout_render_model(&render_model);

    if let Some(slot) = model.slots().first() {
        let _id: UiLayoutSlotId = slot.id();
        let _kind: UiLayoutSlotKind = slot.kind();
        let _order: usize = slot.order();
    }
}

#[test]
fn layout_node_accessors_are_public() {
    let render_model = create_test_render_model();
    let model = layout_render_model(&render_model);

    if let Some(node) = model.nodes().first() {
        let _id: UiLayoutNodeId = node.id();
        let _src_rnd = node.source_render_node();
        let _src_prj = node.source_projection_node();
        let _src_ir = node.source_ir_node();
        let _slot: UiLayoutSlotId = node.slot();
        let _order: usize = node.order();
    }
}

#[test]
fn layout_model_accessors_are_public() {
    let render_model = create_test_render_model();
    let model = layout_render_model(&render_model);

    let _id: UiLayoutModelId = model.id();
    let _src_rnd = model.source_render_model();
    let _src_prj = model.source_projection();
    let _src_ir = model.source_ir_root();
    let _slots: &[UiLayoutSlot] = model.slots();
    let _nodes: &[UiLayoutNode] = model.nodes();
    let _is_empty: bool = model.is_empty();
    let _len: usize = model.len();
}

#[test]
fn layout_render_model_signature_is_locked() {
    let _: fn(&UiRenderModel) -> UiLayoutModel = layout_render_model;
}

#[test]
fn layout_render_model_preserves_model_identity() {
    let render_model = create_test_render_model();
    let layout_model = layout_render_model(&render_model);

    assert_eq!(layout_model.id().raw(), render_model.id().raw());
}

#[test]
fn layout_render_model_preserves_source_projection() {
    let render_model = create_test_render_model();
    let layout_model = layout_render_model(&render_model);

    assert_eq!(
        layout_model.source_projection(),
        render_model.source_projection()
    );
}

#[test]
fn layout_render_model_preserves_source_ir_root() {
    let render_model = create_test_render_model();
    let layout_model = layout_render_model(&render_model);

    assert_eq!(layout_model.source_ir_root(), render_model.source_ir_root());
}

#[test]
fn layout_render_model_preserves_node_references() {
    let render_model = create_test_render_model();
    let layout_model = layout_render_model(&render_model);
    assert_eq!(layout_model.len(), render_model.nodes().len());
}

#[test]
fn layout_render_model_preserves_node_order() {
    let render_model = create_test_render_model();
    let layout_model = layout_render_model(&render_model);

    for (i, node) in layout_model.nodes().iter().enumerate() {
        assert_eq!(node.order(), i);
    }
}

#[test]
fn repeated_layout_render_model_is_deterministic() {
    let render_model = create_test_render_model();

    let layout1 = layout_render_model(&render_model);
    let layout2 = layout_render_model(&render_model);

    assert_eq!(layout1.id().raw(), layout2.id().raw());
    assert_eq!(layout1.slots().len(), layout2.slots().len());
    assert_eq!(layout1.nodes().len(), layout2.nodes().len());
}

#[test]
fn layout_api_exposes_no_geometry_contract() {
    let code = include_str!("../src/layout/base.rs");
    assert!(!code.contains("pub width"));
    assert!(!code.contains("pub height"));
    assert!(!code.contains("pub x:"));
    assert!(!code.contains("pub y:"));
}

#[test]
fn layout_api_exposes_no_draw_event_backend_contract() {
    let code = include_str!("../src/layout/base.rs");
    assert!(!code.contains("draw"));
    assert!(!code.contains("event"));
    assert!(!code.contains("wgpu"));
    assert!(!code.contains("callback"));
}
