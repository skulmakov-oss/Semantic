# PCC-3 Text Closeout

Status: draft closeout note
Track: PCC-3F close PCC-3 text/string phase
Layer: language maturity / closeout
Scope: documentation only
Implementation: out of scope
Owner: language maturity stream

Related:

- `docs/roadmap/language_maturity/pcc3_text_surface_boundary_reset.md`
- `docs/roadmap/language_maturity/pcc3_text_ctf_impact_check.md`
- `docs/roadmap/language_maturity/pcc3_text_7hell_mapping.md`
- `docs/roadmap/language_maturity/practical_core_completion_v0_3.md`
- `#596`
- `#598`
- `#600`
- `#602`
- `#604`
- `#606`

## 1. Purpose

This document closes PCC-3 Text/String Core for the current PCC scope.

It records that the merged PCC-3A through PCC-3E work covers the intended
text gate surfaces and establishes the boundary before PCC-4 eligibility is
considered.

## 2. Closure basis

| PR | Purpose | What it proved | Evidence |
|---|---|---|---|
| `#596` PCC-3-0 | text surface boundary reset | PCC-3 may work on text mechanics without canonizing legacy surface vocabulary. | `docs/roadmap/language_maturity/pcc3_text_surface_boundary_reset.md`; merged PR `#596` |
| `#598` PCC-3A | text/string core gate fixtures | text literals, text binding, text equality, `to_text(...)`, and `text + text` survive the public text gate path. | `tests/pcc3_text_core_gate.rs`; `tests/fixtures/pcc3_text/`; `cargo test -q --test pcc3_text_core_gate`; merged PR `#598` |
| `#600` PCC-3B | text diagnostics and conversion boundaries | invalid text assignments, text/non-text conditions, text concatenation boundaries, comparison mismatches, and unsupported `to_text(record)` paths fail with stable diagnostics. | `tests/pcc3_text_diagnostics.rs`; `cargo test -q --test pcc3_text_diagnostics`; merged PR `#600` |
| `#602` PCC-3C | text lowering and SemCode stability audit | admitted text fixtures emit byte-stable SemCode across repeated compiles, and invalid text fixtures do not produce valid verified SemCode. | `tests/pcc3_text_lowering_stability.rs`; `cargo test -q --test pcc3_text_lowering_stability`; merged PR `#602` |
| `#604` PCC-3D | text VM/verifier/capability/trap impact check | PCC-3 text work did not change VM, verifier, trap taxonomy, determinism, capability policy, or observation / I/O boundary classifications. | `docs/roadmap/language_maturity/pcc3_text_ctf_impact_check.md`; merged PR `#604` |
| `#606` PCC-3E | 7hell mapping for text gates | PCC-3 text coverage maps onto the seven 7hell stages without implementing 7hell. | `docs/roadmap/language_maturity/pcc3_text_7hell_mapping.md`; merged PR `#606` |

## 3. Closed text surface

The following surfaces are closed for the current PCC-3 scope.

### 3.1 Positive pipeline surfaces

- text literals
- text variable binding
- text equality
- text `+` text concatenation
- `to_text(text)`
- `to_text(i32/u32/bool/quad)`
- repeated SemCode byte stability for admitted text fixtures
- `verify` plus `run-smc` coverage for emitted text artifacts

### 3.2 Negative diagnostics surfaces

- text assignment mismatch diagnostics
- scalar assigned from text diagnostics
- text control-flow condition diagnostics
- text `+` non-text diagnostics
- text comparison mismatch diagnostics
- unsupported `to_text(record)` diagnostics
- invalid text compile path does not verify

### 3.3 Trust and qualification surfaces

- CTF impact check passed
- 7hell mapping recorded
- PCC-3-0 surface boundary recorded

## 4. Explicit non-goals / deferred work

PCC-3 closeout does not include:

- Hello World implementation
- `print` implementation
- `observe` implementation
- controlled observation / effect output path
- general stdout
- general I/O
- file I/O
- stdin
- network
- interpolation / formatting
- full text standard library
- text performance benchmark gate
- text golden trace suite
- cross-backend text equivalence suite
- final canonical vocabulary
- `entry` / `state` / `require` / `observe` / `complete` executable grammar
- `#477` M-Hello implementation
- `#478` Surface Vocabulary Audit
- `#479` Lexicon / Density implementation
- Linguist readiness / `#356..#362`
- Workbench / UI / I70
- package builder

Rule:

```text
PCC-3 Text/String Core is closed for the current PCC scope.
Deferred items remain future PCC work or separate trust-lane work.
```

## 5. Current text truth

Current text truth remains:

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
- canonical vocabulary: guarded by PCC-3-0 and future `#478` / `#479` work

This closeout does not expand that truth.

## 6. CTF impact

PCC-3 CTF impact check: `passed`

CTF remains a separate trust lane.

Any future text, observation, Hello World, capability, runtime value, verifier,
SemCode, trap, or determinism expansion must be rechecked against CTF where
relevant.

## 7. 7hell impact

PCC-3 is mapped to all seven 7hell stages, but 7hell is not implemented.

This closeout records qualification mapping only. It does not add a 7hell
command, runner, report format, or harness.

## 8. Observation / Hello boundary

Hello World remains required as proof of life, but it is not implemented by
PCC-3.

Observation boundary remains closed.

The legacy canonical form remains rejected as canonical:

```text
fn main() {
    print("Hello, World!");
    return;
}
```

Canonical direction remains directional / future only unless grammar later
supports it:

```text
entry / state / require / observe / complete
```

## 9. PCC status

```text
PCC-3 Text/String Core: closed
Next phase start status: eligible after maintainer acceptance
```

PCC-3 closeout does not start the next phase.

Do not interpret this as full text system readiness.

## 10. Acceptance checklist

- PCC-3-0..PCC-3E landed
- intended text gate coverage recorded
- closed text surface explicit
- deferred items explicit
- no code / test / fixture changes
- no text truth expansion
- no Hello World
- no `print` / `observe`
- no 7hell implementation
- no UI / Workbench / I70
- no CTF policy changes
- no Linguist readiness
- next phase not started
