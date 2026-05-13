# PCC-3 Text to 7hell Mapping

Status: draft mapping note
Track: PCC-3E 7hell mapping for text gates
Layer: language maturity / qualification mapping
Scope: documentation only
Implementation: out of scope
Owner: language maturity stream

Related:

- `docs/roadmap/language_maturity/pcc3_text_surface_boundary_reset.md`
- `docs/roadmap/language_maturity/pcc3_text_ctf_impact_check.md`
- `docs/roadmap/language_maturity/practical_core_completion_v0_3.md`
- `tests/pcc3_text_core_gate.rs`
- `tests/pcc3_text_diagnostics.rs`
- `tests/pcc3_text_lowering_stability.rs`
- `tests/fixtures/pcc3_text/`
- `7hell_qualification_contract.md`

## 1. Purpose

This document maps PCC-3 text/string coverage to the existing seven 7hell
qualification stages.

The mapping is descriptive only. It records what the already-merged PCC-3-0 to
PCC-3D coverage pressures in the qualification contract.

## 2. Non-goals

This document does not implement:

- `smc 7hell`;
- a 7hell harness;
- a fixture runner;
- JSON output;
- CLI behavior changes;
- VM, verifier, or runtime behavior changes;
- text feature expansion;
- Hello World implementation;
- `print` / `observe` implementation;
- observation / I/O widening;
- canonical vocabulary changes;
- Linguist readiness;
- Workbench, UI, or I70 work.

Rule:

```text
mapping only, no 7hell implementation
```

## 3. Source coverage

PCC-3 text mapping sources:

- `#596` PCC-3-0 text surface boundary reset
- `#598` PCC-3A text/string core gate fixtures
- `#600` PCC-3B text diagnostics and conversion boundaries
- `#602` PCC-3C text lowering / SemCode stability audit
- `#604` PCC-3D text CTF impact check

PCC-3 text coverage files:

- `docs/roadmap/language_maturity/pcc3_text_surface_boundary_reset.md`
- `docs/roadmap/language_maturity/pcc3_text_ctf_impact_check.md`
- `tests/pcc3_text_core_gate.rs`
- `tests/pcc3_text_diagnostics.rs`
- `tests/pcc3_text_lowering_stability.rs`
- `tests/fixtures/pcc3_text/`

Current text truth reviewed:

- text literals: supported
- text binding: supported
- text equality: supported
- `to_text(text)`: supported
- `to_text(i32/u32/bool/quad)`: supported
- text concatenation: supported for `text + text`
- text as control-flow condition: rejected
- text `+` non-text: rejected
- `to_text(record)`: rejected / unsupported
- Hello World: required later, not implemented
- `print` / `observe`: not implemented
- canonical vocabulary: guarded by PCC-3-0 / `#478` / `#479`

## 4. Stage mapping

| Stage | Relevance | PCC-3 source coverage | What it proves | Current status |
|---|---|---|---|---|
| Syntax Hell | direct | `pcc3_text_core_gate.rs` + `pcc3_text_diagnostics.rs` | text literals, text bindings, text equality, and `to_text(...)` surfaces are admitted through the public CLI path, while malformed text forms fail early. | partially covered |
| Type Hell | direct | `pcc3_text_core_gate.rs` + `pcc3_text_diagnostics.rs` | text assignment mismatches, scalar-from-text mismatches, and text control-flow condition mistakes fail with stable diagnostics. | covered |
| Lowering Hell | direct | `pcc3_text_lowering_stability.rs` | admitted text fixtures compile twice to identical `.smc` bytes. | covered |
| Verifier Hell | direct | `pcc3_text_core_gate.rs` + `pcc3_text_lowering_stability.rs` + `pcc3_text_diagnostics.rs` | emitted text artifacts verify through the existing verifier path, and invalid text sources do not produce valid verified SemCode. | covered |
| VM Hell | direct | `pcc3_text_core_gate.rs` + `pcc3_text_lowering_stability.rs` | admitted text artifacts run through `smc run-smc` after verification. | covered |
| Practical Hell | direct | `pcc3_text_core_gate.rs` + `pcc3_text_diagnostics.rs` + `pcc3_text_lowering_stability.rs` | the admitted text surface survives the public pipeline end-to-end, while invalid programs are rejected before valid SemCode can be accepted. | covered |
| User Pain / Diagnostics Hell | direct | `pcc3_text_diagnostics.rs` | invalid text failures report stable, construct-specific diagnostics rather than vague failures. | covered |

## 5. Text feature mapping

| PCC-3 text surface | 7hell pressure | Primary source coverage | Notes |
|---|---|---|---|
| text literals | Syntax Hell, Practical Hell | `pcc3_text_core_gate.rs`, `pcc3_text_lowering_stability.rs` | admitted in the public pipeline and preserved across repeated compiles. |
| text binding | Syntax Hell, Type Hell | `pcc3_text_core_gate.rs`, `pcc3_text_diagnostics.rs` | current bridge syntax exercises text variable binding without defining canonical surface vocabulary. |
| text equality | Syntax Hell, Type Hell, Lowering Hell | `pcc3_text_core_gate.rs`, `pcc3_text_lowering_stability.rs` | same-family text equality is admitted and stable under repeated emission. |
| text `+` text concatenation | Syntax Hell, Lowering Hell, VM Hell | `pcc3_text_core_gate.rs`, `pcc3_text_lowering_stability.rs` | supported current truth is recorded without broadening concatenation policy. |
| `to_text(text)` | Syntax Hell, Practical Hell | `pcc3_text_core_gate.rs`, `pcc3_text_lowering_stability.rs` | identity conversion remains part of current truth and is emission-stable. |
| `to_text(i32/u32/bool/quad)` | Syntax Hell, Practical Hell | `pcc3_text_core_gate.rs`, `pcc3_text_lowering_stability.rs` | scalar conversion support is recorded as already supported current truth. |
| text assignment mismatch diagnostics | Type Hell, User Pain / Diagnostics Hell | `pcc3_text_diagnostics.rs` | stable mismatch diagnostics are asserted for `text <- i32`, `text <- bool`, `text <- quad`, and scalar-from-text cases. |
| scalar assigned from text diagnostics | Type Hell, User Pain / Diagnostics Hell | `pcc3_text_diagnostics.rs` | scalar destinations reject text sources with stable fragments. |
| text control-flow condition diagnostics | Type Hell, User Pain / Diagnostics Hell | `pcc3_text_diagnostics.rs` | `if "hello"` and `while "hello"` reject through the public CLI path. |
| text `+` non-text diagnostics | Type Hell, User Pain / Diagnostics Hell | `pcc3_text_diagnostics.rs` | scalar operands are rejected for text concatenation boundary cases. |
| text comparison mismatch diagnostics | Type Hell, User Pain / Diagnostics Hell | `pcc3_text_diagnostics.rs` | non-text comparison mismatches are rejected with current comparison fragments. |
| unsupported `to_text(record)` diagnostics | Type Hell, User Pain / Diagnostics Hell | `pcc3_text_diagnostics.rs` | unsupported record conversion remains rejected. |
| repeated SemCode byte stability | Lowering Hell, Verifier Hell, VM Hell | `pcc3_text_lowering_stability.rs` | repeated compiles of admitted text fixtures emit identical `.smc` bytes. |
| invalid text sources do not verify | Lowering Hell, Verifier Hell | `pcc3_text_lowering_stability.rs` | invalid text sources fail before becoming valid verified SemCode. |
| CTF impact guard | Verifier Hell, VM Hell, Practical Hell | `pcc3_text_ctf_impact_check.md` | the text surface is recorded as not changing trust classifications. |
| surface boundary guard | Syntax Hell, Type Hell, Practical Hell | `pcc3_text_surface_boundary_reset.md` | canonical text surface vocabulary remains guarded by PCC-3-0 and future surface work. |

## 6. Current truth / non-expansion statement

This document does not expand text truth.

It does not claim:

- Hello World support;
- `print` support;
- `observe` support;
- general stdout;
- interpolation / formatting;
- file I/O;
- stdin;
- network;
- general host ABI;
- canonical `entry` / `observe` syntax as executable;
- full text standard library;
- Linguist readiness;
- final canonical vocabulary.

## 7. Gaps

The mapping is intentionally incomplete in implementation terms.

Not implemented or not covered yet:

- no `smc 7hell`;
- no JSON 7hell output;
- no unified 7hell report;
- no 7hell fixture runner;
- no stage-level pass/fail aggregator;
- no Hello World / controlled observation implementation;
- no `print` / `observe` admission;
- no observation/effect capability path for text output;
- no text formatting/interpolation;
- no full text stdlib;
- no text performance benchmark gate;
- no text golden trace suite;
- no cross-backend text equivalence suite;
- no final canonical vocabulary;
- no Linguist readiness.

These are future work and must not be implemented in this PR.

## 8. PCC impact

PCC-3E does not add text behavior.

It records qualification mapping for already merged PCC-3-0 to PCC-3D
coverage.

It prepares PCC-3F closeout without closing PCC-3 yet.

## 9. Acceptance checklist

- PCC-3 test / doc coverage is mapped to 7hell stages
- all seven stages are represented
- text truth is not expanded
- observation / I/O boundary remains closed
- gaps are explicit
- no command implementation
- no harness implementation
- no code, test, or fixture changes
- no Hello World
- no `print` / `observe`
- no Workbench / UI / I70
- no CTF classification changes
- no Linguist readiness
- PCC-3 not fully closed yet
