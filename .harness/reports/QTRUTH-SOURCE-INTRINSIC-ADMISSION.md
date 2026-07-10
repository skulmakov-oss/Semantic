# QTRUTH-SOURCE-INTRINSIC-ADMISSION

## Status

Completed.

## Summary

Admitted explicit QTruth source intrinsics and lowered them to `IrInstr::QTruth*`.

## Intrinsics

- `qtruth_and(a, b)`
- `qtruth_or(a, b)`
- `qtruth_not(a)`
- `qtruth_impl(a, b)`

## Mapping

- `qtruth_and(a, b)` -> `IrInstr::QTruthAnd`
- `qtruth_or(a, b)` -> `IrInstr::QTruthOr`
- `qtruth_not(a)` -> `IrInstr::QTruthNot`
- `qtruth_impl(a, b)` -> `IrInstr::QTruthImpl`

All intrinsic arguments are admitted only as `quad`, and each intrinsic returns `quad`. Named arguments, wrong arity, and non-quad arguments are rejected.

## Boundary

Only explicit QTruth intrinsics were added.

No parser syntax was added.
No lexer tokens were added.
No sm-format opcode values were changed.
No sm-verify behavior was changed.
No sm-vm behavior was changed.
No sm-emit crate changes were made.
No semantic-core-quad behavior was changed.
No QTruth constant folding was added.
No hidden adapter was added.
No falsity-plane inversion was added.
No EQUIV/NAND/NOR operations were added.
Legacy `&&`, `||`, `!`, and `->` behavior remains unchanged and continues to lower to legacy lattice IR variants.

## Verification

- `cargo test -p sm-front --quiet`
- `cargo test -p sm-ir --quiet`
- `git diff --check`
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1`
- `git status --short`
