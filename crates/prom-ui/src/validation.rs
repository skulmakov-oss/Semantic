//! Minimal Semantic UI AST structural validation seed.
//!
//! This module is a minimal Semantic UI AST structural validation seed.
//! It does not parse.
//! It does not verify Semantic source.
//! It does not type-check.
//! It does not resolve names.
//! It does not perform effect admission.
//! It does not execute.
//! It does not render.
//! It does not calculate layout.
//! It does not handle events.
//! It does not enforce capabilities.
//! It does not call lowering.
//! It does not call parser, verifier, VM, runtime, renderer, or backend code.
//! It does not create semantic truth.

use alloc::vec::Vec;

use crate::model::{UiAst, UiAstNode, UiAstNodeId, UiAstNodeKind};

/// Inert configuration placeholder for the minimal AST validation seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct UiAstValidationConfig;

/// Structured diagnostic kind for the minimal AST validation seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiAstValidationDiagnosticKind {
    /// A node identifier appears more than once in the AST.
    DuplicateNodeId(UiAstNodeId),
    /// A node references a missing parent node.
    MissingParentTarget {
        node_id: UiAstNodeId,
        parent_id: UiAstNodeId,
    },
    /// A node references a missing child node.
    MissingChildTarget {
        node_id: UiAstNodeId,
        child_id: UiAstNodeId,
    },
    /// The parent/child relationship is inconsistent in the AST.
    InconsistentParentChild {
        parent_id: UiAstNodeId,
        child_id: UiAstNodeId,
    },
    /// A node names itself as its own parent.
    SelfParent(UiAstNodeId),
    /// A node names itself as one of its children.
    SelfChild(UiAstNodeId),
    /// More than one root node exists in the AST.
    MultipleRoots,
}

/// Structured diagnostic for a validation failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiAstValidationDiagnostic {
    node_id: Option<UiAstNodeId>,
    kind: UiAstValidationDiagnosticKind,
}

impl UiAstValidationDiagnostic {
    /// Creates a validation diagnostic for a specific AST node.
    pub const fn new(node_id: Option<UiAstNodeId>, kind: UiAstValidationDiagnosticKind) -> Self {
        Self { node_id, kind }
    }

    /// Returns the AST node id associated with the diagnostic, if any.
    pub const fn node_id(&self) -> Option<UiAstNodeId> {
        self.node_id
    }

    /// Returns the diagnostic kind.
    pub const fn kind(&self) -> UiAstValidationDiagnosticKind {
        self.kind
    }
}

/// Collection of structured AST validation diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct UiAstValidationDiagnostics {
    diagnostics: Vec<UiAstValidationDiagnostic>,
}

impl UiAstValidationDiagnostics {
    /// Creates a diagnostics collection from a vector.
    pub fn new(diagnostics: Vec<UiAstValidationDiagnostic>) -> Self {
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
    pub fn diagnostics(&self) -> &[UiAstValidationDiagnostic] {
        &self.diagnostics
    }

    /// Returns an iterator over the diagnostics.
    pub fn iter(&self) -> core::slice::Iter<'_, UiAstValidationDiagnostic> {
        self.diagnostics.iter()
    }

    /// Returns the first diagnostic, if any.
    pub fn first(&self) -> Option<&UiAstValidationDiagnostic> {
        self.diagnostics.first()
    }

    /// Converts the collection into a vector.
    pub fn into_vec(self) -> Vec<UiAstValidationDiagnostic> {
        self.diagnostics
    }
}

/// Result of the minimal AST validation seed.
pub type UiAstValidationResult = Result<(), UiAstValidationDiagnostics>;

/// Validates a minimal inert Semantic UI AST.
///
/// The first seed validates local structural properties only.
pub fn validate_ast(ast: &UiAst, config: &UiAstValidationConfig) -> UiAstValidationResult {
    let _ = config;

    if ast.is_empty() {
        return Ok(());
    }

    let nodes = ast.nodes();
    let mut diagnostics = Vec::new();
    let mut seen_inconsistent_pairs: Vec<(UiAstNodeId, UiAstNodeId)> = Vec::new();
    let mut seen_root = false;

    for (index, node) in nodes.iter().enumerate() {
        if nodes[..index]
            .iter()
            .any(|previous| previous.id() == node.id())
        {
            diagnostics.push(UiAstValidationDiagnostic::new(
                Some(node.id()),
                UiAstValidationDiagnosticKind::DuplicateNodeId(node.id()),
            ));
        }

        if node.kind() == UiAstNodeKind::Root {
            if seen_root {
                diagnostics.push(UiAstValidationDiagnostic::new(
                    Some(node.id()),
                    UiAstValidationDiagnosticKind::MultipleRoots,
                ));
            } else {
                seen_root = true;
            }
        }

        validate_parent_relationship(nodes, node, &mut diagnostics, &mut seen_inconsistent_pairs);
        validate_child_relationships(nodes, node, &mut diagnostics, &mut seen_inconsistent_pairs);
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(UiAstValidationDiagnostics::new(diagnostics))
    }
}

fn validate_parent_relationship(
    nodes: &[UiAstNode],
    node: &UiAstNode,
    diagnostics: &mut Vec<UiAstValidationDiagnostic>,
    seen_inconsistent_pairs: &mut Vec<(UiAstNodeId, UiAstNodeId)>,
) {
    let Some(parent_id) = node.parent() else {
        return;
    };

    if parent_id == node.id() {
        diagnostics.push(UiAstValidationDiagnostic::new(
            Some(node.id()),
            UiAstValidationDiagnosticKind::SelfParent(node.id()),
        ));
        return;
    }

    let Some(parent_node) = find_node(nodes, parent_id) else {
        diagnostics.push(UiAstValidationDiagnostic::new(
            Some(node.id()),
            UiAstValidationDiagnosticKind::MissingParentTarget {
                node_id: node.id(),
                parent_id,
            },
        ));
        return;
    };

    if !parent_node.children().contains(&node.id()) {
        push_inconsistent_pair(
            diagnostics,
            seen_inconsistent_pairs,
            parent_id,
            node.id(),
            Some(node.id()),
        );
    }
}

fn validate_child_relationships(
    nodes: &[UiAstNode],
    node: &UiAstNode,
    diagnostics: &mut Vec<UiAstValidationDiagnostic>,
    seen_inconsistent_pairs: &mut Vec<(UiAstNodeId, UiAstNodeId)>,
) {
    for &child_id in node.children() {
        if child_id == node.id() {
            diagnostics.push(UiAstValidationDiagnostic::new(
                Some(node.id()),
                UiAstValidationDiagnosticKind::SelfChild(node.id()),
            ));
            continue;
        }

        let Some(child_node) = find_node(nodes, child_id) else {
            diagnostics.push(UiAstValidationDiagnostic::new(
                Some(node.id()),
                UiAstValidationDiagnosticKind::MissingChildTarget {
                    node_id: node.id(),
                    child_id,
                },
            ));
            continue;
        };

        if child_node.parent() != Some(node.id()) {
            push_inconsistent_pair(
                diagnostics,
                seen_inconsistent_pairs,
                node.id(),
                child_id,
                Some(node.id()),
            );
        }
    }
}

fn push_inconsistent_pair(
    diagnostics: &mut Vec<UiAstValidationDiagnostic>,
    seen_inconsistent_pairs: &mut Vec<(UiAstNodeId, UiAstNodeId)>,
    parent_id: UiAstNodeId,
    child_id: UiAstNodeId,
    node_id: Option<UiAstNodeId>,
) {
    if seen_inconsistent_pairs
        .iter()
        .any(|&(seen_parent_id, seen_child_id)| {
            seen_parent_id == parent_id && seen_child_id == child_id
        })
    {
        return;
    }

    seen_inconsistent_pairs.push((parent_id, child_id));
    diagnostics.push(UiAstValidationDiagnostic::new(
        node_id,
        UiAstValidationDiagnosticKind::InconsistentParentChild {
            parent_id,
            child_id,
        },
    ));
}

fn find_node(nodes: &[UiAstNode], id: UiAstNodeId) -> Option<&UiAstNode> {
    nodes.iter().find(|node| node.id() == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ast_with_nodes(nodes: impl IntoIterator<Item = UiAstNode>) -> UiAst {
        let mut ast = UiAst::new();
        for node in nodes {
            ast.push_node(node);
        }
        ast
    }

    #[test]
    fn empty_ast_is_valid() {
        let ast = UiAst::new();

        let result = validate_ast(&ast, &UiAstValidationConfig::default());

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn single_root_is_valid() {
        let ast = ast_with_nodes([UiAstNode::new(UiAstNodeId::new(1), UiAstNodeKind::Root)]);

        let result = validate_ast(&ast, &UiAstValidationConfig::default());

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn zero_root_non_empty_ast_is_valid_in_first_seed() {
        let ast = ast_with_nodes([UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Element)]);

        let result = validate_ast(&ast, &UiAstValidationConfig::default());

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn multiple_roots_returns_diagnostic() {
        let ast = ast_with_nodes([
            UiAstNode::new(UiAstNodeId::new(1), UiAstNodeKind::Root),
            UiAstNode::new(UiAstNodeId::new(2), UiAstNodeKind::Root),
        ]);

        let err =
            validate_ast(&ast, &UiAstValidationConfig::default()).expect_err("multiple roots");

        assert!(err.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic.kind(),
            UiAstValidationDiagnosticKind::MultipleRoots
        )));
    }

    #[test]
    fn duplicate_node_id_returns_diagnostic() {
        let ast = ast_with_nodes([
            UiAstNode::new(UiAstNodeId::new(3), UiAstNodeKind::Root),
            UiAstNode::new(UiAstNodeId::new(3), UiAstNodeKind::Element),
        ]);

        let err = validate_ast(&ast, &UiAstValidationConfig::default()).expect_err("duplicate id");

        assert!(err
            .diagnostics()
            .iter()
            .any(|diagnostic| matches!(diagnostic.kind(), UiAstValidationDiagnosticKind::DuplicateNodeId(id) if id == UiAstNodeId::new(3))));
    }

    #[test]
    fn missing_parent_target_returns_diagnostic() {
        let ast = ast_with_nodes([UiAstNode::with_parent(
            UiAstNodeId::new(4),
            UiAstNodeKind::Element,
            UiAstNodeId::new(99),
        )]);

        let err =
            validate_ast(&ast, &UiAstValidationConfig::default()).expect_err("missing parent");

        assert!(err
            .diagnostics()
            .iter()
            .any(|diagnostic| matches!(diagnostic.kind(), UiAstValidationDiagnosticKind::MissingParentTarget { node_id, parent_id } if node_id == UiAstNodeId::new(4) && parent_id == UiAstNodeId::new(99))));
    }

    #[test]
    fn missing_child_target_returns_diagnostic() {
        let mut node = UiAstNode::new(UiAstNodeId::new(5), UiAstNodeKind::Root);
        node.push_child(UiAstNodeId::new(100));
        let ast = ast_with_nodes([node]);

        let err = validate_ast(&ast, &UiAstValidationConfig::default()).expect_err("missing child");

        assert!(err
            .diagnostics()
            .iter()
            .any(|diagnostic| matches!(diagnostic.kind(), UiAstValidationDiagnosticKind::MissingChildTarget { node_id, child_id } if node_id == UiAstNodeId::new(5) && child_id == UiAstNodeId::new(100))));
    }

    #[test]
    fn consistent_parent_child_relationship_is_valid() {
        let mut parent = UiAstNode::new(UiAstNodeId::new(6), UiAstNodeKind::Root);
        parent.push_child(UiAstNodeId::new(7));
        let child = UiAstNode::with_parent(
            UiAstNodeId::new(7),
            UiAstNodeKind::Element,
            UiAstNodeId::new(6),
        );
        let ast = ast_with_nodes([parent, child]);

        let result = validate_ast(&ast, &UiAstValidationConfig::default());

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn parent_without_matching_child_returns_diagnostic() {
        let parent = UiAstNode::new(UiAstNodeId::new(8), UiAstNodeKind::Root);
        let child = UiAstNode::with_parent(
            UiAstNodeId::new(9),
            UiAstNodeKind::Element,
            UiAstNodeId::new(8),
        );
        let ast = ast_with_nodes([parent, child]);

        let err = validate_ast(&ast, &UiAstValidationConfig::default())
            .expect_err("inconsistent relationship");

        assert!(err
            .diagnostics()
            .iter()
            .any(|diagnostic| matches!(diagnostic.kind(), UiAstValidationDiagnosticKind::InconsistentParentChild { parent_id, child_id } if parent_id == UiAstNodeId::new(8) && child_id == UiAstNodeId::new(9))));
    }

    #[test]
    fn child_without_matching_parent_returns_diagnostic() {
        let mut parent = UiAstNode::new(UiAstNodeId::new(10), UiAstNodeKind::Root);
        parent.push_child(UiAstNodeId::new(11));
        let child = UiAstNode::new(UiAstNodeId::new(11), UiAstNodeKind::Element);
        let ast = ast_with_nodes([parent, child]);

        let err = validate_ast(&ast, &UiAstValidationConfig::default())
            .expect_err("inconsistent relationship");

        assert!(err
            .diagnostics()
            .iter()
            .any(|diagnostic| matches!(diagnostic.kind(), UiAstValidationDiagnosticKind::InconsistentParentChild { parent_id, child_id } if parent_id == UiAstNodeId::new(10) && child_id == UiAstNodeId::new(11))));
    }

    #[test]
    fn self_parent_returns_diagnostic() {
        let ast = ast_with_nodes([UiAstNode::with_parent(
            UiAstNodeId::new(12),
            UiAstNodeKind::Element,
            UiAstNodeId::new(12),
        )]);

        let err = validate_ast(&ast, &UiAstValidationConfig::default()).expect_err("self parent");

        assert!(err
            .diagnostics()
            .iter()
            .any(|diagnostic| matches!(diagnostic.kind(), UiAstValidationDiagnosticKind::SelfParent(id) if id == UiAstNodeId::new(12))));
    }

    #[test]
    fn self_child_returns_diagnostic() {
        let mut node = UiAstNode::new(UiAstNodeId::new(13), UiAstNodeKind::Root);
        node.push_child(UiAstNodeId::new(13));
        let ast = ast_with_nodes([node]);

        let err = validate_ast(&ast, &UiAstValidationConfig::default()).expect_err("self child");

        assert!(err
            .diagnostics()
            .iter()
            .any(|diagnostic| matches!(diagnostic.kind(), UiAstValidationDiagnosticKind::SelfChild(id) if id == UiAstNodeId::new(13))));
    }

    #[test]
    fn diagnostics_collect_multiple_failures() {
        let mut first = UiAstNode::new(UiAstNodeId::new(14), UiAstNodeKind::Root);
        first.push_child(UiAstNodeId::new(15));
        let second = UiAstNode::new(UiAstNodeId::new(14), UiAstNodeKind::Element);
        let ast = ast_with_nodes([first, second]);

        let err =
            validate_ast(&ast, &UiAstValidationConfig::default()).expect_err("multiple failures");

        assert!(err.len() >= 2);
        assert!(err
            .diagnostics()
            .iter()
            .any(|diagnostic| matches!(diagnostic.kind(), UiAstValidationDiagnosticKind::DuplicateNodeId(id) if id == UiAstNodeId::new(14))));
        assert!(err
            .diagnostics()
            .iter()
            .any(|diagnostic| matches!(diagnostic.kind(), UiAstValidationDiagnosticKind::MissingChildTarget { node_id, child_id } if node_id == UiAstNodeId::new(14) && child_id == UiAstNodeId::new(15))));
    }

    #[test]
    fn validation_does_not_call_lowering() {
        let ast = UiAst::new();

        let result = validate_ast(&ast, &UiAstValidationConfig::default());

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn validation_config_is_inert() {
        let config = UiAstValidationConfig;

        assert_eq!(config, UiAstValidationConfig::default());
    }

    #[test]
    fn diagnostics_are_structural_and_do_not_panic() {
        let ast = ast_with_nodes([UiAstNode::with_parent(
            UiAstNodeId::new(16),
            UiAstNodeKind::Element,
            UiAstNodeId::new(17),
        )]);

        let err = validate_ast(&ast, &UiAstValidationConfig::default())
            .expect_err("structural diagnostic");

        assert!(err
            .diagnostics()
            .iter()
            .any(|diagnostic| matches!(diagnostic.kind(), UiAstValidationDiagnosticKind::MissingParentTarget { node_id, parent_id } if node_id == UiAstNodeId::new(16) && parent_id == UiAstNodeId::new(17))));
    }

    #[test]
    fn diagnostics_iter_matches_slice_order() {
        let diagnostics = UiAstValidationDiagnostics::new(vec![
            UiAstValidationDiagnostic::new(None, UiAstValidationDiagnosticKind::MultipleRoots),
            UiAstValidationDiagnostic::new(
                None,
                UiAstValidationDiagnosticKind::SelfParent(UiAstNodeId::new(1)),
            ),
        ]);

        let mut iter = diagnostics.iter();
        assert!(matches!(
            iter.next().unwrap().kind(),
            UiAstValidationDiagnosticKind::MultipleRoots
        ));
        assert!(matches!(
            iter.next().unwrap().kind(),
            UiAstValidationDiagnosticKind::SelfParent(_)
        ));
        assert!(iter.next().is_none());
    }

    #[test]
    fn diagnostics_first_returns_first_diagnostic() {
        let diagnostics = UiAstValidationDiagnostics::new(vec![
            UiAstValidationDiagnostic::new(None, UiAstValidationDiagnosticKind::MultipleRoots),
            UiAstValidationDiagnostic::new(
                None,
                UiAstValidationDiagnosticKind::SelfParent(UiAstNodeId::new(1)),
            ),
        ]);

        assert!(matches!(
            diagnostics.first().unwrap().kind(),
            UiAstValidationDiagnosticKind::MultipleRoots
        ));
    }

    #[test]
    fn diagnostics_into_vec_preserves_order() {
        let diagnostics = UiAstValidationDiagnostics::new(vec![
            UiAstValidationDiagnostic::new(None, UiAstValidationDiagnosticKind::MultipleRoots),
            UiAstValidationDiagnostic::new(
                None,
                UiAstValidationDiagnosticKind::SelfParent(UiAstNodeId::new(1)),
            ),
        ]);

        let vec = diagnostics.into_vec();
        assert!(matches!(
            vec[0].kind(),
            UiAstValidationDiagnosticKind::MultipleRoots
        ));
        assert!(matches!(
            vec[1].kind(),
            UiAstValidationDiagnosticKind::SelfParent(_)
        ));
    }

    #[test]
    fn validation_accepts_ast_kinds_without_lowering_subset_filter() {
        let mut root = UiAstNode::new(UiAstNodeId::new(1), UiAstNodeKind::Root);
        root.push_child(UiAstNodeId::new(2));
        root.push_child(UiAstNodeId::new(3));
        root.push_child(UiAstNodeId::new(4));

        let attr = UiAstNode::with_parent(
            UiAstNodeId::new(2),
            UiAstNodeKind::Attribute,
            UiAstNodeId::new(1),
        );
        let bind = UiAstNode::with_parent(
            UiAstNodeId::new(3),
            UiAstNodeKind::Binding,
            UiAstNodeId::new(1),
        );
        let act = UiAstNode::with_parent(
            UiAstNodeId::new(4),
            UiAstNodeKind::Action,
            UiAstNodeId::new(1),
        );

        let ast = ast_with_nodes([root, attr, bind, act]);

        let result = validate_ast(&ast, &UiAstValidationConfig::default());
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn validation_result_does_not_produce_ir() {
        let ast = UiAst::new();
        let result: UiAstValidationResult = validate_ast(&ast, &UiAstValidationConfig::default());
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn cycle_shape_is_not_rejected_in_first_seed() {
        let mut a = UiAstNode::with_parent(
            UiAstNodeId::new(1),
            UiAstNodeKind::Element,
            UiAstNodeId::new(2),
        );
        a.push_child(UiAstNodeId::new(2));

        let mut b = UiAstNode::with_parent(
            UiAstNodeId::new(2),
            UiAstNodeKind::Element,
            UiAstNodeId::new(1),
        );
        b.push_child(UiAstNodeId::new(1));

        let ast = ast_with_nodes([a, b]);

        let result = validate_ast(&ast, &UiAstValidationConfig::default());
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn duplicate_ids_may_accumulate_structural_diagnostics() {
        let root = UiAstNode::new(UiAstNodeId::new(1), UiAstNodeKind::Root);
        let dup = UiAstNode::new(UiAstNodeId::new(1), UiAstNodeKind::Element);

        let ast = ast_with_nodes([root, dup]);

        let err = validate_ast(&ast, &UiAstValidationConfig::default()).expect_err("duplicate id");

        assert!(err.iter().any(|d| matches!(d.kind(), UiAstValidationDiagnosticKind::DuplicateNodeId(id) if id == UiAstNodeId::new(1))));
        assert!(!err.is_empty());
    }
}
