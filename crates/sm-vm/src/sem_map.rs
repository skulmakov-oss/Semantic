//! SHF-P1 (#2030, #2037): persistent ordered storage for `Value::Map`.
//!
//! `SemMap` is a persistent AVL tree whose nodes are shared through `Arc`.
//!
//! - Cloning a map (every register read, `LOAD_VAR`, call argument, record field
//!   read) copies one pointer: O(1).
//! - `insert` walks one root-to-leaf path. Each node on the path is obtained
//!   through `Arc::make_mut`, so a node still shared with another map value is
//!   shallow-copied (its entry and children are `Arc`s) and an unshared node is
//!   updated in place. Either way the cost is O(log n) node operations, whatever
//!   the ownership of the map value is.
//! - Lookups are O(log n) comparisons of `MapKey`.
//!
//! No node reachable from another map value is ever mutated, so every Semantic map
//! value stays logically independent: this is value semantics with structural
//! sharing, not aliasing.
//!
//! Order. Entries are kept in ascending `MapKey` order (`Ord` is derived, so the
//! order is total and platform-independent). Semantic exposes no map iteration,
//! map equality or map serialization (`docs/spec/foundation_stdlib_v0.md`: "no
//! observable map-order promise"), so this order is only visible through the Rust
//! API (`iter`, `Debug`).

use std::cmp::Ordering;
use std::fmt;
use std::sync::Arc;

use crate::semcode_vm::{MapKey, Value};

type Link = Option<Arc<Node>>;

#[derive(Clone)]
struct Node {
    /// Shared so that copying a node on an update path never deep-copies a value.
    entry: Arc<(MapKey, Value)>,
    left: Link,
    right: Link,
    height: u8,
}

fn height(link: &Link) -> u8 {
    link.as_ref().map_or(0, |n| n.height)
}

impl Node {
    fn leaf(key: MapKey, value: Value) -> Self {
        Self {
            entry: Arc::new((key, value)),
            left: None,
            right: None,
            height: 1,
        }
    }

    fn update_height(&mut self) {
        self.height = 1 + height(&self.left).max(height(&self.right));
    }

    fn balance(&self) -> i16 {
        i16::from(height(&self.left)) - i16::from(height(&self.right))
    }
}

/// Persistent ordered map from scalar `MapKey`s to runtime values.
#[derive(Clone, Default)]
pub struct SemMap {
    root: Link,
    len: usize,
}

impl SemMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get(&self, key: &MapKey) -> Option<&Value> {
        let mut cur = self.root.as_deref();
        while let Some(node) = cur {
            cur = match key.cmp(&node.entry.0) {
                Ordering::Less => node.left.as_deref(),
                Ordering::Greater => node.right.as_deref(),
                Ordering::Equal => return Some(&node.entry.1),
            };
        }
        None
    }

    pub fn contains_key(&self, key: &MapKey) -> bool {
        self.get(key).is_some()
    }

    /// Sets `key` to `value`, replacing an existing binding. Other map values that
    /// share structure with `self` are unaffected.
    pub fn insert(&mut self, key: MapKey, value: Value) {
        if insert(&mut self.root, key, value) {
            self.len += 1;
        }
    }

    /// Entries in ascending key order.
    pub fn iter(&self) -> Iter<'_> {
        let mut iter = Iter { stack: Vec::new() };
        iter.push_left(self.root.as_deref());
        iter
    }

    #[cfg(test)]
    fn check_invariants(&self) {
        fn walk(link: &Link, lo: Option<&MapKey>, hi: Option<&MapKey>) -> (u8, usize) {
            let Some(node) = link else { return (0, 0) };
            let key = &node.entry.0;
            assert!(lo.is_none_or(|lo| lo < key), "order violated");
            assert!(hi.is_none_or(|hi| key < hi), "order violated");
            let (hl, nl) = walk(&node.left, lo, Some(key));
            let (hr, nr) = walk(&node.right, Some(key), hi);
            assert_eq!(node.height, 1 + hl.max(hr), "stale height");
            assert!(
                (i16::from(hl) - i16::from(hr)).abs() <= 1,
                "AVL balance violated"
            );
            (node.height, nl + nr + 1)
        }
        let (_, count) = walk(&self.root, None, None);
        assert_eq!(count, self.len, "stale len");
    }
}

/// Returns `true` when a new key was added (as opposed to replaced).
fn insert(link: &mut Link, key: MapKey, value: Value) -> bool {
    let Some(node) = link else {
        *link = Some(Arc::new(Node::leaf(key, value)));
        return true;
    };
    let node = Arc::make_mut(node);
    let added = match key.cmp(&node.entry.0) {
        Ordering::Less => insert(&mut node.left, key, value),
        Ordering::Greater => insert(&mut node.right, key, value),
        Ordering::Equal => {
            node.entry = Arc::new((key, value));
            return false;
        }
    };
    if added {
        rebalance(link);
    }
    added
}

/// Restores the height and AVL balance of the (already unshared) node in `link`.
fn rebalance(link: &mut Link) {
    let Some(node) = link else { return };
    let node = Arc::make_mut(node);
    node.update_height();
    let balance = node.balance();
    if balance > 1 {
        if node.left.as_ref().is_some_and(|l| l.balance() < 0) {
            rotate_left(&mut node.left);
        }
        rotate_right(link);
    } else if balance < -1 {
        if node.right.as_ref().is_some_and(|r| r.balance() > 0) {
            rotate_right(&mut node.right);
        }
        rotate_left(link);
    }
}

fn rotate_right(link: &mut Link) {
    let Some(mut root) = link.take() else { return };
    let root_node = Arc::make_mut(&mut root);
    let Some(mut pivot) = root_node.left.take() else {
        *link = Some(root);
        return;
    };
    let pivot_node = Arc::make_mut(&mut pivot);
    root_node.left = pivot_node.right.take();
    root_node.update_height();
    pivot_node.right = Some(root);
    pivot_node.update_height();
    *link = Some(pivot);
}

fn rotate_left(link: &mut Link) {
    let Some(mut root) = link.take() else { return };
    let root_node = Arc::make_mut(&mut root);
    let Some(mut pivot) = root_node.right.take() else {
        *link = Some(root);
        return;
    };
    let pivot_node = Arc::make_mut(&mut pivot);
    root_node.right = pivot_node.left.take();
    root_node.update_height();
    pivot_node.left = Some(root);
    pivot_node.update_height();
    *link = Some(pivot);
}

/// In-order iterator over a `SemMap`; the stack holds at most one tree height.
pub struct Iter<'a> {
    stack: Vec<&'a Node>,
}

impl<'a> Iter<'a> {
    fn push_left(&mut self, mut cur: Option<&'a Node>) {
        while let Some(node) = cur {
            self.stack.push(node);
            cur = node.left.as_deref();
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a MapKey, &'a Value);

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        self.push_left(node.right.as_deref());
        Some((&node.entry.0, &node.entry.1))
    }
}

/// Builds a map by inserting the entries in order, so a later duplicate key replaces an
/// earlier one (last wins, like `std::collections::BTreeMap`).
///
/// Migration note: the previous `Value::Map(Vec<(MapKey, Value)>)` payload could hold
/// duplicate keys when built by Rust code (the VM itself never produced them), and
/// `MAP_GET` returned the *first* match. `vec.into_iter().rev().collect()` reproduces
/// that first-match lookup for such a vector.
impl FromIterator<(MapKey, Value)> for SemMap {
    fn from_iter<I: IntoIterator<Item = (MapKey, Value)>>(entries: I) -> Self {
        let mut map = SemMap::new();
        for (key, value) in entries {
            map.insert(key, value);
        }
        map
    }
}

/// Content equality: the same bindings, independent of tree shape and of insertion order.
/// Values are compared with `Value::eq` and there is no shared-root shortcut, so a map
/// holding `F64(NaN)` is not equal to itself, as with the previous `Vec` payload.
impl PartialEq for SemMap {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.iter().eq(other.iter())
    }
}

/// Renders as the list of entries in ascending key order, the same shape as the
/// previous `Vec<(MapKey, Value)>` representation.
impl fmt::Debug for SemMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn k(i: i32) -> MapKey {
        MapKey::I32(i)
    }

    /// Deterministic xorshift so the model test needs no RNG dependency.
    fn next(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    #[test]
    fn empty_map() {
        let m = SemMap::new();
        assert!(m.is_empty());
        assert_eq!(m.len(), 0);
        assert_eq!(m.get(&k(1)), None);
        assert!(!m.contains_key(&k(1)));
        assert_eq!(m.iter().count(), 0);
        assert_eq!(format!("{m:?}"), "[]");
    }

    #[test]
    fn insert_get_and_replace() {
        let mut m = SemMap::new();
        m.insert(k(2), Value::I32(20));
        m.insert(k(1), Value::I32(10));
        m.insert(k(2), Value::I32(21));
        assert_eq!(m.len(), 2);
        assert_eq!(m.get(&k(2)), Some(&Value::I32(21)));
        assert_eq!(m.get(&k(1)), Some(&Value::I32(10)));
        assert_eq!(format!("{m:?}"), "[(I32(1), I32(10)), (I32(2), I32(21))]");
        m.check_invariants();
    }

    #[test]
    fn model_equivalence_against_btreemap_including_persistence() {
        let mut state = 0x9E37_79B9_7F4A_7C15u64;
        let mut map = SemMap::new();
        let mut model = BTreeMap::new();
        let mut snapshots: Vec<(SemMap, BTreeMap<MapKey, Value>)> = Vec::new();
        for step in 0..20_000 {
            let key = match next(&mut state) % 3 {
                0 => MapKey::I32((next(&mut state) % 512) as i32 - 256),
                1 => MapKey::Text(format!("k{}", next(&mut state) % 512)),
                _ => MapKey::U32((next(&mut state) % 512) as u32),
            };
            let value = Value::I32(step);
            map.insert(key.clone(), value.clone());
            model.insert(key, value);
            if step % 997 == 0 {
                map.check_invariants();
                snapshots.push((map.clone(), model.clone()));
            }
        }
        map.check_invariants();
        assert_eq!(map.len(), model.len());
        assert!(map.iter().eq(model.iter()));
        // Old versions are unaffected by every later update (value semantics).
        for (old, old_model) in &snapshots {
            old.check_invariants();
            assert_eq!(old.len(), old_model.len());
            assert!(old.iter().eq(old_model.iter()));
        }
    }

    #[test]
    fn updating_a_copy_never_changes_the_original() {
        let original: SemMap = (0..1000).map(|i| (k(i), Value::I32(i))).collect();
        let mut copy = original.clone();
        copy.insert(k(5), Value::I32(-5));
        copy.insert(k(5000), Value::I32(5000));
        assert_eq!(original.get(&k(5)), Some(&Value::I32(5)));
        assert_eq!(original.get(&k(5000)), None);
        assert_eq!(original.len(), 1000);
        assert_eq!(copy.get(&k(5)), Some(&Value::I32(-5)));
        assert_eq!(copy.len(), 1001);
        original.check_invariants();
        copy.check_invariants();
    }

    #[test]
    fn equality_is_by_content_not_insertion_order() {
        let a: SemMap = [(k(1), Value::I32(1)), (k(2), Value::I32(2))]
            .into_iter()
            .collect();
        let b: SemMap = [(k(2), Value::I32(2)), (k(1), Value::I32(1))]
            .into_iter()
            .collect();
        let c: SemMap = [(k(1), Value::I32(1)), (k(2), Value::I32(3))]
            .into_iter()
            .collect();
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn sequential_inserts_stay_logarithmic_in_height() {
        let m: SemMap = (0..100_000).map(|i| (k(i), Value::Unit)).collect();
        m.check_invariants();
        // AVL bound: height < 1.4405 * log2(n + 2).
        let h = m.root.as_ref().map_or(0, |n| n.height);
        assert!(f64::from(h) < 1.4405 * (100_002f64).log2(), "height {h}");
    }

    #[test]
    fn nan_values_keep_equality_non_reflexive() {
        let m: SemMap = [(k(1), Value::F64(f64::NAN))].into_iter().collect();
        let same_tree = m.clone();
        assert_ne!(m, same_tree, "no root-identity shortcut");
        let n: SemMap = [(k(1), Value::F64(1.0))].into_iter().collect();
        assert_eq!(n, n.clone());
    }

    #[test]
    fn duplicate_import_is_last_wins_and_rev_reproduces_first_match() {
        let legacy = vec![
            (k(1), Value::I32(10)),
            (k(2), Value::I32(20)),
            (k(1), Value::I32(11)),
        ];
        let last_wins: SemMap = legacy.clone().into_iter().collect();
        assert_eq!(last_wins.len(), 2);
        assert_eq!(last_wins.get(&k(1)), Some(&Value::I32(11)));
        // The legacy MAP_GET returned the first match for key 1.
        let first_match: SemMap = legacy.into_iter().rev().collect();
        assert_eq!(first_match.get(&k(1)), Some(&Value::I32(10)));
        assert_eq!(first_match.get(&k(2)), Some(&Value::I32(20)));
        first_match.check_invariants();
    }

    #[test]
    fn deep_maps_drop_without_overflow() {
        let m: SemMap = (0..200_000).map(|i| (k(i), Value::I32(i))).collect();
        drop(m);
    }
}
