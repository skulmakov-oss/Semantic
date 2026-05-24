# Quad Drain: Explicit Collapse Policy v0.1

Status: research decision note.
Normative status: non-normative unless promoted into `docs/spec/*` and locked by tests.
Public contract impact: none.
Related documents:

- `docs/research/quad_logic_execution_model_v0_1.md`
- `docs/research/quad_logic_engineering_decisions_v0_1.md`
- `docs/research/quad_logic_safety_semantics_v0_1.md`

## 1. Purpose

This note records the recommended design rule for forced `Quad -> Bool` collapse.

The central problem:

```text
A convenient implicit or semi-implicit Quad -> Bool projection can silently erase N and S.
```

That would violate the core quad law:

```text
No silent collapse of meaning.
```

Therefore, any collapse from `Quad` into `Bool` in guarded or effect-relevant control must explicitly define the fate of `N` and `S`.

## 2. Core rule

Recommended rule:

```text
Forced Quad -> Bool collapse inside Guarded or effect-relevant paths requires an explicit collapse policy for N and S.
```

Short form:

```text
No Quad drain without N/S policy.
```

A developer may not accidentally obtain a flat `Bool` from a `Quad` value in a path that can influence:

- admission;
- capability checks;
- audit behavior;
- Prometheus Boundary effects;
- state mutation;
- resource-sensitive runtime decisions.

## 3. Quad Drain concept

A Quad Drain is an explicit operation that turns a `Quad` into a `Bool` or deterministic trap by declaring how the boundary states are handled.

Conceptual form:

```text
q ?? (N => <policy>, S => <policy>)
```

Example:

```text
if allowed ?? (N => false, S => trap) {
  effect_path()
}
```

Meaning:

| q | Result |
|---|---|
| `T` | `true` |
| `F` | `false` |
| `N` | follows declared `N` policy |
| `S` | follows declared `S` policy |

The exact source syntax is not fixed by this research note.

The important requirement is that `N` and `S` must be handled explicitly.

## 4. Allowed collapse policies

Candidate inline policies:

| Policy | Meaning | Notes |
|---|---|---|
| `true` | collapse to `true` | dangerous; likely denied in effect paths unless explicitly permitted by profile |
| `false` | collapse to `false` | safe for denial, but may hide diagnostics unless audited |
| `trap` | deterministic VM trap | safest for unresolved conflict in guarded execution |
| `audit_then_false` | emit audit obligation, then return `false` | useful for visible denial |
| `deny` | admission/effect denial | verifier-visible denial, not just local bool false |
| `defer` | no decision; require caller policy | useful for staged reasoning |

Default recommendation:

```text
N => false or deny
audit-sensitive N => audit_then_false
S => trap or deny
audit-sensitive S => audit_then_false or trap_with_audit
```

This table is a research candidate, not a final spec.

## 5. Why is_true and has_true are not drains

`is_true(q)` and `has_true(q)` are inspection functions.

They are not safe default collapse operators for effect-relevant control.

Important distinction:

```text
is_true(S)  == false
has_true(S) == true
```

If `if is_true(q)` is used as a hidden shortcut in an effect path, then:

- `N` silently becomes `false`;
- `S` silently becomes `false`;
- the program loses the difference between ordinary denial and conflict/unknown;
- diagnostics and audit may lose the anomaly cause.

Therefore:

```text
is_true/has_true may inspect evidence.
They must not silently drain Quad into Bool in guarded/effect paths.
```

## 6. Profile-dependent rule

Candidate profile behavior:

| Context | `if is_true(q)` | `if q ?? (...)` |
|---|---|---|
| AdmitPure / pure reasoning | allowed, possibly with lint | allowed |
| Strict profile | allowed only if semantically justified and verifier-visible | preferred explicit form |
| Guarded profile | warn or deny if path reaches effect/state mutation | required for effect-relevant path |
| Effect authorization | deny unless projection policy is explicit and approved | required |

Reason:

Pure computation may intentionally inspect evidence.

Effectful or guarded execution needs explicit treatment of unknown and conflict.

## 7. Sema behavior

When source code attempts:

```text
if allowed {
  ...
}
```

where `allowed: Quad`, Sema should reject it.

Recommended diagnostic:

```text
Quad value cannot be implicitly used as branch authority.
Expected Bool, found Quad.

Help:
  Handle N and S explicitly with quad match or explicit collapse policy.

Suggestion:
  if allowed ?? (N => false, S => trap) { ... }
```

Sema may additionally offer pure-compute guidance:

```text
if is_true(allowed) { ... }
```

but only with warning text:

```text
This treats N and S as false and is not suitable for effect authorization.
```

## 8. Verifier behavior

Verifier should deny SemCode when:

- a quad-derived value reaches branch authority without explicit projection or drain policy;
- `is_true` or `has_true` is used as a hidden authorization shortcut;
- `N` can authorize an effect through projection;
- `S` can authorize an effect through `has_true`;
- collapse policy is not deterministic;
- collapse policy omits `N` or `S` in guarded/effect-relevant paths;
- lowering erased whether a bool came from quad projection.

Verifier principle:

```text
A Bool derived from Quad must carry enough origin/policy metadata until admission is complete.
```

## 9. IR and SemCode consequences

IR should distinguish:

```text
BoolConst
BoolOp
QuadProject(kind)
QuadDrain(policy_N, policy_S)
QuadMatch
QuadResolve(policy)
```

SemCode should preserve the difference between:

- ordinary bool branch;
- bool produced by strict projection;
- bool produced by explicit drain;
- bool produced by inspected evidence function;
- authorization predicate produced by safety operator.

Bad lowering:

```text
QuadDrain -> BoolOp
```

Good lowering:

```text
QuadDrain(policy_N, policy_S) -> verifier-visible operation
```

## 10. Authorization rule

Recommended rule:

```text
Effect authorization may not be based on hidden Quad -> Bool collapse.
```

Allowed by default:

```text
AuthAll(... ) == T
```

or an explicit drain policy approved for the profile.

Denied by default:

```text
if is_true(policy_quad) { effect() }
if has_true(policy_quad) { effect() }
if policy_quad { effect() }
```

unless the surrounding profile and verifier-visible policy explicitly allow it.

## 11. Diagnostics examples

### 11.1 Implicit branch

```text
error[E-QUAD-BRANCH]: Quad value cannot be implicitly used as branch authority
  --> src/main.sm:4:4
   |
 4 | if allowed {
   |    ^^^^^^^ expected Bool, found Quad
   |
   = help: handle N and S explicitly
   = suggestion: if allowed ?? (N => false, S => trap) { ... }
```

### 11.2 Ambiguous projection in effect path

```text
error[E-QUAD-AUTH-PROJECTION]: ambiguous Quad projection in effect-relevant path
  --> src/main.sm:9:7
   |
 9 | if is_true(allowed) { write_host_state(); }
   |       ^^^^^^^^^^^^^ N and S collapse to false without audit policy
   |
   = help: use explicit collapse policy or authorization operator
   = suggestion: if allowed ?? (N => audit_then_false, S => trap) { ... }
```

### 11.3 has_true cannot authorize

```text
error[E-QUAD-CONFLICT-AUTH]: conflict evidence cannot authorize effect by default
  --> src/main.sm:12:7
   |
12 | if has_true(allowed) { send_effect(); }
   |       ^^^^^^^^^^^^^ S contains true evidence but is still conflict
   |
   = help: resolve S explicitly before authorization
```

## 12. Evidence needed before spec promotion

| Claim | Evidence needed |
|---|---|
| `if q` is rejected by Sema | sema diagnostic tests |
| SemCode bypass is rejected by Verifier | verifier negative fixture |
| QuadDrain requires N/S policy | parser/sema/verifier fixtures |
| `is_true` is not accepted as hidden authorization | effect-path denial fixture |
| `has_true(S)` cannot authorize effect | projection authorization denial fixture |
| drain policy survives lowering | IR/SemCode visibility test |
| `trap` policy is deterministic | replay-stable trap test |
| `audit_then_false` produces audit obligation | audit obligation fixture |

## 13. Open decisions

| ID | Question | Recommended direction | Risk |
|---|---|---|---|
| OD-QDRAIN-001 | What is the final source syntax? | decide later; keep semantics first | syntax bikeshedding |
| OD-QDRAIN-002 | Are `true` policies allowed for N/S in effect paths? | deny by default | permission by absence/conflict |
| OD-QDRAIN-003 | Is `audit_then_false` runtime effect or admission obligation? | admission-visible obligation | hidden effect risk |
| OD-QDRAIN-004 | Does QuadDrain exist in Strict profile? | only with strong restrictions | weakening strict guarantees |
| OD-QDRAIN-005 | Does derived Bool carry origin metadata? | yes until verifier admission | lost provenance |

## 14. Non-goals

This document does not:

- finalize source syntax;
- require `??` specifically;
- make `is_true` illegal everywhere;
- make all Quad projections effect-unsafe;
- promote QuadDrain into public contract;
- define final diagnostic codes;
- define final SemCode opcodes;
- allow hidden conflict/unknown collapse.

## 15. Summary

QuadDrain is the explicit boundary where a four-state reasoning value may become a two-state control value.

That boundary must not be magical.

Core law:

```text
If Quad becomes Bool, N and S must have an explicit fate.
```

Short form:

```text
No drain without policy.
```
