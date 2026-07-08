# Semantic Roadmap v0.3

This plan turns `EXO_DNA` into executable tasks tied to the current codebase.

## Track A: Frontend & Parser

Goal: complete the arena-first frontend and unified diagnostics.

- [ ] Remove the remaining legacy branches where `Expr/Stmt` are still interpreted as owned trees.
  - Module: `src/frontend.rs`
- [ ] Complete the transition to `ExprId/StmtId/SymbolId` across all public frontend APIs.
  - Module: `src/frontend.rs`
- [ ] Introduce metadata/doc-comment nodes for `Law/Entity`.
  - Module: `src/frontend.rs`
- [ ] Unify parser/type/lowering errors into a rustc-style format.
  - Modules: `src/frontend.rs`, `src/semantics/mod.rs`
- [ ] Prepare REPL parser mode (single-input incremental parse).
  - Module: `src/bin/smc.rs` (+ new REPL module if needed)

Acceptance:

- `cargo test` green;
- diagnostics include `line:col`, context, and caret in all critical paths.

## Track B: Semantics & Type System

Goal: formalize strict Logos semantics and type policy.

- [ ] Define the type lattice: `Int`, `Fx`, `QVec<N>`, `Mask`, `Str`, `Bool`, `Quad`, `Unit`.
  - Module: `src/semantics/mod.rs`
- [ ] Complete `QVec<N>` dimensional compatibility checks (operations, assignments, calls).
  - Module: `src/semantics/mod.rs`
- [ ] Formally separate implicit/explicit cast policy (`Int -> Fx` only implicit).
  - Module: `src/semantics/mod.rs`
- [ ] Check `Law` uniqueness in `Entity`, duplicate `Entity`, and shadowing policy inside `Law`.
  - Module: `src/semantics/mod.rs`
- [ ] Add dead law branch detection (warning) and stable law scheduling by priority.
  - Module: `src/semantics/mod.rs`

Acceptance:

- `smc check` produces a stable report;
- negative tests for mismatch/shadowing/duplicate cases are covered.

## Track C: IR, Bytecode, VM

Goal: stabilize the execution contract and prepare format evolution.

- [ ] Introduce a capability/version table in the SemCode header.
  - Module: `src/semcode_format.rs`
- [ ] Lock the immutable IR boundary after lowering.
  - Modules: `src/frontend.rs`, `src/semantics/mod.rs`
- [ ] Support gate surface in the pipeline (`GateRead/GateWrite/PulseEmit`) with explicit encode/decode policy.
  - Modules: `src/frontend.rs`, `src/semcode_format.rs`, `src/semcode_vm.rs`
- [ ] Extend VM validation for the new format sections.
  - Module: `src/semcode_vm.rs`
- [ ] Prepare compatibility tests across bytecode versions.
  - Tests: `tests/golden_semcode.rs` + new golden sets by version.

Acceptance:

- golden tests are stable;
- header and version parsing is deterministic, with correct errors.

## Track D: no_std Readiness

Goal: reduce dependence on std containers in critical areas.

- [ ] Localize std-only code behind feature gates.
  - Modules: `src/lib.rs`, `src/frontend.rs`, `src/semantics/mod.rs`
- [ ] Prepare no_std-friendly collections/allocators for frontend/semantics.
  - Modules: `src/frontend.rs`, `src/semantics/mod.rs`
- [ ] Add a separate CI profile for `--no-default-features` smoke-check.
  - Config/CI

Acceptance:

- minimal no_std build passes for core layers;
- std-only parts are clearly isolated.

## CLI Milestones

- [x] `smc check <input.sm>` — semantic analysis without writing `.smc`.
  - Module: `src/bin/smc.rs`
- [ ] `smc repl` — interactive mode.
- [ ] `smc explain <error-code>` — diagnostic help.

## Technical Debt (High Priority)

- [ ] Standardize the name-rendering function for `SymbolId` (single conversion point).
- [ ] Reduce duplication between the classic parser and the Logos parser.
- [ ] Extract frontend arena/types into a separate module (`src/frontend/ast.rs`) for readability.

## Definition of Done for v0.3

- `cargo check` and `cargo test` green.
- Golden sets are updated and stable.
- Public frontend/semantics APIs are aligned around the ID model.
- Documentation (`EXO_DNA`, roadmap, diagnostics) is synchronized with the code.
