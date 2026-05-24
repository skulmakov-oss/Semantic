# Quad Logic Safety Semantics v0.1

Status: research orientation document.
Normative status: non-normative unless promoted into `docs/spec/*` and locked by tests.
Public contract impact: none.
Related documents:

- `docs/research/quad_logic_execution_model_v0_1.md`
- `docs/research/quad_logic_engineering_decisions_v0_1.md`
- `docs/research/core_trust_contract_v0_1.md`
- `docs/research/admission_evidence_matrix.md`

## 1. Purpose

This document records an important research boundary:

Semantic Quad Logic is inspired by four-valued evidence logic, but it is not a direct adoption of classical truth-table behavior in safety-critical execution paths.

Semantic needs quad logic for verified execution, admission, capability decisions, audit obligations, and deterministic runtime behavior.

Therefore, some operations must be defined by runtime safety semantics rather than by purely abstract truth-table elegance.

Core statement:

```text
Belnap-style evidence states are useful.
Belnap-style truth tables are not automatically safe for execution admission.
```

Semantic uses quad states as an engineering interface to uncertainty and contradiction.

## 2. The two-bit evidence plane

Semantic uses the following conceptual model:

```text
Quad = (e_f, e_t)
```

| State | Evidence plane | Meaning |
|---|---:|---|
| `N` | `(0, 0)` | no evidence either way |
| `F` | `(1, 0)` | false evidence only |
| `T` | `(0, 1)` | true evidence only |
| `S` | `(1, 1)` | both false and true evidence; conflict |

This model is valuable because it makes uncertainty and contradiction compact, deterministic, and mechanically visible.

However, the evidence plane alone does not decide how a value may affect real execution.

The same quad state may be harmless in pure reasoning and dangerous in an effect authorization path.

## 3. Classical bilattice vs execution semantics

Classical four-valued logic is usually described through abstract lattice operations.

That is useful for knowledge representation.

Semantic has a different goal:

```text
not merely classify information,
but decide whether execution may proceed.
```

In Semantic, a quad value may affect:

- branch authority;
- admission class;
- capability gating;
- resource accounting;
- audit obligations;
- effect authorization;
- VM fail-closed behavior.

This changes the meaning of operators.

A truth-table result that is acceptable for database query logic may be unsafe for runtime authorization.

## 4. N and S are not symmetric in safety space

In an abstract information lattice, `N` and `S` may appear as opposite ends of an information axis.

In Semantic safety semantics, they have very different risk profiles.

| State | Safety interpretation |
|---|---|
| `N` | passive absence of evidence; unsafe only when a decision is required |
| `S` | active contradiction; evidence conflict that must remain visible |

`N` means:

```text
The system does not know enough.
```

`S` means:

```text
The system has contradictory evidence.
```

These are not operationally equivalent.

`N` is a missing-input problem.

`S` is a conflict-integrity problem.

## 5. Conflict must not disappear

Research decision:

```text
S must not be silently neutralized by nearby F, T, or N in safety-critical paths.
```

Rationale:

If a conflict occurred in the reasoning path, the verifier, diagnostics, or explicit policy must be able to see it.

A conflict that is hidden by a later operation becomes a silent semantic bug.

Core law:

```text
Conflict is not merely a truth value.
Conflict is a visibility obligation.
```

## 6. Unknown must not become permission

Research decision:

```text
N must not merge with T into authorization.
```

Example:

```text
local_rule       = T
capability_check = N
```

A pure information merge might be tempted to preserve the known true evidence.

But in an authorization path, this must not become clean `T`.

Safe default:

```text
AuthMerge(T, N) != T
```

Recommended default:

```text
AuthMerge(T, N) = N
```

or an equivalent denial/defer state that remains verifier-visible.

Rationale:

Absence of capability evidence must not become consent.

## 7. Operator classes must be separated

Semantic should not define a single universal `AND`, `OR`, or `MERGE` table for all contexts.

Instead, operator behavior should be classified by intent.

| Operator class | Purpose | Safety priority |
|---|---|---|
| pure truth operation | reason over truth-like evidence | logical consistency |
| evidence merge | combine independent evidence sources | preserve evidence visibility |
| authorization combine | decide whether an effect path may proceed | fail closed |
| diagnostic combine | explain what evidence occurred | preserve causes |
| verifier combine | decide admissibility | preserve denial/conflict reasons |

A pure reasoning expression and a capability authorization predicate should not be forced to use the same collapse rules.

## 8. Safety-priority ordering

For authorization and admission paths, a conservative priority order is useful:

```text
S > N > F > T
```

This is not a truth order.

It is a safety visibility order.

Meaning:

| Priority | Meaning in effect/admission path |
|---:|---|
| `S` | conflict must surface first |
| `N` | insufficient evidence must prevent permission |
| `F` | explicit denial |
| `T` | permission candidate, still subject to capability/audit/quota checks |

This order prevents both dangerous collapses:

```text
S disappearing into F
N merging with T into permission
```

## 9. Authorization combine candidate

For effect-bearing paths, a candidate conservative combine operation is:

```text
AuthAll(a, b)
```

Meaning:

```text
all required predicates must be strictly true,
while uncertainty and conflict remain visible.
```

Candidate table:

| `AuthAll` | N | F | T | S |
|---|---|---|---|---|
| N | N | N | N | S |
| F | N | F | F | S |
| T | N | F | T | S |
| S | S | S | S | S |

Interpretation:

- any `S` makes the result `S`;
- if no `S`, any `N` makes the result `N`;
- if no `S` or `N`, any `F` makes the result `F`;
- only all `T` yields `T`.

This table is intentionally not a classical Belnap truth table.

It is a fail-closed authorization table.

## 10. Effect authorization rule

Recommended admission rule:

```text
An effect path may proceed only when its authorization predicate is strict T.
```

Therefore:

| Final auth quad | Default admission behavior |
|---|---|
| `T` | may proceed only after capability, quota, and audit checks pass |
| `F` | deny |
| `N` | deny or defer; must not permit |
| `S` | deny and surface conflict unless explicit policy exists |

This rule ensures:

```text
has_true(S) does not authorize effects.
is_true(T) is required for default authorization.
```

## 11. Conflict poisoning

Research decision:

```text
S should propagate by default in safety-relevant reasoning.
```

This can be called conflict poisoning.

It means:

```text
If a conflict enters a safety-relevant reasoning path,
it should remain visible until explicit match or resolution.
```

This is similar in spirit to poison-value propagation, but it is semantic rather than numeric.

Non-goal:

```text
This document does not claim that S is identical to NaN.
```

The analogy is only about propagation visibility.

## 12. Unknown isolation

Research decision:

```text
N should isolate permission.
```

`N` is not active contradiction, but it still cannot grant authority.

In an effect path:

```text
T + N must not become T.
```

Possible safe outcomes:

- `N` remains visible;
- the path is denied as insufficient evidence;
- the artifact is admitted only under an explicit guarded profile;
- the program must handle `N` through explicit policy.

Default rule:

```text
Unknown does not authorize.
```

## 13. Why F should not always swallow S

In ordinary boolean reasoning, false often short-circuits conjunction.

In Semantic safety paths, this is dangerous if it hides a conflict.

Example:

```text
A = F
B = S
A AND B = F   // unsafe if it hides B = S
```

A pure denial may be sufficient to stop execution, but not sufficient to preserve evidence.

Semantic must distinguish:

```text
Denied because false
Denied with conflict present
```

If the quad result collapses to `F`, diagnostics and verifier evidence may lose the fact that a conflict occurred.

Recommended safety result:

```text
AuthAll(F, S) = S
```

or an equivalent structured denial that preserves conflict as cause.

## 14. Separation between result and decision

A key design distinction:

```text
Quad result != admission decision.
```

For example:

| Quad result | Admission decision |
|---|---|
| `T` | candidate permit, still checked by capability/audit/quota |
| `F` | deny |
| `N` | deny/defer/guarded depending on policy |
| `S` | deny/conflict unless explicit deterministic policy exists |

This prevents misuse of quad values as direct runtime permissions.

The verifier should map quad-derived predicates into admission classes explicitly.

## 15. Context-sensitive tables

Semantic may need different tables for different contexts.

Candidate families:

| Family | Example | Role |
|---|---|---|
| `TruthAnd` / `TruthOr` | pure reasoning | abstract logical reasoning |
| `EvidenceMerge` | source fusion | preserve evidence planes |
| `AuthAll` / `AuthAny` | authorization | fail-closed permission logic |
| `DiagJoin` | diagnostics | preserve causal evidence |
| `VerifierJoin` | admission | preserve rejection/conflict reasons |

Important rule:

```text
The operation name must reveal its policy class.
```

A generic `AND` over quad values is too ambiguous for safety-critical semantics.

## 16. Verifier consequences

The verifier should reject or deny artifacts where:

- generic quad operators are used in effect authorization without a policy class;
- `N` can become authorization through merge with `T`;
- `S` can disappear before admission/effect decision;
- `has_true(S)` is used as default authorization;
- conflict resolution is not deterministic;
- unknown handling is not explicit in decisive paths.

Verifier principle:

```text
The verifier does not merely check quad syntax.
It checks whether quad uncertainty and conflict can influence execution safely.
```

## 17. IR and SemCode consequences

IR and SemCode should preserve operation class.

Bad lowering:

```text
QuadAuthAll -> generic boolean AND
```

Good lowering:

```text
QuadAuthAll -> verifier-visible safety operator
```

The verifier must be able to see whether a quad operation is:

- pure reasoning;
- evidence merge;
- authorization combine;
- diagnostic join;
- explicit resolution.

Without this distinction, a safe source-level policy can become unsafe after lowering.

## 18. Diagnostics consequences

Diagnostics should not say only:

```text
condition is not true
```

They should preserve cause:

```text
permission denied: unknown evidence reached authorization path
permission denied: conflict evidence reached authorization path
permission denied: strict true required, got S
permission denied: generic quad merge cannot authorize effect
```

This makes quad logic explainable instead of mystical.

## 19. Relation to Core Trust Profile

The Core Trust Profile should use the conservative authorization semantics by default.

Recommended default:

```text
Strict profile:
  T = candidate permit
  F = deny
  N = deny unless explicit unknown policy exists
  S = deny unless explicit conflict policy exists
```

Guarded profiles may allow more flexible handling, but only when:

- the policy is explicit;
- resolution is deterministic;
- audit behavior is defined;
- replay remains stable;
- admission class records the weakened guarantee.

## 20. Evidence needed before spec promotion

| Claim | Evidence needed |
|---|---|
| `AuthAll(T, N) != T` | authorization table tests |
| `AuthAll(F, S)` preserves conflict | conflict visibility tests |
| `S` propagates in safety paths | safety propagation fixtures |
| `N` does not authorize | capability/effect denial fixtures |
| generic quad `AND` is not allowed in effect paths | verifier negative fixtures |
| `has_true(S)` does not authorize by default | projection authorization tests |
| explicit policy can handle `N`/`S` | policy admission fixtures |
| operation class survives lowering | IR/SemCode visibility tests |

## 21. Open decisions

| ID | Question | Recommended direction | Risk |
|---|---|---|---|
| OD-QSAFE-001 | Should safety paths use `AuthAll` instead of generic `AND`? | yes | generic `AND` hides policy |
| OD-QSAFE-002 | Should `S` dominate `F` in authorization combine? | yes, preserve conflict | conflict invisibility |
| OD-QSAFE-003 | Should `N` dominate `F` in authorization combine? | likely yes for evidence visibility | denial reason may become less direct |
| OD-QSAFE-004 | Should pure logic and authorization logic have separate opcodes? | likely yes | opcode growth |
| OD-QSAFE-005 | Should explicit policy produce structured admission notes? | yes | added verifier metadata |

## 22. Non-goals

This document does not:

- reject the usefulness of classical four-valued logic;
- define a final formal algebra for all Semantic quad operations;
- freeze source syntax;
- freeze SemCode binary layout;
- claim every pure reasoning operation must use safety-priority tables;
- claim all `S` states are runtime panics;
- claim all `N` states are errors;
- allow `N` or `S` to authorize effects by default;
- promote these decisions into public contract.

## 23. Summary

Semantic Quad Logic is not merely a four-valued truth system.

It is a safety-aware execution model for reasoning under incomplete or contradictory evidence.

Classical truth tables optimize for abstract logical classification.

Semantic safety tables optimize for deterministic admission, fail-closed effects, and conflict visibility.

Core law:

```text
Do not let elegant truth tables erase dangerous evidence.
```

Short form:

```text
Quad safety semantics are risk-oriented, not truth-table ornamental.
```
