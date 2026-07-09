# Quad Logic Frame QuadroReg32 SWAR AND/OR Map Methods

Status: PASS

## Issue

Implements the third slice of #1407

## Scope

This PR extends `QuadroReg32` in `semantic-core-quad` with explicit SWAR-backed `AND` and `OR` map methods, tested against the scalar map oracle.

This is an isolated feature addition.
No behavior changes to VM, opcode execution, loader, or runtime boundaries. Mask evaluation remains unmodified.
No other SWAR methods have been implemented yet.

## Decisions made

- Sliced PR into SWAR `AND` and `OR` only.
- Implemented `map_and_swar` using truth-plane intersection and falsity-plane union (`(at & bt) | (af | bf)`).
- Implemented `map_or_swar` using truth-plane union and falsity-plane intersection (`(at | bt) | (af & bf)`).
- Tested exhaustively against `RAW_SAMPLES` pairs and `QuadState::ALL` repeated instance pairs, validating them against their corresponding scalar map methods.
- `EQUIV` map is omitted in line with the deferred `EQUIV` policy.
- `IMPLIES`, `NAND`, and `NOR` SWAR methods are omitted for future slices.
- No default aliases were added.

## Verification

- `cargo test -p semantic-core-quad --quiet`
- `git diff --check`
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1`
- `git status --short`

*(If local `cargo clippy --workspace` fails, it's due to unrelated workspace warnings from other crates outside this PR's scope.)*
