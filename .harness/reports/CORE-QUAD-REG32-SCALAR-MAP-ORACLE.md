# Quad Logic Frame QuadroReg32 Scalar Map Oracle

Status: PASS

## Issue

Implements the first slice of #1407

## Scope

This PR extends `QuadroReg32` in `semantic-core-quad` with scalar mapping oracle methods for future SWAR verification.
It uses the `logic_frame` truth tables to map states lane-by-lane.

This is an isolated feature addition.
No behavior changes to VM, opcode execution, loader, or runtime boundaries. Mask evaluation remains unmodified.
No SWAR methods have been implemented yet.

## Decisions made

- Sliced PR into scalar map oracle only.
- Implemented `map_not_scalar`, `map_and_scalar`, `map_or_scalar`, `map_xor_scalar`, `map_implies_scalar`, `map_nand_scalar`, and `map_nor_scalar`.
- Each method loops `0..32` and calls the corresponding `logic_frame` scalar function to update the lane in-place.
- `EQUIV` map is omitted in line with the deferred `EQUIV` policy.
- No SWAR optimizations or default aliases were added.

## Verification

- `cargo test -p semantic-core-quad --quiet`
- `git diff --check`
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1`
- `git status --short`

*(If local `cargo clippy --workspace` fails, it's due to unrelated workspace warnings from other crates outside this PR's scope.)*
