//! Minimal Semantic UI Tree to AST structural bridge.
//!
//! This module provides a deterministic, structural, and inert mapping from
//! `UiTree` to `UiAst`. It does not execute actions, does not lower semantics,
//! does not project models, and does not admit capabilities.

use alloc::vec::Vec;

use crate::model::{UiAst, UiAstNode, UiAstNodeId, UiNodeId, UiNodeKind, UiTree};
use crate::validation::{validate_tree, UiTreeValidationConfig};

/// Structured diagnostic kind for the inert Tree to AST bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiTreeToAstDiagnosticKind {
    /// The input tree failed local structural validation.
    InvalidTree,
    /// The tree node kind is currently unsupported in the AST bridge.
    UnsupportedTreeNodeKind { node_id: UiNodeId, kind: UiNodeKind },
}

/// Structured diagnostic for a Tree to AST bridging failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiTreeToAstDiagnostic {
    kind: UiTreeToAstDiagnosticKind,
}

impl UiTreeToAstDiagnostic {
    /// Creates a bridge diagnostic.
    pub const fn new(kind: UiTreeToAstDiagnosticKind) -> Self {
        Self { kind }
    }

    /// Returns the diagnostic kind.
    pub const fn kind(&self) -> UiTreeToAstDiagnosticKind {
        self.kind
    }
}

/// Collection of structured Tree to AST bridging diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct UiTreeToAstDiagnostics {
    diagnostics: Vec<UiTreeToAstDiagnostic>,
}

impl UiTreeToAstDiagnostics {
    /// Creates a diagnostics collection from a vector.
    pub fn new(diagnostics: Vec<UiTreeToAstDiagnostic>) -> Self {
        Self { diagnostics }
    }

    /// Returns whether there are no diagnostics.
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Returns the number of diagnostics.
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Returns the diagnostics as a slice.
    pub fn diagnostics(&self) -> &[UiTreeToAstDiagnostic] {
        &self.diagnostics
    }

    /// Returns an iterator over the diagnostics.
    pub fn iter(&self) -> core::slice::Iter<'_, UiTreeToAstDiagnostic> {
        self.diagnostics.iter()
    }

    /// Converts the collection into a vector.
    pub fn into_vec(self) -> Vec<UiTreeToAstDiagnostic> {
        self.diagnostics
    }
}

/// Result of the minimal Tree to AST bridge.
pub type UiTreeToAstResult = Result<UiAst, UiTreeToAstDiagnostics>;

/// Bridges a validated inert Semantic UI tree to an inert UI AST.
///
/// This function calls `validate_tree` before bridging. If validation fails,
/// it returns an `InvalidTree` diagnostic without a partial AST.
///
/// Resolution metadata (`Known`, `Unknown`, `Conflict`) is ignored during
/// bridging and does not block structural mapping.
pub fn tree_to_ast(tree: &UiTree) -> UiTreeToAstResult {
    let validation_result = validate_tree(tree, &UiTreeValidationConfig::default());
    if validation_result.is_err() {
        return Err(UiTreeToAstDiagnostics::new(alloc::vec![
            UiTreeToAstDiagnostic::new(UiTreeToAstDiagnosticKind::InvalidTree)
        ]));
    }

    let diagnostics = Vec::new();
    let mut ast_nodes = Vec::with_capacity(tree.len());

    for node in tree.nodes() {
        let ast_kind = match node.kind() {
            UiNodeKind::Root => crate::model::UiAstNodeKind::Root,
            UiNodeKind::Element => crate::model::UiAstNodeKind::Element,
            UiNodeKind::Text => crate::model::UiAstNodeKind::Text,
            UiNodeKind::Fragment => crate::model::UiAstNodeKind::Fragment,
            UiNodeKind::Slot => crate::model::UiAstNodeKind::Fragment,
        };

        let ast_id = UiAstNodeId::new(node.id().raw());
        let mut ast_node = UiAstNode::new(ast_id, ast_kind);

        if let Some(parent_id) = node.parent() {
            ast_node.set_parent(Some(UiAstNodeId::new(parent_id.raw())));
        }

        for &child_id in node.children() {
            ast_node.push_child(UiAstNodeId::new(child_id.raw()));
        }

        ast_nodes.push(ast_node);
    }

    if !diagnostics.is_empty() {
        return Err(UiTreeToAstDiagnostics::new(diagnostics));
    }

    let mut ast = UiAst::new();
    for node in ast_nodes {
        ast.push_node(node);
    }

    Ok(ast)
}
