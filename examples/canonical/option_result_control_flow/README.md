# option_result_control_flow

- benchmark-class application anchor: practical `Option` / `Result` control
  flow over the admitted public surface
- purpose: show deterministic routing through optional, success, and error
  states using explicit constructors and `match`
- demonstrates:
  - `Option(T)` and `Result(T, E)` type surface
  - `Option::Some` / `Option::None`
  - `Result::Ok` / `Result::Err`
  - `match` over admitted standard forms
  - deterministic fallback `_` handling
  - terminal return paths
  - assert-based self-checks
- commands:
  - `cargo run --bin smc -- check examples/canonical/option_result_control_flow/src/main.sm`
  - `cargo run --bin smc -- run examples/canonical/option_result_control_flow/src/main.sm`
  - `cargo run --bin smc -- compile examples/canonical/option_result_control_flow/src/main.sm -o out.smc`
  - `cargo run --bin smc -- verify out.smc`
- expected output:
  - `check` succeeds
  - `run` exits successfully
  - `verify` accepts the compiled `.smc`
- non-goals:
  - no exceptions
  - no async control flow
  - no collection integration
  - no public stdlib policy work for `Option` / `Result`
