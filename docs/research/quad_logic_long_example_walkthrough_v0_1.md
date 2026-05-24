# Quad Logic Long Example Walkthrough v0.1

Status: research example and walkthrough.
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
- `docs/research/quad_logic_research_conclusions_v0_1.md`

## 1. Purpose

This document provides a long, end-to-end example of Quad Logic inside the Semantic admission model.

The goal is to show how the research rules interact in a realistic effect-bearing scenario:

```text
Quad reasoning
  ↓
Safety-aware authorization
  ↓
Explicit QuadDrain
  ↓
SemCode-like lowering
  ↓
Verifier provenance analysis
  ↓
Admission decision
  ↓
VM / Prometheus Boundary behavior
```

This is an example, not a final syntax or opcode specification.

## 2. Scenario

A Semantic program wants to write a structured event into host state through the Prometheus Boundary.

The write is allowed only if several independent checks agree:

1. local policy permits the operation;
2. runtime capability context confirms the token;
3. audit budget is available;
4. no conflict is present in the policy chain;
5. unknown states are explicitly handled;
6. the final effect path is authorized by a verifier-visible safety operation.

The program is running under a candidate `Guarded` profile.

## 3. Actors

| Symbol | Meaning | Possible Quad states |
|---|---|---|
| `local_policy` | static/local rule about whether this operation is allowed | `T/F/N/S` |
| `capability_ctx` | runtime capability context result | `T/F/N/S` |
| `audit_ready` | audit budget and trace path availability | `T/F/N/S` |
| `state_clean` | whether local state is not conflicted | `T/F/N/S` |
| `auth` | combined authorization predicate | `T/F/N/S` |
| `decision` | final Bool after explicit drain | `true/false/trap` |

Important distinction:

```text
auth: Quad
decision: Bool derived from explicit policy
```

## 4. Quad evidence states

The example uses the evidence-plane interpretation:

```text
Quad = (e_f, e_t)
```

| State | Evidence plane | Meaning |
|---|---:|---|
| `N` | `(0,0)` | no evidence |
| `F` | `(1,0)` | false evidence |
| `T` | `(0,1)` | true evidence |
| `S` | `(1,1)` | conflict evidence |

The safety priority for effect/admission paths is:

```text
S > N > F > T
```

This is not truth order.

It is risk/visibility order.

## 5. Authorization operator: AuthAll

For effect-bearing paths, the example uses a safety operator:

```text
AuthAll(a, b)
```

Meaning:

```text
all required predicates must be strictly true,
while unknown and conflict remain visible.
```

Candidate table:

| `AuthAll` | N | F | T | S |
|---|---|---|---|---|
| N | N | N | N | S |
| F | N | F | F | S |
| T | N | F | T | S |
| S | S | S | S | S |

Interpretation:

- any `S` produces `S`;
- otherwise any `N` produces `N`;
- otherwise any `F` produces `F`;
- only all `T` produces `T`.

This operator is not classical boolean `AND` and not a direct Belnap truth table.

It is safety-oriented authorization combination.

## 6. Source-level example: safe version

Conceptual source shape:

```text
fn write_event_guarded(event: Event) -> Result {
    let local_policy: Quad = check_local_policy(event);
    let capability_ctx: Quad = check_capability("PATH_WRITE_HOST");
    let audit_ready: Quad = check_audit_budget(event);
    let state_clean: Quad = check_state_consistency(event);

    let auth_1: Quad = AuthAll(local_policy, capability_ctx);
    let auth_2: Quad = AuthAll(auth_1, audit_ready);
    let auth: Quad = AuthAll(auth_2, state_clean);

    if auth ?? (N => audit_then_false, S => trap) {
        Prometheus.write_host_state(event);
        return ok();
    }

    return denied();
}
```

This code is safe because:

- authorization uses `AuthAll`, not generic `AND`;
- final `Quad -> Bool` is explicit;
- `N` has a declared fate;
- `S` has a declared fate;
- the effect path is controlled by a verifier-visible drain policy;
- audit behavior is declared for unknown state;
- conflict does not silently become false.

## 7. Unsafe source-level anti-example

Conceptual unsafe shape:

```text
fn write_event_bad(event: Event) -> Result {
    let local_policy: Quad = check_local_policy(event);
    let capability_ctx: Quad = check_capability("PATH_WRITE_HOST");

    let auth: Quad = merge(local_policy, capability_ctx);

    if is_true(auth) {
        Prometheus.write_host_state(event);
        return ok();
    }

    return denied();
}
```

Why this is unsafe:

- `merge` is ambiguous;
- `is_true(auth)` is passive inspection;
- `N` and `S` silently collapse to `false`;
- the effect path is controlled by a Bool derived from quad projection;
- the verifier cannot treat this as approved authorization unless additional policy is visible.

Required outcome under Guarded/effect-relevant context:

```text
Deny(E_QUAD_AUTH_PROJECTION)
```

or equivalent structured denial.

## 8. Case analysis

### Case A: all checks are T

```text
local_policy  = T
capability_ctx = T
audit_ready   = T
state_clean   = T
```

Evaluation:

```text
AuthAll(T, T) = T
AuthAll(T, T) = T
AuthAll(T, T) = T
```

Final:

```text
auth = T
QuadDrain(T, N => audit_then_false, S => trap) = true
```

Admission:

```text
AdmitGuarded(effectful, audit_obligation, capability_checked)
```

Runtime behavior:

```text
Prometheus.write_host_state(event)
```

provided capability, quota, and audit checks also pass.

### Case B: local policy permits, capability unknown

```text
local_policy  = T
capability_ctx = N
audit_ready   = T
state_clean   = T
```

Evaluation:

```text
AuthAll(T, N) = N
AuthAll(N, T) = N
AuthAll(N, T) = N
```

Final:

```text
auth = N
QuadDrain(N, N => audit_then_false, S => trap) = false + audit obligation
```

Admission:

```text
AdmitGuarded only if audit_then_false obligation is valid and budgeted
```

Runtime behavior:

```text
record audit: unknown capability evidence reached authorization
return denied
no host write
```

Key point:

```text
T + N does not become T.
```

Unknown capability cannot become permission.

### Case C: local denial, capability conflict

```text
local_policy  = F
capability_ctx = S
audit_ready   = T
state_clean   = T
```

Evaluation:

```text
AuthAll(F, S) = S
AuthAll(S, T) = S
AuthAll(S, T) = S
```

Final:

```text
auth = S
QuadDrain(S, N => audit_then_false, S => trap) = deterministic trap
```

Admission:

```text
AdmitGuarded may be possible only if S_TRAP is verifier-visible and deterministic
```

Runtime behavior:

```text
trap before effect
no host write
```

Why not `F`?

Because the conflict must not disappear just because a denial also exists.

The system must distinguish:

```text
denied because false
denied with conflict present
```

### Case D: all clean except audit unknown

```text
local_policy  = T
capability_ctx = T
audit_ready   = N
state_clean   = T
```

Evaluation:

```text
AuthAll(T, T) = T
AuthAll(T, N) = N
AuthAll(N, T) = N
```

Final:

```text
auth = N
QuadDrain(N, N => audit_then_false, S => trap) = false + audit obligation
```

This case is subtle.

If audit is unavailable, writing an audit record may also be impossible.

Therefore, `audit_then_false` must be an admission-visible obligation, not a hidden runtime effect.

If audit budget/path is absent:

```text
Verifier must deny admission before VM execution.
```

### Case E: state conflict after all permissions are true

```text
local_policy  = T
capability_ctx = T
audit_ready   = T
state_clean   = S
```

Evaluation:

```text
AuthAll(T, T) = T
AuthAll(T, T) = T
AuthAll(T, S) = S
```

Final:

```text
auth = S
QuadDrain(S, N => audit_then_false, S => trap) = deterministic trap
```

This prevents a state-integrity conflict from being hidden by otherwise valid permissions.

## 9. Why not use is_true directly?

Consider:

```text
if is_true(auth) {
    Prometheus.write_host_state(event);
}
```

Projection table:

| auth | is_true(auth) |
|---|---:|
| T | true |
| F | false |
| N | false |
| S | false |

This erases the difference between:

- denial;
- unknown;
- conflict.

For pure computation, this may be acceptable when intentionally used.

For effect-relevant control, this is unsafe.

Therefore:

```text
is_true is inspection.
QuadDrain is policy.
AuthAll is authorization combination.
```

## 10. SemCode-like lowering: safe version

Conceptual SemCode-like form:

```text
.profile Guarded
.capability PATH_WRITE_HOST
.audit_required EVENT_WRITE

.code
    CHECK_LOCAL_POLICY      r1, event          ; r1: Quad
    CHECK_CAPABILITY        r2, PATH_WRITE_HOST ; r2: Quad
    CHECK_AUDIT_BUDGET      r3, EVENT_WRITE   ; r3: Quad
    CHECK_STATE_CLEAN       r4, event          ; r4: Quad

    QUAD_AUTH_ALL           r5, r1, r2         ; r5: QuadAuth
    QUAD_AUTH_ALL           r6, r5, r3         ; r6: QuadAuth
    QUAD_AUTH_ALL           r7, r6, r4         ; r7: QuadAuth

    QUAD_DRAIN              r8, r7, N_AUDIT_THEN_FALSE, S_TRAP ; r8: BoolFromQuadDrain
    JUMP_IF_FALSE           r8, @deny

@effect:
    PROMETHEUS_WRITE        event
    RET_OK

@deny:
    RET_DENIED
```

Important metadata:

```text
r8: BoolProvenance = QuadDrain(N_AUDIT_THEN_FALSE, S_TRAP, origin = r7)
```

Effect path is controlled by an approved drain, not passive inspection.

## 11. SemCode-like lowering: unsafe bypass

Conceptual malicious or bypass-generated SemCode:

```text
.profile Guarded
.capability PATH_WRITE_HOST
.audit_required EVENT_WRITE

.code
    LOAD_QUAD_CONST         r1, QUAD_S
    QUAD_IS_TRUE            r2, r1
    JUMP_IF_FALSE           r2, @deny

@effect:
    PROMETHEUS_WRITE        event
    RET_OK

@deny:
    RET_DENIED
```

Verifier provenance:

```text
r1: Quad(origin = const S)
r2: BoolProvenance = QuadProjection(is_true, r1)
```

Branch:

```text
JUMP_IF_FALSE uses r2
```

Effect path:

```text
PROMETHEUS_WRITE reachable under control derived from passive projection
```

Required result:

```text
Deny(E_QUAD_AUTH_PROJECTION)
```

## 12. Verifier analysis algorithm: conceptual

The verifier does not need to solve arbitrary semantics for this fixture.

It needs targeted provenance tracking.

Conceptual pass:

```text
for instruction in SemCode:
    update type state
    update provenance state
    mark branch control provenance
    mark effect-relevant blocks
    validate that effect paths are controlled only by approved provenance
```

Candidate provenance states:

```text
PlainBool
QuadProjection(kind, origin)
QuadDrain(policy_N, policy_S, origin)
QuadAuth(class, origin)
Unknown
```

Effect path rule:

```text
if branch_controls_effect_path:
    condition provenance must be PlainBool, approved QuadDrain, or approved QuadAuth
    otherwise deny
```

## 13. Sema analysis

Sema should reject direct implicit branch:

```text
if auth { ... }
```

because `auth: Quad`.

Sema should warn or reject effect-path passive projection:

```text
if is_true(auth) {
    Prometheus.write_host_state(event);
}
```

Recommended diagnostic:

```text
Quad inspection cannot control an effect-relevant path.
Use explicit QuadDrain or authorization operator.
```

Sema is not the trust boundary.

It is the developer feedback layer.

The verifier remains final enforcement.

## 14. Admission decision table

| Program shape | Expected admission |
|---|---|
| `AuthAll` + `QuadDrain(N=>audit_then_false,S=>trap)` + effect | `AdmitGuarded` if audit/capability/quota obligations are valid |
| `is_true(auth)` controls effect | `Deny(E_QUAD_AUTH_PROJECTION)` |
| `has_true(auth)` controls effect | `Deny(E_QUAD_CONFLICT_AUTH)` or equivalent |
| `QuadDrain(N=>true,S=>true)` controls effect | deny by default |
| `QuadDrain(N=>false,S=>trap)` controls pure branch | likely admitted under Guarded |
| direct `if quad` | reject in Sema; deny in verifier if encoded |

## 15. Runtime behavior

For admitted safe version:

- VM executes deterministic quad checks;
- `AuthAll` propagates `S` and preserves `N`;
- `QuadDrain` handles `N/S` according to explicit policy;
- Prometheus effect occurs only if final decision is `true` and all capability/audit/quota conditions pass.

For denied unsafe version:

- VM never runs the artifact;
- denial occurs at verifier admission;
- no host effect can occur;
- denial reason should be structured.

## 16. Why this example matters

This example demonstrates the whole Semantic idea in one path:

```text
Meaning is preserved.
Uncertainty is not permission.
Conflict is not hidden.
Inspection is not authorization.
Drain requires policy.
Effect requires admission.
Verifier enforces before VM.
```

It turns the quad research block from abstract design into a concrete execution story.

## 17. Future real tests

Candidate future tests:

```text
test_sema_rejects_if_quad

test_verifier_deny_direct_quad_branch_to_effect

test_verifier_deny_is_true_jump_to_effect

test_verifier_deny_has_true_conflict_authorization

test_verifier_admit_quad_drain_s_trap_to_effect

test_verifier_deny_quad_drain_n_true_to_effect

test_verifier_deny_quad_drain_s_true_to_effect

test_quad_auth_all_preserves_conflict

test_quad_auth_all_unknown_does_not_authorize

test_quad_drain_audit_then_false_requires_audit_budget
```

These tests should be introduced only after the relevant test harness and verifier internals exist.

## 18. Non-goals

This document does not:

- finalize source syntax;
- finalize SemCode opcode names;
- claim the current repository already has these instructions;
- claim verifier provenance tracking already exists;
- replace the first fixture document;
- define final diagnostics;
- promote the model into public specification.

## 19. Summary

This example shows the intended safe pattern:

```text
Quad evidence
  ↓
AuthAll safety combination
  ↓
QuadDrain explicit N/S policy
  ↓
Verifier-approved Bool provenance
  ↓
Effect path
```

and the forbidden pattern:

```text
Quad evidence
  ↓
is_true passive inspection
  ↓
ordinary Bool branch
  ↓
effect path
```

Final law:

```text
A decision that touches reality must be built from admitted meaning, not from collapsed evidence.
```
