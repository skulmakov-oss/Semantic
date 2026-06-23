//! Minimal Semantic UI AST→IR lowering seed.
//!
//! This module is a minimal Semantic UI AST→IR lowering seed.
//! It does not parse.
//! It does not verify.
//! It does not execute.
//! It does not render.
//! It does not calculate layout.
//! It does not handle events.
//! It does not enforce capabilities.
//! It does not call runtime, verifier, VM, renderer, or backend code.
//! It does not create semantic truth.

use alloc::vec::Vec;

use crate::model::{UiAst, UiAstNodeId, UiAstNodeKind, UiIr, UiIrNode, UiIrNodeId, UiIrNodeKind};

/// Inert configuration placeholder for the minimal AST→IR lowering seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct UiLoweringConfig;

/// Structured diagnostic kind for the minimal AST→IR lowering seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiLoweringDiagnosticKind {
    /// The AST node kind is not yet supported by the minimal lowering seed.
    UnsupportedAstNodeKind(UiAstNodeKind),
}

/// Structured diagnostic for a lowering failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiLoweringDiagnostic {
    ast_node_id: UiAstNodeId,
    kind: UiLoweringDiagnosticKind,
}

impl UiLoweringDiagnostic {
    /// Creates a lowering diagnostic for a specific AST node.
    pub const fn new(ast_node_id: UiAstNodeId, kind: UiLoweringDiagnosticKind) -> Self {
        Self { ast_node_id, kind }
    }

    /// Returns the AST node id associated with the diagnostic.
    pub const fn ast_node_id(&self) -> UiAstNodeId {
        self.ast_node_id
    }

    /// Returns the diagnostic kind.
    pub const fn kind(&self) -> UiLoweringDiagnosticKind {
        self.kind
    }
}

/// Collection of structured lowering diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct UiLoweringDiagnostics {
    diagnostics: Vec<UiLoweringDiagnostic>,
}

impl UiLoweringDiagnostics {
    /// Creates a diagnostics collection from a vector.
    pub fn new(diagnostics: Vec<UiLoweringDiagnostic>) -> Self {
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
    pub fn diagnostics(&self) -> &[UiLoweringDiagnostic] {
        &self.diagnostics
    }
}

/// Result of the minimal AST→IR lowering seed.
pub type UiLoweringResult = Result<UiIr, UiLoweringDiagnostics>;

/// Lowers a minimal inert Semantic UI AST into inert Semantic UI IR.
///
/// Supported AST kinds:
/// - Root
/// - Element
/// - Text
/// - Fragment
/// - Attribute
/// - Action
///
/// Unsupported AST kinds return diagnostics:
/// - Binding
pub fn lower_ast_to_ir(ast: &UiAst, config: &UiLoweringConfig) -> UiLoweringResult {
    let _ = config;

    let mut diagnostics = Vec::new();
    for node in ast.nodes() {
        if let Some(diagnostic) = unsupported_node_diagnostic(node.id(), node.kind()) {
            diagnostics.push(diagnostic);
        }
    }

    if !diagnostics.is_empty() {
        return Err(UiLoweringDiagnostics::new(diagnostics));
    }

    let mut ir = UiIr::new();
    for node in ast.nodes() {
        let ir_kind = map_supported_kind(node.kind());
        let mut ir_node = match node.parent() {
            Some(parent) => UiIrNode::with_parent(
                UiIrNodeId::new(node.id().raw()),
                ir_kind,
                UiIrNodeId::new(parent.raw()),
            ),
            None => UiIrNode::new(UiIrNodeId::new(node.id().raw()), ir_kind),
        };
        ir_node.set_source_ast_node_id(Some(node.id()));

        for child in node.children() {
            ir_node.push_child(UiIrNodeId::new(child.raw()));
        }

        ir.push_node(ir_node);
    }

    Ok(ir)
}

const fn map_supported_kind(kind: UiAstNodeKind) -> UiIrNodeKind {
    match kind {
        UiAstNodeKind::Root => UiIrNodeKind::Root,
        UiAstNodeKind::Element => UiIrNodeKind::Element,
        UiAstNodeKind::Text => UiIrNodeKind::Text,
        UiAstNodeKind::Fragment => UiIrNodeKind::Fragment,
        UiAstNodeKind::Attribute => UiIrNodeKind::Property,
        UiAstNodeKind::Action => UiIrNodeKind::Action,
        UiAstNodeKind::Binding => UiIrNodeKind::Root, // unreachable after unsupported diagnostics
    }
}

const fn unsupported_node_diagnostic(
    ast_node_id: UiAstNodeId,
    kind: UiAstNodeKind,
) -> Option<UiLoweringDiagnostic> {
    match kind {
        UiAstNodeKind::Binding => Some(UiLoweringDiagnostic::new(
            ast_node_id,
            UiLoweringDiagnosticKind::UnsupportedAstNodeKind(kind),
        )),
        UiAstNodeKind::Root
        | UiAstNodeKind::Element
        | UiAstNodeKind::Text
        | UiAstNodeKind::Fragment
        | UiAstNodeKind::Attribute
        | UiAstNodeKind::Action => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn supported_ast() -> UiAst {
        let mut ast = UiAst::new();
        let mut root = crate::model::UiAstNode::new(UiAstNodeId::new(1), UiAstNodeKind::Root);
        root.push_child(UiAstNodeId::new(2));
        ast.push_node(root);
        ast.push_node(crate::model::UiAstNode::with_parent(
            UiAstNodeId::new(2),
            UiAstNodeKind::Element,
            UiAstNodeId::new(1),
        ));
        ast.push_node(crate::model::UiAstNode::new(
            UiAstNodeId::new(3),
            UiAstNodeKind::Text,
        ));
        ast.push_node(crate::model::UiAstNode::new(
            UiAstNodeId::new(4),
            UiAstNodeKind::Fragment,
        ));
        ast
    }

    #[test]
    fn empty_ast_lowers_to_empty_ir() {
        let ast = UiAst::new();

        let lowered = lower_ast_to_ir(&ast, &UiLoweringConfig);

        assert_eq!(lowered, Ok(UiIr::new()));
    }

    #[test]
    fn root_lowers_deterministically() {
        let mut ast = UiAst::new();
        ast.push_node(crate::model::UiAstNode::new(
            UiAstNodeId::new(10),
            UiAstNodeKind::Root,
        ));

        let lowered_one = lower_ast_to_ir(&ast, &UiLoweringConfig);
        let lowered_two = lower_ast_to_ir(&ast, &UiLoweringConfig);

        assert_eq!(lowered_one, lowered_two);
    }

    #[test]
    fn root_element_text_fragment_lower_to_matching_ir_kinds() {
        let ast = supported_ast();

        let lowered = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("supported subset");

        assert_eq!(lowered.len(), 4);
        assert_eq!(lowered.nodes()[0].kind(), UiIrNodeKind::Root);
        assert_eq!(lowered.nodes()[1].kind(), UiIrNodeKind::Element);
        assert_eq!(lowered.nodes()[2].kind(), UiIrNodeKind::Text);
        assert_eq!(lowered.nodes()[3].kind(), UiIrNodeKind::Fragment);
    }

    #[test]
    fn parent_and_children_handles_are_mapped_by_raw_value() {
        let mut ast = UiAst::new();
        let mut node = crate::model::UiAstNode::with_parent(
            UiAstNodeId::new(20),
            UiAstNodeKind::Element,
            UiAstNodeId::new(19),
        );
        node.push_child(UiAstNodeId::new(21));
        node.push_child(UiAstNodeId::new(22));
        ast.push_node(node);

        let lowered = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("supported subset");
        let ir_node = &lowered.nodes()[0];

        assert_eq!(ir_node.id(), UiIrNodeId::new(20));
        assert_eq!(ir_node.parent(), Some(UiIrNodeId::new(19)));
        assert_eq!(
            ir_node.children(),
            &[UiIrNodeId::new(21), UiIrNodeId::new(22)]
        );
    }

    #[test]
    fn attribute_lowers_to_ir_property() {
        let mut ast = UiAst::new();
        ast.push_node(crate::model::UiAstNode::new(
            UiAstNodeId::new(30),
            UiAstNodeKind::Attribute,
        ));

        let lowered = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("supported kind");

        assert_eq!(lowered.len(), 1);
        assert_eq!(lowered.nodes()[0].id(), UiIrNodeId::new(30));
        assert_eq!(lowered.nodes()[0].kind(), UiIrNodeKind::Property);
    }

    #[test]
    fn unsupported_binding_returns_diagnostic() {
        let mut ast = UiAst::new();
        ast.push_node(crate::model::UiAstNode::new(
            UiAstNodeId::new(31),
            UiAstNodeKind::Binding,
        ));

        let err = lower_ast_to_ir(&ast, &UiLoweringConfig).expect_err("unsupported kind");

        assert_eq!(err.len(), 1);
        assert_eq!(err.diagnostics()[0].ast_node_id(), UiAstNodeId::new(31));
        assert_eq!(
            err.diagnostics()[0].kind(),
            UiLoweringDiagnosticKind::UnsupportedAstNodeKind(UiAstNodeKind::Binding)
        );
    }

    #[test]
    fn action_lowers_to_ir_action() {
        let mut ast = UiAst::new();
        ast.push_node(crate::model::UiAstNode::new(
            UiAstNodeId::new(32),
            UiAstNodeKind::Action,
        ));

        let lowered = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("supported kind");

        assert_eq!(lowered.len(), 1);
        assert_eq!(lowered.nodes()[0].id(), UiIrNodeId::new(32));
        assert_eq!(lowered.nodes()[0].kind(), UiIrNodeKind::Action);
    }

    #[test]
    fn unsupported_nodes_collect_all_diagnostics() {
        let mut ast = UiAst::new();
        ast.push_node(crate::model::UiAstNode::new(
            UiAstNodeId::new(41),
            UiAstNodeKind::Binding,
        ));

        let err = lower_ast_to_ir(&ast, &UiLoweringConfig).expect_err("unsupported kinds");

        assert_eq!(err.len(), 1);
        assert_eq!(err.diagnostics()[0].ast_node_id(), UiAstNodeId::new(41));
    }

    #[test]
    fn unsupported_nodes_do_not_return_partial_ir() {
        let mut ast = supported_ast();
        ast.push_node(crate::model::UiAstNode::new(
            UiAstNodeId::new(43),
            UiAstNodeKind::Binding,
        ));

        assert!(lower_ast_to_ir(&ast, &UiLoweringConfig).is_err());
    }

    #[test]
    fn effect_boundary_is_not_generated_by_minimal_lowering() {
        let ast = supported_ast();
        let lowered = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("supported subset");

        assert!(lowered
            .nodes()
            .iter()
            .all(|node| node.kind() != UiIrNodeKind::EffectBoundary));
    }

    #[test]
    fn property_and_ir_action_are_generated_by_carrier_lowering() {
        let mut ast = supported_ast();
        ast.push_node(crate::model::UiAstNode::new(
            UiAstNodeId::new(44),
            UiAstNodeKind::Attribute,
        ));
        ast.push_node(crate::model::UiAstNode::new(
            UiAstNodeId::new(45),
            UiAstNodeKind::Action,
        ));
        let lowered = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("supported subset");

        assert!(lowered
            .nodes()
            .iter()
            .any(|node| node.kind() == UiIrNodeKind::Property));
        assert!(lowered
            .nodes()
            .iter()
            .any(|node| node.kind() == UiIrNodeKind::Action));
    }

    #[test]
    fn lowering_does_not_require_conversion_traits() {
        let ast = supported_ast();

        let lowered = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("supported subset");

        assert_eq!(lowered.len(), 4);
    }

    #[test]
    fn lowering_config_is_inert() {
        let config = UiLoweringConfig;

        assert_eq!(config, UiLoweringConfig);
    }

    #[test]
    fn diagnostics_are_structural_and_do_not_panic() {
        let mut ast = UiAst::new();
        ast.push_node(crate::model::UiAstNode::new(
            UiAstNodeId::new(50),
            UiAstNodeKind::Binding,
        ));

        let err = lower_ast_to_ir(&ast, &UiLoweringConfig).expect_err("unsupported kind");

        assert_eq!(err.len(), 1);
        assert_eq!(err.diagnostics()[0].ast_node_id(), UiAstNodeId::new(50));
        assert_eq!(
            err.diagnostics()[0].kind(),
            UiLoweringDiagnosticKind::UnsupportedAstNodeKind(UiAstNodeKind::Binding)
        );
    }
}
