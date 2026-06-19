use crate::model::{
    UiAstNodeId, UiAstNodeKind, UiIr, UiIrNodeId, UiIrNodeKind, UiNodeId, UiNodeKind,
    UiNodeResolution, UiTreeId,
};
use crate::tree_slot_ast_intent::{UiTreeSlotAstIntentEntryId, UiTreeSlotAstIntentModel};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiAstSlotIrIntentModelId(u64);

impl UiAstSlotIrIntentModelId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiAstSlotIrIntentEntryId(u64);

impl UiAstSlotIrIntentEntryId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    pub fn raw(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiAstSlotIrIntentKind {
    IrFragmentLinkedToAstSlotIntent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiAstSlotIrIntentState {
    Deferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiAstSlotIrIntentEntry {
    id: UiAstSlotIrIntentEntryId,
    source_ast_intent_entry_id: UiTreeSlotAstIntentEntryId,
    source_tree_intent_entry_id: crate::tree_slot_intent::UiTreeSlotCarrierIntentEntryId,
    source_tree_id: UiTreeId,
    source_tree_node_id: UiNodeId,
    source_tree_node_kind: UiNodeKind,
    source_tree_resolution: UiNodeResolution,
    source_ast_node_id: UiAstNodeId,
    source_ast_node_kind: UiAstNodeKind,
    ir_node_id: UiIrNodeId,
    ir_node_kind: UiIrNodeKind,
    parent_tree_node_id: Option<UiNodeId>,
    child_tree_node_ids: Vec<UiNodeId>,
    parent_ast_node_id: Option<UiAstNodeId>,
    child_ast_node_ids: Vec<UiAstNodeId>,
    parent_ir_node_id: Option<UiIrNodeId>,
    child_ir_node_ids: Vec<UiIrNodeId>,
    kind: UiAstSlotIrIntentKind,
    state: UiAstSlotIrIntentState,
}

impl UiAstSlotIrIntentEntry {
    pub fn id(&self) -> UiAstSlotIrIntentEntryId {
        self.id
    }
    pub fn source_ast_intent_entry_id(&self) -> UiTreeSlotAstIntentEntryId {
        self.source_ast_intent_entry_id
    }
    pub fn source_tree_intent_entry_id(
        &self,
    ) -> crate::tree_slot_intent::UiTreeSlotCarrierIntentEntryId {
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
    pub fn ir_node_id(&self) -> UiIrNodeId {
        self.ir_node_id
    }
    pub fn ir_node_kind(&self) -> UiIrNodeKind {
        self.ir_node_kind
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
    pub fn kind(&self) -> UiAstSlotIrIntentKind {
        self.kind
    }
    pub fn state(&self) -> UiAstSlotIrIntentState {
        self.state
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiAstSlotIrIntentModel {
    id: UiAstSlotIrIntentModelId,
    entries: Vec<UiAstSlotIrIntentEntry>,
}

impl UiAstSlotIrIntentModel {
    pub fn new(id: UiAstSlotIrIntentModelId) -> Self {
        Self {
            id,
            entries: Vec::new(),
        }
    }
    pub fn id(&self) -> UiAstSlotIrIntentModelId {
        self.id
    }
    pub fn entries(&self) -> &[UiAstSlotIrIntentEntry] {
        &self.entries
    }
    pub fn push_entry(&mut self, entry: UiAstSlotIrIntentEntry) {
        self.entries.push(entry);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiAstSlotIrIntentDiagnosticKind {
    MissingIrNode {
        source_ast_node_id: UiAstNodeId,
    },
    UnexpectedIrNodeKind {
        source_ast_node_id: UiAstNodeId,
        ir_node_id: UiIrNodeId,
        actual_kind: UiIrNodeKind,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiAstSlotIrIntentDiagnostic {
    kind: UiAstSlotIrIntentDiagnosticKind,
    message: String,
}

impl UiAstSlotIrIntentDiagnostic {
    pub fn new(kind: UiAstSlotIrIntentDiagnosticKind, message: String) -> Self {
        Self { kind, message }
    }
    pub fn kind(&self) -> &UiAstSlotIrIntentDiagnosticKind {
        &self.kind
    }
    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiAstSlotIrIntentDiagnostics {
    diagnostics: Vec<UiAstSlotIrIntentDiagnostic>,
}

impl UiAstSlotIrIntentDiagnostics {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }
    pub fn push(&mut self, diagnostic: UiAstSlotIrIntentDiagnostic) {
        self.diagnostics.push(diagnostic);
    }
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
    pub fn diagnostics(&self) -> &[UiAstSlotIrIntentDiagnostic] {
        &self.diagnostics
    }
}

impl Default for UiAstSlotIrIntentDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

pub type UiAstSlotIrIntentResult = Result<UiAstSlotIrIntentModel, UiAstSlotIrIntentDiagnostics>;

pub fn build_ast_slot_ir_intents(
    ast_intents: &UiTreeSlotAstIntentModel,
    ir: &UiIr,
) -> UiAstSlotIrIntentResult {
    let mut model = UiAstSlotIrIntentModel::new(UiAstSlotIrIntentModelId::new(1));
    let mut diagnostics = UiAstSlotIrIntentDiagnostics::new();

    for ast_entry in ast_intents.entries() {
        let raw_id = ast_entry.ast_node_id().raw();
        let target_ir_node_id = UiIrNodeId::new(raw_id);

        let matching_ir_node = ir.nodes().iter().find(|n| n.id() == target_ir_node_id);

        if let Some(ir_node) = matching_ir_node {
            if ir_node.kind() == UiIrNodeKind::Fragment {
                let parent_ir_node_id = ast_entry
                    .parent_ast_node_id()
                    .map(|id| UiIrNodeId::new(id.raw()));

                let child_ir_node_ids = ast_entry
                    .child_ast_node_ids()
                    .iter()
                    .map(|id| UiIrNodeId::new(id.raw()))
                    .collect();

                let entry = UiAstSlotIrIntentEntry {
                    id: UiAstSlotIrIntentEntryId::new(raw_id),
                    source_ast_intent_entry_id: ast_entry.id(),
                    source_tree_intent_entry_id: ast_entry.source_tree_intent_entry_id(),
                    source_tree_id: ast_entry.source_tree_id(),
                    source_tree_node_id: ast_entry.source_tree_node_id(),
                    source_tree_node_kind: ast_entry.source_tree_node_kind(),
                    source_tree_resolution: ast_entry.source_tree_resolution(),
                    source_ast_node_id: ast_entry.ast_node_id(),
                    source_ast_node_kind: ast_entry.ast_node_kind(),
                    ir_node_id: ir_node.id(),
                    ir_node_kind: ir_node.kind(),
                    parent_tree_node_id: ast_entry.parent_tree_node_id(),
                    child_tree_node_ids: ast_entry.child_tree_node_ids().to_vec(),
                    parent_ast_node_id: ast_entry.parent_ast_node_id(),
                    child_ast_node_ids: ast_entry.child_ast_node_ids().to_vec(),
                    parent_ir_node_id,
                    child_ir_node_ids,
                    kind: UiAstSlotIrIntentKind::IrFragmentLinkedToAstSlotIntent,
                    state: UiAstSlotIrIntentState::Deferred,
                };
                model.push_entry(entry);
            } else {
                diagnostics.push(UiAstSlotIrIntentDiagnostic::new(
                    UiAstSlotIrIntentDiagnosticKind::UnexpectedIrNodeKind {
                        source_ast_node_id: ast_entry.ast_node_id(),
                        ir_node_id: ir_node.id(),
                        actual_kind: ir_node.kind(),
                    },
                    "IR node kind is not Fragment".to_string(),
                ));
            }
        } else {
            diagnostics.push(UiAstSlotIrIntentDiagnostic::new(
                UiAstSlotIrIntentDiagnosticKind::MissingIrNode {
                    source_ast_node_id: ast_entry.ast_node_id(),
                },
                "IR node matching AST node not found".to_string(),
            ));
        }
    }

    if !diagnostics.is_empty() {
        Err(diagnostics)
    } else {
        Ok(model)
    }
}
