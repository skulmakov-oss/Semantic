# PCC-2 Numeric Closeout

Status: draft closeout note
Track: PCC-2F close PCC-2 numeric phase
Layer: language maturity / closeout
Scope: documentation only
Implementation: out of scope
Owner: language maturity stream

Related:

- `practical_core_completion_v0_3.md`
- `practical_core_feature_matrix_live_audit.md`
- `pcc2_numeric_ctf_impact_check.md`
- `pcc2_numeric_7hell_mapping.md`
- `tests/pcc2_numeric_core_gate.rs`
- `tests/pcc2_numeric_diagnostics.rs`
- `tests/pcc2_numeric_lowering_stability.rs`
- `tests/fixtures/pcc2_numeric/`

## 1. Purpose

This document closes PCC-2 Numeric Core for the current Practical Core
Completion scope.

It records that the merged PCC-2A..PCC-2E work covers the intended numeric
gate surfaces and establishes the boundary before PCC-3.

## 2. Closure basis

| PR | Purpose | What it proved | Evidence |
|---|---|---|---|
| PCC-2A | numeric core gate fixtures | admitted numeric fixtures survive the public pipeline for the current numeric truth surface. | `tests/pcc2_numeric_core_gate.rs`; `tests/fixtures/pcc2_numeric/`; `cargo test -q --test pcc2_numeric_core_gate`; merged PR `#588` |
| PCC-2B | numeric diagnostics hardening | invalid numeric programs fail with stable, construct-specific diagnostics through the public CLI path. | `tests/pcc2_numeric_diagnostics.rs`; `cargo test -q --test pcc2_numeric_diagnostics`; merged PR `#589` |
| PCC-2C | lowering / SemCode stability audit | repeated compiles of admitted numeric fixtures emit byte-stable SemCode and verify through the public CLI path. | `tests/pcc2_numeric_lowering_stability.rs`; `cargo test -q --test pcc2_numeric_lowering_stability`; merged PR `#590` |
| PCC-2D | numeric VM / verifier / capability / trap impact check | PCC-2 numeric work did not alter CTF trust classifications or execution-trust boundaries. | `docs/roadmap/language_maturity/pcc2_numeric_ctf_impact_check.md`; merged PR `#591` |
| PCC-2E | 7hell mapping | PCC-2 numeric coverage is mapped to the seven 7hell qualification stages without implementing 7hell. | `docs/roadmap/language_maturity/pcc2_numeric_7hell_mapping.md`; merged PR `#592` |

## 3. Closed numeric surface

The following surfaces are closed for the current PCC-2 scope.

### 3.1 Positive pipeline surfaces

- `i32` literals
- `i32` arithmetic
- `i32` comparisons
- numeric expressions in bool control-flow conditions through comparisons
- `u32` basic equality path
- `f64` basic arithmetic / public pipeline path
- `fx` basic literal / arithmetic / equality public pipeline path

### 3.2 Negative diagnostics surfaces

- assignment mismatch diagnostics
- arithmetic mismatch diagnostics
- comparison mismatch diagnostics
- bool / numeric condition diagnostics
- invalid numeric compile path does not verify

### 3.3 Stability and qualification surfaces

- repeated SemCode byte stability for admitted numeric fixtures
- `verify` plus `run-smc` coverage for emitted numeric artifacts
- CTF impact check passed
- 7hell mapping recorded

## 4. Explicit non-goals / deferred work

PCC-2 closeout does not include:

- full `u32` arithmetic;
- full `f64` math builtin qualification;
- full `fx` fixed-point policy;
- `fx` scale / rounding / overflow / division policy closure;
- implicit numeric conversions;
- cross-family numeric arithmetic;
- numeric overflow trap taxonomy expansion;
- numeric golden trace suite;
- cross-backend numeric equivalence suite;
- numeric performance benchmark gate;
- 7hell command implementation;
- 7hell JSON output;
- 7hell harness;
- PCC-3 text / string phase;
- Workbench / UI / I70;
- package builder.

Rule:

```text
PCC-2 Numeric Core is closed for the current PCC scope.
Deferred items remain future PCC work or separate trust-lane work.
```

## 5. Current numeric truth

```text
i32: supported for literals, arithmetic, comparisons, numeric control-flow conditions
u32: basic equality path only
f64: basic arithmetic / public pipeline path
fx: basic literal / arithmetic / equality public pipeline path
```

This truth is restated here without expansion.

## 6. CTF impact

PCC-2 CTF impact check: `passed`

CTF remains a separate trust lane.

Any future numeric expansion that affects traps, determinism, runtime values,
verifier behavior, SemCode, or capability policy must be checked against the
CTF lane.

This does not mean CTF is closed forever.

This does not mean future numeric expansion is exempt from CTF.

## 7. 7hell impact

PCC-2 is mapped to all seven 7hell stages, but 7hell is not implemented.

This closeout records qualification mapping only. It does not add a 7hell
command, runner, report format, or harness.

## 8. PCC status

```text
PCC-2 Numeric Core: closed
PCC-3 start status: eligible after maintainer acceptance
```

PCC-3 may become eligible after maintainer acceptance, but PCC-3 is not started
by this PR.

Do not interpret this as full numeric readiness.

## 9. Acceptance checklist

- PCC-2A..PCC-2E landed
- intended numeric gate coverage recorded
- closed numeric surface explicit
- deferred items explicit
- no code, test, or fixture changes
- no numeric truth expansion
- no 7hell implementation
- no Workbench / UI / I70
- no CTF policy changes
- PCC-3 not started
