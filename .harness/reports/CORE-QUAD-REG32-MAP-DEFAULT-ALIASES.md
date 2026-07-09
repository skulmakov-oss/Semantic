# Quad Logic Frame QuadroReg32 Map Default Aliases

Status: PASS

## Issue

Implements `#1436` (Follows #1407)

## Scope

Promotes the proven `QuadroReg32` SWAR truth-table methods to explicit default `map_*` aliases.

This is an isolated feature addition.
No behavior changes to VM, opcode execution, loader, or runtime boundaries. Mask evaluation remains unmodified.

## Decisions made

- Created thin public aliases for proven operations: `map_not`, `map_xor`, `map_and`, `map_or`, `map_implies`, `map_nand`, `map_nor`.
- All aliases are `const fn` and delegate directly to their `_swar` equivalents.
- Tests prove that for both `RAW_SAMPLES` combinations and `QuadState::ALL` identical-filled states, the default aliases produce output strictly equal to the corresponding `_swar` and `_scalar` methods.
- `EQUIV` map remains excluded per policy. No `map_equiv`, `map_equiv_scalar`, `map_equiv_swar`, `equiv()`, or `EQUIV_LUT` were added.

## Verification

- `cargo test -p semantic-core-quad --quiet`
- `git diff --check`
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1`
- `git status --short`

*(If local `cargo clippy --workspace` fails, it's due to unrelated workspace warnings from other crates outside this PR's scope.)*
