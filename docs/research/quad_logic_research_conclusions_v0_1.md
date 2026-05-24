# Quad Logic Research Conclusions v0.1

Status: research conclusion document.
Normative status: non-normative unless promoted into `docs/spec/*` and locked by tests.
Public contract impact: none.
Related documents:

- `docs/research/core_trust_contract_v0_1.md`
- `docs/research/admission_evidence_matrix.md`
- `docs/research/quad_logic_execution_model_v0_1.md`
- `docs/research/quad_logic_engineering_decisions_v0_1.md`
- `docs/research/quad_logic_safety_semantics_v0_1.md`
- `docs/research/quad_drain_explicit_collapse_policy_v0_1.md`
- `docs/research/quad_drain_first_admission_fixture_v0_1.md`

## 1. Purpose

This document records the current research conclusions from the Quad Logic / Quad Drain / Admission discussion.

It is not a public specification.

It is a consolidation layer that explains what the current research documents imply for Semantic architecture, verifier design, Sema behavior, admission profiles, and future tests.

## 2. Core conclusion

Semantic now has two foundational pillars:

```text
Semantic executes only what it can admit.
Semantic preserves meaning until a verifier-visible policy resolves it.
```

The first pillar defines the right to execute.

The second pillar defines the right to collapse meaning into decision.

Together:

```text
No admission — no execution.
No silent collapse of meaning.
No drain without policy.
Inspection is not permission.
```

## 3. The trust-kernel shape

The current research set forms an early trust-kernel shape:

```text
Core Trust Contract
  ↓
Admission Evidence Matrix
  ↓
Quad Logic Execution Model
  ↓
Quad Logic Engineering Decisions
  ↓
Quad Logic Safety Semantics
  ↓
Quad Drain Explicit Collapse Policy
  ↓
First Quad Drain Admission Fixture
```

This chain matters because it moves the work from abstract concept toward evidence:

```text
thesis → invariant → decision → safety semantics → collapse policy → fixture shape
```

This is the correct path for Semantic: research first, spec second, tests as the lock.

## 4. Three laws of the current Quad model

The current Quad research block can be summarized by three laws.

### 4.1 Semantic executes only what it can admit

An emitted SemCode artifact is not automatically executable.

It becomes executable only after admission.

### 4.2 No silent collapse of meaning

`Quad` states carry uncertainty and conflict.

They must not silently collapse into `Bool`, `F`, `T`, permission, denial, or branch authority.

### 4.3 No drain without policy

If a `Quad` becomes a `Bool`, the fate of `N` and `S` must be explicit.

A collapse without an explicit `N/S` policy is not a safe collapse.

## 5. Quad is not just four-valued logic

The current model does not merely say:

```text
Semantic uses four-valued logic.
```

It says:

```text
Semantic uses quad states as safety-visible execution states.
```

`N`, `F`, `T`, and `S` are not decorative values.

They are verifier-visible reasoning states that may affect:

- control flow;
- effect authorization;
- capability decisions;
- audit obligations;
- admission class;
- deterministic replay;
- VM fail-closed behavior.

## 6. Practical meaning of N/F/T/S

| State | Mathematical intuition | Semantic safety meaning |
|---|---|---|
| `N` | no evidence | unknown; cannot silently authorize |
| `F` | false evidence | denial or negative evidence |
| `T` | true evidence | candidate permission, still subject to capability/audit/quota |
| `S` | both true and false evidence | conflict; visibility obligation |

The most important distinction:

```text
S is not merely more information.
S is a conflict that must remain visible.
```

## 7. Why Semantic diverges from classical Belnap tables

Classical four-valued logic is useful as a conceptual ancestor.

But Semantic cannot blindly inherit classical truth tables in effect/admission paths.

Reason:

```text
Semantic is not only classifying information.
Semantic is deciding whether execution may proceed.
```

A truth table that is elegant for knowledge representation may be unsafe for runtime admission.

Therefore, Semantic needs safety-specific operator families.

## 8. Risk-oriented ordering

The research documents introduced a risk/visibility ordering:

```text
S > N > F > T
```

This is not a truth order.

It is a safety-priority order.

| Priority | Meaning |
|---:|---|
| `S` | conflict must surface first |
| `N` | insufficient evidence must block permission |
| `F` | explicit denial |
| `T` | permission candidate, still checked by capability/audit/quota |

This order prevents two dangerous collapses:

```text
S disappearing into F
N merging with T into permission
```

## 9. Inspection, Drain, and Authorization

The most important engineering distinction is:

```text
Inspection != Drain != Authorization
```

### 9.1 Inspection

Inspection functions observe evidence:

```text
is_true(q)
has_true(q)
is_conflict(q)
```

They answer questions about the state.

They do not define the fate of `N` and `S`.

### 9.2 Drain

Drain is an explicit `Quad -> Bool` boundary:

```text
q ?? (N => ..., S => ...)
```

Drain declares what happens to uncertainty and conflict.

### 9.3 Authorization

Authorization is a safety decision for effect/admission paths.

It should use approved safety operators or approved drain policies, not passive inspection.

## 10. Why is_true is dangerous in effect paths

`is_true(q)` is safe as inspection, but unsafe as a hidden effect-path drain.

Truth table:

| q | `is_true(q)` |
|---|---:|
| `T` | true |
| `F` | false |
| `N` | false |
| `S` | false |

The problem:

```text
F, N, and S collapse into the same Bool false.
```

That erases the difference between:

```text
ordinary denial
unknown evidence
conflict evidence
```

Therefore:

```text
is_true(q) is inspection.
QuadDrain is policy.
Authorization requires approved safety semantics.
```

## 11. Provenance tracking conclusion

The verifier needs to understand not only that a value is `Bool`, but where that `Bool` came from.

Candidate internal model:

```text
PlainBool
BoolFromQuadProjection
BoolFromQuadDrain
BoolFromQuadAuth
BoolFromUnknownOrigin
```

This enables a key rule:

```text
A Bool derived from passive Quad inspection must not control an effect-relevant path by default.
```

This does not require full unrestricted program proof.

It requires targeted provenance tracking for the dangerous path:

```text
Quad → Bool → Branch → Effect
```

## 12. Sema and Verifier split

The research now clearly separates ergonomics from trust enforcement.

| Layer | Responsibility |
|---|---|
| Parser | build syntax without type policy |
| Sema | reject or diagnose implicit `Quad` branch authority for developer UX |
| IR | preserve quad origin and operation class |
| SemCode | expose projection/drain/auth operation class to verifier |
| Verifier | enforce admission boundary even if SemCode bypasses Sema |
| VM | execute only admitted meaning and trap closed on impossible/corrupt state |

Key split:

```text
Sema explains.
Verifier enforces.
```

## 13. The first negative fixture is correctly chosen

The first candidate fixture tests the core failure mode:

```text
Quad S
  ↓
is_true(S) -> false
  ↓
Bool controls branch
  ↓
effect path exists
```

Required verifier result:

```text
Deny(E_QUAD_AUTH_PROJECTION)
```

This fixture is important because it verifies:

- passive inspection is not drain;
- conflict cannot disappear before an effect path;
- Sema can be bypassed but Verifier cannot;
- effect control requires approved provenance;
- the admission boundary exists before VM execution.

## 14. Positive sibling is equally important

The positive sibling uses explicit drain:

```text
QUAD_DRAIN r2, r1, N_FALSE, S_TRAP
```

This demonstrates the approved route:

```text
N has explicit fate.
S has explicit fate.
Verifier can admit under Guarded profile.
```

Expected result candidate:

```text
AdmitGuarded(...)
```

The negative fixture proves the boundary.

The positive sibling proves the intended escape hatch.

Both are required to avoid false confidence.

## 15. Current hypothesis maturity

| Hypothesis | Current maturity | Conclusion |
|---|---:|---|
| Verifier-first admission | L2 | ready for spec extraction candidate |
| Strict/Guarded admission split | L2 | needs profile formalization |
| Quad as execution-domain primitive | L2 | strong design candidate |
| No silent collapse of meaning | L2 | core invariant candidate |
| Safety semantics over classical truth tables | L2 | strong architectural distinction |
| Quad Drain | L2 | ready for first fixture implementation planning |
| Quad provenance tracking | L1/L2 | next verifier design target |
| First negative fixture | L1/L2 | ready to become real test when harness exists |
| Proof-Carrying SemCode | L1 | keep as future research track |

Current status:

```text
Design candidate, not public contract.
```

## 16. Main risk

The largest remaining risk is not mathematical.

It is ergonomic.

If Semantic allows convenient shortcuts such as:

```text
if is_true(q) { effect(); }
```

in effect-relevant paths, then the system will reintroduce silent collapse through the UX layer.

This would weaken the whole Quad model.

Therefore, developer convenience must not override the invariant:

```text
No silent collapse of meaning.
```

## 17. Required next implementation concepts

Before the first real tests can be written, the project likely needs:

- raw SemCode fixture construction;
- verifier test entry point;
- effect-relevant instruction classification;
- `Bool` provenance metadata;
- quad projection metadata;
- quad drain policy representation;
- structured denial reason codes;
- admission decision result shape;
- deterministic guarded trap semantics.

This document does not claim that these already exist.

## 18. Recommended next steps

Recommended order:

1. define minimal `AdmissionDecision` shape;
2. define minimal denial reason code for `E_QUAD_AUTH_PROJECTION`;
3. define internal `BoolProvenance` model;
4. define conceptual raw SemCode fixture builder or equivalent test harness;
5. implement the negative fixture from `quad_drain_first_admission_fixture_v0_1.md`;
6. implement the positive sibling with explicit `S_TRAP`;
7. only then promote the relevant parts into `docs/spec/*`.

## 19. Falsifiability criteria

The current Quad hypothesis is weakened if:

- practical code requires frequent implicit `Quad -> Bool` coercion;
- `N` and `S` disappear before diagnostics or verifier admission;
- `is_true` becomes the common workaround for effect-path control;
- SemCode cannot preserve quad-origin metadata;
- verifier cannot distinguish passive inspection from explicit drain;
- effect paths can be controlled by unapproved quad projections;
- negative fixture denial requires VM execution instead of verifier admission.

## 20. Final conclusion

Semantic is no longer only a language for computation.

The current research direction makes Semantic a verified execution substrate for decisions under incomplete or contradictory information.

Core conclusion:

```text
If Core Trust Contract defines the right to execute,
Quad Logic defines the right to decide.
```

Final short form:

```text
Semantic must not merely compute truth.
Semantic must preserve meaning until a verified policy permits decision.
```
