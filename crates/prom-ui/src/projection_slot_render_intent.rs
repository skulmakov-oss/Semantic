use crate::ast_slot_ir_intent::UiAstSlotIrIntentEntryId;
use crate::ir_slot_projection_intent::{
    UiIrSlotProjectionIntentEntryId, UiIrSlotProjectionIntentModel,
};
use crate::model::{
    UiAstNodeId, UiAstNodeKind, UiIrNodeId, UiIrNodeKind, UiNodeId, UiNodeKind, UiNodeResolution,
    UiTreeId,
};
use crate::projection::{UiProjectedNodeId, UiProjectedNodeKind};
use crate::renderer::{UiRenderModel, UiRenderNodeId, UiRenderNodeKind};
use crate::tree_slot_ast_intent::UiTreeSlotAstIntentEntryId;
use crate::tree_slot_intent::UiTreeSlotCarrierIntentEntryId;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiProjectionSlotRenderIntentModelId(u64);

impl UiProjectionSlotRenderIntentModelId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiProjectionSlotRenderIntentEntryId(u64);

impl UiProjectionSlotRenderIntentEntryId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    pub fn raw(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiProjectionSlotRenderIntentKind {
    RenderNodeLinkedToProjectionSlotIntent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiProjectionSlotRenderIntentState {
    Deferred,
}

#[derive(Debug, Clone)]
pub struct UiProjectionSlotRenderIntentEntry {
    id: UiProjectionSlotRenderIntentEntryId,
    source_projection_intent_entry_id: UiIrSlotProjectionIntentEntryId,
    source_ir_intent_entry_id: UiAstSlotIrIntentEntryId,
    source_ast_intent_entry_id: UiTreeSlotAstIntentEntryId,
    source_tree_intent_entry_id: UiTreeSlotCarrierIntentEntryId,
    source_tree_id: UiTreeId,
    source_tree_node_id: UiNodeId,
    source_tree_node_kind: UiNodeKind,
    source_tree_resolution: UiNodeResolution,
    source_ast_node_id: UiAstNodeId,
    source_ast_node_kind: UiAstNodeKind,
    source_ir_node_id: UiIrNodeId,
    source_ir_node_kind: UiIrNodeKind,
    projected_node_id: UiProjectedNodeId,
    projected_node_kind: UiProjectedNodeKind,
    render_node_id: UiRenderNodeId,
    render_node_kind: UiRenderNodeKind,
    render_source_projection_node: UiProjectedNodeId,
    render_source_ir_node: Option<UiIrNodeId>,
    parent_tree_node_id: Option<UiNodeId>,
    child_tree_node_ids: Vec<UiNodeId>,
    parent_ast_node_id: Option<UiAstNodeId>,
    child_ast_node_ids: Vec<UiAstNodeId>,
    parent_ir_node_id: Option<UiIrNodeId>,
    child_ir_node_ids: Vec<UiIrNodeId>,
    parent_projected_node_id: Option<UiProjectedNodeId>,
    child_projected_node_ids: Vec<UiProjectedNodeId>,
    kind: UiProjectionSlotRenderIntentKind,
    state: UiProjectionSlotRenderIntentState,
}

impl UiProjectionSlotRenderIntentEntry {
    pub fn id(&self) -> UiProjectionSlotRenderIntentEntryId {
        self.id
    }
    pub fn source_projection_intent_entry_id(&self) -> UiIrSlotProjectionIntentEntryId {
        self.source_projection_intent_entry_id
    }
    pub fn source_ir_intent_entry_id(&self) -> UiAstSlotIrIntentEntryId {
        self.source_ir_intent_entry_id
    }
    pub fn source_ast_intent_entry_id(&self) -> UiTreeSlotAstIntentEntryId {
        self.source_ast_intent_entry_id
    }
    pub fn source_tree_intent_entry_id(&self) -> UiTreeSlotCarrierIntentEntryId {
        self.source_tree_intent_entry_id
    }
    pub fn source_tree_id(&self) -> UiTreeId {
        self.source_tree_id
    }
    pub fn source_tree_node_id(&self) -> UiNodeId {
        self.source_tree_node_id
    }
    pub fn source_tree_node_kind(&self) -> UiNodeKind {
        self.source_tree_node_kind
    }
    pub fn source_tree_resolution(&self) -> UiNodeResolution {
        self.source_tree_resolution
    }
    pub fn source_ast_node_id(&self) -> UiAstNodeId {
        self.source_ast_node_id
    }
    pub fn source_ast_node_kind(&self) -> UiAstNodeKind {
        self.source_ast_node_kind
    }
    pub fn source_ir_node_id(&self) -> UiIrNodeId {
        self.source_ir_node_id
    }
    pub fn source_ir_node_kind(&self) -> UiIrNodeKind {
        self.source_ir_node_kind
    }
    pub fn projected_node_id(&self) -> UiProjectedNodeId {
        self.projected_node_id
    }
    pub fn projected_node_kind(&self) -> UiProjectedNodeKind {
        self.projected_node_kind
    }
    pub fn render_node_id(&self) -> UiRenderNodeId {
        self.render_node_id
    }
    pub fn render_node_kind(&self) -> UiRenderNodeKind {
        self.render_node_kind
    }
    pub fn render_source_projection_node(&self) -> UiProjectedNodeId {
        self.render_source_projection_node
    }
    pub fn render_source_ir_node(&self) -> Option<UiIrNodeId> {
        self.render_source_ir_node
    }
    pub fn parent_tree_node_id(&self) -> Option<UiNodeId> {
        self.parent_tree_node_id
    }
    pub fn child_tree_node_ids(&self) -> &[UiNodeId] {
        &self.child_tree_node_ids
    }
    pub fn parent_ast_node_id(&self) -> Option<UiAstNodeId> {
        self.parent_ast_node_id
    }
    pub fn child_ast_node_ids(&self) -> &[UiAstNodeId] {
        &self.child_ast_node_ids
    }
    pub fn parent_ir_node_id(&self) -> Option<UiIrNodeId> {
        self.parent_ir_node_id
    }
    pub fn child_ir_node_ids(&self) -> &[UiIrNodeId] {
        &self.child_ir_node_ids
    }
    pub fn parent_projected_node_id(&self) -> Option<UiProjectedNodeId> {
        self.parent_projected_node_id
    }
    pub fn child_projected_node_ids(&self) -> &[UiProjectedNodeId] {
        &self.child_projected_node_ids
    }
    pub fn kind(&self) -> UiProjectionSlotRenderIntentKind {
        self.kind
    }
    pub fn state(&self) -> UiProjectionSlotRenderIntentState {
        self.state
    }
}

#[derive(Debug, Clone)]
pub struct UiProjectionSlotRenderIntentModel {
    id: UiProjectionSlotRenderIntentModelId,
    entries: Vec<UiProjectionSlotRenderIntentEntry>,
}

impl UiProjectionSlotRenderIntentModel {
    pub fn id(&self) -> UiProjectionSlotRenderIntentModelId {
        self.id
    }
    pub fn entries(&self) -> &[UiProjectionSlotRenderIntentEntry] {
        &self.entries
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiProjectionSlotRenderIntentDiagnosticKind {
    MissingRenderNode {
        source_projection_node_id: UiProjectedNodeId,
    },
    UnexpectedRenderNodeKind {
        source_projection_node_id: UiProjectedNodeId,
        render_node_id: UiRenderNodeId,
        actual_kind: UiRenderNodeKind,
    },
    UnexpectedRenderMarkers {
        source_projection_node_id: UiProjectedNodeId,
        render_node_id: UiRenderNodeId,
    },
}

#[derive(Debug, Clone)]
pub struct UiProjectionSlotRenderIntentDiagnostic {
    kind: UiProjectionSlotRenderIntentDiagnosticKind,
}

impl UiProjectionSlotRenderIntentDiagnostic {
    pub fn kind(&self) -> &UiProjectionSlotRenderIntentDiagnosticKind {
        &self.kind
    }
}

#[derive(Debug, Default)]
pub struct UiProjectionSlotRenderIntentDiagnostics {
    diagnostics: Vec<UiProjectionSlotRenderIntentDiagnostic>,
}

impl UiProjectionSlotRenderIntentDiagnostics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push(&mut self, diagnostic: UiProjectionSlotRenderIntentDiagnostic) {
        self.diagnostics.push(diagnostic);
    }
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = &UiProjectionSlotRenderIntentDiagnostic> {
        self.diagnostics.iter()
    }
}

pub type UiProjectionSlotRenderIntentResult =
    Result<UiProjectionSlotRenderIntentModel, UiProjectionSlotRenderIntentDiagnostics>;

pub fn build_projection_slot_render_intents(
    projection_intents: &UiIrSlotProjectionIntentModel,
    render_model: &UiRenderModel,
) -> UiProjectionSlotRenderIntentResult {
    let mut diagnostics = UiProjectionSlotRenderIntentDiagnostics::new();
    let mut entries = Vec::new();

    for proj_intent in projection_intents.entries() {
        let projected_node_id = proj_intent.projected_node_id();

        // Find the matching render node
        let mut matched_render_node = None;
        for render_node in render_model.nodes() {
            if render_node.source_projection_node() == projected_node_id {
                matched_render_node = Some(render_node);
                break;
            }
        }

        if let Some(render_node) = matched_render_node {
            if render_node.kind() != UiRenderNodeKind::Fragment {
                diagnostics.push(UiProjectionSlotRenderIntentDiagnostic {
                    kind: UiProjectionSlotRenderIntentDiagnosticKind::UnexpectedRenderNodeKind {
                        source_projection_node_id: projected_node_id,
                        render_node_id: render_node.id(),
                        actual_kind: render_node.kind(),
                    },
                });
            } else if !render_node.markers().is_empty() {
                diagnostics.push(UiProjectionSlotRenderIntentDiagnostic {
                    kind: UiProjectionSlotRenderIntentDiagnosticKind::UnexpectedRenderMarkers {
                        source_projection_node_id: projected_node_id,
                        render_node_id: render_node.id(),
                    },
                });
            } else {
                let entry_id = UiProjectionSlotRenderIntentEntryId::new(projected_node_id.raw());

                let entry = UiProjectionSlotRenderIntentEntry {
                    id: entry_id,
                    source_projection_intent_entry_id: proj_intent.id(),
                    source_ir_intent_entry_id: proj_intent.source_ir_intent_entry_id(),
                    source_ast_intent_entry_id: proj_intent.source_ast_intent_entry_id(),
                    source_tree_intent_entry_id: proj_intent.source_tree_intent_entry_id(),
                    source_tree_id: proj_intent.source_tree_id(),
                    source_tree_node_id: proj_intent.source_tree_node_id(),
                    source_tree_node_kind: proj_intent.source_tree_node_kind(),
                    source_tree_resolution: proj_intent.source_tree_resolution(),
                    source_ast_node_id: proj_intent.source_ast_node_id(),
                    source_ast_node_kind: proj_intent.source_ast_node_kind(),
                    source_ir_node_id: proj_intent.source_ir_node_id(),
                    source_ir_node_kind: proj_intent.source_ir_node_kind(),
                    projected_node_id: proj_intent.projected_node_id(),
                    projected_node_kind: proj_intent.projected_node_kind(),
                    render_node_id: render_node.id(),
                    render_node_kind: render_node.kind(),
                    render_source_projection_node: render_node.source_projection_node(),
                    render_source_ir_node: render_node.source_ir_node(),
                    parent_tree_node_id: proj_intent.parent_tree_node_id(),
                    child_tree_node_ids: proj_intent.child_tree_node_ids().to_vec(),
                    parent_ast_node_id: proj_intent.parent_ast_node_id(),
                    child_ast_node_ids: proj_intent.child_ast_node_ids().to_vec(),
                    parent_ir_node_id: proj_intent.parent_ir_node_id(),
                    child_ir_node_ids: proj_intent.child_ir_node_ids().to_vec(),
                    parent_projected_node_id: proj_intent.parent_projected_node_id(),
                    child_projected_node_ids: proj_intent.child_projected_node_ids().to_vec(),
                    kind: UiProjectionSlotRenderIntentKind::RenderNodeLinkedToProjectionSlotIntent,
                    state: UiProjectionSlotRenderIntentState::Deferred,
                };
                entries.push(entry);
            }
        } else {
            diagnostics.push(UiProjectionSlotRenderIntentDiagnostic {
                kind: UiProjectionSlotRenderIntentDiagnosticKind::MissingRenderNode {
                    source_projection_node_id: projected_node_id,
                },
            });
        }
    }

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    Ok(UiProjectionSlotRenderIntentModel {
        id: UiProjectionSlotRenderIntentModelId::new(1),
        entries,
    })
}
