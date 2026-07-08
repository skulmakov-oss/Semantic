# Quad Logic Frame scalar AND/OR LUTs Creation

Status: PASS

## Issue

Implements the second slice of #1406

## Scope

This PR extends the `logic_frame` module in `semantic-core-quad` with generated scalar LUT truth-tables for `AND` and `OR` operations.
This is the second small slice of #1406 after the merged `NOT` LUT PR.

This is an isolated feature addition.
No behavior changes to VM, opcode execution, loader, or runtime boundaries. Mask evaluation remains unmodified.

## Changed files

- `.harness/current.task.yaml`
- `.harness/reports/CORE-QUAD-SCALAR-AND-OR-LUTS.md`
- `crates/semantic-core-quad/src/logic_frame.rs`

## Decisions made

- Sliced PR into `AND` and `OR` implementations only, no `XOR`, `IMPLIES`, `EQUIV`, `NAND`, `NOR` to minimize risk.
- Implemented `AND_LUT` and `OR_LUT` arrays natively using `const fn` evaluators. The generation explicitly leverages the truth and falsity planes according to the formulas:
    - AND: `truth = a.truth & b.truth`, `falsity = a.falsity | b.falsity`
    - OR: `truth = a.truth | b.truth`, `falsity = a.falsity & b.falsity`
- Exported `and` and `or` methods exposing `QuadState` types.
- Expanded `tests` module confirming logic formulas over the full 16 state pair domain, including commutativity and identity expectations.

## Verification

- `cargo test -p semantic-core-quad --quiet`
- `git diff --check`
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1`
- `git status --short`

*(If local `cargo clippy --workspace` fails, it's due to unrelated workspace warnings from other crates outside this PR's scope.)*
