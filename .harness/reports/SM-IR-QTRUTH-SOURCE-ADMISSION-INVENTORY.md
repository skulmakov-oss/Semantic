# SM-IR-QTRUTH-SOURCE-ADMISSION-INVENTORY

## Status

Completed.

## Current state

### Legacy Quad IR construction

Legacy Quad instructions are constructed in `crates/sm-ir/src/legacy_lowering.rs`:

- `UnaryOp::Not` with `Type::Quad` constructs `IrInstr::QNot`.
- `BinaryOp::AndAnd` with `Type::Quad` constructs `IrInstr::QAnd`.
- `BinaryOp::OrOr` with `Type::Quad` constructs `IrInstr::QOr`.
- `BinaryOp::Implies` with two `Type::Quad` operands constructs `IrInstr::QImpl`.

The frontend typechecker accepts the existing quad surface for `!`, `&&`, `||`, and `->`. The parser builds `Expr::Unary`/`Expr::Binary` nodes for these operators, and the lexer recognizes `&&`, `||`, and `->`. Quad literals `N`, `F`, `T`, and `S`, plus the `quad` type, are also existing source constructs.

Therefore legacy `QAnd`, `QOr`, `QNot`, and `QImpl` are reachable from source syntax today.

### QTruth source reachability

No source parser, frontend typechecker, or sm-ir lowering path currently constructs `IrInstr::QTruthAnd`, `IrInstr::QTruthOr`, `IrInstr::QTruthNot`, or `IrInstr::QTruthImpl`.

The current `QTruth*` occurrences are IR enum representation, encoding, CrystalFold passthrough, envelope tests, and manual-IR tests. QTruth operations are therefore manual-IR-only at present.

## Boundary

The existing `&&`, `||`, `!`, and `->` source operators must continue to map to legacy lattice `QAnd`, `QOr`, `QNot`, and `QImpl`. They must not be silently reinterpreted as QTruth operations.

The following remain unchanged for a later implementation slice:

- sm-format opcode values;
- sm-verify admission behavior;
- sm-vm execution behavior;
- sm-emit ownership;
- semantic-core-quad truth-map semantics;
- legacy lattice behavior and opcode mapping.

No crate changes were made by this audit.

## Recommended next implementation slice

Recommended title: `feat(sm-front/sm-ir): add explicit QTruth source admission intrinsic`

The smallest safe source admission is a dedicated, explicitly named intrinsic or builtin contract, with source/typecheck/lowering changes reviewed together. Reusing `&&`, `||`, `!`, or `->` would erase the required distinction between legacy lattice and explicit truth-map operations.

Until that explicit source contract is named and specified, QTruth should remain manual-IR-only. A later implementation should be allowed to touch only the selected parser/typecheck/lowering owners and should still forbid sm-format, sm-verify, sm-vm, sm-emit, semantic-core-quad, and legacy lattice rewrites.

## Risks

- Mapping QTruth to existing operators could silently route truth-map operations through lattice aliases.
- Touching sm-format is unnecessary because QTruth opcode slots already exist.
- Adding parser syntax before the semantic admission contract is selected could create an unstable public surface.
- A generic builtin name without an explicit Quad/truth-map contract could blur the `quad` meaning boundary.

## Verification

- `git diff --check`
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1`
- `git status --short`
