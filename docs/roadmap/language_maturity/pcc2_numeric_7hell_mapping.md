# PCC-2 Numeric to 7hell Mapping

Status: draft mapping note
Track: PCC-2E 7hell mapping for numeric gates
Layer: language maturity / qualification mapping
Scope: documentation only
Implementation: out of scope
Owner: language maturity stream

Related:

- `tests/pcc2_numeric_core_gate.rs`
- `tests/pcc2_numeric_diagnostics.rs`
- `tests/pcc2_numeric_lowering_stability.rs`
- `tests/fixtures/pcc2_numeric/`
- `pcc2_numeric_ctf_impact_check.md`
- `7hell_qualification_contract.md`
- `practical_core_completion_v0_3.md`

## 1. Purpose

This document maps PCC-2 numeric coverage to the existing 7hell qualification
stages.

The mapping is descriptive only. It records what the already-merged PCC-2A to
PCC-2D coverage pressures in the qualification contract.

## 2. Non-goals

This document does not implement:

- `smc 7hell`;
- a 7hell harness;
- a fixture runner;
- JSON 7hell output;
- CLI behavior changes;
- VM, verifier, or runtime behavior changes;
- numeric feature expansion;
- PCC-3 start;
- Workbench, UI, or I70 work.

Rule:

```text
mapping only, no 7hell implementation
```

## 3. Source coverage

PCC-2 numeric mapping sources:

- `#588` PCC-2A numeric core gate fixtures
- `#589` PCC-2B numeric diagnostics hardening
- `#590` PCC-2C numeric lowering / SemCode stability audit
- `#591` PCC-2D numeric CTF impact check

PCC-2 numeric coverage files:

- `tests/pcc2_numeric_core_gate.rs`
- `tests/pcc2_numeric_diagnostics.rs`
- `tests/pcc2_numeric_lowering_stability.rs`
- `tests/fixtures/pcc2_numeric/`
- `docs/roadmap/language_maturity/pcc2_numeric_ctf_impact_check.md`

Current numeric truth reviewed:

- `i32`: literals, arithmetic, comparisons, numeric control-flow conditions
- `u32`: basic equality path only
- `f64`: basic arithmetic / public pipeline path
- `fx`: basic literal / arithmetic / equality public pipeline path

## 4. Stage mapping

| Stage | Relevance | PCC-2 source coverage | What it proves | Current status |
|---|---|---|---|---|
| Syntax Hell | direct | `pcc2_numeric_core_gate.rs` + `pcc2_numeric_diagnostics.rs` | admitted numeric literals and expressions parse through the public pipeline, and invalid numeric forms are rejected early. | partially covered |
| Type Hell | direct | `pcc2_numeric_core_gate.rs` + `pcc2_numeric_diagnostics.rs` | numeric assignment mismatches, comparison mismatches, and bool / numeric condition mistakes fail with stable diagnostics. | covered |
| Lowering Hell | direct | `pcc2_numeric_lowering_stability.rs` | admitted numeric fixtures lower / emit deterministically and repeated compiles produce identical `.smc` bytes. | covered |
| Verifier Hell | direct | `pcc2_numeric_lowering_stability.rs` + `pcc2_numeric_core_gate.rs` | emitted numeric artifacts verify through the existing verifier path before `run-smc`. | covered |
| VM Hell | direct | `pcc2_numeric_core_gate.rs` + `pcc2_numeric_lowering_stability.rs` | admitted numeric artifacts run through the existing VM path and remain stable across repeated emission. | covered |
| Practical Hell | direct | `pcc2_numeric_core_gate.rs` + `pcc2_numeric_lowering_stability.rs` + `pcc2_numeric_diagnostics.rs` | the admitted numeric surface survives the public pipeline end-to-end, and invalid numeric programs do not become valid verified SemCode. | covered |
| User Pain / Diagnostics Hell | direct | `pcc2_numeric_diagnostics.rs` | invalid numeric failures report stable, construct-specific diagnostics rather than vague failures. | covered |

## 5. Numeric feature mapping

| PCC-2 numeric surface | 7hell pressure | Primary source coverage | Notes |
|---|---|---|---|
| `i32` literals | Syntax Hell | `pcc2_numeric_core_gate.rs`, `pcc2_numeric_lowering_stability.rs` | admitted numeric surface in the public pipeline. |
| `i32` arithmetic | Syntax Hell, Lowering Hell, VM Hell | `pcc2_numeric_core_gate.rs`, `pcc2_numeric_lowering_stability.rs` | repeated compiles produce stable SemCode for admitted `i32` arithmetic examples. |
| `i32` comparisons | Type Hell, Practical Hell | `pcc2_numeric_core_gate.rs`, `pcc2_numeric_lowering_stability.rs` | comparison forms are part of the admitted numeric gate. |
| numeric expressions in bool conditions via comparison | Type Hell, Practical Hell | `pcc2_numeric_core_gate.rs`, `pcc2_numeric_diagnostics.rs` | bool control-flow conditions are driven by numeric comparison, not by implicit numeric truthiness. |
| `u32` basic equality | Syntax Hell, Practical Hell | `pcc2_numeric_core_gate.rs`, `pcc2_numeric_lowering_stability.rs` | only the basic equality path is confirmed in current truth. |
| `f64` basic arithmetic | Syntax Hell, Lowering Hell, VM Hell | `pcc2_numeric_core_gate.rs`, `pcc2_numeric_lowering_stability.rs` | basic arithmetic / public pipeline coverage is confirmed for the current fixture set. |
| `fx` basic literal / arithmetic / equality | Syntax Hell, Type Hell, Lowering Hell | `pcc2_numeric_core_gate.rs`, `pcc2_numeric_diagnostics.rs`, `pcc2_numeric_lowering_stability.rs` | basic `fx` admission is covered, but not a full fixed-point policy. |
| assignment mismatch diagnostics | User Pain / Diagnostics Hell | `pcc2_numeric_diagnostics.rs` | stable error fragments are asserted for invalid numeric assignment cases. |
| arithmetic mismatch diagnostics | User Pain / Diagnostics Hell | `pcc2_numeric_diagnostics.rs` | stable error fragments are asserted for mixed or unsupported arithmetic cases. |
| comparison mismatch diagnostics | User Pain / Diagnostics Hell | `pcc2_numeric_diagnostics.rs` | stable error fragments are asserted for invalid comparison cases. |
| bool / numeric condition diagnostics | Type Hell, User Pain / Diagnostics Hell | `pcc2_numeric_diagnostics.rs` | invalid numeric conditions are rejected through the public CLI route. |
| repeated SemCode byte stability | Lowering Hell, Verifier Hell, VM Hell | `pcc2_numeric_lowering_stability.rs` | repeated compiles of admitted numeric fixtures emit identical `.smc` bytes. |
| invalid numeric sources do not verify | Lowering Hell, Verifier Hell | `pcc2_numeric_lowering_stability.rs` | invalid numeric sources do not produce valid verified SemCode. |
| CTF impact guard | Verifier Hell, VM Hell, Practical Hell | `pcc2_numeric_ctf_impact_check.md` | numeric phase impact is recorded as passed without trust classification changes. |

## 6. Current truth / non-expansion statement

This document does not expand numeric truth.

It does not claim:

- full `u32` arithmetic;
- full `f64` math builtin coverage;
- full `fx` fixed-point policy;
- implicit numeric conversions;
- cross-family numeric arithmetic;
- numeric backend portability proof;
- PCC-3 readiness.

## 7. Gaps

The mapping is intentionally incomplete in implementation terms.

Not implemented or not covered here:

- no `smc 7hell`;
- no JSON 7hell output;
- no unified 7hell report;
- no 7hell fixture runner;
- no stage-level pass/fail aggregator;
- no full `fx` fixed-point policy;
- no `u32` arithmetic gate;
- no full `f64` math builtin qualification;
- no numeric overflow / trap taxonomy expansion;
- no numeric golden trace suite;
- no cross-backend numeric equivalence suite.

These are future work and remain out of scope for PCC-2E.

## 8. PCC impact

PCC-2E does not add numeric behavior.

It records qualification mapping for the already merged PCC-2A, PCC-2B,
PCC-2C, and PCC-2D coverage.

It prepares PCC-2F closeout without claiming that 7hell is implemented.

## 9. Acceptance checklist

- PCC-2 test / doc coverage is mapped to 7hell stages.
- All seven stages are represented.
- Numeric truth is not expanded.
- Gaps are explicit.
- No 7hell command is implemented.
- No harness is implemented.
- No code, test, or fixture changes are introduced.
- Workbench, UI, and I70 remain untouched.
- CTF classifications remain unchanged.
- PCC-3 not started.
