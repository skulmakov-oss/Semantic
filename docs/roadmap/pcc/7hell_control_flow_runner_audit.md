# 7hell Control-flow Runner Audit

Status: PCC-CF-6B preflight audit

This note identifies how the current `7hell` runner is structured before any
control-flow qualification wiring is added.

It is intentionally descriptive. It does not change the runner. It exists so
the next patch can be small and deliberate instead of speculative.

## Goal

Identify where the current `7hell` runner defines fixture selection, pass/fail
expectations, and reporting.

## Findings

### Runner entrypoint

- `tools/7hell/run.ps1`
- `tools/7hell/run.sh`

### Fixture registry

- none

The current runner does not expose a fixture registry or table-driven group
selector. It is a linear shell script with hardcoded gate commands.

### Group selector

- none

There is no `--group`-style selector and no group-dispatch layer in the current
runner scripts.

### Positive fixture model

Observed behavior:

- Hell 6 hardcodes specific positive source files to compile into
  `target/7hell/`
- success is determined by command exit status
- outputs are cleaned up after the smoke step

### Negative fixture model

Observed behavior:

- Hell 4 and other diagnostics live in separate test suites
- the runner itself does not provide a dedicated negative corpus dispatcher
- failure is controlled by the underlying `cargo test` / `cargo run` exit code

### Exit code policy

Observed behavior:

- each shell step uses command exit status as the pass/fail boundary
- the runner fails fast on the first non-zero exit code
- there is no separate per-fixture result registry in the script

### Report format

Observed behavior:

- human-readable gate headings are printed for Hell 1 through Hell 7
- success is reported with `PASS: Hell N`
- a final `ALL 7 GATES PASSED!` banner is printed when the linear script
  completes

## Recommended minimal integration strategy

The smallest safe way to add a control-flow qualification group is to extend
the existing Hell 6 smoke step or add one new fixed step in the shell scripts,
for example:

- compile the existing canonical control-flow examples explicitly;
- run `cargo test --test pcc_control_flow_negative`;
- keep the runner linear and fail-fast.

Do not invent a new group registry unless the runner is redesigned to support
one.

## Decision

Proceed with PCC-CF-6B wiring only as a fixed runner step, not as a new
registry-based group system.

Current implementation target:

- wire `cargo test --test pcc_control_flow_negative` into Hell 6 in both
  `tools/7hell/run.ps1` and `tools/7hell/run.sh`

## Non-Goals

- No runner redesign.
- No new CLI group selector in this issue.
- No change to `tools/7hell` gate ordering.
- No fixture duplication when existing canonical / negative test coverage can
  be reused.
