# PCC-3 Text CTF Impact Check

Status: guard record
Track: PCC-3-D
Layer: language maturity / trust lane
Scope: documentation only
Implementation: out of scope
Owner: language maturity stream

Related:

- `docs/roadmap/language_maturity/practical_core_completion_v0_3.md`
- `#596`
- `#598`
- `#600`
- `#602`

## 1. Purpose

This document records the PCC-3 text/string VM/verifier/capability/trap impact check.

It records whether the merged PCC-3 text work affected VM runtime value surfaces, verifier assumptions, trap taxonomy, determinism matrix, SemCode opcode/capability surfaces, CTF trust classifications, or the observation / I/O boundary.

## 2. Input PRs

| PR | What changed | Code changed | VM/verifier/CTF docs changed | Observation / I/O touched | CTF touched statement |
|---|---|---|---|---|---|
| `#596` PCC-3-0 | Recorded the text surface boundary reset before PCC-3. | No. Docs-only. | No. | No. | `CTF touched: none` |
| `#598` PCC-3A | Added PCC-3 text/string gate tests and fixtures for current text truth. | Yes. Test harness and fixtures only. | No. | No. | `CTF touched: none` |
| `#600` PCC-3B | Hardened text diagnostics and conversion-boundary tests. | Yes. Test harness and fixtures only. | No. | No. | `CTF touched: none` |
| `#602` PCC-3C | Added lowering and SemCode stability tests for current text fixtures. | Yes. Test harness only. | No. | No. | `CTF touched: none` |

## 3. Current text truth reviewed

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
- canonical vocabulary: not frozen by PCC-3A/B/C

This review does not claim:

- general stdout
- general observation
- `print`
- `observe`
- interpolation / formatting
- file I/O
- stdin
- network
- general host ABI
- canonical `entry` / `observe` syntax as executable
- Linguist readiness

## 4. Impact matrix

| Surface | PCC-3A/B/C impact | Result |
|---|---|---|
| Runtime values | Existing text runtime values exercised, no new runtime value kind added | no classification change |
| SemCode opcodes | Existing text/string emission exercised, no new opcode introduced | no classification change |
| Verifier assumptions | Emitted text artifacts verify through existing verifier path | no verifier widening |
| VM execution | Admitted text artifacts run through existing VM path | no VM widening |
| Trap taxonomy | No trap taxonomy row changed | no classification change |
| Determinism matrix | Repeated `.smc` byte stability covered by tests, matrix unchanged | no classification change |
| Capability/effect policy | No observation / print / general I/O capability added | no capability change |
| Observation boundary | Hello World / `print` / `observe` not implemented | boundary remains closed |
| CLI surface | Public `check` / `run` / `compile` / `verify` / `run-smc` routes used, no CLI expansion | no CLI widening |
| UI / Workbench / I70 | Untouched | frozen |
| Linguist readiness | Untouched | deferred |

## 5. Guard result

PCC-3 text CTF impact check result: `passed`

Meaning:

- PCC-3A/B/C did not change VM, verifier, trap, determinism, capability, or trust classifications.
- PCC-3A/B/C added tests and fixtures only.
- Text runtime behavior was exercised through the existing public pipeline.
- Observation / I/O remained closed.
- Any future text, observation, Hello World, or surface-vocabulary expansion must be rechecked against CTF where relevant.

## 6. CTF boundary

CTF remains authoritative for:

- trap taxonomy
- determinism matrix
- verifier-first policy
- golden trace policy
- capability/effect denial policy
- observation/effect admission policy

PCC-3 does not own those classifications.

## 7. Open follow-up / not closed by this PR

This PR does not close all PCC-3.
This PR does not implement Hello World.
This PR does not implement `print` / `observe`.
This PR does not start `#477`.
This PR does not close `#478` or `#479`.
This PR does not start Linguist readiness.
This PR does not define final canonical vocabulary.
This PR does not add general I/O.
This PR does not modify trap or determinism docs.

## 8. Acceptance checklist

- PCC-3 text impact check recorded
- no CTF classifications changed
- trap taxonomy untouched
- determinism matrix untouched
- VM / verifier / runtime untouched
- no capability widening
- no observation / I/O widening
- no CLI widening
- no UI / Workbench / I70
- no Linguist readiness
- Hello World not implemented
- `print` / `observe` not implemented

## 9. Boundary summary

```text
PCC-3A/B/C exercised existing text surfaces.
CTF classifications remain unchanged.
Observation / I/O remains closed.
```
