# Implementation Plan: Quad Logic Calculator (Corrective)

## Overview
This plan outlines the redesign of the Quad Logic Calculator to ensure strict ownership boundaries: Semantic (`.sm`) will exclusively own the state machine and logical decisions (including Quad native logic), while Rust will be strictly relegated to parsing the `proj.sm` via the canonical pipeline and acting as the event transport/Shell Player shell.

## Open Questions

> [!WARNING]
> **Semantic VM Invocation:** In the current Semantic v0 spec, there is no public `invoke(args)` function in `sm-vm` to call an arbitrary Semantic function from Rust with an `action_id`. If `smc run-smc` is the only way to execute semantic bytecode, should the Rust bridge spawn `smc run-smc` as a subprocess passing the `action_id` via a side-channel (e.g. stdout/stdin or a file), or should we use `sm-vm` directly and inject the state as a host context/observable? What is the expected mechanical boundary for passing the `admitted action intent` into the `.sm` logic and receiving the updated state?

> [!WARNING]
> **Updating the Projection:** After Semantic computes the new state, how should Rust mutate the `UiProjectionArtifact` given that Grammar v0 cannot bind state automatically? Are we expected to generate a `ProjectionPatch` (like `ReferenceContour` does) on the Rust side based on the Semantic state output, or should the Semantic script itself generate the Patch operations?

## Proposed Changes

### `examples/experimental/quad_logic_calculator/src/calculator.sm`
- **State Machine Definition**: Define a strict `Record` or `Enum` for the calculator state (mode, current_input, stored_operand, etc.).
- **Transitions**: Implement transition functions that take `(State, ActionId) -> State`.
- **Quad Native Operations**: Use the native `!a`, `a && b`, `a || b`, `a -> b`, `a == b`, `a != b` operators instead of manual truth tables.
- **Evaluation State**: Maintain explicit `N`, `T`, `F`, `S` states for the calculation outcome.

### `examples/experimental/quad_logic_calculator/src/quad_calc.proj.sm`
- Create a complete and compliant grammar v0 structure utilizing allowed roles.
- Include proper nesting: application surface, mode selector, expression display, result display, arithmetic controls, quad controls, etc.

### `examples/experimental/quad_logic_calculator/src/main.rs`
- Remove all hard-coded `UiProjectionArtifact` construction.
- Load `quad_calc.proj.sm` via the canonical pipeline: `compile_projection_source_to_bundle_v0` -> `activate_projection_bundle_v0_gate_d`.
- Establish a `DesktopSession` shell player using `prom_ui_runtime`.
- Implement an interaction loop that captures input events, produces `SemanticIntent`, admits them through an admission boundary, passes the action to `calculator.sm`, and then applies the resulting state back to the UI via projection patches.

## Verification Plan

### Automated Tests
- Run `cargo test -p quad_logic_calculator`
- Run `smc check`, `smc compile`, `smc verify` on the `.sm` files.
- Add local tests in `main.rs` that simulate input streams and verify deterministic state transitions across all required quad states (`N/F/T/S`) and division-by-zero rejection.

### Manual Verification
- Compile and run the desktop app `cargo run -p quad_logic_calculator` to ensure visual layout correctness and interaction loops function without panics.
