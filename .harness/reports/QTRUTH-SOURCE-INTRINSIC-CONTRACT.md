# QTRUTH-SOURCE-INTRINSIC-CONTRACT

## Status

Completed.

## Purpose

Define explicit source admission for QTruth operations without changing parser, frontend, verifier, VM, format, or runtime behavior.

## Intrinsic contract

| Intrinsic | Arity | Argument types | Return type | Future IR target |
|---|---:|---|---|---|
| qtruth_and | 2 | quad, quad | quad | IrInstr::QTruthAnd |
| qtruth_or | 2 | quad, quad | quad | IrInstr::QTruthOr |
| qtruth_not | 1 | quad | quad | IrInstr::QTruthNot |
| qtruth_impl | 2 | quad, quad | quad | IrInstr::QTruthImpl |

The contract requires explicit quad operands and a quad result. There is no implicit bool conversion or numeric conversion.

## Legacy non-interference

The existing source operators remain legacy lattice operators:

| Source operator | Existing IR target |
|---|---|
| && | IrInstr::QAnd |
| || | IrInstr::QOr |
| ! | IrInstr::QNot |
| -> | IrInstr::QImpl |

These operators must not be silently reinterpreted as QTruth operations.

## Forbidden fallback

QTruth intrinsics must not be implemented by:

- mapping to QAnd/QOr/QNot/QImpl;
- using lattice aliases;
- using hidden adapters;
- inverting falsity planes;
- changing semantic-core-quad;
- changing opcode values;
- adding EQUIV/NAND/NOR.

## Recommended implementation slice

Recommended next implementation title:

`feat(sm-front/sm-ir): admit explicit QTruth intrinsics`

The implementation should add only `qtruth_and`, `qtruth_or`, `qtruth_not`, and `qtruth_impl` admission, with tests proving they lower to the matching `IrInstr::QTruth*` variants while legacy operators continue to lower to `IrInstr::Q*`.

The later PR should touch only the exact parser/typechecker/lowering owners discovered during implementation, plus its harness/report files. It must not broaden into format, verifier, VM, runtime, or legacy lattice changes.

## Boundary

This slice is specification only.

No crate files were changed.
No source syntax was added.
No parser behavior was added.
No frontend lowering was added.
No verifier behavior was changed.
No VM behavior was changed.
No sm-format opcode values were changed.
No semantic-core-quad behavior was changed.
No legacy lattice behavior was changed.

## Verification

- git diff --check
- pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1
- git status --short
