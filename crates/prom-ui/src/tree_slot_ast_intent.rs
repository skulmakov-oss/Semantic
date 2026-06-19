use crate::model::{
    UiAst, UiAstNodeId, UiAstNodeKind, UiNodeId, UiNodeKind, UiNodeResolution, UiTreeId,
};
use crate::tree_slot_intent::{UiTreeSlotCarrierIntentEntryId, UiTreeSlotCarrierIntentModel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiTreeSlotAstIntentModelId(pub u64);

impl UiTreeSlotAstIntentModelId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiTreeSlotAstIntentEntryId(pub u64);

impl UiTreeSlotAstIntentEntryId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiTreeSlotAstIntentKind {
    AstFragmentLinkedToTreeSlotIntent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiTreeSlotAstIntentState {
    Deferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTreeSlotAstIntentEntry {
    id: UiTreeSlotAstIntentEntryId,

    source_tree_intent_entry_id: UiTreeSlotCarrierIntentEntryId,
    source_tree_id: UiTreeId,
    source_tree_node_id: UiNodeId,
    source_tree_node_kind: UiNodeKind,
    source_tree_resolution: UiNodeResolution,

    ast_node_id: UiAstNodeId,
    ast_node_kind: UiAstNodeKind,

    parent_tree_node_id: Option<UiNodeId>,
    child_tree_node_ids: Vec<UiNodeId>,

    parent_ast_node_id: Option<UiAstNodeId>,
    child_ast_node_ids: Vec<UiAstNodeId>,

    kind: UiTreeSlotAstIntentKind,
    state: UiTreeSlotAstIntentState,
}

impl UiTreeSlotAstIntentEntry {
    pub fn id(&self) -> UiTreeSlotAstIntentEntryId {
        self.id
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

    pub fn ast_node_id(&self) -> UiAstNodeId {
        self.ast_node_id
    }

    pub fn ast_node_kind(&self) -> UiAstNodeKind {
        self.ast_node_kind
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

    pub fn kind(&self) -> UiTreeSlotAstIntentKind {
        self.kind
    }

    pub fn state(&self) -> UiTreeSlotAstIntentState {
        self.state
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTreeSlotAstIntentModel {
    id: UiTreeSlotAstIntentModelId,
    entries: Vec<UiTreeSlotAstIntentEntry>,
}

impl UiTreeSlotAstIntentModel {
    pub fn new(id: UiTreeSlotAstIntentModelId) -> Self {
        Self {
            id,
            entries: Vec::new(),
        }
    }

    pub fn id(&self) -> UiTreeSlotAstIntentModelId {
        self.id
    }

    pub fn entries(&self) -> &[UiTreeSlotAstIntentEntry] {
        &self.entries
    }

    pub fn push_entry(&mut self, entry: UiTreeSlotAstIntentEntry) {
        self.entries.push(entry);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiTreeSlotAstIntentDiagnosticKind {
    MissingAstNode {
        source_tree_node_id: UiNodeId,
    },
    UnexpectedAstNodeKind {
        source_tree_node_id: UiNodeId,
        ast_node_id: UiAstNodeId,
        actual_kind: UiAstNodeKind,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTreeSlotAstIntentDiagnostic {
    kind: UiTreeSlotAstIntentDiagnosticKind,
    message: String,
}

impl UiTreeSlotAstIntentDiagnostic {
    pub fn new(kind: UiTreeSlotAstIntentDiagnosticKind, message: String) -> Self {
        Self { kind, message }
    }

    pub fn kind(&self) -> &UiTreeSlotAstIntentDiagnosticKind {
        &self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiTreeSlotAstIntentDiagnostics {
    diagnostics: Vec<UiTreeSlotAstIntentDiagnostic>,
}

impl UiTreeSlotAstIntentDiagnostics {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn push(&mut self, diagnostic: UiTreeSlotAstIntentDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn diagnostics(&self) -> &[UiTreeSlotAstIntentDiagnostic] {
        &self.diagnostics
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

pub type UiTreeSlotAstIntentResult =
    Result<UiTreeSlotAstIntentModel, UiTreeSlotAstIntentDiagnostics>;

pub fn build_tree_slot_ast_intents(
    tree_intents: &UiTreeSlotCarrierIntentModel,
    ast: &UiAst,
) -> UiTreeSlotAstIntentResult {
    let mut model =
        UiTreeSlotAstIntentModel::new(UiTreeSlotAstIntentModelId::new(tree_intents.id().raw()));
    let mut diagnostics = UiTreeSlotAstIntentDiagnostics::new();

    for tree_entry in tree_intents.entries() {
        let raw_id = tree_entry.source_node_id().raw();
        let target_ast_id = UiAstNodeId::new(raw_id);

        let mut matching_ast_node = None;
        for ast_node in ast.nodes() {
            if ast_node.id().raw() == raw_id {
                matching_ast_node = Some(ast_node);
                break;
            }
        }

        if let Some(ast_node) = matching_ast_node {
            if ast_node.kind() != UiAstNodeKind::Fragment {
                diagnostics.push(UiTreeSlotAstIntentDiagnostic::new(
                    UiTreeSlotAstIntentDiagnosticKind::UnexpectedAstNodeKind {
                        source_tree_node_id: tree_entry.source_node_id(),
                        ast_node_id: target_ast_id,
                        actual_kind: ast_node.kind(),
                    },
                    "AST node matching Tree Slot intent is not a Fragment".to_string(),
                ));
            } else {
                let parent_ast_node_id = tree_entry.parent().map(|p| UiAstNodeId::new(p.raw()));
                let child_ast_node_ids = tree_entry
                    .children()
                    .iter()
                    .map(|c| UiAstNodeId::new(c.raw()))
                    .collect();

                let new_entry = UiTreeSlotAstIntentEntry {
                    id: UiTreeSlotAstIntentEntryId::new(raw_id),
                    source_tree_intent_entry_id: tree_entry.id(),
                    source_tree_id: tree_entry.source_tree_id(),
                    source_tree_node_id: tree_entry.source_node_id(),
                    source_tree_node_kind: tree_entry.source_node_kind(),
                    source_tree_resolution: tree_entry.source_resolution(),
                    ast_node_id: target_ast_id,
                    ast_node_kind: ast_node.kind(),
                    parent_tree_node_id: tree_entry.parent(),
                    child_tree_node_ids: tree_entry.children().to_vec(),
                    parent_ast_node_id,
                    child_ast_node_ids,
                    kind: UiTreeSlotAstIntentKind::AstFragmentLinkedToTreeSlotIntent,
                    state: UiTreeSlotAstIntentState::Deferred,
                };
                model.push_entry(new_entry);
            }
        } else {
            diagnostics.push(UiTreeSlotAstIntentDiagnostic::new(
                UiTreeSlotAstIntentDiagnosticKind::MissingAstNode {
                    source_tree_node_id: tree_entry.source_node_id(),
                },
                "No AST node found matching Tree Slot intent".to_string(),
            ));
        }
    }

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    Ok(model)
}
