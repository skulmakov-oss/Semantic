# Quad Logic Engine v1 Compatibility Closeout

## Purpose

Record the compatibility policy after the v1 implementation slices landed on
`main`, without widening stable, serialized, GPU, or visual guarantees.

## Issue

- Closes #1413: compatibility rollout policy.
- Parent: #1404, the v1 roadmap umbrella.

## Starting main commit

`77c7ac9b23169e2c66a71f098f0a87ff18e13c2f`

## Landed implementation inventory

- PR #1478: compatibility, mask bridge, and delta split.
- PR #1479: qualified core `QuadTile128` layout.
- PR #1480: deterministic tile truth-map lifting.
- PR #1481: in-place bank truth-map helpers.

## Stable public items

- `QuadState` with frozen `N/F/T/S` encoding `00/01/10/11`.
- `QuadroReg32` and its 32-lane packed representation.
- Scalar truth-map operations as the correctness oracle.
- Separate truth-map and knowledge-lattice API families.
- `QuadTile128` as qualified CPU/core semantic storage: `repr(C, align(16))`,
  32 bytes, truth plane at offset 0, falsity plane at offset 16.
- `QuadroBank<N>` and `QuadTileBank<N>` container and indexing semantics.

Stable means the documented v1 contract; it does not claim GPU upload ABI or
serialized cross-version compatibility.

## Additive compatibility items

- `QuadMask32` / `QuadLaneMask32` dense logical mask compatibility surface.
- `QuadPhysicalMask32` validated packed physical mask with explicit conversion.
- `PlaneDelta32` for truth/falsity plane membership transitions.
- `ExactStateDelta32` for exact N/F/T/S transitions.
- Tile maps: `map_not`, `map_xor`, `map_and`, `map_or`, `map_implies`,
  `map_nand`, `map_nor`.
- Matching in-place helpers for both bank types.

No legacy public item was removed.

## Legacy compatibility surfaces retained

`StateDelta32`, `StateDelta128`, `QuadMask32`, and ambiguous names such as
`entered_true`, `left_true`, and `raw_delta` remain documented compatibility
surfaces. They are not silently redefined. Lattice methods remain distinct from
truth-map methods.

## Feature qualification

- `std`: tested.
- `no_std`: check-qualified through `--no-default-features`.
- `serde`: compile/test-qualified under `--all-features`.
- Serialized cross-version compatibility: not claimed.

## Core capsule smoke path

`semantic-core-capsule` is the minimum downstream smoke consumer. Its passing
tests provide baseline integration evidence, not total external compatibility.

## EQUIV deferral

EQUIV remains deferred and is not part of the qualified v1 public API. A future
EQUIV API requires a semantic-policy decision, explicit naming, qualification
vectors, and compatibility review.

## Core versus visual layout boundary

`QuadTile128` is canonical CPU/core semantic storage, not a GPU upload ABI. No
Pod, byte-cast, WGPU, WGSL, or serialized-layout guarantee follows from its
core layout. GPU transport representation and visual adapter work remain in
#1417.

## Remaining open work

- #1412: qualification matrix and benchmark plan.
- #1417: GPU transport representation.
- #1404: umbrella closeout.

None of these is complete as part of this closeout.

## Explicit non-claims

This closeout does not claim EQUIV, GPU transport, visual adapter behavior,
benchmark completion, total external compatibility, serialized cross-version
compatibility, or completion of #1412, #1417, or #1404.

## #1413 acceptance mapping

| Acceptance criterion | Concrete evidence |
| --- | --- |
| Compatibility rollout policy exists | `v1_compatibility_rollout.md` is the normative source of truth. |
| Stable public items listed | Current v1 registry and this closeout's stable-items section. |
| Ambiguous names documented | Retained-name policy for masks, deltas, and legacy event names. |
| Additive-first policy preserved | Additive inventory and breaking-change rules. |
| `no_std`/`std`/`serde` posture documented | Feature qualification sections in both policy and closeout. |
| Core capsule smoke path present | Minimum downstream smoke consumer and commands are recorded. |
