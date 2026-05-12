# PCC-1 Control Flow Closeout

Status: draft closeout note
Track: PCC-1F close PCC-1 control-flow phase
Layer: language maturity / closeout
Scope: documentation only
Implementation: out of scope
Owner: language maturity stream

Related:

- `practical_core_completion_v0_3.md`
- `practical_core_feature_matrix_live_audit.md`
- `pcc1_control_flow_7hell_mapping.md`
- `tests/pcc1_control_flow_gate.rs`
- `tests/pcc1_control_flow_lowering_stability.rs`
- `tests/pcc1_control_flow_diagnostics.rs`
- `tests/fixtures/pcc1_control_flow/`

## 1. Purpose

This document closes PCC-1 Control Flow Core for the current Practical Core
Completion scope.

It records that the merged PCC-1A..PCC-1E work covers the intended
control-flow gate surfaces and establishes the boundary before PCC-2.

## 2. Closure basis

| PR | Purpose | What it proved | Evidence |
|---|---|---|---|
| PCC-1A | dedicated control-flow gate fixtures | `if / else`, `while`, statement `loop`, `break`, and `continue` survive the public control-flow gate path. | `tests/pcc1_control_flow_gate.rs`; `tests/fixtures/pcc1_control_flow/`; `cargo test -q --test pcc1_control_flow_gate`; merged PR `#566` |
| PCC-1B | nested loop-control diagnostics | nested `while` / `loop` behavior and invalid loop-control scope exits remain stable. | `tests/pcc1_control_flow_gate.rs`; `tests/fixtures/pcc1_control_flow/`; `cargo test -q --test pcc1_control_flow_gate`; merged PR `#567` |
| PCC-1C | lowering / emission stability audit | repeated compiles of admitted control-flow fixtures emit byte-stable SemCode and verify through the public CLI path. | `tests/pcc1_control_flow_lowering_stability.rs`; `cargo test -q --test pcc1_control_flow_lowering_stability`; merged PR `#575` |
| PCC-1D | diagnostic hardening | invalid control-flow programs fail with stable, useful, construct-specific diagnostics through the public CLI path. | `tests/pcc1_control_flow_diagnostics.rs`; `cargo test -q --test pcc1_control_flow_diagnostics`; merged PR `#576` |
| PCC-1E | 7hell mapping | PCC-1 control-flow coverage is mapped to the seven 7hell qualification stages without implementing 7hell. | `docs/roadmap/language_maturity/pcc1_control_flow_7hell_mapping.md`; merged PR `#577` |

## 3. Closed control-flow surface

The following surfaces are closed for the current PCC-1 scope.

### 3.1 Positive pipeline surfaces

- `if / else` positive pipeline
- `while` positive pipeline
- statement `loop` positive pipeline
- `break` inside loop
- `continue` inside loop
- nested `while`
- nested `loop`
- inner `break`
- inner `continue`
- outer `continue` after inner `break`

### 3.2 Negative diagnostics surfaces

- `break` outside loop rejection
- `continue` outside loop rejection
- `break` after nested loop exits rejection
- `continue` after nested loop exits rejection
- `while` non-bool condition rejection
- `if` quad / non-bool condition rejection

### 3.3 Stability and qualification surfaces

- repeated SemCode byte stability for control-flow fixtures
- `verify` plus `run-smc` coverage for emitted artifacts
- diagnostic stability for invalid control-flow cases
- 7hell mapping coverage

## 4. Explicit non-goals / deferred work

PCC-1 closeout does not include:

- `break` with value;
- labeled loops;
- `for` loops;
- `match` control-flow closeout;
- advanced CFG optimization;
- exhaustive path analysis;
- unreachable-code diagnostics;
- 7hell command implementation;
- 7hell JSON output;
- 7hell harness;
- PCC-2 numeric work;
- Workbench / UI / I70;
- CTF policy changes.

Rule:

```text
PCC-1 Control Flow Core is closed for the current PCC scope.
Deferred items remain future PCC work or separate trust-lane work.
```

## 5. 7hell impact

PCC-1 is mapped to 7hell stages, but 7hell is not implemented.

This closeout records qualification mapping only. It does not add a 7hell
command, runner, report format, or harness.

## 6. CTF impact

No CTF trust policy changes are introduced by this closeout PR.

CTF guard remains a separate issue / trust lane.

This document does not claim CTF is closed by PCC-1F.

## 7. PCC status

```text
PCC-1 Control Flow Core: closed
PCC-2 Numeric Core start status: eligible after maintainer acceptance
```

PCC-2 may begin only after maintainer acceptance of this closeout and after
deciding whether to resolve guard issues first.

Do not interpret this as full language readiness.

## 8. Acceptance checklist

- PCC-1A..PCC-1E landed
- all intended control-flow gate coverage recorded
- deferred items explicit
- no code, test, or fixture changes
- no 7hell implementation
- no Workbench / UI / I70
- no CTF policy changes
- PCC-2 not started

