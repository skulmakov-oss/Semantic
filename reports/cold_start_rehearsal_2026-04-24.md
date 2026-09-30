# Cold-Start Rehearsal 2026-04-24

Status: completed onboarding rehearsal note for `PR-D3`

## Goal

Verify that the current onboarding path can be followed in a fresh checkout
without undocumented steps.

This note is evidence for the current onboarding surface only.

It does not:

- widen the release promise
- promote current-`main` behavior into `published stable`
- replace `docs/roadmap/v1_readiness.md` or
  `reports/g1_release_scope_statement.md`

## Rehearsal Setup

Fresh rehearsal worktree:

- `<local clone>/EXOcode-d3` (machine-local absolute path redacted by Issue #1969)

Cold build setup:

- fresh worktree from current `origin/main`
- dedicated cold target dir:
  - `<local clone>/EXOcode-d3/.cold_start_target`

Repository docs exercised:

- `docs/getting_started.md`
- `docs/examples_index.md`

Environment observed during rehearsal:

- OS: `Microsoft Windows NT 10.0.26300.0`
- `rustc 1.93.1 (01f6ddf75 2026-02-11)`
- `cargo 1.93.1 (083ac5135 2025-12-15)`

## Executed Path

The following onboarding path was executed successfully:

1. `cargo build --bin smc --bin svm`
2. create minimal `program.sm`
3. `cargo run --bin smc -- check program.sm`
4. `cargo run --bin smc -- run program.sm`
5. `cargo run --bin smc -- compile program.sm -o program.smc`
6. `cargo run --bin smc -- verify program.smc`
7. `cargo run --bin smc -- run-smc program.smc`
8. `cargo run --bin svm -- disasm program.smc`
9. `cargo run --bin smc -- check examples/canonical/cli_batch_core/src/main.sm`
10. `cargo run --bin smc -- run examples/canonical/cli_batch_core/src/main.sm`
11. `cargo run --bin smc -- compile examples/canonical/cli_batch_core/src/main.sm -o cli_batch_core.smc`
12. `cargo run --bin smc -- verify cli_batch_core.smc`
13. `cargo test -q --test canonical_examples`
14. `cargo test -q --test public_api_contracts`

Generated rehearsal artifacts were removed after the run:

- `program.sm`
- `program.smc`
- `cli_batch_core.smc`

## Timings

Observed wall-clock timings:

- `build_public_entrypoints`: `52.91s`
- `check_minimal_program`: `0.62s`
- `run_minimal_program`: `0.58s`
- `compile_minimal_program`: `0.62s`
- `verify_minimal_program`: `0.51s`
- `run_verified_minimal_program`: `0.36s`
- `disasm_minimal_program`: `0.29s`
- `check_canonical_cli_batch_core`: `0.31s`
- `run_canonical_cli_batch_core`: `0.34s`
- `compile_verify_canonical_cli_batch_core`: `0.56s`
- `canonical_examples_test`: `3.79s`
- `public_api_contracts_test`: `1.89s`

## Observed Friction

No blocking onboarding defect was found in the exercised path.

Non-blocking friction observed:

- the first cold build took roughly `53s` on the capture machine
- repeated `cargo run` invocations emitted existing `dead_code` warnings from:
  - `crates/sm-front/src/typecheck.rs`
  - `crates/sm-vm/src/semcode_vm.rs`
- the warnings did not block the path, but they add noise to the first-run
  experience

## Verdict

The documented onboarding path in:

- `docs/getting_started.md`
- `docs/examples_index.md`

is currently reproducible on a fresh worktree and does not require an
undocumented workaround.

Current D3 reading:

- green

Contingency reading for `PR-D4.1`:

- not triggered by this rehearsal

## Scope Note

This rehearsal confirms the current onboarding path only.

It does not claim:

- broader public-release readiness
- broader executable-module authoring
- broader practical-programming scope beyond the current qualified contour
