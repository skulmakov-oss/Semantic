# VM-QTRUTH-SOURCE-INTRINSIC-E2E

## Status

Completed.

## Summary

Added sm-vm end-to-end tests proving source-level QTruth intrinsics compile, verify, and execute through the VM.

## Covered intrinsics

- qtruth_and
- qtruth_or
- qtruth_not
- qtruth_impl

## Boundary

This slice is test-only.

No VM behavior was changed.
No verifier behavior was changed.
No source admission was changed.
No parser or lexer behavior was changed.
No sm-format opcode values were changed.
No sm-front files were changed.
No sm-ir files were changed.
No sm-emit files were changed.
No semantic-core-quad behavior was changed.
No QTruth constant folding was added.
No hidden adapter was added.
No falsity-plane inversion was added.
No EQUIV/NAND/NOR operations were added.
Legacy QAnd/QOr/QNot/QImpl behavior was unchanged.

## Verification

- cargo test -p sm-vm --quiet
- git diff --check
- pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1
- git status --short
