# Admission Evidence Matrix

Status: research evidence planning document.
Normative status: non-normative unless promoted into `docs/spec/*` and locked by tests.
Public contract impact: none.
Related research document: `docs/research/core_trust_contract_v0_1.md`.

## 1. Purpose

This document maps the Core Trust Contract v0.1 research claims to the evidence required before any claim can be promoted into a public specification.

The goal is to keep research, specification, implementation, and public contract separated.

```text
Research suggests
  ↓
Evidence filters
  ↓
Spec accepts
  ↓
Tests lock
```

## 2. Evidence maturity levels

| Level | Meaning | Claim strength |
|---|---|---|
| L0 — Idea | Discussion only | speculative |
| L1 — Research note | Written in `docs/research/*` | research candidate |
| L2 — Design candidate | Boundaries and non-goals are written | design candidate |
| L3 — Implemented slice | Partial implementation exists | implementation-backed candidate |
| L4 — Tested evidence | Positive and negative tests exist | evidence-backed candidate |
| L5 — Public contract | Spec plus tests plus stable behavior | public contract |

Rule:

```text
Research documents may contain L0-L4 claims.
Public behavior claims require L5.
```

## 3. Core admission claims

| ID | Claim | Current level | Evidence needed | Next action |
|---|---|---:|---|---|
| ADM-001 | No admission means no execution | L1/L2 | fixture proving VM entry requires verifier admission | define admission gate fixtures |
| ADM-002 | Compiled SemCode is not automatically admitted | L1/L2 | emitted-but-not-admitted artifact rejection cases | add compile-vs-admit cases |
| ADM-003 | Verifier should return an admission class | L1 | design decision and tests per class | draft `AdmissionDecision` shape |
| ADM-004 | Strict admission requires bounded execution | L1/L2 | bounded positive cases and unbounded rejection cases | define strict profile rules |
| ADM-005 | Guarded admission permits deterministic quota traps | L1/L2 | replay-stable quota trap cases | define guarded trap semantics |
| ADM-006 | Verifier does not solve arbitrary halting | L1 | explicit non-goal and strict/guarded split | preserve as design boundary |
| ADM-007 | VM executes only matching admitted artifact/profile | L1/L2 | profile and artifact compatibility fixtures | design admission token concept |
| ADM-008 | VM fails closed on corrupted admitted state | L1 | integrity mismatch fixtures | define fail-closed invariant |

## 4. Capability and audit claims

| ID | Claim | Current level | Evidence needed | Next action |
|---|---|---:|---|---|
| CAP-001 | No capability means no external effect | L1/L2 | valid artifact rejected when required capability is absent | build capability matrix |
| CAP-002 | Declared capabilities are checked against effective context | L1/L2 | declared-but-not-granted rejection cases | define effective context fixture |
| CAP-003 | Capability manifest must be well formed | L1 | malformed manifest rejection cases | define manifest validation errors |
| AUD-001 | No effect without audit | L1/L2 | effect path requires audit obligation and budget | build audit matrix |
| AUD-002 | Audit obligations are part of admission | L1 | admission result carries audit obligations | define audit obligation model |
| AUD-003 | Controlled observation is not general stdout | L1/L2 | observation path remains capability/audit bound | connect to observation case study |

## 5. Resource and determinism claims

| ID | Claim | Current level | Evidence needed | Next action |
|---|---|---:|---|---|
| RES-001 | Strict admission requires statically sufficient quotas | L1 | resource certificate and quota sufficiency cases | define certificate MVP |
| RES-002 | Guarded admission uses deterministic runtime quota traps | L1/L2 | same artifact/profile/quota gives same trap | add replay cases |
| RES-003 | Unbounded behavior is rejected from strict profile | L1/L2 | loop/recursion strict-profile rejection cases | define strict rejection errors |
| RES-004 | Bounded loops require checkable upper bounds | L1 | bounded loop positive and negative cases | specify bounded loop rules |
| DET-001 | Same artifact plus same profile/context gives same result | L1/L2 | golden replay suite | add deterministic replay fixtures |
| DET-002 | Runtime traps are deterministic | L1/L2 | replay-stable trap records | define canonical trap record |
| DET-003 | Profile mismatch prevents execution | L1 | profile compatibility fixtures | add profile mismatch cases |

## 6. Mutation and state claims

| ID | Claim | Current level | Evidence needed | Next action |
|---|---|---:|---|---|
| MUT-001 | No silent mutation | L1/L2 | explicit state-transition fixtures | map mutation-bearing constructs |
| MUT-002 | State changes are visible in IR/SemCode contract | L1 | lowering and verifier fixtures | define state-transition contract |
| MUT-003 | Ownership and mutation discipline are checked before VM execution | L1/L2 | overlap and invalid-mutation cases | connect to ownership evidence |

## 7. Proof-Carrying SemCode claims

| ID | Claim | Current level | Evidence needed | Next action |
|---|---|---:|---|---|
| PCCS-001 | SemCode may carry checkable resource evidence | L1 | candidate certificate shape | keep research-only for now |
| PCCS-002 | Compiler may produce evidence, verifier validates it | L1 | certificate consistency cases | define verifier trust boundary |
| PCCS-003 | Invalid certificates prevent admission | L1 | malformed or mismatched certificate cases | create certificate rejection matrix |
| PCCS-004 | Proof-Carrying SemCode is optional for early MVP | L1 | explicit non-goal in first spec candidate | avoid blocking Core Trust Freeze |

## 8. Candidate fixture groups

Candidate future test groups:

```text
admission gate fixtures
compile-vs-admit fixtures
capability matrix fixtures
audit matrix fixtures
strict profile bounds fixtures
guarded quota trap fixtures
post-verify integrity fixtures
deterministic replay fixtures
certificate validation fixtures
```

These are planning placeholders only. They do not imply current implementation.

## 9. Promotion candidates

| Candidate spec | Source claims | Promotion condition |
|---|---|---|
| `docs/spec/admission.md` | ADM-* | admission behavior is tested |
| `docs/spec/capability_manifest.md` | CAP-* | capability matrix exists |
| `docs/spec/audit_obligations.md` | AUD-* | audit matrix exists |
| `docs/spec/resource_certificates.md` | RES-* and PCCS-* | certificate MVP exists |
| `docs/spec/determinism_profile.md` | DET-* | replay behavior is locked |
| `docs/spec/mutation_discipline.md` | MUT-* | explicit mutation fixtures exist |

## 10. Open decisions

| ID | Question | Risk |
|---|---|---|
| OD-ADM-001 | Should `AdmissionDecision` be public API or internal verifier result first? | premature API freeze |
| OD-ADM-002 | Is effectful admission a separate class or an overlay? | class explosion vs clarity |
| OD-RES-001 | What is the first resource certificate MVP? | overbuilding proof-carrying model too early |
| OD-DET-001 | What is the canonical trap record? | unstable replay assertions |
| OD-CAP-001 | Where is opcode-to-capability mapping owned? | drift between verifier and docs |

## 11. Immediate next actions

1. Define `AdmissionDecision` as a research candidate.
2. Define strict vs guarded profile terms.
3. Draft capability matrix.
4. Draft audit matrix.
5. Identify the smallest admission fixture suite.
6. Avoid promotion to `docs/spec/*` until negative tests exist.

## 12. Summary

The Core Trust Contract v0.1 states the platform thesis.

This evidence matrix defines how that thesis becomes engineering evidence.

The matrix is intentionally conservative: it allows Semantic to research strong admission claims without accidentally treating them as implemented public guarantees.
