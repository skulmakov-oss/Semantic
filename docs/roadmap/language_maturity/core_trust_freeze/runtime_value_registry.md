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
| `text` | planned | PCC-3 | Literal / equality / concat / length only at PCC-3. |
| `record` | planned | PCC-4 | Requires PCC-3.5 carrier note. |
| `ADT` | planned | PCC-5 | Basic constructor + match path. |
| `Option(T)` | planned | PCC-6 | Built on ADT / match assumptions. |
| `Result(T,E)` | planned | PCC-6 | Built on ADT / match assumptions. |
| `Sequence<T>` | planned | PCC-7 | Dynamic container; not record semantics. |
| `Map<K,V>` | planned | PCC-7 | Dynamic container; key policy required. |
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
