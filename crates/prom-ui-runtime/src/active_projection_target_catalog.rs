//! `ActiveProjectionTargetCatalog`, owned by `prom-ui-runtime`.
//!
//! Implements the runtime-owned catalog responsibility frozen in
//! `docs/spec/ui/shell_player_session_state_v0.md` section 7.1.9.6: the
//! sole stage-5 membership source, constructed exactly once from one
//! `prom_ui::shell_bridge::ActivationTargetSnapshot` value (itself derived
//! only from `prom-ui`'s `PreparedActiveProjectionTargets` producer), and
//! attached immutably to the activated session context. It is never
//! constructed from patch operations, a manifest, or raw caller-supplied
//! IDs outside that one snapshot.
//!
//! Membership uses `BTreeSet`, not `HashMap`/`HashSet`: lookups are
//! deterministic and never depend on hash-iteration order.
#![allow(dead_code)]

use alloc::collections::BTreeSet;

use prom_ui::shell_bridge::ActivationTargetSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ActiveProjectionTargetCatalog {
    node_anchors: BTreeSet<u64>,
    binding_anchors: BTreeSet<(u64, u32)>,
    collection_anchors: BTreeSet<u64>,
}

impl ActiveProjectionTargetCatalog {
    /// The empty catalog: no declared targets are members. Used as the
    /// deterministic zero value, e.g. for sessions with no activated
    /// projection structure.
    pub(crate) const fn empty() -> Self {
        Self {
            node_anchors: BTreeSet::new(),
            binding_anchors: BTreeSet::new(),
            collection_anchors: BTreeSet::new(),
        }
    }

    /// Constructs the catalog from one activation-scoped snapshot. This is
    /// the sole construction path other than [`Self::empty`].
    pub(crate) fn from_snapshot(snapshot: &ActivationTargetSnapshot) -> Self {
        Self {
            node_anchors: snapshot.node_anchor_ids.iter().copied().collect(),
            binding_anchors: snapshot.binding_anchor_ids.iter().copied().collect(),
            collection_anchors: snapshot.collection_anchor_ids.iter().copied().collect(),
        }
    }

    pub(crate) fn contains_node(&self, node: u64) -> bool {
        self.node_anchors.contains(&node)
    }

    pub(crate) fn contains_binding(&self, node: u64, slot: u32) -> bool {
        self.binding_anchors.contains(&(node, slot))
    }

    pub(crate) fn contains_collection(&self, collection: u64) -> bool {
        self.collection_anchors.contains(&collection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn snapshot() -> ActivationTargetSnapshot {
        ActivationTargetSnapshot {
            node_anchor_ids: vec![1, 2, 3],
            binding_anchor_ids: vec![(2, 0)],
            collection_anchor_ids: vec![3],
        }
    }

    #[test]
    fn test_empty_catalog_contains_nothing() {
        let catalog = ActiveProjectionTargetCatalog::empty();
        assert!(!catalog.contains_node(1));
        assert!(!catalog.contains_binding(1, 0));
        assert!(!catalog.contains_collection(1));
    }

    #[test]
    fn test_from_snapshot_membership() {
        let catalog = ActiveProjectionTargetCatalog::from_snapshot(&snapshot());
        assert!(catalog.contains_node(1));
        assert!(catalog.contains_node(2));
        assert!(catalog.contains_node(3));
        assert!(!catalog.contains_node(4));

        assert!(catalog.contains_binding(2, 0));
        assert!(!catalog.contains_binding(2, 1));
        assert!(!catalog.contains_binding(1, 0));

        assert!(catalog.contains_collection(3));
        assert!(!catalog.contains_collection(2));
    }

    #[test]
    fn test_determinism() {
        let a = ActiveProjectionTargetCatalog::from_snapshot(&snapshot());
        let b = ActiveProjectionTargetCatalog::from_snapshot(&snapshot());
        assert_eq!(a, b);
    }
}
