use crate::model::UiIrNodeId;
use crate::projection::UiProjectedNodeId;
use crate::projection::UiProjectionArtifactId;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutGeometryModelId {
    raw: u64,
}

impl UiLayoutGeometryModelId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }

    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutGeometryNodeId {
    raw: u64,
}

impl UiLayoutGeometryNodeId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }

    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiLayoutGeometryRect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl UiLayoutGeometryRect {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutGeometryNode {
    id: UiLayoutGeometryNodeId,
    source_layout_node: UiLayoutNodeId,
    source_layout_slot: UiLayoutSlotId,
    source_render_node: UiRenderNodeId,
    source_projection_node: Option<UiProjectedNodeId>,
    source_ir_node: Option<UiIrNodeId>,
    rect: UiLayoutGeometryRect,
    order: usize,
}

impl UiLayoutGeometryNode {
    pub fn id(&self) -> UiLayoutGeometryNodeId {
        self.id
    }

    pub fn source_layout_node(&self) -> UiLayoutNodeId {
        self.source_layout_node
    }

    pub fn source_layout_slot(&self) -> UiLayoutSlotId {
        self.source_layout_slot
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

    pub fn rect(&self) -> UiLayoutGeometryRect {
        self.rect
    }

    pub fn order(&self) -> usize {
        self.order
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutGeometryModel {
    id: UiLayoutGeometryModelId,
    source_layout_model: UiLayoutModelId,
    source_render_model: UiRenderModelId,
    source_projection: UiProjectionArtifactId,
    source_ir_root: Option<UiIrNodeId>,
    nodes: Vec<UiLayoutGeometryNode>,
}

impl UiLayoutGeometryModel {
    pub fn id(&self) -> UiLayoutGeometryModelId {
        self.id
    }

    pub fn source_layout_model(&self) -> UiLayoutModelId {
        self.source_layout_model
    }

    pub fn source_render_model(&self) -> UiRenderModelId {
        self.source_render_model
    }

    pub fn source_projection(&self) -> UiProjectionArtifactId {
        self.source_projection
    }

    pub fn source_ir_root(&self) -> Option<UiIrNodeId> {
        self.source_ir_root
    }

    pub fn nodes(&self) -> &[UiLayoutGeometryNode] {
        &self.nodes
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

pub fn build_layout_geometry(model: &UiLayoutModel) -> UiLayoutGeometryModel {
    let mut nodes = Vec::with_capacity(model.nodes().len());

    for layout_node in model.nodes() {
        nodes.push(UiLayoutGeometryNode {
            id: UiLayoutGeometryNodeId::new(layout_node.id().raw()),
            source_layout_node: layout_node.id(),
            source_layout_slot: layout_node.slot(),
            source_render_node: layout_node.source_render_node(),
            source_projection_node: layout_node.source_projection_node(),
            source_ir_node: layout_node.source_ir_node(),
            rect: UiLayoutGeometryRect::default(),
            order: layout_node.order(),
        });
    }

    UiLayoutGeometryModel {
        id: UiLayoutGeometryModelId::new(model.id().raw()),
        source_layout_model: model.id(),
        source_render_model: model.source_render_model(),
        source_projection: model.source_projection(),
        source_ir_root: model.source_ir_root(),
        nodes,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutConstraintsModelId {
    raw: u64,
}

impl UiLayoutConstraintsModelId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }

    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutConstraintId {
    raw: u64,
}

impl UiLayoutConstraintId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }

    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiLayoutConstraintKind {
    #[default]
    Unresolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiLayoutConstraintState {
    #[default]
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutConstraintDeclaration {
    id: UiLayoutConstraintId,
    source_layout_node: UiLayoutNodeId,
    source_layout_slot: UiLayoutSlotId,
    source_geometry_node: UiLayoutGeometryNodeId,
    source_render_node: UiRenderNodeId,
    source_projection_node: Option<UiProjectedNodeId>,
    source_ir_node: Option<UiIrNodeId>,
    kind: UiLayoutConstraintKind,
    state: UiLayoutConstraintState,
    order: usize,
}

impl UiLayoutConstraintDeclaration {
    pub fn id(&self) -> UiLayoutConstraintId {
        self.id
    }

    pub fn source_layout_node(&self) -> UiLayoutNodeId {
        self.source_layout_node
    }

    pub fn source_layout_slot(&self) -> UiLayoutSlotId {
        self.source_layout_slot
    }

    pub fn source_geometry_node(&self) -> UiLayoutGeometryNodeId {
        self.source_geometry_node
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

    pub fn kind(&self) -> UiLayoutConstraintKind {
        self.kind
    }

    pub fn state(&self) -> UiLayoutConstraintState {
        self.state
    }

    pub fn order(&self) -> usize {
        self.order
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutConstraintsModel {
    id: UiLayoutConstraintsModelId,
    source_layout_model: UiLayoutModelId,
    source_geometry_model: UiLayoutGeometryModelId,
    source_render_model: UiRenderModelId,
    source_projection: UiProjectionArtifactId,
    source_ir_root: Option<UiIrNodeId>,
    declarations: Vec<UiLayoutConstraintDeclaration>,
}

impl UiLayoutConstraintsModel {
    pub fn id(&self) -> UiLayoutConstraintsModelId {
        self.id
    }

    pub fn source_layout_model(&self) -> UiLayoutModelId {
        self.source_layout_model
    }

    pub fn source_geometry_model(&self) -> UiLayoutGeometryModelId {
        self.source_geometry_model
    }

    pub fn source_render_model(&self) -> UiRenderModelId {
        self.source_render_model
    }

    pub fn source_projection(&self) -> UiProjectionArtifactId {
        self.source_projection
    }

    pub fn source_ir_root(&self) -> Option<UiIrNodeId> {
        self.source_ir_root
    }

    pub fn declarations(&self) -> &[UiLayoutConstraintDeclaration] {
        &self.declarations
    }

    pub fn is_empty(&self) -> bool {
        self.declarations.is_empty()
    }

    pub fn len(&self) -> usize {
        self.declarations.len()
    }
}

pub fn build_layout_constraints(model: &UiLayoutModel) -> UiLayoutConstraintsModel {
    let geometry_model = build_layout_geometry(model);
    let mut declarations = Vec::with_capacity(geometry_model.nodes().len());

    for geometry_node in geometry_model.nodes() {
        declarations.push(UiLayoutConstraintDeclaration {
            id: UiLayoutConstraintId::new(geometry_node.id().raw()),
            source_layout_node: geometry_node.source_layout_node(),
            source_layout_slot: geometry_node.source_layout_slot(),
            source_geometry_node: geometry_node.id(),
            source_render_node: geometry_node.source_render_node(),
            source_projection_node: geometry_node.source_projection_node(),
            source_ir_node: geometry_node.source_ir_node(),
            kind: UiLayoutConstraintKind::Unresolved,
            state: UiLayoutConstraintState::Unresolved,
            order: geometry_node.order(),
        });
    }

    UiLayoutConstraintsModel {
        id: UiLayoutConstraintsModelId::new(model.id().raw()),
        source_layout_model: model.id(),
        source_geometry_model: geometry_model.id(),
        source_render_model: model.source_render_model(),
        source_projection: model.source_projection(),
        source_ir_root: model.source_ir_root(),
        declarations,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutSizingModelId {
    raw: u64,
}

impl UiLayoutSizingModelId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }

    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutSizingEntryId {
    raw: u64,
}

impl UiLayoutSizingEntryId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }

    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiLayoutSizingKind {
    #[default]
    Unresolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiLayoutSizingState {
    #[default]
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutSizingEntry {
    id: UiLayoutSizingEntryId,
    source_layout_node: UiLayoutNodeId,
    source_layout_slot: UiLayoutSlotId,
    source_geometry_node: UiLayoutGeometryNodeId,
    source_constraint_declaration: UiLayoutConstraintId,
    source_render_node: UiRenderNodeId,
    source_projection_node: Option<UiProjectedNodeId>,
    source_ir_node: Option<UiIrNodeId>,
    kind: UiLayoutSizingKind,
    state: UiLayoutSizingState,
    order: usize,
}

impl UiLayoutSizingEntry {
    pub fn id(&self) -> UiLayoutSizingEntryId {
        self.id
    }

    pub fn source_layout_node(&self) -> UiLayoutNodeId {
        self.source_layout_node
    }

    pub fn source_layout_slot(&self) -> UiLayoutSlotId {
        self.source_layout_slot
    }

    pub fn source_geometry_node(&self) -> UiLayoutGeometryNodeId {
        self.source_geometry_node
    }

    pub fn source_constraint_declaration(&self) -> UiLayoutConstraintId {
        self.source_constraint_declaration
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

    pub fn kind(&self) -> UiLayoutSizingKind {
        self.kind
    }

    pub fn state(&self) -> UiLayoutSizingState {
        self.state
    }

    pub fn order(&self) -> usize {
        self.order
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutSizingModel {
    id: UiLayoutSizingModelId,
    source_layout_model: UiLayoutModelId,
    source_geometry_model: UiLayoutGeometryModelId,
    source_constraints_model: UiLayoutConstraintsModelId,
    source_render_model: UiRenderModelId,
    source_projection: UiProjectionArtifactId,
    source_ir_root: Option<UiIrNodeId>,
    entries: Vec<UiLayoutSizingEntry>,
}

impl UiLayoutSizingModel {
    pub fn id(&self) -> UiLayoutSizingModelId {
        self.id
    }

    pub fn source_layout_model(&self) -> UiLayoutModelId {
        self.source_layout_model
    }

    pub fn source_geometry_model(&self) -> UiLayoutGeometryModelId {
        self.source_geometry_model
    }

    pub fn source_constraints_model(&self) -> UiLayoutConstraintsModelId {
        self.source_constraints_model
    }

    pub fn source_render_model(&self) -> UiRenderModelId {
        self.source_render_model
    }

    pub fn source_projection(&self) -> UiProjectionArtifactId {
        self.source_projection
    }

    pub fn source_ir_root(&self) -> Option<UiIrNodeId> {
        self.source_ir_root
    }

    pub fn entries(&self) -> &[UiLayoutSizingEntry] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

pub fn build_layout_sizing(model: &UiLayoutModel) -> UiLayoutSizingModel {
    let geometry_model = build_layout_geometry(model);
    let constraints_model = build_layout_constraints(model);
    let mut entries = Vec::with_capacity(constraints_model.declarations().len());

    for (index, ((declaration, layout_node), geometry_node)) in constraints_model
        .declarations()
        .iter()
        .zip(model.nodes())
        .zip(geometry_model.nodes())
        .enumerate()
    {
        entries.push(UiLayoutSizingEntry {
            id: UiLayoutSizingEntryId::new(declaration.id().raw()),
            source_layout_node: layout_node.id(),
            source_layout_slot: layout_node.slot(),
            source_geometry_node: geometry_node.id(),
            source_constraint_declaration: declaration.id(),
            source_render_node: layout_node.source_render_node(),
            source_projection_node: layout_node.source_projection_node(),
            source_ir_node: layout_node.source_ir_node(),
            kind: UiLayoutSizingKind::Unresolved,
            state: UiLayoutSizingState::Unresolved,
            order: index,
        });
    }

    UiLayoutSizingModel {
        id: UiLayoutSizingModelId::new(model.id().raw()),
        source_layout_model: model.id(),
        source_geometry_model: geometry_model.id(),
        source_constraints_model: constraints_model.id(),
        source_render_model: model.source_render_model(),
        source_projection: model.source_projection(),
        source_ir_root: model.source_ir_root(),
        entries,
    }
}
