# rule_state_decision

- purpose: structured practical program — record-oriented rule/state decision
  logic with explicit `Result(T, E)` handling, a separate validation phase, and
  compact orchestration
- language profile: Rust-like executable surface
- supported status: `qualified limited release` (parses, type-checks, compiles,
  verifies, and runs on current `main`); also the first application-completeness
  anchor covered by `tests/canonical_examples.rs`
- demonstrates (`docs/spec/source_style.md`):
  - B.1 top-level order: data (`DecisionContext`) -> domain transformation
    (`decide`) -> validation (`validate_verdict`) -> orchestration (`main`)
  - B.5 compact guard returns in `decide`
  - contextual `Result::Ok` / `Result::Err` and explicit `match` settlement
  - B.4 blank lines separating construction, decision, and validation phases
    inside `main`
- commands:
  - `cargo run --bin smc -- check examples/canonical/rule_state_decision/src/main.sm`
  - `cargo run --bin smc -- run examples/canonical/rule_state_decision/src/main.sm`
  - `cargo run --bin smc -- compile examples/canonical/rule_state_decision/src/main.sm -o out.smc`
  - `cargo run --bin smc -- verify out.smc`
- expected result:
  - `check` passes with 0 warnings
  - `run` exits 0 (all `assert`s hold)
  - `compile` + `verify` accept the emitted `.smc`
- non-claims:
  - no host effects, UI, or package/release packaging work
  - no Logos syntax
