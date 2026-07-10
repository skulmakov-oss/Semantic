# SM-IR-QTRUTH-SEMCODE-ENVELOPE-GUARD

## Status

Completed.

## Summary

Added sm-ir tests proving QTruth IR instructions survive full SemCode envelope emission.

## Covered instructions

- IrInstr::QTruthAnd
- IrInstr::QTruthOr
- IrInstr::QTruthNot
- IrInstr::QTruthImpl

## Boundary

This slice is an sm-ir SemCode envelope guard only.

No sm-format opcode values were changed.
No sm-verify behavior was changed.
No sm-vm behavior was changed.
No sm-emit crate changes were made.
No semantic-core-quad behavior was changed.
No parser or frontend syntax was added.
No Cargo dependencies were added.
No QTruth constant folding was added.
No legacy lattice QAnd/QOr/QNot/QImpl behavior was changed.

## Verification

- cargo test -p sm-ir --quiet
- git diff --check
- pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1
- git status --short
