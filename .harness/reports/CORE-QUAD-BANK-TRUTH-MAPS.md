# Harness Report: CORE-QUAD-BANK-TRUTH-MAPS

## Status
- **Starting main commit:** `f925475cf2ba40fca4706c933f9d7b77956ae82f`
- **Branch:** `core-quad/bank-truth-map-helpers`

## Existing Map Inventory
- Reg maps: `map_not`, `map_xor`, `map_and`, `map_or`, `map_implies`, `map_nand`, `map_nor`
- Tile maps: `map_not`, `map_xor`, `map_and`, `map_or`, `map_implies`, `map_nand`, `map_nor`

## Added `QuadroBank<N>` Inventory
- `map_not_inplace`
- `map_xor_inplace`
- `map_and_inplace`
- `map_or_inplace`
- `map_implies_inplace`
- `map_nand_inplace`
- `map_nor_inplace`

## Added `QuadTileBank<N>` Inventory
- `map_not_inplace`
- `map_xor_inplace`
- `map_and_inplace`
- `map_or_inplace`
- `map_implies_inplace`
- `map_nand_inplace`
- `map_nor_inplace`

## Implementation Strategy
- Implemented as in-place methods (`&mut self` and optional `&Self` for binary maps).
- Binary operations use `self.iter_mut().zip(other.iter().copied())` elementwise application.
- Unary operations use `self.iter_mut()` and mutate each element in-place.

## Contracts
- **Element-order:** Index `i` is combined only with index `i`. No rotation, reversal, or cross-element contamination.
- **Zero-length behavior:** Tested and verified as safe no-ops.
- **Lattice separation:** Confirmed through matrix and boundary testing.
- **EQUIV deferral:** Explicitly omitted.
- **no_std & serde:** Full compatibility retained. No allocation or `unsafe` code added.

## Local Verification Commands
- `cargo +1.93.1 fmt --all --check`
- `cargo +1.93.1 clippy --workspace --all-targets -- -D warnings`
- `cargo test -p semantic-core-quad bank_default_ -- --nocapture`
- `cargo test -p semantic-core-quad --quiet`
- `cargo check -p semantic-core-quad --no-default-features`
- `cargo test -p semantic-core-quad --all-features --quiet`
- `cargo test -p semantic-core-capsule --quiet`
- `cargo test --test legacy_guards --quiet`
- `cargo test --all-targets --quiet`
- `git diff --check`
- `scripts/harness-check.ps1`

## Changed Files
- `.harness/current.task.yaml`
- `.harness/reports/CORE-QUAD-BANK-TRUTH-MAPS.md`
- `crates/semantic-core-quad/src/lib.rs`
- `docs/roadmap/core_quad/bank_truth_map_helpers.md`

## Explicit Non-Changes
- No EQUIV.
- No changes to `QuadroBank<N>` or `QuadTileBank<N>` fields, structs, constructors, getters/setters, layout, masks, delta APIs, reg truth semantics, tile truth semantics, or lattice helpers.
- No dynamic-length mismatch handling.
- No allocation, unsafe code, dependencies, runtime dispatch, or direct plane optimization.
