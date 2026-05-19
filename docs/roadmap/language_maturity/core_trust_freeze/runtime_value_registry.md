# CTF-1 — RuntimeValue Registry

Status: draft registry
Parent lane: `core_trust_freeze/index.md`

## Purpose

This file tracks the runtime value families that the verifier, VM, runtime-core, traces, and canonical examples must treat as part of the current execution surface.

It is not a full VM implementation document. It is a trust registry.

## Current registry shape

Use the following table during PCC live audit.

| Runtime value family | Status | PCC owner | Notes |
|---|---|---|---|
| `unit` | audit-needed | PCC-0.5 | Confirm current carrier and trace behavior. |
| `bool` | audit-needed | PCC-0.5 | Confirm stable VM representation. |
| `quad` | audit-needed | PCC-0.5 | Confirm `N/F/T/S` behavior and branch restrictions. |
| `i32` | audit-needed | PCC-2 | Confirm arithmetic and trap behavior. |
| `u32` | audit-needed | PCC-2 | Confirm arithmetic and conversion policy. |
| `f64` | audit-needed | PCC-2 | Confirm deterministic profile and capability assumptions. |
| `fx` | audit-needed | PCC-2 | Confirm fixed-point carrier and arithmetic scope. |
| `text` | freeze-candidate | PCC-3 / PCC-8 | Literal / equality / concat / length and admitted helper/text surface only. |
| `record` | freeze-candidate | PCC-4 | Current record seams / fixture-backed surface only. |
| `ADT` | freeze-candidate | PCC-5 | Current constructor + basic match surface only. |
| `Option(T)` | freeze-candidate | PCC-6 | Current standard-form Option only. |
| `Result(T,E)` | freeze-candidate | PCC-6 | Current standard-form Result only. |
| `Sequence<T>` | freeze-candidate | PCC-7 | Current admitted Sequence fixture-backed surface only. |
| `Map<K,V>` | freeze-candidate | PCC-7 | Current admitted Map baseline only; missing-key / iteration / quota policy remains open. |
| project manifest metadata | audit-needed | PCC-9 | Current Semantic.package manifest baseline only; not a runtime value unless later represented in VM/runtime. |
| closure values | out-of-pcc unless audited | post-PCC / current-main audit | Do not widen in PCC unless explicitly pulled in. |
| host handles | out-of-pcc | separate runtime boundary scope | Do not mix into practical core. |

## Status meanings

| Status | Meaning |
|---|---|
| `audit-needed` | Appears to exist, but PCC cannot rely on it until live audit confirms behavior. |
| `planned` | Belongs to a PCC phase. |
| `freeze-candidate` | Behavior is stable enough to protect from silent change. |
| `frozen` | Public or release-facing contract. |
| `out-of-pcc` | Not part of this plan. |

## PR update rule

A PR must update this registry if it:

- adds a runtime value;
- removes a runtime value;
- changes value encoding;
- changes equality or display behavior;
- changes trace representation;
- changes verifier admission assumptions;
- changes VM trap behavior for the value.

## Explicit debug boundary

`debug_render` must not be registered as a language value conversion.

```text
debug_render != to_text
debug_render is internal tooling only
```

Any public conversion belongs to PCC-8 Stdlib v0.

## CTF-WP2 PCC-4..PCC-9 Runtime Value Sync

PCC-4..PCC-9 closeouts did not add new runtime behavior in this PR.

WP2 records trust status based on existing PCC fixture evidence.

`freeze-candidate` means protected from silent change, not full public release freeze.

Broad ergonomics and future widening remain out of scope.

Boundary notes:

- record: current fields / literals / access fixture surface only;
- ADT: current constructors / basic match only;
- Option / Result: standard forms only, no exception semantics;
- Sequence: current positive / negative fixture-backed operations only;
- Map: current baseline only; missing-key behavior, iteration policy, and memory/quota remain open;
- text / to_text: no universal reflection; debug_render remains internal-only;
- Project Model: Semantic.package baseline is project-adjacent evidence, not a runtime value family unless runtime representation is later introduced.
