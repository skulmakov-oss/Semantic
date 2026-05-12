# PCC-2 Numeric CTF Impact Check

Status: draft guard note
Track: PCC-2D numeric VM / verifier / capability / trap impact check
Layer: language maturity / trust guard
Scope: documentation only
Implementation: out of scope
Owner: language maturity stream

Related:

- `practical_core_completion_v0_3.md`
- `core_trust_freeze/trap_taxonomy.md`
- `core_trust_freeze/determinism_matrix.md`
- `pcc1_control_flow_ctf_guard.md`
- `tests/pcc2_numeric_core_gate.rs`
- `tests/pcc2_numeric_diagnostics.rs`
- `tests/pcc2_numeric_lowering_stability.rs`

## 1. Purpose

This document records the PCC-2 numeric VM / verifier / capability / trap
impact check.

It states whether the merged PCC-2A, PCC-2B, and PCC-2C work changed any Core
Trust Freeze classifications or execution-trust boundaries.

## 2. Input PRs

| PR | What changed | Code changed? | VM / verifier / CTF docs changed? | CTF touched |
|---|---|---|---|---|
| `#588` PCC-2A | Added the numeric public pipeline gate fixtures and seeded current numeric truth for the admitted PCC-2 numeric surface. | No production code; tests and fixtures only. | No. | `CTF touched: none` |
| `#589` PCC-2B | Added numeric diagnostic hardening tests and targeted numeric fixtures for invalid programs. | No production code; tests and fixtures only. | No. | `CTF touched: none` |
| `#590` PCC-2C | Added numeric lowering / SemCode stability audit coverage for admitted numeric fixtures. | No production code; tests only. | No. | `CTF touched: none` |

## 3. Current numeric truth reviewed

The current numeric truth reviewed for PCC-2 is:

- `i32`: literals, arithmetic, comparisons, numeric control-flow conditions
- `u32`: basic equality path only
- `f64`: basic arithmetic / public pipeline path
- `fx`: basic literal / arithmetic / equality public pipeline path

This document does not expand numeric truth.
It does not claim full `u32` arithmetic.
It does not claim full `f64` math builtins beyond the current fixtures.
It does not claim a full fixed-point policy for `fx`.

## 4. Impact matrix

| Surface | PCC-2A/B/C impact | Result |
|---|---|---|
| Runtime values | Existing numeric runtime values were exercised; no new runtime value kind was added. | no classification change |
| SemCode opcodes | Existing numeric emission was exercised; no new opcode was introduced by PCC-2A/B/C. | no classification change |
| Verifier assumptions | Emitted numeric artifacts verify through the existing verifier path. | no verifier widening |
| VM execution | Admitted numeric artifacts run through the existing VM path. | no VM widening |
| Trap taxonomy | No trap taxonomy row changed. | no classification change |
| Determinism matrix | Repeated `.smc` byte stability is covered by tests; the matrix itself is unchanged. | no classification change |
| Capability / effect policy | Numeric tests do not add effects or capabilities. | no capability change |
| CLI surface | Public `check` / `run` / `compile` / `verify` / `run-smc` routes were used; no CLI expansion occurred. | no CLI widening |
| UI / Workbench / I70 | Untouched. | frozen |

## 5. Guard result

```text
PCC-2 numeric CTF impact check result: passed
```

Meaning:

- PCC-2A/B/C did not change CTF trap taxonomy classifications.
- PCC-2A/B/C did not change CTF determinism matrix classifications.
- PCC-2A/B/C did not change verifier-first, VM, runtime, or capability trust policy.
- PCC-2A/B/C added tests and fixtures only for numeric qualification and stability.
- Numeric runtime behavior was exercised through the existing public pipeline.
- Any future numeric feature expansion must be rechecked against the CTF lane.

## 6. CTF boundary

CTF remains authoritative for:

- trap taxonomy;
- determinism matrix;
- verifier-first policy;
- golden trace policy;
- capability / effect denial policy.

PCC-2 does not own those classifications.

This document does not modify CTF trust policy.

## 7. Open follow-up / not closed by this PR

This PR does not close all PCC-2.

This PR does not start PCC-3.

This PR does not implement 7hell.

This PR does not define full `fx` fixed-point policy.

This PR does not add `u32` arithmetic.

This PR does not add new `f64` math coverage beyond the current fixtures.

This PR does not modify trap or determinism docs.

## 8. Acceptance checklist

- PCC-2 numeric impact check recorded
- no CTF classifications changed
- trap taxonomy untouched
- determinism matrix untouched
- VM / verifier / runtime untouched
- no capability widening
- no CLI widening
- no UI / Workbench / I70
- PCC-3 not started
