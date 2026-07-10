# Quad Logic Engine v1 Compatibility & Rollout Policy

This document is the normative compatibility source of truth for the v1
rollout. Landed on `main` means implemented and evidenced; it does not widen
the stable or serialized-compatibility promise.

## 1. Ownership boundary

- **Canonical owner:** `semantic-core-quad`
- **Compatibility-only owner:** `ton618-core`
- **Qualification consumer:** `semantic-core-capsule`
- **GPU/visual transport owner:** tracked separately by #1417

Compatibility dimensions remain distinct: source/API compatibility, semantic
compatibility, binary ABI/layout compatibility, and serialized-data
compatibility.

## 2. Current v1 registry

### `QuadState`

- The public enum remains stable.
- The encoding is frozen: `N = 00`, `F = 01`, `T = 10`, `S = 11`.
- No encoding migration is authorized.

### `QuadroReg32`

- The public type and 32-lane packed representation remain stable.
- Scalar operations remain the correctness oracle.
- Default `map_*` APIs currently route through the qualified SWAR implementation.
- Lattice APIs remain separate from truth-map APIs.
- No EQUIV API is exposed.

### Mask model

The landed compatibility bridge is:

```text
QuadMask32 / QuadLaneMask32
    dense logical lane mask compatibility surface

QuadPhysicalMask32
    validated packed physical mask
```

- `QuadMask32` remains available for compatibility.
- `QuadLaneMask32` is an additive alias.
- `QuadPhysicalMask32` rejects odd physical bits.
- Conversion between dense and physical masks is explicit.
- No legacy public item was removed.

The same additive-first posture applies to the existing `QuadMask128`
compatibility surface; its interpretation must not change silently.

### Delta model

The landed explicit types are:

```text
PlaneDelta32
ExactStateDelta32
StateDelta32
StateDelta128
```

- `PlaneDelta32` owns truth-plane/falsity-plane membership transitions.
- `ExactStateDelta32` owns exact N/F/T/S transitions.
- `StateDelta32` remains a compatibility structure with documented mixed semantics.
- `StateDelta128` remains a compatibility structure for 128-lane deltas.
- Ambiguous legacy names are retained, not silently redefined.

Names such as `entered_true`, `left_true`, and `raw_delta` remain compatibility
surfaces until a dedicated semantic decision and migration are approved.

### `QuadTile128`

The qualified core representation is:

```text
#[repr(C, align(16))]
size: 32 bytes
alignment: 16 bytes
truth-plane offset: 0
falsity-plane offset: 16
```

This is the canonical CPU/core semantic-storage layout. It is not the GPU
upload ABI. No Pod, byte-cast, WGPU, WGSL, or serialized-layout guarantee
follows from the core layout. GPU transport remains tracked separately by
#1417.

### Tile truth maps

The landed default tile APIs are:

```text
map_not
map_xor
map_and
map_or
map_implies
map_nand
map_nor
```

They currently use four-register decomposition as the deterministic reference
lifting path.

### Bank truth maps

The landed in-place helpers for both `QuadroBank<N>` and `QuadTileBank<N>` are:

```text
map_not_inplace
map_xor_inplace
map_and_inplace
map_or_inplace
map_implies_inplace
map_nand_inplace
map_nor_inplace
```

Operations are elementwise, preserve ordering, perform no allocation, and
leave structures and indexing semantics unchanged.

## 3. EQUIV policy

**EQUIV remains deferred and is not part of the qualified v1 public API.**

No EQUIV truth table or API is implied here. Any future EQUIV API requires a
dedicated semantic-policy decision, explicit naming, qualification vectors,
and compatibility review.

## 4. Additive-first and breaking-change rules

The v1 rollout preserves existing public names and behavior while adding
qualified mask, delta, tile-map, bank-helper, and qualification surfaces.
Ambiguous names are documented compatibility names before any removal or
renaming. A change to encoding, masks, delta meaning, tile layout, truth-map
outputs, or compatibility names requires a dedicated issue, migration note,
and regression evidence.

## 5. Feature guarantees

- `std`: tested.
- `no_std`: check-qualified through `cargo check -p semantic-core-quad --no-default-features`.
- `serde`: compile/test-qualified under `--all-features`.
- Serialized cross-version compatibility is not claimed.

## 6. Core capsule qualification path

`semantic-core-capsule` remains the minimum downstream smoke consumer. Its
passing tests provide baseline integration evidence, not proof of total
external compatibility.

Minimum rollout checks remain:

```text
cargo test -p semantic-core-quad --quiet
cargo check -p semantic-core-quad --no-default-features
cargo test -p semantic-core-quad --all-features --quiet
cargo test -p semantic-core-capsule --quiet
```

## 7. Remaining rollout order

Qualification tests and benchmark planning are tracked by #1412. GPU transport
representation and visual adapter work remain tracked by #1417. Umbrella
roadmap closeout remains tracked by #1404.
