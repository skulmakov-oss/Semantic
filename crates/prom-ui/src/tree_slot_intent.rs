use crate::model::{UiNodeId, UiNodeKind, UiNodeResolution, UiTree, UiTreeId};
use crate::validation::{validate_tree, UiTreeValidationConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiTreeSlotCarrierIntentModelId(u64);

impl UiTreeSlotCarrierIntentModelId {
    pub fn new(raw: u64) -> Self {
        Self(raw)
    }
    pub fn raw(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiTreeSlotCarrierIntentEntryId(u64);

impl UiTreeSlotCarrierIntentEntryId {
    pub fn new(raw: u64) -> Self {
        Self(raw)
    }
    pub fn raw(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiTreeSlotCarrierIntentKind {
    StructuralSlotBoundary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiTreeSlotCarrierIntentState {
    Deferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTreeSlotCarrierIntentEntry {
    id: UiTreeSlotCarrierIntentEntryId,
    source_tree_id: UiTreeId,
    source_node_id: UiNodeId,
    source_node_kind: UiNodeKind,
    source_resolution: UiNodeResolution,
    parent: Option<UiNodeId>,
    children: Vec<UiNodeId>,
    kind: UiTreeSlotCarrierIntentKind,
    state: UiTreeSlotCarrierIntentState,
}

impl UiTreeSlotCarrierIntentEntry {
    pub fn id(&self) -> UiTreeSlotCarrierIntentEntryId {
        self.id
    }
    pub fn source_tree_id(&self) -> UiTreeId {
        self.source_tree_id
    }
    pub fn source_node_id(&self) -> UiNodeId {
        self.source_node_id
    }
    pub fn source_node_kind(&self) -> UiNodeKind {
        self.source_node_kind
    }
    pub fn source_resolution(&self) -> UiNodeResolution {
        self.source_resolution
    }
    pub fn parent(&self) -> Option<UiNodeId> {
        self.parent
    }
    pub fn children(&self) -> &[UiNodeId] {
        &self.children
    }
    pub fn kind(&self) -> UiTreeSlotCarrierIntentKind {
        self.kind
    }
    pub fn state(&self) -> UiTreeSlotCarrierIntentState {
        self.state
    }
}

#[derive(Debug, Clone)]
pub struct UiTreeSlotCarrierIntentModel {
    id: UiTreeSlotCarrierIntentModelId,
    entries: Vec<UiTreeSlotCarrierIntentEntry>,
}

impl UiTreeSlotCarrierIntentModel {
    pub fn id(&self) -> UiTreeSlotCarrierIntentModelId {
        self.id
    }
    pub fn entries(&self) -> &[UiTreeSlotCarrierIntentEntry] {
        &self.entries
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiTreeSlotCarrierIntentDiagnosticKind {
    TreeValidationFailed,
}

#[derive(Debug, Clone)]
pub struct UiTreeSlotCarrierIntentDiagnostic {
    kind: UiTreeSlotCarrierIntentDiagnosticKind,
    message: String,
}

impl UiTreeSlotCarrierIntentDiagnostic {
    pub fn kind(&self) -> UiTreeSlotCarrierIntentDiagnosticKind {
        self.kind.clone()
    }
    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone)]
pub struct UiTreeSlotCarrierIntentDiagnostics {
    diagnostics: Vec<UiTreeSlotCarrierIntentDiagnostic>,
}

impl UiTreeSlotCarrierIntentDiagnostics {
    pub fn diagnostics(&self) -> &[UiTreeSlotCarrierIntentDiagnostic] {
        &self.diagnostics
    }
}

pub type UiTreeSlotCarrierIntentResult =
    Result<UiTreeSlotCarrierIntentModel, UiTreeSlotCarrierIntentDiagnostics>;

pub fn build_tree_slot_carrier_intents(tree: &UiTree) -> UiTreeSlotCarrierIntentResult {
    // 1. Validate tree
    if let Err(errs) = validate_tree(tree, &UiTreeValidationConfig::default()) {
        let diag = UiTreeSlotCarrierIntentDiagnostic {
            kind: UiTreeSlotCarrierIntentDiagnosticKind::TreeValidationFailed,
            message: format!(
                "Tree validation failed: {} error(s)",
                errs.diagnostics().len()
            ),
        };
        return Err(UiTreeSlotCarrierIntentDiagnostics {
            diagnostics: vec![diag],
        });
    }

    // 2. Build intents
    let mut entries = Vec::new();
    for node in tree.nodes() {
        if node.kind() == UiNodeKind::Slot {
            let entry = UiTreeSlotCarrierIntentEntry {
                id: UiTreeSlotCarrierIntentEntryId::new(node.id().raw()),
                source_tree_id: tree.id(),
                source_node_id: node.id(),
                source_node_kind: node.kind(),
                source_resolution: node.resolution(),
                parent: node.parent(),
                children: node.children().to_vec(),
                kind: UiTreeSlotCarrierIntentKind::StructuralSlotBoundary,
                state: UiTreeSlotCarrierIntentState::Deferred,
            };
            entries.push(entry);
        }
    }

    Ok(UiTreeSlotCarrierIntentModel {
        id: UiTreeSlotCarrierIntentModelId::new(tree.id().raw()),
        entries,
    })
}
