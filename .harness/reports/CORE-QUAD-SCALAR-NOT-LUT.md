# Quad Logic Frame scalar NOT LUT Creation

Status: PASS

## Issue

Implements the first slice of #1406

## Scope

This PR adds the `logic_frame` module skeleton to `semantic-core-quad` and implements the generated scalar LUT truth-table for the `NOT` operation.

This is an isolated feature addition.
No behavior changes to VM, opcode execution, loader, or runtime boundaries. Mask evaluation remains unmodified.

## Changed files

- `.harness/current.task.yaml`
- `.harness/reports/CORE-QUAD-SCALAR-NOT-LUT.md`
- `crates/semantic-core-quad/src/lib.rs`
- `crates/semantic-core-quad/src/logic_frame.rs`

## Decisions made

- Sliced PR into `NOT` implementation only to minimize risk.
- Implemented `NOT_LUT` using `QuadState` explicitly.
- Added `tests` submodule ensuring `NOT` output correctly conforms to the Logic Frame v1 specification.

## Fixup: Style Cleanup

- `cargo fmt/clippy` cleanup: Removed trailing whitespace and replaced `#[inline(always)]` with `#[inline]`.
- No behavior change.
- No public API change.
- No VM/opcode/runtime changes.

## Verification

- `cargo test -p semantic-core-quad --quiet`
- `git diff --check`
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1`
- `git status --short`
