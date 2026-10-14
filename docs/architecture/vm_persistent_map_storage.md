# Semantic VM: Persistent Map Storage (SHF-P1)

Status: implemented on branch `shf-r/p1-persistent-map` (SHF-R #2030, finding F1; research track #2037). Draft PR, awaiting OWNER review. **Merge is not authorized by this document.**

Decision owner: repository OWNER (OWNER DIRECTION "SHF-R Qualification and Rust VM Optimization", item 2). The public API change in §6 is disclosed for an explicit OWNER contract decision.

## 1. Problem and evidence

`Value::Map` was `Vec<(MapKey, Value)>`, with no sharing:

| Operation | Previous cost | Why |
|---|---|---|
| any read of a map register / local (`get_reg`, `LOAD_VAR`, call argument, `RECORD_GET` of a map field) | O(n) | deep `Vec` clone |
| `MAP_CONTAINS`, `MAP_GET` | O(n) | linear scan, after the O(n) register clone |
| `MAP_SET` | O(n) | filter-clone of every other pair, after the O(n) register clone |

Building an n-entry symbol table is therefore Θ(n²). Native Linux qualification of `main` @ `94c5c50` (posted on #2030) measured `m = map_set(m, i, i)` at 14.5 s for 16k and 55.3 s for 32k entries (time exponent 1.93, Steps exponent 1.00).

## 2. The ownership constraint

OWNER correction: `Arc<BTreeMap<MapKey, Value>>` alone does not give logarithmic persistent updates. The update path is:

```text
LOAD_VAR  rM, m        ; clone of the local      -> Arc count >= 2
MAP_SET   rD, rM, k, v ; get_reg(rM) clones again -> Arc count >= 3
STORE_VAR m, rD
```

When `MAP_SET` runs, the map's storage is always shared with at least the local `m` and the register `rM`. `Arc::make_mut` on a whole-map container would copy all n entries on every insertion, which is still Θ(n²). The representation has to make an update cheap **while the old version is still alive**.

## 3. Options considered

| | Option | Lookup | Update under sharing | Covers maps in records / arguments / returns | Verifier or planner interaction |
|---|---|---|---|---|---|
| A | `Arc<BTreeMap>` + `make_mut` | O(log n) | **O(n)** (whole-map copy, §2) | – | none |
| B | A + extend SHF-P3 verified consumption to `LOAD_VAR`→`MAP_SET`→`STORE_VAR` | O(log n) | O(log n) **only** for the accepted local triple; O(n) otherwise | **no**: `ctx with { syms: map_set(ctx.syms, …) }`, `declare(ctx, name) -> Ctx` and every call boundary still copy | new acceptance conditions, plan sites, failure-path proof |
| C | **persistent balanced tree with `Arc` nodes, path copying** | O(log n) | **O(log n)** whatever the ownership | **yes**: a copy is O(1) everywhere | none |
| D | HAMT / hash trie | ~O(1) | O(log₃₂ n) | yes | needs a hash function frozen as part of determinism; no ordered iteration |

B is the SHF-P3 mechanism and is attractive for sequences, where the dominant pattern is a local accumulator. For maps it fails the measured workload: a compiler's symbol table lives inside a context record and crosses function boundaries (§7, `symbol_table` and `state_record_map`). D adds a hashing contract (algorithm, seed, collision order) that would itself have to be frozen for cross-platform determinism, while C only relies on the already-derived total order of `MapKey`.

**Decision: C.** It is the simplest structure that removes the asymptotic bottleneck for every admitted way a map value can be held, and it requires no change to SemCode, the verifier, the consumption planner or any quota.

## 4. Design

`crates/sm-vm/src/sem_map.rs` defines `SemMap`, a persistent AVL tree:

```text
SemMap { root: Option<Arc<Node>>, len: usize }
Node   { entry: Arc<(MapKey, Value)>, left, right: Option<Arc<Node>>, height: u8 }
```

- **Copy:** `SemMap::clone` clones one `Arc`. It is O(1).
- **Lookup:** iterative descent by `MapKey::cmp`. It is O(log n).
- **Insert:** a recursive descent in which each node on the path is obtained through `Arc::make_mut`.
  - A node still shared with another map value is shallow-copied: its entry and children are `Arc`s, so the copy is O(1) and **never deep-copies a stored `Value`**.
  - A node owned only by this map is updated in place.
  - Height update and at most two rotations per level follow, also through `make_mut`.
  - Total cost: O(log n) node operations plus one new entry allocation.
- **Replace:** an existing key gets a new entry `Arc`. The old entry stays reachable from older versions.
- **Drop:** the default recursive drop, bounded by the AVL height (< 1.4405·log₂(n+2), so under 40 for 10⁸ entries).

The four VM handlers change only in how they reach the storage:

| Opcode | Before | After |
|---|---|---|
| `MAP_EMPTY` | `Value::Map(Vec::new())` | `Value::Map(SemMap::new())` |
| `MAP_CONTAINS` | `pairs.iter().any(..)` | `map.contains_key(&key)` |
| `MAP_GET` | `pairs.iter().find(..)`, clone of the value or the default register | `map.get(&key)`, the same clone or default |
| `MAP_SET` | filter-clone + push | `map.insert(key, val)` on the register copy |

Operand decoding order, register reads, type checks, error kinds and error texts are unchanged line by line.

## 5. Invariants and proof obligations

1. **Value semantics.** A node reachable from more than one map value is never mutated: `Arc::make_mut` copies it first. Every mutation therefore affects only nodes uniquely owned by the map being updated, so no other Semantic value can observe it. This is the same argument as SHF-P3 for `Arc<Vec<Value>>`, applied per node. Tests: `updating_a_copy_never_changes_the_original`, persistence snapshots in `model_equivalence_against_btreemap_including_persistence` (20 000 random operations, 21 retained versions re-checked at the end), and the source-level `map_value_semantics_and_counters_are_unchanged`.
2. **Functional equivalence.** `get`, `contains`, `insert` and replacement agree with `std::collections::BTreeMap` on randomized mixed-family keys (model test). AVL order, balance, height and length invariants are re-checked during the model test.
3. **Determinism.** Tree shape depends only on the sequence of inserted keys and on the derived `Ord` of `MapKey`. That order is total and platform-independent; `Text` is compared by bytes. No hashing, no addresses, no randomness.
4. **Observable map order.** Semantic exposes no map iteration, equality or serialization. `value_eq` on two maps still returns `TypeMismatchRuntime("Map values are not comparable with == / !=")`, and `foundation_stdlib_v0.md` "makes no observable map-order promise". The ascending order is visible only through the Rust API (`SemMap::iter`, `Debug`).
5. **Accounting.** No instruction gains or loses a Steps, Calls, Frames or effect charge. `tests/shf_p1_persistent_map.rs` pins Steps/Calls/Frames/effects to the exact values measured on `main` @ `94c5c50`.
6. **Verifier-first, capabilities.** Untouched: no opcode, SemCode revision, verifier rule or capability change.
7. **Send + Sync.** `SemMap` holds only `Arc`s of `Send + Sync` data. The existing compile-time guard on `Value` keeps covering it.

## 6. Public API change (OWNER APPROVED — Policy B)

The OWNER has formally approved **Policy B** (SHF-P1 final contract decision):
- Canonical representation is `Value::Map(SemMap)` in `crates/sm-vm/src/semcode_vm.rs`.
- Single-index persistent AVL tree with `Arc`-shared nodes.
- Keys are unique and canonically ordered by `MapKey::Ord`.
- `FromIterator` implements last-write-wins semantics.
- Rust iteration and `Debug` yield ascending `MapKey` order.
- Content-based equality preserving nested `F64(NaN)` non-reflexivity without pointer-identity shortcutting.
- Preserved Semantic program behavior and deterministic accounting.

`tests/golden_snapshots/public_api/sm_vm_semcode_vm.txt`:

```diff
+pub use crate::sem_map::{Iter as SemMapIter, SemMap};
-Map(Vec<(MapKey, Value)>),
+Map(SemMap),
```

New snapshot `tests/golden_snapshots/public_api/sm_vm_sem_map.txt`, registered in `tests/public_api_contracts.rs`, covering:
- `SemMap::{new, len, is_empty, get, contains_key, insert, iter}`;
- the `Iter` type;
- `Clone`, `Default`, `PartialEq` (content equality, independent of tree shape), `Debug` and `FromIterator<(MapKey, Value)>`.

**Migration** for out-of-tree Rust code:
- `Value::Map(vec)` → `Value::Map(vec.into_iter().collect())` (last-write-wins for duplicates);
- for legacy duplicate-bearing vectors where first-match was relied upon: `Value::Map(vec.into_iter().rev().collect())`;
- `pairs.iter()` → `map.iter()`, which yields `(&MapKey, &Value)` in ascending key order;
- destructuring: match `Value::Map(ref map)` and query via `SemMap::{get, contains_key, iter, len}` instead of slice indexing;
- equality: `Value::PartialEq` now compares by content (identical bindings compare equal regardless of original insertion sequence);
- diagnostic formatting: `Debug` formats pairs in ascending `MapKey` order.

No other workspace crate constructs or destructures `Value::Map`.

### 6.1 Behaviour matrix (Semantic-visible vs Rust-visible)

| Behaviour | Before (`Vec<(MapKey, Value)>`) | After (`SemMap`) | Semantic-visible? |
|---|---|---|---|
| `MAP_SET` on an existing key | removes every matching pair, appends the new pair | replaces the binding | no difference: lookup result identical |
| `MAP_GET` hit / miss | first matching pair / default register | the single binding / default register | identical |
| `MAP_GET` default register | read only on a miss | read only on a miss (same `match` arm) | identical; an uninitialized default register still errors only on a miss |
| Operand decode order, key conversion, errors | `map_key_from_value` after the map type check | unchanged, line by line | identical |
| Snapshot isolation (old versions, arguments, record fields) | deep copies | structural sharing; shared nodes never mutated | identical (tests) |
| Duplicate keys | representable when built by Rust code; never produced by the VM | not representable; `FromIterator` is last-wins; `.rev().collect()` reproduces the old first-match lookup | not reachable from Semantic |
| Rust iteration / `Debug` order | last-write order (replacement moves to the end) | ascending `MapKey` order | observable only in Rust diagnostics: any error message that formats a map will display pairs in ascending key order |
| Rust `Value::PartialEq` on maps | pairwise in vector order: same bindings in a different insertion order compare **unequal** | bindings in key order: same bindings compare **equal** regardless of insertion order | not observable: `value_eq` on maps is `TypeMismatchRuntime("Map values are not comparable with == / !=")`, unchanged |
| NaN values | `F64(NaN)` makes map equality non-reflexive | unchanged: element-wise `Value::eq`, no `Arc::ptr_eq` shortcut | test `nan_values_keep_equality_non_reflexive` |
| Ordinal / order index | none | none (no replacement-order index, so no ordinal-exhaustion policy is needed) | — |

**Diagnostic formatting note.** `Debug` renders as a list of `(key, value)` pairs in ascending key order instead of insertion/last-write order. Consequently, in any diagnostic or error-formatting path where an invalid operation (e.g. malformed verifier-admitted bytecode passing a map as a key in `map_key_from_value`) formats a map into an error string, the formatted pairs inside that error text will reflect ascending `MapKey` order rather than the previous insertion sequence. Complete text equivalence is therefore not claimed for diagnostic strings that embed formatted maps; however, the structured error kind, error classification, and message framing remain identical.

### 6.2 Downstream callers

No other workspace crate constructs, destructures or iterates `Value::Map`. The only uses are the four handlers in `sm-vm`, plus `value_eq` and the ABI conversion, which match `Value::Map(_)`. Out-of-tree Rust code is source-incompatible at three points:
- construction: `Value::Map(vec)` → `Value::Map(vec.into_iter().collect())`, or `.rev().collect()` to keep first-match semantics for a vector with duplicates;
- pattern matching on the `Vec`: use `SemMap::{get, contains_key, iter, len}` instead;
- reliance on insertion-order equality or iteration order.

This is a forward-only change at the public enum payload, gated by the OWNER contract decision.

## 6A. Relation to the Codex drafts #2041 and #2042

**#2041** (SEM-RVM-001 architecture research) recommends a structurally shared deterministic balanced ordered tree. This PR implements that family, with these differences:

| | #2041 recommendation | This PR |
|---|---|---|
| Structure | persistent B-tree (prototype: `im` 15.1.0, external) | in-tree persistent AVL, std only: no dependency change (dependency changes are forbidden by the envelope) |
| Replacement order | retained via a second persistent tree keyed by an update ordinal (checked `u64`, panics at exhaustion in the probe) | not retained. Rust iteration is key order. No ordinal, so no exhaustion policy |
| Duplicates | duplicate-preserving import requested as an open decision | duplicates not representable; last-wins import plus a documented first-match migration path |
| Equality | ordered logical pairs, no root shortcut | key-ordered pairs, no root shortcut (NaN test) |
| Evidence | storage prototype only, sizes 16/128/1024; explicitly no VM integration | integrated in the VM; end-to-end `smc run` A/B on 50 + 3 workloads up to 128k entries; counters identical |

- **Advantages of AVL here:**
  - no dependency;
  - a small auditable implementation (about 260 lines including documentation, plus tests);
  - an O(1) clone;
  - simple invariants checked in tests.
- **Disadvantages:**
  - a binary tree has more nodes and pointer hops than a B-tree, so lookups are cache-unfriendly at large n;
  - an insert allocates one node per level when the path is shared.
- **Remaining risks:**
  - The OWNER may require insertion-order iteration for Rust callers. That would need an order index, which reopens #2041's ordinal question.
  - Memory for many retained versions is O(versions × log n) extra nodes. This was not measured here beyond peak RSS.

**#2042** (compatible copy elision) borrows the map register (`get_reg_ref`) in `MAP_GET`/`MAP_CONTAINS`/`MAP_SET` and clones surviving pairs once.
- With this PR, a map clone is one `Arc` increment, so the borrowed read saves only a refcount operation. `MAP_SET` must own a handle anyway in order to path-copy.
- #2042's map-handler changes edit the same lines and are **superseded** by this PR if both are accepted. Whichever lands second needs a trivial conflict resolution in favour of `SemMap`.
- #2042's test `map_update_preserves_order_duplicates_and_nested_isolation` asserts `Vec` payload semantics and would need to be retired with it.
- The generic `get_reg_ref` helper is independent of map storage and is worth retaining separately. Borrowed reads for `RECORD_GET` and other read-only consumers are relevant to P2 (#2046).

This PR does not modify #2041 or #2042.

## 7. Evidence

Native Linux x86-64 (Intel Xeon @ 2.10 GHz, 4 vCPU, container), rustc 1.97.1, release. `smc run … --envelope trusted-compiler --max-steps 200000000 --metrics json`. Self-asserting workloads: the result is checked by `assert`, so `ok` means semantically correct. 3 rounds, alternating order. Base = `main` @ `94c5c50`. Times are `nondeterministic.wall_ms` (compile + verify + execute), informative only.

**Deterministic equivalence.** 50 workloads × 3 rounds × 2 builds. Outcome, Steps, Calls, peak Frames, effect calls and quota fields are identical: **0 mismatches**.

| Workload (n = 16 000) | Steps | base | P1 | base / P1 | peak RSS base → P1 |
|---|---:|---:|---:|---:|---|
| `map_set`: `m = map_set(m, i, i)` | 224 017 | 14 615 ms | 32.5 ms | ×450 | 25.3 → 13.1 MB |
| `map_get`: build, then `map_contains` + `map_get` per key | 624 023 | 34 530 ms | 60.5 ms | ×571 | 22.3 → 12.9 MB |
| `map_versions`: build, keep the old version, overwrite all keys | 480 035 | 50 638 ms | 71.1 ms | ×713 | 30.0 → 14.0 MB |
| `state_record_map`: `c = c with { syms: map_set(c.syms, text, id) }` | 400 034 | 111 682 ms | 58.6 ms | ×1 904 | 40.6 → 13.2 MB |
| non-map controls (`seq_push`, `seq_read`, `record_with`, `text_scan`, `lex_calls`, `state_record_seq`) | — | — | — | ×0.87–×1.49 (noise) | unchanged |

**Small-map control** (5 rounds; 20 000 iterations × (k `map_set` + k `map_get`) on a fresh map): k = 2: ×1.05 (noise), k = 8: ×1.26, k = 32: ×1.99. Counters are identical, so there is no small-map regression.

**Scaling of P1** (1 round, n = 64k → 128k; Steps exponent 1.00 everywhere):

| Family | n = 128 000 | time exponent |
|---|---:|---:|
| `map_set` | 333 ms | 1.14 |
| `map_get` | 542 ms | 0.98 |
| `map_versions` | 613 ms | 0.83 |
| `state_record_map` | 527 ms | 0.81 |

Base exponents for the same map families were 1.9–2.0. `state_record_seq` (P2) is unchanged by design (×0.92–×1.41, noise).

**Gates.**
- `cargo test -p sm-vm --lib`: 197 passed, including 9 new `sem_map` tests.
- `tests/shf_p1_persistent_map.rs`: counters pinned to `main`.
- `public_api_contracts`: snapshot diff limited to §6.
- `cargo fmt --check` and `clippy -p sm-vm --all-targets -D warnings`: clean.
- Workspace results are reported in the PR.

## 8. Limitations and non-goals

- **No aggregate heap bound** is claimed, as for every `trusted-compiler` execution (`docs/spec/quotas.md`). Step quotas limit only instruction count and call frame depth, and do not establish a general or aggregate memory bound. Persistence keeps old versions alive while they are referenced: memory is O(live versions × log n) extra nodes, not O(n) per version.
- Map element reads (`MAP_GET`) still clone the stored `Value`. That is O(size of that value), as before.
- **Not changed:** sequences (SHF-P3), records (P2, separate design), map iteration/removal/length (not admitted by the language), quotas, and the CLI.
- Constant factors: for a uniquely owned map, a per-insert path copy allocates about log₂ n small nodes. A later, separately qualified extension of SHF-P3 consumption to `MAP_SET` could make that path unique and update it in place; it is not required to remove the asymptotic bottleneck and is out of scope.
