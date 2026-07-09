# VM Quad Opcode semantic-core-quad Bridge

Status: PASS

## Issue

Implements `#1438` (Follows #1436)

## Scope

This is the first VM bridge slice after the proven SWAR backend and default aliases.
It routes VM scalar quad opcode execution through the canonical `semantic-core-quad` `QuadroReg32` default map aliases.

## Decisions made

- `semantic-core-quad` added as a dependency of `sm-vm`.
- Internal bridge added in `semcode_vm.rs` to convert between VM `QuadVal` and `semantic_core_quad::QuadState`.
- `quad_not`, `quad_and`, `quad_or`, `quad_implies` local scalar helpers now use `QuadroReg32` lane 0.
- VM opcode execution updated so:
  - `Opcode::QNot` uses `quad_not`
  - `Opcode::QAnd` uses `quad_and`
  - `Opcode::QOr` uses `quad_or`
  - `Opcode::QImpl` uses `quad_implies`
- Added tests to prove that the VM bridge behavior exactly matches the existing expected truth behavior (tested over all possible QuadVal inputs and pairs).

## Restrictions Honored

- Opcode IDs are unchanged.
- SemCode format is unchanged.
- Verifier and emitter are unchanged.
- No EQUIV API or opcode was introduced.
- No NAND/NOR opcodes were added.
- Public `QuadVal` enum shape is unchanged.

## Verification

- `cargo test -p sm-vm --quiet`
- `cargo test -p semantic-core-quad --quiet`
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1`
