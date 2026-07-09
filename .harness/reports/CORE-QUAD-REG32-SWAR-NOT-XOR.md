# Quad Logic Frame QuadroReg32 SWAR NOT/XOR Map Methods

Status: PASS

## Issue

Implements the second slice of #1407

## Scope

This PR extends `QuadroReg32` in `semantic-core-quad` with explicit SWAR-backed `NOT` and `XOR` map methods, tested against the scalar map oracle.

This is an isolated feature addition.
No behavior changes to VM, opcode execution, loader, or runtime boundaries. Mask evaluation remains unmodified.
No other SWAR methods have been implemented yet.

## Decisions made

- Sliced PR into SWAR `NOT` and `XOR` only.
- Implemented `map_not_swar` using `self.inverse()`.
- Implemented `map_xor_swar` using raw bitwise XOR.
- Tested exhaustively against `RAW_SAMPLES` and `QuadState::ALL` repeated instances, validating them against their corresponding scalar map methods.
- `EQUIV` map is omitted in line with the deferred `EQUIV` policy.
- `AND`, `OR`, `IMPLIES`, `NAND`, and `NOR` SWAR methods are omitted for future slices.
- No default aliases were added.

## Verification

- `cargo test -p semantic-core-quad --quiet`
- `git diff --check`
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1`
- `git status --short`

*(If local `cargo clippy --workspace` fails, it's due to unrelated workspace warnings from other crates outside this PR's scope.)*
