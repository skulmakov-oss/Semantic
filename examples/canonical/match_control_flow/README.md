# match_control_flow

- benchmark-class application anchor: practical `match`-driven control-flow
  over `quad`
- purpose: show explicit quad-state branching without implicit truthiness
- demonstrates:
  - `match` over `quad`
  - explicit `T / F / N / S` handling
  - deterministic branch selection
  - terminal return paths through ordinary control flow
  - ordinary `if` inside a `match` branch
- commands:
  - `cargo run --bin smc -- check examples/canonical/match_control_flow/src/main.sm`
  - `cargo run --bin smc -- run examples/canonical/match_control_flow/src/main.sm`
  - `cargo run --bin smc -- compile examples/canonical/match_control_flow/src/main.sm -o out.smc`
  - `cargo run --bin smc -- verify out.smc`
- expected output:
  - `check` succeeds
  - `run` exits successfully
  - `verify` accepts the compiled `.smc`
- non-goals:
  - no Option / Result surface
  - no pattern guards
  - no fallthrough semantics
- no implicit quad truthiness
- self-check via assertions rather than return code
