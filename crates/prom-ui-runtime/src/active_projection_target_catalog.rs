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
            node_anchors: snapshot.node_anchor_ids().iter().copied().collect(),
            binding_anchors: snapshot.binding_anchor_ids().iter().copied().collect(),
            collection_anchors: snapshot.collection_anchor_ids().iter().copied().collect(),
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
    use prom_ui::shell_bridge::demo_activation_snapshot;

    // `ActivationTargetSnapshot` is opaque outside `prom-ui`: it has no
    // public constructor here, so these tests build the catalog from the
    // one authentic snapshot producer (`demo_activation_snapshot`) rather
    // than a hand-authored fixture. Its fixed shape is: node anchors
    // 1,2,3,4; one binding anchor (2, 0); one collection anchor 4.

    #[test]
    fn test_empty_catalog_contains_nothing() {
        let catalog = ActiveProjectionTargetCatalog::empty();
        assert!(!catalog.contains_node(1));
        assert!(!catalog.contains_binding(1, 0));
        assert!(!catalog.contains_collection(1));
    }

    #[test]
    fn test_from_snapshot_membership() {
        let catalog = ActiveProjectionTargetCatalog::from_snapshot(&demo_activation_snapshot());
        assert!(catalog.contains_node(1));
        assert!(catalog.contains_node(2));
        assert!(catalog.contains_node(3));
        assert!(catalog.contains_node(4));
        assert!(!catalog.contains_node(999));

        assert!(catalog.contains_binding(2, 0));
        assert!(!catalog.contains_binding(2, 1));
        assert!(!catalog.contains_binding(1, 0));

        assert!(catalog.contains_collection(4));
        assert!(!catalog.contains_collection(2));
    }

    #[test]
    fn test_determinism() {
        let a = ActiveProjectionTargetCatalog::from_snapshot(&demo_activation_snapshot());
        let b = ActiveProjectionTargetCatalog::from_snapshot(&demo_activation_snapshot());
        assert_eq!(a, b);
    }

    /// Proves the P1 fix: `ActivationTargetSnapshot` cannot be constructed
    /// from arbitrary raw ids. There is no struct-literal, `Default`,
    /// `From<Vec<_>>`, or raw-parts constructor available here — the only
    /// way to obtain one is the authentic `demo_activation_snapshot`
    /// producer, so a caller cannot fabricate catalog membership for ids
    /// that were never declared by validated projection-owned evidence.
    /// (Documentation test: absence of a fabrication path is proven by this
    /// module compiling without one, not by a runtime assertion.)
    #[test]
    fn test_catalog_only_ever_built_from_authentic_snapshot() {
        let snapshot = demo_activation_snapshot();
        let catalog = ActiveProjectionTargetCatalog::from_snapshot(&snapshot);
        // Node 999 was never declared anywhere in the authentic fixture;
        // there is no way to inject it into `snapshot` from this crate.
        assert!(!catalog.contains_node(999));
        assert!(!catalog.contains_binding(999, 0));
        assert!(!catalog.contains_collection(999));
    }
}
