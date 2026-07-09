# Quad Logic Frame scalar NAND/NOR LUT Creation

Status: PASS

## Issue

Implements the fifth slice of #1406

## Scope

This PR extends the `logic_frame` module in `semantic-core-quad` with generated scalar LUT truth-tables for the `NAND` and `NOR` operations.
This is the fifth small slice of #1406 after `NOT`, `AND/OR`, `XOR`, and `IMPLIES`.

This is an isolated feature addition.
No behavior changes to VM, opcode execution, loader, or runtime boundaries. Mask evaluation remains unmodified.

## Decisions made

- Sliced PR into `NAND` and `NOR` implementations only. `EQUIV` is left out as it's deferred/separately named.
- Implemented `NAND_LUT` and `NOR_LUT` arrays natively using `const fn` evaluators.
- Generated purely from derived operations according to specs: `NAND = NOT(AND)` and `NOR = NOT(OR)`.
- No new primitive implication or equivalence semantics were implemented.
- Exported `nand` and `nor` methods exposing `QuadState` types.
- Expanded `tests` module confirming logic formulas over the full 16 state pair domain against the derived operations, commutativity for both, and exact discrete identity checks.

## Verification

- `cargo test -p semantic-core-quad --quiet`
- `git diff --check`
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1`
- `git status --short`

*(If local `cargo clippy --workspace` fails, it's due to unrelated workspace warnings from other crates outside this PR's scope.)*
