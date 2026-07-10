# Quad Logic Engine v1 Compatibility & Rollout Policy

## 1. Ownership Boundary
* **canonical owner:** `semantic-core-quad`
* **compatibility-only owner:** `ton618-core`
* **qualification consumer:** `semantic-core-capsule`

## 2. Compatibility Dimensions
The policy distinguishes the following compatibility dimensions. An item may be semantically and source-compatible while its ABI/layout remains unqualified.
* **source/API compatibility:** The ability to compile against the public API without breakage.
* **semantic compatibility:** The logical execution and behavior remaining correct according to specifications.
* **binary ABI/layout compatibility:** The in-memory or on-disk byte representation of data structures.
* **serialized-data compatibility:** The ability to correctly serialize/deserialize data across versions or environments (e.g. via `serde`).

## 3. Public API Registry

### `QuadState`
* **semantic contract:** stable
* **binary ABI/layout:** numeric representation or ABI must only be claimed if already explicitly qualified elsewhere.

### `QuadroReg32`
* **source/API and lane semantics:** stable
* **binary ABI/layout:** not independently frozen by this policy unless backed by an explicit representation contract.
* **forbidden silent change:** silent semantic reinterpretation remains forbidden.

### `QuadMask32`
* **status:** migration-candidate (ambiguous u64 mask)
* **v1 guarantee:** strict isolation
* **migration note:** will be split into physical vs logical mask types in upcoming PRs.

### `QuadTile128`
* **semantic contract:** lane count and semantic behavior are stable.
* **binary ABI/layout:** alignment and binary/GPU transport layout are under review in #1417.
* **allowed change:** an explicit qualified layout change is permitted through #1417.
* **forbidden silent change:** silent layout mutation remains forbidden.

### `QuadMask128`
* **status:** migration-candidate (ambiguous u128 mask)
* **v1 guarantee:** strict isolation
* **migration note:** will be split into physical vs logical mask types in upcoming PRs.

### `StateDelta32`
* **status:** migration-candidate (diff of two 32-lane regs)
* **migration note:** will split into exact-state and plane-delta types in upcoming PRs.

### `StateDelta128`
* **status:** migration-candidate (diff of two 128-lane regs)
* **migration note:** will split into exact-state and plane-delta types in upcoming PRs.

### `QuadroBank<N>` and `QuadTileBank<N>`
* **semantic contract:** container and indexing semantics are preserved.
* **binary ABI/layout:** not declared frozen by this policy.
* **allowed change:** layout changes require explicit qualification and regression evidence.

## 4. Additive-first rollout policy
Early v1 changes are strictly bound to an additive-first approach. They may add:
* typed mask wrappers
* new exact-state delta types
* new plane-delta types
* new tile truth-map APIs
* new bank helpers
* qualification tests

They must not *initially* remove or silently redefine existing APIs. Removal or renaming of legacy APIs is only permitted through an explicit, qualified breaking-change process (see Section 9).

## 5. Ambiguous-name policy
* `QuadMask32`: Currently an ambiguous raw mask. Will migrate to separate dense lane masks and physical packed masks.
* `StateDelta32`: Currently a raw diff mask. Will migrate to separate exact-state events and plane events.
* `entered_true` / `left_true`: Currently vague exact/plane boundaries. Will migrate to explicit plane event APIs.
* `raw_delta`: Currently raw bitwise XOR. Will be migrated or isolated to prevent ambiguous event semantics.
* `map_*`: truth-table operations.
* `join` / `meet` / `inverse`: knowledge-lattice operations.
Do not mix truth-table outputs with knowledge-lattice outputs under the same API family.

## 6. Feature guarantees
* `std`: currently verified (default feature flag)
* `no_std`: check-qualified for the current crate under `cargo check -p semantic-core-quad --no-default-features`; full target/runtime qualification is not claimed.
* `serde`: feature compilation verified via `--all-features`, but full round-trip semantic compatibility is not yet formally qualified.

## 7. Core capsule qualification path
Minimum checks required after each `core-quad` rollout PR:
* `cargo test -p semantic-core-quad --quiet`
* `cargo check -p semantic-core-quad --no-default-features`
* `cargo test -p semantic-core-quad --all-features --quiet`
* `cargo test -p semantic-core-capsule --quiet`

*(Note: Passing `semantic-core-capsule` tests provides baseline downstream evidence, but does not prove total compatibility for all external consumers. It acts as an initial integration invariant.)*

## 8. PR Sequence
The controlled merge order:
1. compatibility policy
2. typed mask bridge
3. delta split
4. core tile layout decision
5. tile truth maps
6. bank helpers
7. qualification and benches
8. optional default-backend decision in a separate PR

## 9. Breaking-change rule
Any future change that modifies:
* state encoding
* mask interpretation
* delta meaning
* tile layout
* truth-map output
* legacy VM/source semantics

must require:
* dedicated issue
* explicit compatibility classification
* migration note
* golden/regression evidence
* separate PR
