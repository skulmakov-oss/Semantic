# SEM-RVM-002: concrete persistent Map contract proposal

Status: OWNER CONTRACT REVIEW. No production payload replacement, dependency
change, quota change or merge is authorized by this document.
Reference main: `94c5c50d16c3f2efc7691456755461bedae83b18`.
Compatible predecessor: PR #2042, `8db588bea6395839eb543aee453c2b2657e856d5`.

## Recommended implementation

Use a focused in-tree persistent balanced ordered tree with immutable Arc nodes.
The experimental implementation uses AVL balancing, without new dependencies.
Two indexes share `Arc<MapKey>` and `Arc<Value>` entries:

1. Key index: MapKey -> ordered identifiers for occurrences of that key.
2. Order index: identifier -> (shared key, shared value).

Canonical unique-key Maps have one occurrence per key. An update removes the old
order entry, inserts a new last entry, and copies only affected balanced paths.
Old roots remain immutable. Clone shares two roots in O(1). All fields stay
private; no mutable pair/value references escape. Arc supports Send + Sync when
the existing Value and MapKey satisfy those properties.

AVL is a specific implementation candidate, not a promise that the short
experimental implementation is production-qualified. Production code must test
balance/height invariants, deletion rotations, drop behavior, duplicate imports,
ordinal compaction and error ordering. Avoid optional storage backends or a
small-map hybrid until real workloads justify them.

For n unique keys: lookup/contains O(log n) key comparisons; SET O(log n) path
updates plus key/value ownership costs; logical clone O(1). Construction is
O(n log n) node work. Comparing text keys can examine O(L) bytes per comparison;
building a key from a VM register still incurs its existing clone cost. GET
clones the selected Value as required, which can copy owned nested payloads.
Import/export/equality are not logarithmic operations.

## Exact proposed Rust API

These signatures are proposed, not landed:

```rust
#[derive(Clone, Default)]
pub struct MapValue { /* private roots and ordering state */ }

impl MapValue {
    pub fn new() -> Self;
    pub fn from_pairs(pairs: Vec<(MapKey, Value)>) -> Self;
    pub fn get(&self, key: &MapKey) -> Option<&Value>;
    pub fn contains_key(&self, key: &MapKey) -> bool;
    pub fn updated(&self, key: MapKey, value: Value) -> Self;
    pub fn len(&self) -> usize; // number of ordered pairs, including duplicates
    pub fn is_empty(&self) -> bool;
    pub fn iter_pairs(&self) -> impl Iterator<Item = (&MapKey, &Value)> + '_;
    pub fn to_pairs(&self) -> Vec<(MapKey, Value)>;
    pub fn into_pairs(self) -> Vec<(MapKey, Value)>;
}
impl From<Vec<(MapKey, Value)>> for MapValue { /* preserves all pairs */ }
impl PartialEq for MapValue { /* ordered logical comparison */ }
impl std::fmt::Debug for MapValue { /* list of ordered pairs */ }

pub enum Value {
    // all other variants unchanged
    Map(MapValue),
}
```

`iter_pairs` order is import order followed by replacement-at-end updates.
`get` returns the first matching occurrence. `updated` removes every occurrence
of that key and appends its replacement. Duplicate imports are preserved,
not silently canonicalized. For k duplicates, removing occurrences costs
O(k log n); an Arc<Vec<identifier>> occurrence-list import can cost O(k²) for
a pathological all-duplicate input, so production import must group occurrences
in one pass before building immutable lists. Unique-key construction retains
the stated bound. No mutable iterator, DerefMut or IndexMut is proposed.

Equality compares ordered key/value pairs even when roots are identical.
Nested NaN remains non-reflexive. Debug should preserve the existing ordered
pair display shape; its traversal/materialization cost is diagnostic, not a
lookup guarantee. `into_pairs` may clone shared Values: no zero-copy export
promise is made.

## Ordinal exhaustion without a new runtime error

Proposed policy: before allocating the last exhausted u64 identifier, compact
the current order to consecutive identifiers starting at zero, rebuild both
indexes preserving every pair, then perform the update. No silent wrap, panic
policy, stale identifier or changed observable order is permitted.

On currently tested 32/64-bit targets a Vec of nonzero-sized Map pairs cannot
contain u64::MAX entries; the compaction count therefore fits below exhaustion.
Compaction through the current prototype's `to_pairs` also clones owned nested
payloads; its cost includes total payload bytes, not merely the number of nodes.
Production code should rebuild indexes from shared entries to avoid those clones.
Production code must make the supported-width/count argument explicit and test
the boundary with a small artificial identifier limit. Rebase is exceptional
O(n) or O(n log n), not an O(log n) worst-case SET guarantee. It changes physical
storage only; old snapshots keep their roots. This policy requires explicit
OWNER contract acceptance. It does not create a new guest error/quota default.

## Migration and compatibility

`Value::Map(Vec<...>)` -> `Value::Map(MapValue)` is source-breaking. Associated
helper constructors cannot preserve old enum construction or Vec destructuring.
Do not conceal the break by changing snapshots alone.

Example migration:

```diff
- let value = Value::Map(pairs);
+ let value = Value::Map(MapValue::from_pairs(pairs));

  if let Value::Map(map) = value {
-     for (key, value) in map.iter() { /* read */ }
+     for (key, value) in map.iter_pairs() { /* read */ }
  }

- let Value::Map(mut pairs) = value else { return; };
- pairs.push((key, new_value));
+ let Value::Map(map) = value else { return; };
+ let updated = map.updated(key, new_value);
```

The last example is appropriate only when the caller intends SET semantics.
Arbitrary Vec edits must migrate via explicit `into_pairs`/`from_pairs` to retain
duplicates and order. Exporting and reimporting every VM local/argument would
restore quadratic cost and is not the internal execution design.

Tracked repository inventory at #2042 found direct Map construction/destructuring
only in `crates/sm-vm/src/semcode_vm.rs` and its `examples/map_cost.rs`; public
contract snapshots also record the payload. Numerous sm_vm consumers use other
variants or wildcard matches. This inventory covers this repository only, not
unknown external Rust users. Before production migration, rerun the full source
inventory and API golden tests; update the sm-vm re-export, exact snapshot and
affected constructors together in a reviewed forward-only API change.
The current `pub use semcode_vm::*` already routes runtime exports; the new
MapValue must be publicly exported from that module. Snapshot targets are
`tests/golden_snapshots/public_api/sm_vm_semcode_vm.txt` and, if export routing
changes, `sm_vm_lib.txt`. Add a dedicated MapValue signature snapshot/test so
the new private module's public methods are not omitted by the existing parser.

A source-compatible private InternalValue engine is feasible in principle:
convert legacy Values only at public boundaries and retain persistent Maps
through registers, locals, aggregates, captures, arguments and returns. It is
a broader engine refactor because Frame.locals and aggregate carriers expose
Value. A register-only cache does not solve local-heavy map_set loops. No
claim of impossibility is made; this alternative trades a larger maintained
internal/public conversion boundary for preserving external Rust syntax.
The smallest focused proposal is the explicit MapValue payload migration.

## Dependency assessment

The evaluated `im = 15.1.0` remains a research dependency only. Upstream
`bodil/im-rs` is archived (GitHub records archival May 3, 2026; API confirms
archived=true, last push August 19, 2024). License metadata is MPL-2.0/MPL-2.0+.
The crates.io API records 15.1.0 publication on April 29, 2022.
The installed package declares Rust 1.46 and edition 2018; the probe builds
with the repository's Rust 1.97.1. Locked direct/transitive support includes
bitmaps 2.1.0, sized-chunks 0.6.5, typenum, rand_core, rand_xoshiro and
version_check. A crate-wide dependency must not be assumed deterministic merely
because its ordered-map API is deterministic; audit the actually used tree
paths, allocator behavior and thread-safe Arc mode separately.

Primary source: https://github.com/bodil/im-rs . The package has optional serde,
rayon, refpool and testing dependencies; none are a production requirement here.
This evaluation is not a security vulnerability audit or license clearance.
Semantic's VM module currently compiles under `#[cfg(feature = "std")]`; the
crate itself retains its `cfg_attr(..., no_std)` configuration without that
module. Put the new MapValue module behind the same std boundary and preserve
`cargo check --no-default-features`. This does not promise a no_std VM. In-tree
Arc storage preserves current VM std/allocator requirements.

Recommendation: avoid a new archived production dependency; qualify the small
in-tree ordered tree against the existing prototype and Vec oracle. This adds
maintenance responsibility for balancing/deletion and must be reviewed honestly.
No Cargo manifests or locks are part of the recommended first implementation.

## Qualification and next bounded work

Actual VM experiment changes the payload only in a disposable Linux clone,
without committing or pushing the breaking implementation. Identical admitted
compiler-generated SemCode executes with explicit identical Steps budget
2,000,000, all other VerifiedLocal quotas unchanged, no capabilities or host
effects. Source generation, binaries, patch, raw timings, counters and artifact
hashes are retained as experiment provenance. The instrumentation allocator
counts requested bytes and extra live allocations, not RSS or production wall
speed. Concurrent build load and only three interleaved samples limit timing
precision. Consult the accompanying evidence before making scaling claims.
Dynamic guest instruction counts in these successful runs equal recorded Steps:
the VM charges one Step after each opcode decode before dispatch. Calls remain
zero in the pure loops. The full differential acceptance matrix is still pending
the production contract decision; successful loop assertions are not that matrix.

Full production acceptance still requires the complete VM differential matrix:
all key families, missing/default behavior, duplicate external arguments,
register overlaps, nested aggregates/NaN, snapshot function arguments/returns,
long keys, error/quota boundaries, exact Steps/Calls and verifier admission.
No new wire format, opcode, capability, verifier rule or quota default is needed.

Required OWNER CONTRACT GO is narrow: approve the exact MapValue public payload,
constructors/read access/conversions, duplicate/order/equality policy, exhaustion
compaction and forward-only snapshot/downstream migration. No dependency GO is
requested for the recommended in-tree option. Then implement a separate focused
PR touching sm-vm storage/opcodes/tests, exact public API snapshots and normative
contract evidence within the existing Harness. Keep #2042 and #2041 unmerged.

Record+Sequence is a separate subsequent investigation: verify moving a nested
field only with proven ownership, preserving OWN0 borrow/write rejection,
aliases, old snapshots and failure behavior. No implementation is folded into
the Map proposal or prototype.
