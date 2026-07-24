//! Shared stable-target anchor value types. See contract section 7.1.3.
//!
//! These are the three conceptual target classes shared by both
//! transition-scoped (`PreparedProjectionPatchTargets`) and activation-scoped
//! (`PreparedActiveProjectionTargets`) prepared evidence. They carry only
//! stable identity coordinates, never patch values or semantic content.
#![allow(dead_code)]

use crate::binding_graph::BindingSlot;
use crate::contract_primitives::StaticNodeId;

/// The stable node identity targeted by a node-availability operation, or
/// declared as an activatable node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct NodeAnchor {
    node: StaticNodeId,
}

impl NodeAnchor {
    pub(crate) const fn new(node: StaticNodeId) -> Self {
        Self { node }
    }

    pub(crate) const fn node(self) -> StaticNodeId {
        self.node
    }
}

/// The stable node identity and binding-slot identity targeted by a
/// binding-value operation, or declared as an activatable binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct BindingAnchor {
    node: StaticNodeId,
    slot: BindingSlot,
}

impl BindingAnchor {
    pub(crate) const fn new(node: StaticNodeId, slot: BindingSlot) -> Self {
        Self { node, slot }
    }

    pub(crate) const fn node(self) -> StaticNodeId {
        self.node
    }

    pub(crate) const fn slot(self) -> BindingSlot {
        self.slot
    }
}

/// The stable collection-node identity targeted by a collection operation,
/// or explicitly declared as a collection anchor. Explicit declaration is
/// the sole source for activation-scoped `CollectionAnchor` membership; see
/// `collection_anchor_declarations`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct CollectionAnchor {
    collection: StaticNodeId,
}

impl CollectionAnchor {
    pub(crate) const fn new(collection: StaticNodeId) -> Self {
        Self { collection }
    }

    pub(crate) const fn collection(self) -> StaticNodeId {
        self.collection
    }
}

/// The target class carried by one manifest entry. See contract section
/// 7.1.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TargetAnchor {
    Node(NodeAnchor),
    Binding(BindingAnchor),
    Collection(CollectionAnchor),
}
