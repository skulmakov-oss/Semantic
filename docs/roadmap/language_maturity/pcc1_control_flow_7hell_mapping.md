# PCC-1 Control Flow to 7hell Mapping

Status: draft mapping note
Track: PCC-1E 7hell mapping for control-flow gates
Layer: language maturity / qualification mapping
Scope: documentation only
Implementation: out of scope
Owner: language maturity stream

Related:

- `tests/pcc1_control_flow_gate.rs`
- `tests/pcc1_control_flow_lowering_stability.rs`
- `tests/pcc1_control_flow_diagnostics.rs`
- `tests/fixtures/pcc1_control_flow/`
- `7hell_qualification_contract.md`
- `practical_core_completion_v0_3.md`

## 1. Purpose

This document maps PCC-1 control-flow gate coverage to the existing 7hell
qualification stages.

The mapping is descriptive only. It records what the already-merged PCC-1A to
PCC-1D test coverage pressures in the qualification contract.

## 2. Non-goals

This document does not implement:

- `smc 7hell`;
- a 7hell harness;
- fixture execution changes;
- CLI behavior changes;
- VM, verifier, or runtime behavior changes;
- Workbench, UI, or I70 work.

Rule:

```text
mapping only, no 7hell implementation
```

## 3. Source coverage

PCC-1 control-flow mapping sources:

- `tests/pcc1_control_flow_gate.rs`
- `tests/pcc1_control_flow_lowering_stability.rs`
- `tests/pcc1_control_flow_diagnostics.rs`
- `tests/fixtures/pcc1_control_flow/`

PCC-1 coverage summary:

- PCC-1A: dedicated control-flow gate fixtures
- PCC-1B: nested loop-control diagnostics
- PCC-1C: lowering/emission stability audit
- PCC-1D: control-flow diagnostic hardening

## 4. Stage mapping

| Stage | Relevance | PCC-1 source coverage | What it proves | Current status |
|---|---|---|---|---|
| Syntax Hell | direct | `pcc1_control_flow_gate.rs` + `pcc1_control_flow_diagnostics.rs` | `if`, `while`, `loop`, `break`, and `continue` parse in the public control-flow gate set, while malformed loop-control forms fail early. | covered |
| Type Hell | direct | `pcc1_control_flow_gate.rs` + `pcc1_control_flow_diagnostics.rs` | `while` requires a bool condition, `if` quad/non-bool conditions are rejected, and invalid loop-control usage gets stable diagnostics. | covered |
| Lowering Hell | direct | `pcc1_control_flow_lowering_stability.rs` | repeated compiles of admitted control-flow fixtures emit stable SemCode bytes. | covered |
| Verifier Hell | direct | `pcc1_control_flow_gate.rs` + `pcc1_control_flow_lowering_stability.rs` | emitted SemCode verifies through the public CLI path before `run-smc`. | covered |
| VM Hell | direct | `pcc1_control_flow_gate.rs` + `pcc1_control_flow_lowering_stability.rs` | verified control-flow SemCode runs through the public CLI path and remains deterministic across repeated compilation. | covered |
| Practical Hell | direct | `pcc1_control_flow_gate.rs` + `pcc1_control_flow_lowering_stability.rs` + `pcc1_control_flow_diagnostics.rs` | control-flow programs survive the public pipeline end-to-end, and invalid programs are rejected before valid SemCode can be accepted. | covered |
| User Pain / Diagnostics Hell | direct | `pcc1_control_flow_diagnostics.rs` | invalid control-flow failures report stable, construct-specific diagnostics rather than vague failures. | covered |

## 5. Control-flow feature mapping

| PCC-1 control-flow feature | 7hell pressure | Primary source coverage | Notes |
|---|---|---|---|
| if / else | Syntax Hell, Type Hell, Practical Hell | `pcc1_control_flow_gate.rs`, `pcc1_control_flow_lowering_stability.rs`, `pcc1_control_flow_diagnostics.rs` | Valid `if / else` fixtures prove public pipeline coverage; non-bool/quad condition diagnostics prove stable rejection. |
| while | Syntax Hell, Type Hell, Practical Hell | `pcc1_control_flow_gate.rs`, `pcc1_control_flow_lowering_stability.rs`, `pcc1_control_flow_diagnostics.rs` | Bool-condition handling and repeatable SemCode emission are covered. |
| statement loop | Syntax Hell, Lowering Hell, VM Hell | `pcc1_control_flow_gate.rs`, `pcc1_control_flow_lowering_stability.rs` | Statement `loop` plus `break` / `continue` is exercised through the public CLI path. |
| break | Type Hell, Diagnostics Hell | `pcc1_control_flow_gate.rs`, `pcc1_control_flow_diagnostics.rs` | Valid inner `break` is accepted in nested control-flow, invalid `break` outside a loop rejects stably. |
| continue | Type Hell, Diagnostics Hell | `pcc1_control_flow_gate.rs`, `pcc1_control_flow_diagnostics.rs` | Valid inner `continue` is accepted in nested control-flow, invalid `continue` outside a loop rejects stably. |
| nested while | Lowering Hell, VM Hell | `pcc1_control_flow_gate.rs`, `pcc1_control_flow_lowering_stability.rs` | Nested loop scope behavior is covered by positive and repeated-pipeline tests. |
| nested loop | Lowering Hell, VM Hell | `pcc1_control_flow_gate.rs`, `pcc1_control_flow_lowering_stability.rs` | Nested `loop` + `continue` and `loop` + `break` / outer `continue` are part of the gate. |
| break outside loop | Type Hell, User Pain / Diagnostics Hell | `pcc1_control_flow_gate.rs`, `pcc1_control_flow_diagnostics.rs` | Stable rejection path with a specific construct-level message. |
| continue outside loop | Type Hell, User Pain / Diagnostics Hell | `pcc1_control_flow_gate.rs`, `pcc1_control_flow_diagnostics.rs` | Stable rejection path with a specific construct-level message. |
| non-bool while condition | Type Hell | `pcc1_control_flow_gate.rs`, `pcc1_control_flow_diagnostics.rs` | The control-flow gate now proves bool-condition enforcement is stable. |
| quad / non-bool if condition | Type Hell | `pcc1_control_flow_diagnostics.rs` | The `if T` path documents quad-conditioned rejection using the public CLI. |
| repeated SemCode stability | Lowering Hell, Verifier Hell, VM Hell | `pcc1_control_flow_lowering_stability.rs` | Repeated compiles of the same source produce identical emitted bytes and verified execution remains stable. |

## 6. Gaps

The mapping is intentionally incomplete in implementation terms.

Not implemented here:

- no `smc 7hell`;
- no JSON 7hell output;
- no unified 7hell report;
- no 7hell fixture runner;
- no stage-level pass/fail aggregator.

These are future work and remain out of scope for PCC-1E.

## 7. PCC impact

PCC-1E does not add new control-flow behavior.

It records qualification mapping for the already merged PCC-1A, PCC-1B,
PCC-1C, and PCC-1D coverage.

It prepares PCC-1F closeout without claiming that 7hell is implemented.

## 8. Acceptance checklist

- PCC-1 control-flow coverage is mapped to 7hell stages.
- All seven stages are represented.
- Gaps are explicit.
- No 7hell command is implemented.
- No harness is implemented.
- No code, test, or fixture changes are introduced.
- Workbench, UI, and I70 remain untouched.
- CTF remains untouched.
