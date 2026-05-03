# Changelog

All notable changes to this project are documented in this file.

## Unreleased

### Added (post-v1.1.1 language-maturity subtracks)

- **D-wave baseline** (application-completeness program, D-wave):
  - `Map(K, V)` persistent lookup tables — PR #404, 2026-05-03
    - `map_empty()`, `map_contains`, `map_get`, `map_set` (functional update)
    - admitted key types: `i32`, `u32`, `bool`, `text`, `quad`
    - SemCode format `SEMCOD14`, capability `CAP_MAP_VALUES = 1 << 15`
  - deterministic seeded PRNG — PR #405, 2026-05-03
    - `random_seed(seed: i32)` — seeds the VM PRNG
    - `random_next_i32(lo: i32, hi: i32) -> i32` — deterministic bounded value in `[lo, hi)`
    - xorshift64 algorithm (period 2⁶⁴−1); range computed through `i64` to handle full `i32` span
    - SemCode format `SEMCOD15`, capability `CAP_PRNG = 1 << 16`

- **M9.4 Richer Pattern Surface**: five new pattern forms across owner layer, parser, and typecheck.
  - `MatchPattern::Wildcard` — bare `_` matches any scrutinee, binds nothing
  - `MatchPattern::Or(alts)` — pipe-separated alternatives `P1 | P2 | P3`
  - `MatchPattern::IntRange(IntRangePattern)` — integer range `1..=5` / `1..5`; admitted for `i32`/`u32` scrutinees
  - `TuplePatternItem::Nested(items)` — recursive tuple destructuring `let (a, (b, c)) = ...`
  - `Expr::IfLet(IfLetExpr)` — `if let Pat = expr { } else { }` binding guard
  - `TokenKind::Pipe` — bare `|` now lexes as a token (was error)
  - exhaustiveness: Wildcard covers all variants; Or unwraps alternatives for variant coverage
  - done-boundary: `M9.4 closes at owner layer + parser admission + typecheck for all five forms`

- **M9.2 Traits (static)**: traits/impls now have full owner-layer representation,
  parser admission, and static typecheck support.
  - `trait` and `impl` declarations admitted at top level
  - `TraitDecl`, `ImplDecl`, `TraitBound`, `TraitMethodSig` AST nodes
  - `<T: TraitName>` bound syntax on generic function type parameters
  - `validate_trait_coherence`: rejects duplicate `(trait, for_type)` impl pairs
  - `validate_impl_conformance`: rejects impls with missing methods or wrong return types
  - bound satisfaction check at generic call sites
  - runtime dispatch, trait objects, specialization, and blanket impls remain deferred
  - done-boundary: `M9.2 closes at static trait admission + coherence/conformance + bound satisfaction`

## v1.1.1 - 2026-04-01

### Added
- `v0.1` density-surface wave landed in `main`, including expression-valued
  control, guarded control, composition/call density, flow primitives,
  assertion contracts, const declarations, and expanded numeric literal forms.
- `v0.2` contract/data-core wave landed in `main`, including:
  - tuple literals and tuple types
  - tuple destructuring bind/assignment and tuple `let-else`
  - nominal ADT declarations and constructors
  - ADT match core plus exhaustiveness enforcement
  - `Option(T)` / `Result(T, E)` standard forms and match ergonomics
  - first-wave function contracts: `requires`, `ensures`, and narrow
    `invariant`
  - first-wave units of measure for supported numeric families
- record-layer waves landed in `main`, including canonical nominal records,
  field access, pass/return, equality-safe comparisons, destructuring, narrow
  record `let-else`, immutable copy-with, and shorthand/punning ergonomics.
- `v0.3` schema/boundary-core wave landed in `main`, including:
  - canonical schema declarations with record/tagged-union forms, role markers,
    and version metadata
  - deterministic validation-plan ownership and derived validation checks
  - canonical config-contract parsing and validation paths
  - deterministic generated API-contract artifacts
  - deterministic schema compatibility classification and migration metadata
  - deterministic generated wire-contract artifacts for tagged wire unions and
    record patch types

### Changed
- GitHub roadmap hygiene was normalized so implemented `v0.1`, `v0.2`, `v0.3`,
  density, and record-layer milestones no longer remain open after landing in
  `main`.
- The repository now carries an explicit post-`v0.3` release-freeze checkpoint
  in `docs/roadmap/language_maturity/release_freeze_post_v03_checkpoint.md`.

### Notes
- This release was cut from exact source commit `087f2f6`.
- Published assets were validated as:
  - `smc.exe`
  - `svm.exe`
  - `semantic-language-windows-x64-v1.1.1.zip`
- Downloaded release assets passed the stable smoke matrix before publish.

## v1.0.0 - 2026-02-14

### Added
- `f64` type support in frontend, type checker, AST, and IR.
- Float literals and unary `+` / `-`.
- `f64` arithmetic operators: `+`, `-`, `*`, `/` with precedence parsing.
- SemCode v1 format marker: `SEMCODE1`.
- New math opcodes: `LOAD_F64`, `ADD_F64`, `SUB_F64`, `MUL_F64`, `DIV_F64`.
- VM builtin dispatch for `sin`, `cos`, `tan`, `sqrt`, `abs`, `pow`.
- New v1 golden fixture: `tests/golden_v1/calculator.sm` + `.smc`.
- New example program: `examples/calculator.sm`.

### Changed
- SemCode parser/VM now accept both `SEMCODE0` and `SEMCODE1`.
- Bytecode emitter automatically selects:
  - `SEMCODE0` for v0-compatible programs.
  - `SEMCODE1` when v1 math opcodes are used.
- Golden tests extended to include v1 fixture.

### Compatibility
- Backward compatibility preserved: existing `SEMCODE0` binaries remain executable.

## v0.1.0 - 2026-02-14

### Added
- Toolchain v0 baseline: parser, type-checker, IR lowering, SemCode emitter, VM, CLI.
- SemCode v0 bytecode format (`SEMCODE0`) and golden byte-for-byte tests.

### Notes
- v0 is frozen on branch `release/v0` and tag `v0.1.0`.
