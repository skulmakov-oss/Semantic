# SEM-RVM-001: persistent Map storage proposal

Status: architectural proposal for OWNER CONTRACT REVIEW, not production
implementation authorization or merge authorization. Base:
`94c5c50d16c3f2efc7691456755461bedae83b18`.

## Owner decision and boundaries

Option 2 authorizes architectural evaluation and a separate reviewable PR.
It does not authorize silently breaking `Value::Map(Vec<(MapKey, Value)>)`,
updating public API snapshots, adding production dependencies, or changing
SemCode, verifier admission, capabilities, Calls or heap/quota contracts.
The compatible copy-elision implementation is a separate PR; it does not
eliminate linear lookup or full-copy persistent updates.

## Current obligations

The normative `semantic.foundation.std/0.1` selects persistent Map values,
exact same-family i32/u32/bool/text/quad keys, explicit defaults, and no
language iteration, equality or ordering API. MAP_SET returns an independent
value. MAP_GET ignores its default register on a hit. Diagnostics, operand
validation order and verifier-first execution must stay unchanged.

The Rust API has additional observable behavior: callers construct/destructure
the Vec payload; `Value::PartialEq` compares pair order; SET removes all matching
duplicates and appends the new pair; GET selects the first matching pair.
Externally constructed duplicate pairs are possible. These obligations are
not erased merely because ordinary Semantic source produces unique-key Maps.
Floating-point NaN values make `Value::PartialEq` non-reflexive: comparing
shared roots cannot substitute for comparing ordered values.

## Alternatives

| Model | Lookup | Shared persistent update | Clone | Main limitation |
|---|---|---|---|---|
| Current Vec | O(n) key comparisons | O(n) owned pair/value copying | O(n) | Quadratic repeated construction |
| Arc<Vec> | O(n) | O(n) on shared update | O(1) | Sharing alone does not fix search/update |
| Arc<BTreeMap> | O(log n) comparisons | O(n) tree copy when shared | O(1) | Does not meet full-copy elimination goal |
| Structurally shared balanced ordered tree | O(log n) comparisons | O(log n) path copies for unique keys | O(1) | Node/order-index overhead, comparison cost |
| Deterministic HAMT | Shallow hash-trie traversal normally | Copies affected trie path normally | O(1) | Full collisions can require O(n); fixed hashing exposes collision risk |

Text comparison/hashing has byte-length cost. These bounds do not promise
constant-time handling of arbitrarily long keys or cloning returned payloads.
No randomized iteration may be exposed. A HAMT requires explicit canonical
key encoding and an independent order representation if Rust order is retained.

## Isolated evaluation

The prototype uses `im = 15.1.0` outside the Semantic workspace: persistent
B-tree and HAMT implementations, plus std Arc<BTreeMap> and Vec controls.
No production dependency or repository manifest was changed. Evaluation of
one library is not approval to adopt it; maintenance, license/transitive
dependency review and a dependency OWNER GO remain necessary before adoption.

Source: `tests/fixtures/rvm001_storage/prototype.rs`.
Raw data: `tests/fixtures/rvm001_storage/storage.csv`.
Lock data: `tests/fixtures/rvm001_storage/prototype.lock`.

All models use the same immutable `Arc<MapKey>` and `Arc<Value>` payloads.
The benchmark shares a single 256-byte text value across entries, deliberately
isolating container/storage cost; it does not measure arbitrary deep nested
payload workloads. Both persistent prototypes and Arc<BTreeMap> include a
second persistent tree keyed by update ordinal to retain replacement-at-end
order. Vec retains order directly. Order-index costs are included.

Windows x86-64, Intel Core i5-9300H, Rust 1.97.1, release, sizes 16/128/1024,
nine samples per case/model, models interleaved by sample. Counters are
instrumented allocation requests, not RSS. Default GlobalAlloc reallocation
behavior is included. Differential checks cover all five key families,
100 updates, previous snapshots and ordered output; an additional 128-key
constant-hash test verifies HAMT collision handling. Ordered equality with
NaN is checked without a shared-root shortcut.

At 1024 entries (instrumented medians; raw ranges remain in CSV):

| Case | Vec | Arc<BTreeMap> | Persistent B-tree | HAMT |
|---|---:|---:|---:|---:|
| Build 1024 entries, ms | 26.680 | 42.362 | 7.699 | 6.545 |
| 1000 last-key lookups, ms | 4.319 | 0.435 | 0.196 | 0.033 |
| 100 independent insertions, ms | 5.274 | 8.219 | 0.860 | 0.686 |
| 100 independent replacements, ms | 4.359 | 8.339 | 1.005 | 0.844 |
| 1000 clones, ms | 39.296 | 0.036 | 0.035 | 0.056 |
| Build retained container bytes above setup | 49104 | 111576 | 126240 | 400584 |
| Extra bytes while retaining 100 updated versions | 2433072 | 5009400 | 833184 | 831600 |

These are storage-prototype results, not VM speedup factors. Persistent storage
reduces copied work and retained-version amplification here, but costs more
for a single live Map. HAMT's base container is particularly larger in this
implementation. Small-map results must be considered separately; there is
no unconditional claim that persistent structures beat a short Vec.

## Actual VM/compiler evidence and its limits

The separate compatible implementation benchmark runs identical verifier-admitted
SemCode on pristine/candidate VMs. It includes compiler-generated Map construction
loops, increasing-size read/update workloads and the independently authored
Semantic SemSeed lexer/parser/checker/interpreter. Its actual O0 artifact is
`edad5ca66ea2cb668211498f2cccffa87bbe3a09f1c74e649e4005fed616d9e2`.
For the assignment program returning 14, both executions report Steps 11785,
Calls 344, peak Frames 8, EffectCalls 2. SemSeed timings are INCONCLUSIVE;
allocations change only from 31463 to 31411.

The storage prototypes have not replaced the production VM or executed its
full compiler pipeline. Synthetic symbol-key/model workloads cannot establish
the end-to-end benefit of the future payload change. Actual compiler and VM
integration measurements, nested aggregates, snapshot-heavy workloads, and
small-map controls are mandatory acceptance gates for a later implementation.
This evidence limitation is explicit, rather than presenting prototype timing
as successful production integration.

## Recommended architecture for contract review

Prefer a structurally shared deterministic balanced ordered tree as the initial
design, with immutable shared keys/values and a persistent order index. It
eliminates whole-container copying and avoids an attacker-controlled hash
collision boundary. HAMT remains a measured alternative, but requires a
separate collision/resource argument; the probe's fixed FNV hash is deliberately
not a production security recommendation.

An encapsulated `MapValue` would own private indexes and expose functional
`get`, `contains`, `set`, ordered import/export and iteration for Rust callers.
Production equality must compare ordered logical pairs without root-identity
shortcuts. Values can be shared internally but returned owned values must
retain existing copy-by-value semantics, including nested Map/Record/ADT/Tuple.

The order representation is still a design review obligation. The probe uses
checked u64 ordinals and panics at exhaustion; that is NOT an approved production
error or quota policy. Duplicate-preserving import requires recording every
ordered occurrence, first-hit lookup and removing all occurrences on SET.
Deleting k duplicate occurrences can legitimately cost O(k log n); the O(log n)
update claim is limited to canonical unique-key Maps. No canonicalization,
rejection of old Rust inputs or ordinal-overflow behavior is silently approved.

## Public API change and migration

`Value::Map(MapValue)` changes construction and destructuring at a public enum
payload boundary. It is source-breaking even if Semantic source behavior is
identical. Adding another public Value variant also affects exhaustive matches.
Neither change is implemented here. Merely adjusting API snapshots would hide
the break and is prohibited.

Before implementation, OWNER must explicitly approve:

1. The new payload/API and a forward-only compatibility/migration policy.
2. Exact duplicate import, ordered equality/export and ordinal semantics.
3. Dependency adoption, if a library is chosen; the active dependency boundary
   currently forbids production manifest changes.
4. Any downstream inventory/migration changes requiring a separate scope.

A nonbreaking private internal-value representation with legacy Value conversion
at public boundaries is an alternative, but public Frame locals, arguments and
aggregate carriers make it a broader VM refactor. Materializing at every local
or argument boundary would surrender the performance gain. That broader refactor
is not silently introduced in this proposal.

## Later implementation acceptance gates

- Same admitted SemCode bytes, verifier rules, capabilities, outcomes and quota
  counters; error ordering and default-register laziness unchanged.
- All key families, Unicode/long text keys, duplicate public inputs, NaN,
  nested aggregates, register aliases, locals/arguments/captures and Send+Sync.
- Persistent value isolation and exact ordered pair export/reference comparison.
- Actual compiler/VM end-to-end measurements, allocation/live-memory accounting,
  multiple sizes and retained versions, matched release builds and raw ranges.
- Fresh adversarial review, current public API qualification, hosted CI and
  separate OWNER MERGE GO. No quota-policy shortcut to better measurements.

## Reproduction of the isolated probe

Create an external Cargo project (outside the Semantic repository). Copy the
prototype to `src/main.rs` and lock data to `Cargo.lock`. Use a Cargo manifest
with package name `rvm-storage-prototype`, version `0.0.0`, edition `2021`,
`im = "=15.1.0"`, and `sm-vm = { path = "<this checkout>/crates/sm-vm" }`.
Run `cargo +1.97.1 run --locked --release`; assertions must pass before the
CSV is interpreted. Do not copy that manifest into Semantic or change its
dependencies to run this probe.
