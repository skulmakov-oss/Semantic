# rule_state_decision

- benchmark-class application anchor: deterministic policy/admission decision
  logic on the admitted surface
- purpose: record-oriented rule/state decision logic with explicit
  `Result(T, E)` handling
- demonstrates:
  - nominal records
  - `quad`
  - contextual `Result::Ok` / `Result::Err`
  - explicit `match` settlement
  - deterministic decision path with `assert(verdict == T)`
- commands:
  - `cargo run --bin smc -- check examples/canonical/rule_state_decision/src/main.sm`
  - `cargo run --bin smc -- run examples/canonical/rule_state_decision/src/main.sm`
  - `cargo run --bin smc -- compile examples/canonical/rule_state_decision/src/main.sm -o out.smc`
  - `cargo run --bin smc -- verify out.smc`
- expected output:
  - `check` succeeds
  - `run` exits successfully
  - `verify` accepts the compiled `.smc`
- non-goals:
  - no host effects
  - no UI
  - no package or release packaging work
