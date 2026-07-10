# SM-IR-QTRUTH-ENCODING

## Status

Completed.

## Summary

Encoded explicit QTruth IR instructions to their reserved QTruth opcodes.

## Mapping

- IrInstr::QTruthAnd -> Opcode::QTruthAnd
- IrInstr::QTruthOr -> Opcode::QTruthOr
- IrInstr::QTruthNot -> Opcode::QTruthNot
- IrInstr::QTruthImpl -> Opcode::QTruthImpl

## Boundary

This slice is IR-to-bytecode encoding only.

No sm-format opcode values were changed.
No sm-verify behavior was changed.
No sm-vm behavior was changed.
No sm-emit crate changes were made.
No semantic-core-quad behavior was changed.
No parser or frontend syntax was added.
No QTruth constant folding was added.
No legacy lattice QAnd/QOr/QNot/QImpl behavior was changed.

## Verification

- cargo test -p sm-ir --quiet
- git diff --check
- pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1
- git status --short
