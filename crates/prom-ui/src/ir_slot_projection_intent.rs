use crate::ast_slot_ir_intent::{UiAstSlotIrIntentEntryId, UiAstSlotIrIntentModel};
use crate::model::{
    UiAstNodeId, UiAstNodeKind, UiIrNodeId, UiIrNodeKind, UiNodeId, UiNodeKind, UiNodeResolution,
    UiTreeId,
};
use crate::projection::{UiProjectedNodeId, UiProjectedNodeKind, UiProjectionArtifact};
use crate::tree_slot_ast_intent::UiTreeSlotAstIntentEntryId;
use crate::tree_slot_intent::UiTreeSlotCarrierIntentEntryId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiIrSlotProjectionIntentModelId(u64);

impl UiIrSlotProjectionIntentModelId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiIrSlotProjectionIntentEntryId(u64);

impl UiIrSlotProjectionIntentEntryId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    pub fn raw(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiIrSlotProjectionIntentKind {
    ProjectedFragmentLinkedToIrSlotIntent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiIrSlotProjectionIntentState {
    Deferred,
}

#[derive(Debug, Clone)]
pub struct UiIrSlotProjectionIntentEntry {
    id: UiIrSlotProjectionIntentEntryId,
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
    parent_tree_node_id: Option<UiNodeId>,
    child_tree_node_ids: Vec<UiNodeId>,
    parent_ast_node_id: Option<UiAstNodeId>,
    child_ast_node_ids: Vec<UiAstNodeId>,
    parent_ir_node_id: Option<UiIrNodeId>,
    child_ir_node_ids: Vec<UiIrNodeId>,
    parent_projected_node_id: Option<UiProjectedNodeId>,
    child_projected_node_ids: Vec<UiProjectedNodeId>,
    kind: UiIrSlotProjectionIntentKind,
    state: UiIrSlotProjectionIntentState,
}

impl UiIrSlotProjectionIntentEntry {
    pub fn id(&self) -> UiIrSlotProjectionIntentEntryId {
        self.id
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
    pub fn kind(&self) -> UiIrSlotProjectionIntentKind {
        self.kind
    }
    pub fn state(&self) -> UiIrSlotProjectionIntentState {
        self.state
    }
}

#[derive(Debug, Clone)]
pub struct UiIrSlotProjectionIntentModel {
    id: UiIrSlotProjectionIntentModelId,
    entries: Vec<UiIrSlotProjectionIntentEntry>,
}

impl UiIrSlotProjectionIntentModel {
    pub fn id(&self) -> UiIrSlotProjectionIntentModelId {
        self.id
    }
    pub fn entries(&self) -> &[UiIrSlotProjectionIntentEntry] {
        &self.entries
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiIrSlotProjectionIntentDiagnosticKind {
    MissingProjectedNode {
        source_ir_node_id: UiIrNodeId,
    },
    UnexpectedProjectedNodeKind {
        source_ir_node_id: UiIrNodeId,
        projected_node_id: UiProjectedNodeId,
        actual_kind: UiProjectedNodeKind,
    },
}

#[derive(Debug, Clone)]
pub struct UiIrSlotProjectionIntentDiagnostic {
    kind: UiIrSlotProjectionIntentDiagnosticKind,
}

impl UiIrSlotProjectionIntentDiagnostic {
    pub fn kind(&self) -> &UiIrSlotProjectionIntentDiagnosticKind {
        &self.kind
    }
}

#[derive(Debug, Default)]
pub struct UiIrSlotProjectionIntentDiagnostics {
    diagnostics: Vec<UiIrSlotProjectionIntentDiagnostic>,
}

impl UiIrSlotProjectionIntentDiagnostics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push(&mut self, diagnostic: UiIrSlotProjectionIntentDiagnostic) {
        self.diagnostics.push(diagnostic);
    }
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = &UiIrSlotProjectionIntentDiagnostic> {
        self.diagnostics.iter()
    }
}

pub type UiIrSlotProjectionIntentResult =
    Result<UiIrSlotProjectionIntentModel, UiIrSlotProjectionIntentDiagnostics>;

pub fn build_ir_slot_projection_intents(
    ir_intents: &UiAstSlotIrIntentModel,
    projection: &UiProjectionArtifact,
) -> UiIrSlotProjectionIntentResult {
    let mut diagnostics = UiIrSlotProjectionIntentDiagnostics::new();
    let mut entries = Vec::new();

    for ir_intent in ir_intents.entries() {
        let ir_node_id = ir_intent.ir_node_id();

        // Find the matching projected node
        let mut matched_projected_node = None;
        for projected_node in projection.nodes() {
            if projected_node.source_ir_node_id() == Some(ir_node_id) {
                matched_projected_node = Some(projected_node);
                break;
            }
        }

        if let Some(projected_node) = matched_projected_node {
            if projected_node.kind() != UiProjectedNodeKind::Fragment {
                diagnostics.push(UiIrSlotProjectionIntentDiagnostic {
                    kind: UiIrSlotProjectionIntentDiagnosticKind::UnexpectedProjectedNodeKind {
                        source_ir_node_id: ir_node_id,
                        projected_node_id: projected_node.id(),
                        actual_kind: projected_node.kind(),
                    },
                });
            } else {
                let entry_id = UiIrSlotProjectionIntentEntryId::new(ir_node_id.raw());
                let parent_projected_node_id = projected_node.parent();
                let child_projected_node_ids = projected_node.children().to_vec();

                let entry = UiIrSlotProjectionIntentEntry {
                    id: entry_id,
                    source_ir_intent_entry_id: ir_intent.id(),
                    source_ast_intent_entry_id: ir_intent.source_ast_intent_entry_id(),
                    source_tree_intent_entry_id: ir_intent.source_tree_intent_entry_id(),
                    source_tree_id: ir_intent.source_tree_id(),
                    source_tree_node_id: ir_intent.source_tree_node_id(),
                    source_tree_node_kind: ir_intent.source_tree_node_kind(),
                    source_tree_resolution: ir_intent.source_tree_resolution(),
                    source_ast_node_id: ir_intent.source_ast_node_id(),
                    source_ast_node_kind: ir_intent.source_ast_node_kind(),
                    source_ir_node_id: ir_intent.ir_node_id(),
                    source_ir_node_kind: ir_intent.ir_node_kind(),
                    projected_node_id: projected_node.id(),
                    projected_node_kind: projected_node.kind(),
                    parent_tree_node_id: ir_intent.parent_tree_node_id(),
                    child_tree_node_ids: ir_intent.child_tree_node_ids().to_vec(),
                    parent_ast_node_id: ir_intent.parent_ast_node_id(),
                    child_ast_node_ids: ir_intent.child_ast_node_ids().to_vec(),
                    parent_ir_node_id: ir_intent.parent_ir_node_id(),
                    child_ir_node_ids: ir_intent.child_ir_node_ids().to_vec(),
                    parent_projected_node_id,
                    child_projected_node_ids,
                    kind: UiIrSlotProjectionIntentKind::ProjectedFragmentLinkedToIrSlotIntent,
                    state: UiIrSlotProjectionIntentState::Deferred,
                };
                entries.push(entry);
            }
        } else {
            diagnostics.push(UiIrSlotProjectionIntentDiagnostic {
                kind: UiIrSlotProjectionIntentDiagnosticKind::MissingProjectedNode {
                    source_ir_node_id: ir_node_id,
                },
            });
        }
    }

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    Ok(UiIrSlotProjectionIntentModel {
        id: UiIrSlotProjectionIntentModelId::new(1),
        entries,
    })
}
