# Quad Logic Frame scalar XOR LUT Creation

Status: PASS

## Issue

Implements the third slice of #1406

## Scope

This PR extends the `logic_frame` module in `semantic-core-quad` with a generated scalar LUT truth-table for the `XOR` operation.
This is the third small slice of #1406 after the merged `NOT` and `AND/OR` LUT PRs.

This is an isolated feature addition.
No behavior changes to VM, opcode execution, loader, or runtime boundaries. Mask evaluation remains unmodified.

## Changed files

- `.harness/current.task.yaml`
- `.harness/reports/CORE-QUAD-SCALAR-XOR-LUT.md`
- `crates/semantic-core-quad/src/logic_frame.rs`

## Decisions made

- Sliced PR into `XOR` implementation only, no `IMPLIES`, `EQUIV`, `NAND`, `NOR` to minimize risk.
- Implemented `XOR_LUT` array natively using a `const fn` evaluator. The generation explicitly utilizes the `a.bits() ^ b.bits()` raw-code derived policy.
- Exported `xor` method exposing `QuadState` types.
- Expanded `tests` module confirming logic formulas over the full 16 state pair domain against `raw_xor`, including commutativity and identity expectations (e.g. `xor(x, x) == N` and `xor(N, x) == x`).

## Verification

- `cargo test -p semantic-core-quad --quiet`
- `git diff --check`
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1`
- `git status --short`

*(If local `cargo clippy --workspace` fails, it's due to unrelated workspace warnings from other crates outside this PR's scope.)*
