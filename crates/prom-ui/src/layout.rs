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
    let mut order = 0;

    for render_node in model.nodes() {
        nodes.push(UiLayoutNode {
            id: UiLayoutNodeId::new(render_node.id().raw()),
            source_render_node: render_node.id(),
            source_projection_node: Some(render_node.source_projection_node()),
            source_ir_node: render_node.source_ir_node(),
            slot: UiLayoutSlotId::new(2),
            order,
        });
        order += 1;
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
