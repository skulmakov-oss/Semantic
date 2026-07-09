# Quad Logic Frame EQUIV Policy Guard

Status: PASS

## Issue

Implements the sixth slice of #1406

## Scope

This PR extends the `logic_frame` module in `semantic-core-quad` with an explicit policy guard for the deferred `EQUIV` operation.
This is the sixth small slice of #1406 after `NOT`, `AND/OR`, `XOR`, `IMPLIES`, and `NAND/NOR`.

This is an isolated test/documentation addition.
No behavior changes to VM, opcode execution, loader, or runtime boundaries. Mask evaluation remains unmodified.

## Decisions made

- Sliced PR into `EQUIV` policy guard only.
- Documented that `EQUIV` is intentionally not exposed as `equiv`.
- Added the constant marker `EQUIV_POLICY = "deferred_or_separately_named"`.
- Added test `test_equiv_policy_is_deferred` asserting the constant marker.
- Explicitly refrained from implementing `EQUIV_LUT` or an `equiv()` public API, preventing unqualified equivalence operations.
- No existing LUT semantics were modified.

## Verification

- `cargo test -p semantic-core-quad --quiet`
- `git diff --check`
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1`
- `git status --short`

*(If local `cargo clippy --workspace` fails, it's due to unrelated workspace warnings from other crates outside this PR's scope.)*
