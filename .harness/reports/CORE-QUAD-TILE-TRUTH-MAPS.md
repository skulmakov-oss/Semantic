# Harness Report: CORE-QUAD-TILE-TRUTH-MAPS

## Status

- **Starting main commit:** `a17c3a86c3d2c666b015c32bf3e3d83707299707`
- **Branch:** `core-quad/tile-truth-map-lifting`
- **Implementation commit:** `bbfbf8be1ee68caae77964910dedfa7ea55bb156`
- **Pull request:** `#1480`
- **Issue slice:** first of two slices for `#1411`

## Register Map Inventory

The existing default `QuadroReg32` truth maps used as the oracle are:

- `map_not`
- `map_xor`
- `map_and`
- `map_or`
- `map_implies`
- `map_nand`
- `map_nor`

## Added Tile Map Inventory

`QuadTile128` now exposes:

- `map_not`
- `map_xor`
- `map_and`
- `map_or`
- `map_implies`
- `map_nand`
- `map_nor`

`map_equiv` remains explicitly deferred under the current v1 compatibility policy.

## Implementation Strategy

Each tile method follows the deterministic reference path:

1. convert `QuadTile128` into four `QuadroReg32` values with `to_regs()`;
2. apply the corresponding default register-level `map_*` operation;
3. reconstruct the tile with `from_regs()`.

The implementation:

- performs no allocation;
- uses no `unsafe`;
- adds no direct `u128` plane formulas;
- does not duplicate scalar truth tables;
- remains compatible with the crate's `no_std` posture.

## Test Inventory

- `tile_default_unary_map_matches_reg_oracle`
- `tile_default_binary_maps_match_reg_oracle`
- `tile_default_matrix_transitions`
- `tile_default_boundary_lanes`
- `tile128_lattice_stability`

The tests prove:

- tile NOT matches the four-register oracle;
- all six binary tile maps match the four-register oracle;
- all `4 × 4` repeated-state binary combinations match register behavior;
- register-boundary lane ordering is preserved;
- `join`, `meet`, and `inverse` retain their knowledge-lattice behavior.

## Semantic Separation

Truth maps remain distinct from:

- `join`
- `meet`
- `inverse`
- `raw_delta`

No truth-map operation was implemented as a renamed lattice operation.

## Verification

Local fail-fast verification passed:

- `cargo +1.93.1 fmt --all --check`
- `cargo +1.93.1 clippy --workspace --all-targets -- -D warnings`
- `cargo test -p semantic-core-quad tile_default_ -- --nocapture`
- `cargo test -p semantic-core-quad --quiet`
- `cargo check -p semantic-core-quad --no-default-features`
- `cargo test -p semantic-core-quad --all-features --quiet`
- `cargo test -p semantic-core-capsule --quiet`
- `cargo test --test legacy_guards --quiet`
- `cargo test --all-targets --quiet`
- `git diff --check`
- `scripts/harness-check.ps1`

GitHub Actions run `#4351` passed all eight jobs at implementation head:

```text
bbfbf8be1ee68caae77964910dedfa7ea55bb156
```

## Changed Files in PR

* `.harness/current.task.yaml`
* `.harness/reports/CORE-QUAD-TILE-TRUTH-MAPS.md`
* `crates/semantic-core-quad/src/lib.rs`
* `docs/roadmap/core_quad/tile_truth_map_lifting.md`

## Explicit Non-Changes

* No `map_equiv`.
* No `QuadroBank<N>` map helpers.
* No `QuadTileBank<N>` map helpers.
* No tile layout change.
* No mask API change.
* No delta API change.
* No register truth-map semantic change.
* No direct plane optimization.
* No GPU transport implementation.
* No VM or runtime change.
* No dependency addition.
