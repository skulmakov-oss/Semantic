//! Inert Semantic UI foundation model.
//!
//! This module defines the first local Semantic UI model layer: UI Tree, UI AST,
//! and UI IR. The types are structural handles only. They do not render, do not
//! execute, do not perform admission, and do not integrate with parser,
//! verifier, VM, runtime, Workbench, or Semantic Studio.

use alloc::vec::Vec;

/// Stable local identifier for a UI node.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UiNodeId(pub u64);

impl UiNodeId {
    /// Creates a new node identifier from a raw value.
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw identifier value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Stable local identifier for a UI tree instance.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UiTreeId(pub u64);

impl UiTreeId {
    /// Creates a new tree identifier from a raw value.
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw identifier value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Stable local identifier for a UI AST node.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UiAstNodeId(pub u64);

impl UiAstNodeId {
    /// Creates a new AST-node identifier from a raw value.
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw identifier value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Stable local identifier for a UI IR node.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UiIrNodeId(pub u64);

impl UiIrNodeId {
    /// Creates a new IR-node identifier from a raw value.
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw identifier value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Closed structural vocabulary for the inert UI Tree layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UiNodeKind {
    Root,
    Element,
    Text,
    Fragment,
    Slot,
}

/// Inert UI Tree node with local parent/child handles.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiNode {
    pub id: UiNodeId,
    pub kind: UiNodeKind,
    pub parent: Option<UiNodeId>,
    pub children: Vec<UiNodeId>,
}

impl UiNode {
    /// Creates an inert UI Tree node with no parent or children.
    pub fn new(id: UiNodeId, kind: UiNodeKind) -> Self {
        Self {
            id,
            kind,
            parent: None,
            children: Vec::new(),
        }
    }

    /// Creates an inert UI Tree node with a parent handle.
    pub fn with_parent(id: UiNodeId, kind: UiNodeKind, parent: UiNodeId) -> Self {
        Self {
            id,
            kind,
            parent: Some(parent),
            children: Vec::new(),
        }
    }

    /// Returns the optional parent handle.
    pub const fn parent(&self) -> Option<UiNodeId> {
        self.parent
    }

    /// Returns the child handles.
    pub fn children(&self) -> &[UiNodeId] {
        &self.children
    }

    /// Updates the parent handle without adding semantics.
    pub fn set_parent(&mut self, parent: Option<UiNodeId>) {
        self.parent = parent;
    }

    /// Appends a child handle without traversal semantics.
    pub fn push_child(&mut self, child: UiNodeId) {
        self.children.push(child);
    }
}

/// Inert UI Tree container.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiTree {
    pub id: UiTreeId,
    pub nodes: Vec<UiNode>,
}

impl UiTree {
    /// Creates an empty UI Tree container for the given tree id.
    pub fn new(id: UiTreeId) -> Self {
        Self {
            id,
            nodes: Vec::new(),
        }
    }

    /// Returns the tree id.
    pub const fn id(&self) -> UiTreeId {
        self.id
    }

    /// Returns whether the tree has no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the number of stored nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the stored nodes.
    pub fn nodes(&self) -> &[UiNode] {
        &self.nodes
    }

    /// Appends an inert node to the tree.
    pub fn push_node(&mut self, node: UiNode) {
        self.nodes.push(node);
    }
}

/// Closed structural vocabulary for the inert UI AST layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UiAstNodeKind {
    Root,
    Element,
    Text,
    Fragment,
    Attribute,
    Binding,
    Action,
}

/// Inert UI AST node with local parent/child handles.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiAstNode {
    pub id: UiAstNodeId,
    pub kind: UiAstNodeKind,
    pub parent: Option<UiAstNodeId>,
    pub children: Vec<UiAstNodeId>,
}

impl UiAstNode {
    /// Creates an inert UI AST node with no parent or children.
    pub fn new(id: UiAstNodeId, kind: UiAstNodeKind) -> Self {
        Self {
            id,
            kind,
            parent: None,
            children: Vec::new(),
        }
    }

    /// Creates an inert UI AST node with a parent handle.
    pub fn with_parent(id: UiAstNodeId, kind: UiAstNodeKind, parent: UiAstNodeId) -> Self {
        Self {
            id,
            kind,
            parent: Some(parent),
            children: Vec::new(),
        }
    }

    /// Returns the optional parent handle.
    pub const fn parent(&self) -> Option<UiAstNodeId> {
        self.parent
    }

    /// Returns the child handles.
    pub fn children(&self) -> &[UiAstNodeId] {
        &self.children
    }

    /// Updates the parent handle without adding semantics.
    pub fn set_parent(&mut self, parent: Option<UiAstNodeId>) {
        self.parent = parent;
    }

    /// Appends a child handle without parser or lowering semantics.
    pub fn push_child(&mut self, child: UiAstNodeId) {
        self.children.push(child);
    }
}

/// Inert UI AST container.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct UiAst {
    pub nodes: Vec<UiAstNode>,
}

impl UiAst {
    /// Creates an empty UI AST container.
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Returns whether the AST has no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the number of stored nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the stored AST nodes.
    pub fn nodes(&self) -> &[UiAstNode] {
        &self.nodes
    }

    /// Appends an inert AST node.
    pub fn push_node(&mut self, node: UiAstNode) {
        self.nodes.push(node);
    }
}

/// Closed structural vocabulary for the inert UI IR layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UiIrNodeKind {
    Root,
    Element,
    Text,
    Fragment,
    Property,
    Action,
    EffectBoundary,
}

/// Inert UI IR node with local parent/child handles.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiIrNode {
    pub id: UiIrNodeId,
    pub kind: UiIrNodeKind,
    pub parent: Option<UiIrNodeId>,
    pub children: Vec<UiIrNodeId>,
}

impl UiIrNode {
    /// Creates an inert UI IR node with no parent or children.
    pub fn new(id: UiIrNodeId, kind: UiIrNodeKind) -> Self {
        Self {
            id,
            kind,
            parent: None,
            children: Vec::new(),
        }
    }

    /// Creates an inert UI IR node with a parent handle.
    pub fn with_parent(id: UiIrNodeId, kind: UiIrNodeKind, parent: UiIrNodeId) -> Self {
        Self {
            id,
            kind,
            parent: Some(parent),
            children: Vec::new(),
        }
    }

    /// Returns the optional parent handle.
    pub const fn parent(&self) -> Option<UiIrNodeId> {
        self.parent
    }

    /// Returns the child handles.
    pub fn children(&self) -> &[UiIrNodeId] {
        &self.children
    }

    /// Updates the parent handle without adding semantics.
    pub fn set_parent(&mut self, parent: Option<UiIrNodeId>) {
        self.parent = parent;
    }

    /// Appends a child handle without execution semantics.
    pub fn push_child(&mut self, child: UiIrNodeId) {
        self.children.push(child);
    }
}

/// Inert UI IR container.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct UiIr {
    pub nodes: Vec<UiIrNode>,
}

impl UiIr {
    /// Creates an empty UI IR container.
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Returns whether the IR has no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the number of stored nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the stored IR nodes.
    pub fn nodes(&self) -> &[UiIrNode] {
        &self.nodes
    }

    /// Appends an inert IR node.
    pub fn push_node(&mut self, node: UiIrNode) {
        self.nodes.push(node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_id_round_trip() {
        let id = UiNodeId::new(7);

        assert_eq!(id.raw(), 7);
        assert_eq!(UiNodeId(7), id);
    }

    #[test]
    fn tree_id_round_trip() {
        let id = UiTreeId::new(11);

        assert_eq!(id.raw(), 11);
        assert_eq!(UiTreeId(11), id);
    }

    #[test]
    fn empty_tree_has_id_and_no_nodes() {
        let tree = UiTree::new(UiTreeId::new(1));

        assert_eq!(tree.id(), UiTreeId::new(1));
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert!(tree.nodes().is_empty());
    }

    #[test]
    fn tree_accepts_inert_node() {
        let mut tree = UiTree::new(UiTreeId::new(2));
        let mut node = UiNode::new(UiNodeId::new(21), UiNodeKind::Element);
        node.set_parent(Some(UiNodeId::new(20)));
        node.push_child(UiNodeId::new(22));

        tree.push_node(node.clone());

        assert_eq!(tree.len(), 1);
        assert_eq!(tree.nodes()[0], node);
    }

    #[test]
    fn ast_starts_empty() {
        let ast = UiAst::new();

        assert!(ast.is_empty());
        assert_eq!(ast.len(), 0);
        assert!(ast.nodes().is_empty());
    }

    #[test]
    fn ast_accepts_inert_node() {
        let mut ast = UiAst::new();
        let mut node = UiAstNode::new(UiAstNodeId::new(31), UiAstNodeKind::Element);
        node.set_parent(Some(UiAstNodeId::new(30)));
        node.push_child(UiAstNodeId::new(32));

        ast.push_node(node.clone());

        assert_eq!(ast.len(), 1);
        assert_eq!(ast.nodes()[0], node);
    }

    #[test]
    fn ir_starts_empty() {
        let ir = UiIr::new();

        assert!(ir.is_empty());
        assert_eq!(ir.len(), 0);
        assert!(ir.nodes().is_empty());
    }

    #[test]
    fn ir_accepts_inert_node() {
        let mut ir = UiIr::new();
        let mut node = UiIrNode::new(UiIrNodeId::new(41), UiIrNodeKind::Property);
        node.set_parent(Some(UiIrNodeId::new(40)));
        node.push_child(UiIrNodeId::new(42));

        ir.push_node(node.clone());

        assert_eq!(ir.len(), 1);
        assert_eq!(ir.nodes()[0], node);
    }

    #[test]
    fn node_parent_and_children_are_handles_only() {
        let mut node =
            UiNode::with_parent(UiNodeId::new(50), UiNodeKind::Fragment, UiNodeId::new(49));
        node.push_child(UiNodeId::new(51));
        node.push_child(UiNodeId::new(52));

        assert_eq!(node.parent(), Some(UiNodeId::new(49)));
        assert_eq!(node.children(), &[UiNodeId::new(51), UiNodeId::new(52)]);
    }

    #[test]
    fn ast_and_ir_are_not_runtime_or_renderer_bound() {
        let mut ast = UiAst::new();
        ast.push_node(UiAstNode::new(UiAstNodeId::new(60), UiAstNodeKind::Action));

        let mut ir = UiIr::new();
        ir.push_node(UiIrNode::new(
            UiIrNodeId::new(70),
            UiIrNodeKind::EffectBoundary,
        ));

        assert_eq!(ast.len(), 1);
        assert_eq!(ast.nodes()[0].kind, UiAstNodeKind::Action);
        assert_eq!(ir.len(), 1);
        assert_eq!(ir.nodes()[0].kind, UiIrNodeKind::EffectBoundary);
    }
}
