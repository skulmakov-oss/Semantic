# Quad Logic Frame scalar IMPLIES LUT Creation

Status: PASS

## Issue

Implements the fourth slice of #1406

## Scope

This PR extends the `logic_frame` module in `semantic-core-quad` with a generated scalar LUT truth-table for the `IMPLIES` operation.
This is the fourth small slice of #1406 after `NOT`, `AND/OR`, and `XOR`.

This is an isolated feature addition.
No behavior changes to VM, opcode execution, loader, or runtime boundaries. Mask evaluation remains unmodified.

## Decisions made

- Sliced PR into `IMPLIES` implementation only. `EQUIV` is left out as it's deferred/separately named.
- Implemented `IMPLIES_LUT` array natively using a `const fn` evaluator.
- Used the frozen derived compatibility policy: `implies(a, b) == not(a).join(b)`.
- No primitive implication semantics were implemented or introduced.
- Exported `implies` method exposing `QuadState` types.
- Expanded `tests` module confirming logic formulas over the full 16 state pair domain against `not(a).join(b)`. Added checks for directionality (`T -> F != F -> T`) and exact values.

## Verification

- `cargo test -p semantic-core-quad --quiet`
- `git diff --check`
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1`
- `git status --short`

*(If local `cargo clippy --workspace` fails, it's due to unrelated workspace warnings from other crates outside this PR's scope.)*
