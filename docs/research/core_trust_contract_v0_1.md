# Core Trust Contract v0.1

Status: research orientation document.
Normative status: non-normative unless promoted into `docs/spec/*`.
Public contract impact: none.
Evidence maturity: research candidate; requires evidence matrix, negative tests, and spec promotion before public contract claims.

## 1. Core statement

Semantic is not a language that attempts to make arbitrary programs safe.

Semantic is a verified execution platform that admits only those programs whose execution rights can be checked against a declared semantic, capability, resource, and audit contract.

Short form:

```text
Semantic executes only what it can admit.
```

Operational law:

```text
No admission — no execution.
No capability — no effect.
No audit — no contact with reality.
```

## 2. Compiler vs verifier boundary

The compiler answers:

```text
Can this source be understood and lowered?
```

The verifier answers:

```text
Does this SemCode artifact have the right to execute?
```

This distinction is central. A successfully compiled artifact is not automatically an admitted program.

```text
source accepted      != admitted
IR lowered           != admitted
SemCode emitted      != admitted
SemCode verified     == candidate for execution
SemCode admitted     == may enter VM runtime under its admission class
```

## 3. Trust model

Semantic follows a software admission model rather than a hardware enclave model.

```text
trusted because isolated  -> hardware TEE model
trusted because admitted  -> Semantic admission model
```

Semantic trust is produced by the verified relationship between:

- SemCode artifact;
- runtime profile;
- capability context;
- resource quotas;
- audit obligations;
- deterministic VM execution rules.

The VM does not become trustworthy by executing arbitrary code defensively. It becomes trustworthy by executing only admitted SemCode under a known profile.

## 4. Admission hypothesis

A strict admission decision can be described as:

```text
AdmittedStrict(SemCode, Profile, CapCtx, Quotas)
⇔
    StructuralValid(SemCode)
∧   SemanticallyWellFormed(SemCode)
∧   DeterministicUnder(Profile)
∧   CapabilitiesDeclaredAndAllowed(SemCode, CapCtx)
∧   EffectsAuditable(SemCode)
∧   QuotasStaticallySufficient(SemCode, Quotas)
∧   TerminationBounded(SemCode)
∧   NoSilentMutation(SemCode)
```

A guarded admission decision can be described as:

```text
AdmittedGuarded(SemCode, Profile, CapCtx, Quotas)
⇔
    StructuralValid(SemCode)
∧   SemanticallyWellFormed(SemCode)
∧   DeterministicUnder(Profile)
∧   CapabilitiesDeclaredAndAllowed(SemCode, CapCtx)
∧   EffectsAuditable(SemCode)
∧   RuntimeQuotaTrapDeterministic(Quotas)
∧   NoSilentMutation(SemCode)
```

The guarded form does not claim static quota sufficiency. It admits execution with deterministic runtime quota traps.

## 5. Halting problem boundary

The verifier is not required to solve the halting problem.

Semantic must not claim to statically prove termination or resource sufficiency for arbitrary programs.

Instead, Semantic separates execution into admission classes:

1. strict admission: execution is statically bounded before VM entry;
2. guarded admission: structure, capability, effect, and determinism are verified, while quota exhaustion is handled by deterministic runtime trap;
3. denial: artifacts with undeclared effects, unverifiable capabilities, nondeterministic dependencies, or unbounded resource behavior are rejected from strict execution.

Core principle:

```text
The verifier does not prove arbitrary programs safe.
The verifier admits only programs with checkable execution rights.
```

## 6. Admission classes

Verifier output should be richer than a boolean admit/deny decision.

Candidate admission classes:

| Class | Meaning | Execution consequence |
|---|---|---|
| `Deny(reason)` | Artifact has no right to execute | VM must not run it |
| `AdmitPure` | Pure deterministic computation, no external effects | VM may execute without Prometheus effect gates |
| `AdmitStrict(certificate)` | Statically bounded execution is proven | VM may execute under strict profile |
| `AdmitGuarded(runtime_limits)` | Runtime quota traps are deterministic | VM may execute with quota guards |
| `AdmitEffectful(token, audit_obligations)` | Effects are capability-gated and auditable | VM may request Prometheus boundary effects only through the admitted token |

This model prevents silent promotion from research claim to public behavior. Each admission class must be tied to tests and specification before becoming normative.

## 7. VM fail-closed responsibility

Admission is the primary safety boundary. The VM remains responsible for cheap fail-closed integrity checks.

The VM should not re-prove the whole safety contract in the hot path. It should verify that it is executing the same artifact that was admitted.

Required fail-closed checks may include:

- artifact hash or admission token match;
- SemCode version/profile compatibility;
- section bounds sanity;
- opcode class sanity;
- deterministic trap on impossible state;
- denial on post-verification artifact mutation.

VM invariant:

```text
The VM executes admitted SemCode, or it traps closed.
It must not silently continue from corrupted or non-admitted state.
```

## 8. Capability and audit boundary

A valid SemCode artifact may still be denied if its requested effects exceed the effective capability context.

Capability admission must check at least:

- declared capability manifest;
- effective capability context;
- effect opcode requirements;
- audit budget requirements;
- Prometheus boundary policy;
- absence of hidden or implicit host effects.

Acceptance criterion candidate:

```text
The verifier must reject structurally valid SemCode when requested effects exceed the effective capability context.
```

Audit law:

```text
No effect without audit.
```

This is especially important for observation paths. Controlled observation must not become a hidden general stdout channel.

## 9. Resource certificates and Proof-Carrying SemCode

Proof-Carrying SemCode is a candidate model for scaling admission.

In this model, SemCode may carry checkable evidence:

- type and effect manifest;
- capability manifest;
- resource certificate;
- loop or recursion bound;
- termination measure;
- audit obligations.

The compiler or frontend may do expensive reasoning, but the verifier must independently validate the certificate against the artifact and active policy.

Rule:

```text
Compiler may produce evidence.
Verifier must validate evidence.
VM may only execute admitted artifacts.
```

## 10. Strict profile loop rule

Unbounded `while` is not part of the Core Trust Profile.

Strict admission should prefer constructs with explicit finite bounds:

```text
repeat max N
for item in bounded(collection, max = N)
recursion with decreasing measure
finite match/case
bounded verified iterator
```

The verifier does not ask whether an arbitrary loop will eventually stop. It asks whether a valid upper bound or termination measure is present and checkable.

## 11. Research-to-spec transition

This document is a research foundation, not yet a public specification.

Promotion path:

```text
Research suggests
  ↓
Evidence filters
  ↓
Spec accepts
  ↓
Tests lock
```

Before promotion to `docs/spec/*`, the following evidence should exist:

- malformed SemCode denial tests;
- structurally valid but capability-invalid SemCode denial tests;
- audit-required effect without audit budget denial tests;
- strict profile denial for unbounded loops;
- guarded profile deterministic quota trap tests;
- post-verify artifact mutation fail-closed tests;
- golden deterministic replay fixtures for admitted artifacts.

## 12. Non-goals

This document does not claim that Semantic:

- makes arbitrary programs safe;
- solves the halting problem;
- replaces hardware TEEs;
- provides production AI safety guarantees;
- makes all verifier decisions complete;
- defines a final public contract;
- finalizes SemCode binary layout or verifier algorithms.

## 13. Current decision

Semantic should be specified around admission rather than unrestricted execution.

The verifier is the right-to-execute boundary.

SemCode is the verifiable artifact.

The VM is the deterministic executor of admitted artifacts.

The Prometheus boundary is the controlled gateway to real effects.

Audit is the trace of contact with reality.

Core decision:

```text
Semantic does not execute arbitrary programs safely.
Semantic admits only programs whose execution rights can be verified.
```
