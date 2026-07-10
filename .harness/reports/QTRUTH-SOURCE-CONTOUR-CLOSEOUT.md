# QTRUTH-SOURCE-CONTOUR-CLOSEOUT

## Status

Completed.

## Summary

The QTruth source-level execution contour is closed.

Source-level QTruth intrinsics now compile, verify, lower to explicit QTruth IR/opcodes, and execute through the VM.

## Completed PR chain

| PR | Role |
|---:|---|
| #1455 | Added sm-format QTruth opcode enum slots |
| #1457 | Locked verifier/vm unsupported-boundary behavior |
| #1459 | Added sm-vm QTruth opcode execution |
| #1461 | Added sm-ir explicit QTruth instruction representation |
| #1463 | Encoded sm-ir QTruth instructions to QTruth opcodes |
| #1465 | Guarded full sm-ir SemCode envelope emission |
| #1467 | Inventoried QTruth source admission path |
| #1469 | Defined explicit QTruth source intrinsic contract |
| #1471 | Admitted qtruth_* source intrinsics in sm-front/sm-ir |
| #1473 | Added source-level VM E2E execution guard |

## Final accepted source surface

Accepted QTruth source intrinsics:

- qtruth_and(a, b)
- qtruth_or(a, b)
- qtruth_not(a)
- qtruth_impl(a, b)

All arguments are quad.
All results are quad.
No implicit bool or numeric conversion is admitted.

## Pipeline

qtruth_* source intrinsic
-> sm-front builtin type admission
-> sm-ir lowering
-> IrInstr::QTruth*
-> Opcode::QTruth*
-> SemCode envelope
-> sm-verify admission
-> sm-vm execution
-> observable Quad result

## Legacy non-interference

Existing source operators remain legacy lattice operators:

| Source operator | Legacy IR |
|---|---|
| && | IrInstr::QAnd |
| || | IrInstr::QOr |
| ! | IrInstr::QNot |
| -> | IrInstr::QImpl |

They must not be silently reinterpreted as QTruth operations.

## Explicit non-goals

The following are intentionally out of scope:

- EQUIV
- NAND
- NOR
- parser operator syntax for QTruth
- lexer tokens for QTruth
- hidden adapters
- falsity-plane inversion
- routing through lattice aliases
- changing semantic-core-quad truth maps
- changing sm-format opcode values
- changing sm-verify behavior
- changing sm-vm behavior
- changing legacy QAnd/QOr/QNot/QImpl behavior

## Remaining future work

No functional work is required for the current QTruth source-level contour.

Any future QTruth expansion must be a separate explicitly scoped contour.

## Boundary

This closeout is audit-only.

No crate files were changed.
No docs were changed.
No source behavior was changed.
No parser or lexer behavior was changed.
No verifier behavior was changed.
No VM behavior was changed.
No sm-format opcode values were changed.
No semantic-core-quad behavior was changed.
No legacy lattice behavior was changed.

## Verification

- git diff --check
- pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1
- git status --short
