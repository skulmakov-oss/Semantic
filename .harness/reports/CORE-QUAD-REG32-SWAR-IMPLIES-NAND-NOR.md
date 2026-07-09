# Quad Logic Frame QuadroReg32 SWAR IMPLIES/NAND/NOR Map Methods

Status: PASS

## Issue

Implements the fourth slice of #1407

## Scope

This PR extends `QuadroReg32` in `semantic-core-quad` with explicit SWAR-backed `IMPLIES`, `NAND`, and `NOR` map methods, tested against the scalar map oracle.

This is an isolated feature addition.
No behavior changes to VM, opcode execution, loader, or runtime boundaries. Mask evaluation remains unmodified.

## Decisions made

- Sliced PR into SWAR `IMPLIES`, `NAND`, and `NOR` only.
- Implemented `map_implies_swar` using `self.map_not_swar().join(other)`.
- Implemented `map_nand_swar` using `self.map_and_swar(other).map_not_swar()`.
- Implemented `map_nor_swar` using `self.map_or_swar(other).map_not_swar()`.
- Tested exhaustively against `RAW_SAMPLES` pairs, `QuadState::ALL` repeated instance pairs, and exact check rules, validating them against their corresponding scalar map methods.
- `EQUIV` map is omitted in line with the deferred `EQUIV` policy.
- No default aliases were added.

## Verification

- `cargo test -p semantic-core-quad --quiet`
- `git diff --check`
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1`
- `git status --short`

*(If local `cargo clippy --workspace` fails, it's due to unrelated workspace warnings from other crates outside this PR's scope.)*
