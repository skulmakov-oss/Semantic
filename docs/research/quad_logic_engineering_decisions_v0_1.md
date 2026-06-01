# Quad Logic Engineering Decisions v0.1

Status: research decision note.
Normative status: non-normative unless promoted into `docs/spec/*` and locked by tests.
Public contract impact: none.
Related document: `docs/research/quad_logic_execution_model_v0_1.md`.

## 1. Purpose

This note records engineering decisions and recommended resolutions that follow from the Quad Logic Execution Model v0.1.

The goal is to turn the quad-logic research thesis into concrete design pressure without prematurely freezing the public specification.

Core thesis inherited from the execution model:

```text
No silent collapse of meaning.
```

## 2. Evidence-plane interpretation

Quad values are modeled as two evidence bits:

```text
q = (e_f, e_t)
```

| State | `(e_f, e_t)` | Meaning |
|---|---:|---|
| `N` | `(0, 0)` | no false evidence, no true evidence |
| `F` | `(1, 0)` | false evidence only |
| `T` | `(0, 1)` | true evidence only |
| `S` | `(1, 1)` | both false and true evidence / conflict |

This model is powerful because it turns reasoning states into compact evidence planes.

Important caution:

```text
The evidence-plane model is a research execution model.
It does not freeze SemCode binary layout or ABI encoding.
```

## 3. Quad negation decision

Recommended decision:

```text
quad_not(e_f, e_t) = (e_t, e_f)
```

Truth table:

| q | evidence | not q | evidence |
|---|---:|---|---:|
| `N` | `(0,0)` | `N` | `(0,0)` |
| `F` | `(1,0)` | `T` | `(0,1)` |
| `T` | `(0,1)` | `F` | `(1,0)` |
| `S` | `(1,1)` | `S` | `(1,1)` |

Rationale:

- absence of evidence remains absence of evidence;
- false evidence becomes true evidence under negation;
- true evidence becomes false evidence under negation;
- conflict remains conflict because both evidence planes are present.

Engineering implication:

```text
quad_not should be representable as a cheap deterministic bit-plane operation.
```

This is a performance direction, not a public ABI guarantee.

## 4. Projection discipline

Recommended decision:

```text
is_true(q)  means q == T
has_true(q) means q == T || q == S
```

Critical distinction:

```text
is_true(S)  == false
has_true(S) == true
```

Security rationale:

A conflict state must not authorize an effect simply because true evidence is present.

`has_true` is evidence inspection.

`is_true` is strict truth projection.

These are different operations and must remain different through sema, IR, SemCode, verifier, and diagnostics.

## 5. Authorization rule

Recommended default for capability/effect authorization:

```text
Only strict T may authorize an effect-bearing path by default.
```

Default conservative table:

| Quad state in authorization predicate | Default behavior |
|---|---|
| `T` | may proceed only if capability, quota, and audit checks also pass |
| `F` | deny |
| `N` | deny or require explicit unknown policy |
| `S` | deny or require explicit conflict policy |

Rationale:

- `N` means the system does not know enough;
- `S` means the system has contradictory evidence;
- neither state should silently become permission.

Policy escape hatch:

```text
N and S may be handled only through explicit verifier-visible policy.
```

## 6. OD-QUAD-001 recommended resolution: where to reject `if q`

Question:

```text
Should implicit quad branching be rejected at parse, sema, or verifier?
```

Recommended resolution:

```text
Primary rejection: Sema.
Final rejection: Verifier.
```

Layer responsibility:

| Layer | Responsibility |
|---|---|
| Parser | builds AST; should not need to know final expression type |
| Sema | detects `Quad` used where `Bool` branch authority is required |
| Verifier | rejects SemCode that attempts to branch on quad-derived authority without explicit projection/match |

Rationale:

- Sema gives fast developer feedback and IDE-quality diagnostics.
- Verifier protects the VM from bypass compilers or hand-authored SemCode.

Decision principle:

```text
Developer ergonomics at Sema.
Trust boundary enforcement at Verifier.
```

## 7. OD-QUAD-004 recommended resolution: conflict propagation

Question:

```text
Should S propagate by default?
```

Recommended resolution:

```text
Yes. S should propagate by default in reasoning operations unless an explicit resolution policy is present.
```

Rationale:

Conflict is semantic evidence, not a disposable error flag.

If a reasoning flow contains contradiction, the contradiction should remain visible until explicitly matched, resolved, denied, or audited.

Engineering analogy:

```text
S behaves like a semantic poison value for unresolved contradiction.
```

Caution:

This is an analogy for propagation behavior, not a claim that `S` is identical to floating-point `NaN`.

Recommended rule:

```text
Conflict must float upward until explicit quad_match or quad_resolve(policy).
```

## 8. Unknown propagation rule

Recommended decision:

```text
N should preserve uncertainty by default.
```

`N` must not silently become `F`.

Default behavior depends on operation class:

| Context | Default treatment of `N` |
|---|---|
| pure reasoning | propagate or remain visible |
| boolean branch | reject unless explicit projection |
| authorization | deny unless explicit unknown policy |
| diagnostics | report unknown reached decisive context |

Rationale:

Unknown is not false.

Lack of evidence must not become denial, permission, or branch authority without explicit policy.

## 9. Operation-class policy

Recommended propagation defaults:

| Operation class | `N` behavior | `S` behavior |
|---|---|---|
| `quad_not` | preserves `N` | preserves `S` |
| evidence merge | combines evidence | may produce or preserve `S` |
| evidence intersection | keeps shared evidence | may reduce or preserve depending on rule |
| strict projection | explicit only | `S` not strict true/false |
| quad match | explicit handling required | explicit handling required |
| effect authorization | deny by default | deny by default |
| conflict resolution | requires policy | resolves only under explicit policy |

This table should later become a spec candidate only after tests exist.

## 10. Verifier consequences

The verifier should reject or deny:

- implicit quad-to-bool branch authority;
- implicit `S` authorization;
- implicit `N` authorization;
- effect paths depending on ambiguous quad collapse;
- conflict resolution without explicit deterministic policy;
- lowering artifacts where quad origin is erased before admission.

Verifier acceptance should require:

- explicit projection;
- explicit quad match;
- explicit conflict policy;
- explicit unknown policy where decisive behavior is required;
- deterministic resolution semantics.

## 11. IR and SemCode consequences

Recommended rule:

```text
Quad origin must remain visible until projection, match, or resolution is explicit.
```

IR should not erase quad state by lowering it prematurely to bool.

SemCode should preserve enough structure for verifier admission:

- quad constants;
- quad operation class;
- projection kind;
- match coverage;
- resolution policy reference;
- effect path dependency if applicable.

Non-goal:

```text
This note does not define final instruction encoding.
```

## 12. Diagnostics consequences

Diagnostics should distinguish:

| Diagnostic | Meaning |
|---|---|
| implicit quad branch | `Quad` used where `Bool` is required |
| conflict as authorization | `S` reached permission/effect path |
| unknown as authorization | `N` reached permission/effect path |
| ambiguous projection | projection kind is not explicit |
| lost quad origin | lowering erased evidence state before verifier admission |

Suggested diagnostic tone:

```text
Quad value cannot be used as branch authority.
Use explicit quad match or strict projection.
```

For authorization:

```text
Conflict or unknown cannot authorize an effect by default.
Provide an explicit policy or handle N/S before the effect path.
```

## 13. Evidence needed before spec promotion

| Decision | Evidence required |
|---|---|
| Sema rejects `if q` | sema diagnostic tests |
| Verifier rejects quad branch bypass | SemCode-level negative fixture |
| `quad_not` table | golden evaluation tests |
| `is_true` vs `has_true` | projection behavior tests |
| `S` propagation | conflict propagation fixtures |
| `N` preservation | unknown-vs-false fixtures |
| strict T authorization | capability/effect denial fixtures |
| explicit policy escape hatch | policy-visible admission fixtures |

## 14. Non-goals

This note does not:

- finalize source syntax;
- finalize SemCode encoding;
- claim zero-overhead on all targets;
- require every quad operation to be bit-plane encoded;
- turn `S` into a runtime panic by default;
- turn `N` into `F`;
- allow effect authorization through `has_true(S)`;
- promote these recommendations into public contract.

## 15. Summary

Quad logic gives Semantic a compact evidence-plane foundation for reasoning under incomplete or contradictory information.

But the safety value does not come merely from having four states.

The safety value comes from preserving those states until an explicit, verifier-visible decision handles them.

Core engineering laws:

```text
No silent collapse of meaning.
No implicit quad branch authority.
No conflict-based authorization by default.
No unknown-based authorization by default.
Sema explains.
Verifier enforces.
VM executes admitted meaning.
```
