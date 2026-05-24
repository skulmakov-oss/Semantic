# Quad Logic Execution Model v0.1

Status: research orientation document.
Normative status: non-normative unless promoted into `docs/spec/*` and locked by tests.
Public contract impact: none.
Related documents:

- `docs/research/core_trust_contract_v0_1.md`
- `docs/research/admission_evidence_matrix.md`

## 1. Core thesis

Quad logic in Semantic is not a convenience enum and not a surface-language trick.

It is an execution-domain state model for reasoning-oriented programs.

Semantic uses four explicit truth/evidence states:

| Symbol | Working name | Meaning |
|---|---|---|
| `N` | null / unknown | no sufficient evidence for true or false |
| `F` | false | evidence supports false |
| `T` | true | evidence supports true |
| `S` | superposed / conflict | evidence supports both true and false, or the state is contradictory |

Core statement:

```text
Quad value != implicit boolean branch authority.
```

A quad value carries semantic evidence state. It must not be silently collapsed into `Bool`.

## 2. Why quad logic exists

Classical boolean logic is too lossy for reasoning programs.

A boolean can represent:

```text
true
false
```

Reasoning systems often need to represent:

```text
unknown
false
true
conflict
```

Semantic makes these states first-class so that uncertainty and contradiction remain visible to the compiler, verifier, IR, SemCode, and VM.

The goal is not to make logic decorative. The goal is to prevent hidden collapse of meaning.

## 3. Conceptual evidence-plane model

A useful research model is to treat a quad value as two evidence bits:

| State | False evidence | True evidence | Meaning |
|---|---:|---:|---|
| `N` | 0 | 0 | no evidence |
| `F` | 1 | 0 | false evidence only |
| `T` | 0 | 1 | true evidence only |
| `S` | 1 | 1 | both false and true evidence |

This model is conceptual in this document. It does not finalize SemCode binary layout or public ABI.

However, it gives a precise intuition:

```text
N = no evidence
F = false evidence
T = true evidence
S = conflicting / combined evidence
```

This is why `N` must not be treated as `F`, and `S` must not be treated as `T` or `F` without an explicit policy.

## 4. Bool and Quad are distinct

`Bool` and `Quad` are different semantic types.

| Type | Domain | Intended use |
|---|---|---|
| `Bool` | `{ false, true }` | ordinary binary control after certainty is established |
| `Quad` | `{ N, F, T, S }` | reasoning state, evidence state, conflict-aware predicate |

Candidate invariant:

```text
Bool is branch authority.
Quad is reasoning evidence.
```

Therefore:

```text
if bool_value { ... }        // allowed in boolean control
if quad_value { ... }        // rejected or denied in strict profile
match quad_value { ... }     // allowed if quad handling is explicit
```

## 5. Control-flow rule

Research rule candidate:

```text
Boolean branching requires Bool.
Quad branching requires explicit quad match or explicit projection.
```

This protects the verifier from ambiguous control flow.

A quad condition cannot silently decide which branch is authoritative.

| Quad state | Silent bool coercion | Required behavior |
|---|---|---|
| `N` | unsafe | explicit unknown handling |
| `F` | unsafe unless projected | explicit false handling |
| `T` | unsafe unless projected | explicit true handling |
| `S` | unsafe | explicit conflict handling |

Safe patterns:

```text
match q {
  N => handle_unknown(),
  F => handle_false(),
  T => handle_true(),
  S => handle_conflict(),
}

if is_strict_true(q) {
  ...
}
```

The exact surface syntax is not fixed by this research document.

## 6. Projection functions

Quad-to-bool conversion must be explicit and named.

Candidate projections:

| Projection | Returns true when | Risk if misused |
|---|---|---|
| `is_unknown(q)` | `q == N` | may confuse unknown with denial if policy is unclear |
| `is_false(q)` | `q == F` | strict false only |
| `is_true(q)` | `q == T` | strict true only |
| `is_conflict(q)` | `q == S` | conflict detection |
| `has_false(q)` | `q == F || q == S` | sees false evidence even in conflict |
| `has_true(q)` | `q == T || q == S` | sees true evidence even in conflict |
| `is_known(q)` | `q == F || q == T || q == S` | treats conflict as known, not resolved |
| `is_resolved(q)` | `q == F || q == T` | excludes unknown and conflict |

Important distinction:

```text
is_true(S)      == false
has_true(S)     == true
is_conflict(S)  == true
```

This prevents `S` from becoming implicit `T` merely because it contains true evidence.

## 7. Candidate primitive operations

This document does not freeze final names or opcodes, but it identifies operation classes that should remain visible through lowering.

| Operation class | Meaning |
|---|---|
| `quad_not` | swaps true and false evidence; preserves `N` and `S` |
| `quad_merge` | combines evidence from two sources |
| `quad_intersect` | keeps evidence common to two sources |
| `quad_project` | explicit projection to `Bool` |
| `quad_match` | explicit four-state control |
| `quad_resolve` | policy-driven conflict/unknown resolution |
| `quad_assert` | verifier/runtime assertion over quad state |

Candidate negation table:

| q | not q |
|---|---|
| `N` | `N` |
| `F` | `T` |
| `T` | `F` |
| `S` | `S` |

`S` remains `S` under negation because both evidence planes are present.

## 8. Conflict is not an exception

`S` is not a panic state by default.

It is a meaningful reasoning state.

A conflict may be:

- propagated;
- matched;
- explicitly resolved;
- denied by verifier in strict contexts;
- converted through an explicit policy;
- recorded for audit or diagnostics.

What must not happen:

```text
S silently becomes T
S silently becomes F
S is ignored because bool-only control demanded a branch
```

Conflict handling must be explicit and deterministic.

## 9. Unknown is not false

`N` must not be silently treated as `F`.

Unknown means the system lacks sufficient evidence.

In reasoning programs, confusing unknown with false is a semantic bug.

Candidate invariant:

```text
N != F
```

This matters for verifier-first execution because lack of evidence must not become permission, denial, or branch authority unless an explicit policy says so.

## 10. Determinism requirement

Quad evaluation must be deterministic.

Given:

```text
same SemCode
same runtime profile
same capability context
same inputs
```

quad operations must produce the same result.

No quad operation may depend on:

- host time;
- random choice;
- unordered hash iteration;
- hidden environment state;
- nondeterministic conflict resolution.

If conflict resolution requires policy, that policy must be explicit and verifier-visible.

## 11. Verifier responsibilities

The verifier should be able to reject or classify artifacts that misuse quad values.

Candidate verifier checks:

| Check | Purpose |
|---|---|
| reject implicit quad-to-bool branch | prevents hidden evidence collapse |
| require explicit projection for bool control | makes branch authority visible |
| require exhaustive quad handling where needed | prevents unhandled `N` or `S` |
| preserve conflict visibility through lowering | prevents `S` from disappearing silently |
| validate projection semantics | ensures projection behavior is known |
| deny nondeterministic conflict resolution | preserves replay stability |
| deny ambiguous effect authorization | prevents `N` or `S` from authorizing real effects implicitly |

Verifier principle:

```text
A reasoning state may be unresolved or conflicting.
It must not become silently decisive.
```

## 12. IR implications

Quad logic should remain visible in IR.

If a source-level quad construct is lowered too early into ordinary booleans, the verifier loses the ability to inspect reasoning state.

Candidate IR requirements:

- represent `Quad` as a distinct type;
- represent quad constants `N`, `F`, `T`, `S` distinctly;
- represent quad projections explicitly;
- represent quad match explicitly;
- preserve enough metadata for verifier diagnostics;
- avoid lowering `Quad` to `Bool` before projection is validated.

Candidate IR operation classes:

```text
QuadConst(N/F/T/S)
QuadNot
QuadMerge
QuadIntersect
QuadProject(kind)
QuadMatch(arms)
QuadResolve(policy)
```

These are research names, not final API.

## 13. SemCode implications

SemCode should encode quad operations in a verifier-visible way.

The verifier should not need to reverse-engineer whether a boolean branch originally came from a quad value.

SemCode implications:

- quad constants should be distinguishable from bool constants;
- quad operation classes should be recognizable;
- projection operations should be explicit;
- quad match should expose handling of `N`, `F`, `T`, and `S`;
- effect-bearing paths should reveal whether their authorization depends on quad projection or resolution.

Non-goal:

```text
This document does not define final SemCode binary encoding.
```

## 14. VM implications

The VM should execute quad operations deterministically and cheaply.

VM responsibilities:

- execute quad primitive operations according to fixed tables or fixed bit-plane rules;
- never treat `Quad` as `Bool` implicitly;
- trap closed on malformed or impossible encoded quad states if such states can appear;
- preserve replay stability for quad operations;
- avoid hidden host dependencies in conflict resolution.

VM non-responsibility:

```text
The VM should not be responsible for discovering implicit quad-to-bool misuse.
That is verifier territory.
```

The VM executes admitted SemCode. The verifier decides whether quad usage is admissible.

## 15. Relation to admission

Quad logic affects admission because control decisions may depend on reasoning states.

Admission should distinguish:

| Case | Admission impact |
|---|---|
| pure quad computation | admissible if deterministic |
| explicit quad match | admissible if handling is complete or policy-defined |
| explicit projection | admissible if projection semantics are known |
| implicit quad-to-bool coercion | deny or reject from strict profile |
| unresolved `S` in effect path | deny unless explicit conflict policy exists |
| unresolved `N` in permission path | deny unless explicit unknown policy exists |

Important rule candidate:

```text
No external effect may be authorized by an implicit quad collapse.
```

## 16. Effect and capability boundary

Quad states may appear in reasoning that leads toward effects.

But capability and audit admission must not be decided by ambiguous quad collapse.

Conservative candidate policy:

| Capability predicate state | Default safe behavior |
|---|---|
| `T` | may proceed only if capability context also allows |
| `F` | deny |
| `N` | deny or require explicit unknown policy |
| `S` | deny or require explicit conflict-resolution policy |

Research candidate:

```text
Only explicit strict T may authorize effect-bearing paths by default.
N and S require explicit policy.
```

This is not yet a public spec.

## 17. Diagnostics implications

Quad misuse should produce precise diagnostics.

Candidate diagnostic classes:

| Diagnostic | Meaning |
|---|---|
| implicit quad branch | `Quad` used where `Bool` is required |
| non-exhaustive quad match | one or more states are not handled |
| ambiguous projection | projection policy is not explicit |
| conflict reaches effect path | `S` reaches capability/effect decision without policy |
| unknown reaches permission path | `N` reaches authorization decision without policy |
| nondeterministic resolution | conflict/unknown resolution depends on unstable input |

Diagnostics should prefer repair guidance:

```text
use explicit match over N/F/T/S
use is_true(q) if strict true is intended
use has_true(q) only if conflict-aware true evidence is intended
provide explicit conflict policy before effect authorization
```

## 18. Evidence matrix

| Claim | Current maturity | Evidence needed |
|---|---:|---|
| Quad values are not implicit booleans | L1/L2 | parser/sema/verifier rejection tests |
| `N` is distinct from `F` | L1/L2 | evaluation and branch tests |
| `S` is preserved as conflict | L1/L2 | propagation and match tests |
| quad match is explicit | L1 | syntax/IR/SemCode fixtures |
| projection is explicit | L1/L2 | projection tests and diagnostics |
| effect authorization cannot use implicit quad collapse | L1 | admission denial fixtures |
| conflict resolution is deterministic | L1 | replay tests with explicit policy |
| VM executes quad operations deterministically | L1/L2 | golden evaluation tests |

## 19. Candidate fixture groups

Future fixture groups may include:

```text
quad constants
quad negation
quad merge/intersection
quad projection
quad match exhaustiveness
implicit quad branch rejection
unknown-vs-false distinction
conflict propagation
quad in capability predicate
quad in audit/effect path
quad deterministic replay
```

These are planning placeholders only.

## 20. Open decisions

| ID | Question | Options | Risk |
|---|---|---|---|
| OD-QUAD-001 | Should `if q` be rejected at parse, sema, or verifier? | parse / sema / verifier | wrong layer ownership |
| OD-QUAD-002 | Should quad match require exhaustiveness by default? | always / strict profile only / context-dependent | ergonomics vs safety |
| OD-QUAD-003 | What projection names are canonical? | `is_true` / `strict_true` / `has_true` variants | user confusion |
| OD-QUAD-004 | Should `S` propagate by default? | propagate / deny in strict contexts / policy-defined | conflict invisibility |
| OD-QUAD-005 | Should `N` in permission paths deny by default? | deny / require policy / profile-specific | accidental permission |
| OD-QUAD-006 | Where is quad operation ownership located? | `sm-quad` / IR / VM / verifier split | semantic drift |

## 21. Falsifiability criteria

The quad-logic hypothesis is weakened if:

- quad values require frequent implicit boolean coercion for practical code;
- `N` and `F` collapse in common control flow;
- `S` cannot be preserved through IR and SemCode;
- verifier cannot identify quad-derived branch authority;
- effect authorization can happen through ambiguous quad projection;
- deterministic replay diverges for quad programs;
- diagnostics cannot explain unknown/conflict handling clearly.

## 22. Non-goals

This document does not:

- finalize SemCode binary layout;
- finalize source syntax;
- require all programs to use quad logic;
- claim full formal verification of all quad programs;
- define the complete bilattice theory of Semantic;
- replace `Bool` with `Quad` everywhere;
- make conflict an error by default;
- make unknown equivalent to false.

## 23. Research decision

Quad logic should be treated as a first-class execution-domain primitive.

The verifier must be able to see where quad values influence control, capability, resource, audit, and effect paths.

Core decision:

```text
Semantic must preserve uncertainty and conflict until an explicit, verifier-visible decision resolves them.
```

Short form:

```text
No silent collapse of meaning.
```
