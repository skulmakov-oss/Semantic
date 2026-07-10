# Harness Report: CORE-QUAD-TILE-TRUTH-MAPS

## Overview
- **Starting main commit:** a17c3a86c3d2c666b015c32bf3e3d83707299707
- **Branch:** core-quad/tile-truth-map-lifting
- **PR:** (pending)

## Inventory
Existing register map inventory:
`map_not`, `map_xor`, `map_and`, `map_or`, `map_implies`, `map_nand`, `map_nor`

Added tile map inventory:
`map_not`, `map_xor`, `map_and`, `map_or`, `map_implies`, `map_nand`, `map_nor`

**EQUIV deferral:** `map_equiv` explicitly omitted per v1 compatibility policy.

## Strategy
Tile methods convert to four registers, apply register mapping, and reconstruct the tile. This is the deterministic reference implementation.

## Tests
- `tile_default_unary_map_matches_reg_oracle`
- `tile_default_binary_maps_match_reg_oracle`
- `tile_default_matrix_transitions`
- `tile_default_boundary_lanes`
- `tile128_lattice_stability`

Lattice method preservation (`join`, `meet`, `inverse`) proved unchanged.

## Posture
- `no_std` compatibility remains intact.
- `serde` compatibility remains intact.

## Verification
- Pinned fmt/clippy: passed
- Tests (all, quiet, nocapture): passed
- Legacy guards: passed
- Harness check: passed
- Diff check: passed

## Changed Files
- `.harness/current.task.yaml`
- `.harness/reports/CORE-QUAD-TILE-TRUTH-MAPS.md`
- `crates/semantic-core-quad/src/lib.rs`
- `docs/roadmap/core_quad/tile_truth_map_lifting.md`

## Explicit Non-Changes
- EQUIV maps
- Bank-level maps (`QuadroBank`, `QuadTileBank`)
- Plane optimizations
- GPU transport