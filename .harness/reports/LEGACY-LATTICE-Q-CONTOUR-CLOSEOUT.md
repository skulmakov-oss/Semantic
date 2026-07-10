# LEGACY-LATTICE-Q-CONTOUR-CLOSEOUT

## Status

Completed.

## Summary

The legacy lattice Q operator contour is recorded as a separate stable contour from QTruth v1.

Existing source operators for quad operands remain legacy lattice operators.

## Legacy source surface

| Source operator | Operand type | Result type | Legacy IR target |
|---|---|---|---|
| && | quad, quad | quad | IrInstr::QAnd |
| || | quad, quad | quad | IrInstr::QOr |
| ! | quad | quad | IrInstr::QNot |
| -> | quad, quad | quad | IrInstr::QImpl |

## Relationship to QTruth v1

QTruth v1 is a separate explicit truth-map contour.

Accepted QTruth source surface:

- qtruth_and(a, b)
- qtruth_or(a, b)
- qtruth_not(a)
- qtruth_impl(a, b)

QTruth source intrinsics lower to IrInstr::QTruth*.

Legacy source operators lower to IrInstr::Q*.

These two contours must remain separate.

## Non-interference rule

The following must not happen silently:

- && must not become qtruth_and.
- || must not become qtruth_or.
- ! must not become qtruth_not.
- -> must not become qtruth_impl.
- qtruth_* must not lower to QAnd/QOr/QNot/QImpl.
- QAnd/QOr/QNot/QImpl must not be reinterpreted as QTruth.

## Explicit non-goals

The following are intentionally out of scope:

- EQUIV
- NAND
- NOR
- parser operator syntax for QTruth
- lexer tokens for QTruth
- hidden adapters
- falsity-plane inversion
- routing legacy lattice through QTruth
- routing QTruth through legacy lattice
- changing semantic-core-quad truth maps
- changing sm-format opcode values
- changing sm-verify behavior
- changing sm-vm behavior
- changing legacy QAnd/QOr/QNot/QImpl behavior

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
No QTruth behavior was changed.
No legacy lattice behavior was changed.

## Verification

- git diff --check
- pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1
- git status --short
