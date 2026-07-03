# loop_control_flow

- benchmark-class application anchor: practical loop-driven control-flow over
  admitted `while`, `loop`, `break`, and `continue`
- purpose: show deterministic looping with mutable rebinding, early return, and
  explicit loop termination
- demonstrates:
  - `while` with a `bool` condition
  - statement `loop`
  - `break;` and `continue;` in admitted loop bodies
  - mutable rebinding inside loops
  - terminal return paths after loops
  - assert-based self-checks
- commands:
  - `cargo run --bin smc -- check examples/canonical/loop_control_flow/src/main.sm`
  - `cargo run --bin smc -- run examples/canonical/loop_control_flow/src/main.sm`
  - `cargo run --bin smc -- compile examples/canonical/loop_control_flow/src/main.sm -o out.smc`
  - `cargo run --bin smc -- verify out.smc`
- expected output:
  - `check` succeeds
  - `run` exits successfully
  - `verify` accepts the compiled `.smc`
- non-goals:
  - no `break expr;`
  - no loop-expression surface
  - no `for` surface
  - no labeled loops
