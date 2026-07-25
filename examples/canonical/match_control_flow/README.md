# match_control_flow

- purpose: compact quad decision program showing `record`-carried configuration,
  `quad` domain values with the same syntactic dignity as `bool`/`i32`, compact
  guard returns, and a `match`-driven dispatch/aggregation pass
- language profile: Rust-like executable surface
- supported status: `qualified limited release` (parses, type-checks, compiles,
  verifies, and runs on current `main`)
- demonstrates (`docs/spec/source_style.md`):
  - B.1 top-level order: data (`CycleConfig`) -> domain transformations
    (`state_from_index`, `dispatch_code`) -> validation
    (`validate_distribution`) -> orchestration (`main`)
  - B.5 compact guard returns (`if slot == 0 { return N; }`)
  - B.6 compact `match` arms with the required `_` default arm
  - B.10 `main` limited to construction, orchestration, and final validation
- commands:
  - `cargo run --bin smc -- check examples/canonical/match_control_flow/src/main.sm`
  - `cargo run --bin smc -- run examples/canonical/match_control_flow/src/main.sm`
  - `cargo run --bin smc -- compile examples/canonical/match_control_flow/src/main.sm -o out.smc`
  - `cargo run --bin smc -- verify out.smc`
- expected result:
  - `check` passes with 0 warnings
  - `run` exits 0 (all `assert`s hold)
  - `compile` + `verify` accept the emitted `.smc`
- non-claims:
  - does not demonstrate `Option` / `Result`, pattern guards, or Logos syntax
  - `CycleConfig` is an ordinary executable `record`, not a `System` block —
    `System` is Logos-only and does not appear in this file
