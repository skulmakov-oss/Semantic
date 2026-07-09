# SM-IR-QTRUTH-REPRESENTATION

## Status

Completed.

## Summary

Added explicit sm-ir representation for QTruth operations.

## Added representation

- QTruthAnd
- QTruthOr
- QTruthNot
- QTruthImpl

## Boundary

This slice is IR representation only.

No sm-format opcode values were changed.
No sm-verify behavior was changed.
No sm-vm behavior was changed.
No sm-emit lowering was added.
No semantic-core-quad behavior was changed.
No legacy lattice QAnd/QOr/QNot/QImpl behavior was changed.
CrystalFold was updated only to keep exhaustive IR matching representation-preserving for QTruth operations.

No QTruth constant folding was added.

## Verification

- cargo test -p sm-ir --quiet
- git diff --check
- pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1
- git status --short
