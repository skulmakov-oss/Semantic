# Quad Drain First Admission Fixture v0.1

Status: research test-design document.
Normative status: non-normative unless promoted into `tests/*` and `docs/spec/*`.
Public contract impact: none.
Related documents:

- `docs/research/core_trust_contract_v0_1.md`
- `docs/research/admission_evidence_matrix.md`
- `docs/research/quad_logic_execution_model_v0_1.md`
- `docs/research/quad_logic_engineering_decisions_v0_1.md`
- `docs/research/quad_logic_safety_semantics_v0_1.md`
- `docs/research/quad_drain_explicit_collapse_policy_v0_1.md`

## 1. Purpose

This document defines the first candidate negative admission fixture for Quad Drain enforcement.

The fixture tests the invariant:

```text
Passive quad inspection must not authorize or control an effect-relevant path.
```

Short form:

```text
Inspection is not Drain.
```

If this fixture is not denied by the verifier, the Quad Drain policy is only documentation, not an execution boundary.

## 2. Threat model

The source compiler may correctly reject `if q` at Sema.

However, an attacker or bypass compiler may generate SemCode directly.

Therefore, the verifier must reject SemCode that converts `Quad` to `Bool` through passive inspection and then uses that Bool to control an effect path.

The test is not about user ergonomics.

It is about the verifier as the final admission boundary.

## 3. Core invariant under test

```text
A Bool derived from Quad inspection is not ordinary Bool for effect-relevant control.
```

The verifier must preserve provenance:

```text
Bool origin = QuadInspected(...)
```

until it can prove the value was produced by an explicit, approved drain or policy operation.

## 4. Negative fixture: deny bypass via is_true inspection

Candidate test name:

```text
test_verifier_deny_is_true_jump_to_effect
```

### 4.1 Conceptual SemCode

```text
.profile Guarded
.capability PATH_WRITE_HOST

.code
    LOAD_QUAD_CONST   r1, QUAD_S
    QUAD_IS_TRUE      r2, r1
    JUMP_IF_FALSE     r2, @label_deny

@label_effect:
    PROMETHEUS_WRITE  [0x00FF], 0x42
    RET

@label_deny:
    RET
```

### 4.2 Why this must be denied

`QUAD_IS_TRUE` is passive inspection.

It answers:

```text
is q strictly T?
```

It does not declare the fate of `N` or `S`.

For `QUAD_S`, the result is:

```text
is_true(S) == false
```

But using that `false` as ordinary control destroys the distinction between:

```text
ordinary false
authorization denied because conflict was present
```

If the effect path is controlled by this Bool, the verifier must reject the artifact.

## 5. Required verifier provenance tracking

Candidate internal provenance labels:

| Provenance | Meaning |
|---|---|
| `PlainBool` | ordinary Bool not derived from Quad |
| `QuadProjection(kind, origin)` | Bool derived from explicit projection such as `is_true` |
| `QuadDrain(policy_N, policy_S, origin)` | Bool derived from explicit drain policy |
| `QuadAuth(predicate_class, origin)` | Bool/Quad authorization predicate produced by safety operator |
| `Unknown` | provenance not known; unsafe in effect paths |

For the negative fixture:

```text
LOAD_QUAD_CONST r1, QUAD_S
```

establishes:

```text
r1: Quad(origin = const S)
```

Then:

```text
QUAD_IS_TRUE r2, r1
```

establishes:

```text
r2: Bool(origin = QuadProjection(is_true, r1))
```

Then:

```text
JUMP_IF_FALSE r2, @label_deny
```

uses a `QuadProjection` Bool as control authority.

When the alternate path reaches:

```text
PROMETHEUS_WRITE
```

the verifier sees that the effect path is controlled by an unsafe projection.

Required result:

```text
Deny(E_QUAD_AUTH_PROJECTION)
```

or equivalent structured denial.

## 6. Control-flow interpretation

Even though the concrete constant is `QUAD_S`, the verifier should not rely only on constant folding to deny this fixture.

The deeper rule is data-flow based:

```text
A branch predicate derived from Quad inspection cannot guard an effect path unless an explicit drain or authorization operator is present.
```

This must hold for:

- constant `S`;
- runtime-computed `S`;
- runtime-computed `N`;
- unknown quad values;
- values loaded from admitted but unresolved reasoning state.

## 7. Positive sibling: allow explicit drain with S trap

Candidate positive test name:

```text
test_verifier_admit_quad_drain_s_trap_to_effect
```

### 7.1 Conceptual SemCode

```text
.profile Guarded
.capability PATH_WRITE_HOST

.code
    LOAD_QUAD_CONST   r1, QUAD_S
    QUAD_DRAIN        r2, r1, N_FALSE, S_TRAP
    JUMP_IF_FALSE     r2, @label_deny

@label_effect:
    PROMETHEUS_WRITE  [0x00FF], 0x42
    RET

@label_deny:
    RET
```

### 7.2 Why this may be admitted

`QUAD_DRAIN` is not passive inspection.

It explicitly states:

```text
N => false
S => deterministic trap
```

Therefore, an unresolved conflict cannot silently become `false` or authorize the effect path.

If `r1 == S`, execution must trap deterministically before branch control can silently proceed.

Required result candidate:

```text
AdmitGuarded(runtime_limits, drain_policy)
```

or equivalent admission class.

## 8. Rust test sketch

This sketch is intentionally conceptual. It does not claim that the current repository already exposes these exact APIs.

```rust
#[test]
fn test_verifier_deny_is_true_jump_to_effect() {
    let mut artifact = SemCodeArtifact::new(Profile::Guarded);

    artifact.add_instruction(Op::LoadQuadConst {
        reg: 1,
        val: Quad::S,
    });

    artifact.add_instruction(Op::QuadIsTrue {
        dst_bool_reg: 2,
        src_quad_reg: 1,
    });

    artifact.add_instruction(Op::JumpIfFalse {
        condition_reg: 2,
        target: 5,
    });

    artifact.add_instruction(Op::PrometheusWrite {
        address: 0x00FF,
        value: 0x42,
    });

    artifact.add_instruction(Op::Ret);

    let result = Verifier::validate(&artifact);

    assert!(matches!(result, AdmissionDecision::Deny(_)));

    if let AdmissionDecision::Deny(reason) = result {
        assert_eq!(reason.code, "E_QUAD_AUTH_PROJECTION");
    }
}
```

Positive sibling sketch:

```rust
#[test]
fn test_verifier_admit_quad_drain_s_trap_to_effect() {
    let mut artifact = SemCodeArtifact::new(Profile::Guarded);

    artifact.add_instruction(Op::LoadQuadConst {
        reg: 1,
        val: Quad::S,
    });

    artifact.add_instruction(Op::QuadDrain {
        dst_bool_reg: 2,
        src_quad_reg: 1,
        on_unknown: DrainPolicy::False,
        on_conflict: DrainPolicy::Trap,
    });

    artifact.add_instruction(Op::JumpIfFalse {
        condition_reg: 2,
        target: 5,
    });

    artifact.add_instruction(Op::PrometheusWrite {
        address: 0x00FF,
        value: 0x42,
    });

    artifact.add_instruction(Op::Ret);

    let result = Verifier::validate(&artifact);

    assert!(matches!(result, AdmissionDecision::AdmitGuarded(_)));
}
```

## 9. Expected denial reason

Candidate denial code:

```text
E_QUAD_AUTH_PROJECTION
```

Meaning:

```text
A Bool produced by passive Quad inspection reached an effect-relevant control path.
Use explicit QuadDrain or an approved authorization operator.
```

Alternative names:

```text
E_QUAD_LOST_PROVENANCE
E_QUAD_IMPLICIT_DRAIN
E_QUAD_EFFECT_PROJECTION
```

Recommended first name:

```text
E_QUAD_AUTH_PROJECTION
```

because it identifies the effect-relevant misuse directly.

## 10. Why this is the first fixture

This is the minimal negative fixture because it connects all major research laws:

| Law | Covered? |
|---|---:|
| No silent collapse of meaning | yes |
| Inspection is not Drain | yes |
| No conflict-based authorization by default | yes |
| Sema explains, Verifier enforces | yes |
| VM executes admitted meaning only | yes |
| No effect without explicit safe path | yes |

It is also small enough to become the first real test once the verifier test harness exists.

## 11. Required implementation hooks

Before this becomes a real test, the implementation likely needs:

- a way to build raw SemCode test artifacts;
- a verifier entry point for fixture artifacts;
- a representation of quad projection provenance;
- a representation of explicit drain policy;
- a way to classify effect-relevant instructions;
- structured denial reason codes;
- an admission decision enum or equivalent result model.

This document does not assume these already exist.

## 12. Acceptance criteria

The fixture is accepted as implemented only when:

1. the negative SemCode artifact is structurally valid;
2. the verifier denies it before VM execution;
3. the denial reason identifies passive quad projection reaching an effect path;
4. the positive sibling with explicit `S_TRAP` drain is admitted or reaches the expected guarded admission class;
5. deterministic trap behavior is tested separately for the positive sibling;
6. no VM execution is required to discover the negative artifact misuse.

## 13. Non-goals

This document does not:

- define final SemCode opcode names;
- define final Rust test APIs;
- require immediate code implementation;
- claim the current repository already supports this fixture;
- finalize diagnostic code names;
- define all QuadDrain policies;
- replace the broader admission evidence matrix.

## 14. Summary

This fixture is the first physical test shape for Quad Drain enforcement.

It proves the most important boundary:

```text
Passive inspection cannot smuggle Quad uncertainty or conflict into effect control.
```

Short form:

```text
Inspection is not permission.
```
