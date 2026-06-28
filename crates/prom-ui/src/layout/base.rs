use crate::model::UiIrNodeId;
use crate::projection::UiProjectedNodeId;
use crate::renderer::{UiRenderModel, UiRenderModelId, UiRenderNodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutModelId {
    raw: u64,
}

impl UiLayoutModelId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }
    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutNodeId {
    raw: u64,
}

impl UiLayoutNodeId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }
    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutSlotId {
    raw: u64,
}

impl UiLayoutSlotId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }
    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLayoutSlotKind {
    Root,
    Node,
    Metadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutSlot {
    id: UiLayoutSlotId,
    kind: UiLayoutSlotKind,
    order: usize,
}

impl UiLayoutSlot {
    pub fn id(&self) -> UiLayoutSlotId {
        self.id
    }
    pub fn kind(&self) -> UiLayoutSlotKind {
        self.kind
    }
    pub fn order(&self) -> usize {
        self.order
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutNode {
    id: UiLayoutNodeId,
    source_render_node: UiRenderNodeId,
    source_projection_node: Option<UiProjectedNodeId>,
    source_ir_node: Option<UiIrNodeId>,
    slot: UiLayoutSlotId,
    order: usize,
}

impl UiLayoutNode {
    pub fn id(&self) -> UiLayoutNodeId {
        self.id
    }
    pub fn source_render_node(&self) -> UiRenderNodeId {
        self.source_render_node
    }
    pub fn source_projection_node(&self) -> Option<UiProjectedNodeId> {
        self.source_projection_node
    }
    pub fn source_ir_node(&self) -> Option<UiIrNodeId> {
        self.source_ir_node
    }
    pub fn slot(&self) -> UiLayoutSlotId {
        self.slot
    }
    pub fn order(&self) -> usize {
        self.order
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutModel {
    id: UiLayoutModelId,
    source_render_model: UiRenderModelId,
    source_projection: crate::projection::UiProjectionArtifactId,
    source_ir_root: Option<UiIrNodeId>,
    slots: Vec<UiLayoutSlot>,
    nodes: Vec<UiLayoutNode>,
}

impl UiLayoutModel {
    pub fn id(&self) -> UiLayoutModelId {
        self.id
    }
    pub fn source_render_model(&self) -> UiRenderModelId {
        self.source_render_model
    }
    pub fn source_projection(&self) -> crate::projection::UiProjectionArtifactId {
        self.source_projection
    }
    pub fn source_ir_root(&self) -> Option<UiIrNodeId> {
        self.source_ir_root
    }
    pub fn slots(&self) -> &[UiLayoutSlot] {
        &self.slots
    }
    pub fn nodes(&self) -> &[UiLayoutNode] {
        &self.nodes
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

pub fn layout_render_model(model: &UiRenderModel) -> UiLayoutModel {
    let layout_model_id = UiLayoutModelId::new(model.id().raw());

    let mut slots = Vec::new();

    slots.push(UiLayoutSlot {
        id: UiLayoutSlotId::new(1),
        kind: UiLayoutSlotKind::Root,
        order: 0,
    });

    if !model.nodes().is_empty() {
        slots.push(UiLayoutSlot {
            id: UiLayoutSlotId::new(2),
            kind: UiLayoutSlotKind::Node,
            order: 1,
        });
    }

    let mut nodes = Vec::with_capacity(model.nodes().len());

    for (order, render_node) in model.nodes().iter().enumerate() {
        nodes.push(UiLayoutNode {
            id: UiLayoutNodeId::new(render_node.id().raw()),
            source_render_node: render_node.id(),
            source_projection_node: Some(render_node.source_projection_node()),
            source_ir_node: render_node.source_ir_node(),
            slot: UiLayoutSlotId::new(2),
            order,
        });
    }

    UiLayoutModel {
        id: layout_model_id,
        source_render_model: model.id(),
        source_projection: model.source_projection(),
        source_ir_root: model.source_ir_root(),
        slots,
        nodes,
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutInspectionPresentationId {
    raw: u64,
}

impl UiLayoutInspectionPresentationId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }
    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutInspectionSectionId {
    raw: u64,
}

impl UiLayoutInspectionSectionId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }
    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutInspectionItemId {
    raw: u64,
}

impl UiLayoutInspectionItemId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }
    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLayoutInspectionSectionKind {
    Model,
    Slots,
    Nodes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLayoutInspectionItemKind {
    ModelIdentity,
    SourceReference,
    Slot,
    Node,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutInspectionSection {
    id: UiLayoutInspectionSectionId,
    kind: UiLayoutInspectionSectionKind,
    order: usize,
}

impl UiLayoutInspectionSection {
    pub fn id(&self) -> UiLayoutInspectionSectionId {
        self.id
    }
    pub fn kind(&self) -> UiLayoutInspectionSectionKind {
        self.kind
    }
    pub fn order(&self) -> usize {
        self.order
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutInspectionItem {
    id: UiLayoutInspectionItemId,
    section: UiLayoutInspectionSectionId,
    kind: UiLayoutInspectionItemKind,
    source_layout_slot: Option<UiLayoutSlotId>,
    source_layout_node: Option<UiLayoutNodeId>,
    source_render_node: Option<UiRenderNodeId>,
    source_projection_node: Option<UiProjectedNodeId>,
    source_ir_node: Option<UiIrNodeId>,
    order: usize,
}

impl UiLayoutInspectionItem {
    pub fn id(&self) -> UiLayoutInspectionItemId {
        self.id
    }
    pub fn section(&self) -> UiLayoutInspectionSectionId {
        self.section
    }
    pub fn kind(&self) -> UiLayoutInspectionItemKind {
        self.kind
    }
    pub fn source_layout_slot(&self) -> Option<UiLayoutSlotId> {
        self.source_layout_slot
    }
    pub fn source_layout_node(&self) -> Option<UiLayoutNodeId> {
        self.source_layout_node
    }
    pub fn source_render_node(&self) -> Option<UiRenderNodeId> {
        self.source_render_node
    }
    pub fn source_projection_node(&self) -> Option<UiProjectedNodeId> {
        self.source_projection_node
    }
    pub fn source_ir_node(&self) -> Option<UiIrNodeId> {
        self.source_ir_node
    }
    pub fn order(&self) -> usize {
        self.order
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutInspectionPresentation {
    id: UiLayoutInspectionPresentationId,
    source_layout_model: UiLayoutModelId,
    source_render_model: UiRenderModelId,
    source_projection: crate::projection::UiProjectionArtifactId,
    source_ir_root: Option<UiIrNodeId>,
    sections: Vec<UiLayoutInspectionSection>,
    items: Vec<UiLayoutInspectionItem>,
}

impl UiLayoutInspectionPresentation {
    pub fn id(&self) -> UiLayoutInspectionPresentationId {
        self.id
    }
    pub fn source_layout_model(&self) -> UiLayoutModelId {
        self.source_layout_model
    }
    pub fn source_render_model(&self) -> UiRenderModelId {
        self.source_render_model
    }
    pub fn source_projection(&self) -> crate::projection::UiProjectionArtifactId {
        self.source_projection
    }
    pub fn source_ir_root(&self) -> Option<UiIrNodeId> {
        self.source_ir_root
    }
    pub fn sections(&self) -> &[UiLayoutInspectionSection] {
        &self.sections
    }
    pub fn items(&self) -> &[UiLayoutInspectionItem] {
        &self.items
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

pub fn present_layout_inspection(model: &UiLayoutModel) -> UiLayoutInspectionPresentation {
    let presentation_id = UiLayoutInspectionPresentationId::new(model.id().raw());

    let mut sections = Vec::new();

    let model_section_id = UiLayoutInspectionSectionId::new(1);
    let slots_section_id = UiLayoutInspectionSectionId::new(2);
    let nodes_section_id = UiLayoutInspectionSectionId::new(3);

    sections.push(UiLayoutInspectionSection {
        id: model_section_id,
        kind: UiLayoutInspectionSectionKind::Model,
        order: 0,
    });

    sections.push(UiLayoutInspectionSection {
        id: slots_section_id,
        kind: UiLayoutInspectionSectionKind::Slots,
        order: 1,
    });

    sections.push(UiLayoutInspectionSection {
        id: nodes_section_id,
        kind: UiLayoutInspectionSectionKind::Nodes,
        order: 2,
    });

    let mut items = Vec::new();
    let mut item_order = 0;

    items.push(UiLayoutInspectionItem {
        id: UiLayoutInspectionItemId::new(1),
        section: model_section_id,
        kind: UiLayoutInspectionItemKind::ModelIdentity,
        source_layout_slot: None,
        source_layout_node: None,
        source_render_node: None,
        source_projection_node: None,
        source_ir_node: None,
        order: item_order,
    });
    item_order += 1;

    items.push(UiLayoutInspectionItem {
        id: UiLayoutInspectionItemId::new(2),
        section: model_section_id,
        kind: UiLayoutInspectionItemKind::SourceReference,
        source_layout_slot: None,
        source_layout_node: None,
        source_render_node: None,
        source_projection_node: None,
        source_ir_node: None,
        order: item_order,
    });
    item_order += 1;

    for slot in model.slots() {
        items.push(UiLayoutInspectionItem {
            id: UiLayoutInspectionItemId::new(10_000u64.wrapping_add(slot.id().raw())),
            section: slots_section_id,
            kind: UiLayoutInspectionItemKind::Slot,
            source_layout_slot: Some(slot.id()),
            source_layout_node: None,
            source_render_node: None,
            source_projection_node: None,
            source_ir_node: None,
            order: item_order,
        });
        item_order += 1;
    }

    for node in model.nodes() {
        items.push(UiLayoutInspectionItem {
            id: UiLayoutInspectionItemId::new(20_000u64.wrapping_add(node.id().raw())),
            section: nodes_section_id,
            kind: UiLayoutInspectionItemKind::Node,
            source_layout_slot: None,
            source_layout_node: Some(node.id()),
            source_render_node: Some(node.source_render_node()),
            source_projection_node: node.source_projection_node(),
            source_ir_node: node.source_ir_node(),
            order: item_order,
        });
        item_order += 1;
    }

    UiLayoutInspectionPresentation {
        id: presentation_id,
        source_layout_model: model.id(),
        source_render_model: model.source_render_model(),
        source_projection: model.source_projection(),
        source_ir_root: model.source_ir_root(),
        sections,
        items,
    }
}
