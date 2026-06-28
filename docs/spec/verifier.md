# Verifier Specification

Status: draft v0
Admission owner: `sm-verify`

## Purpose

This document defines the current SemCode admission contract before standard VM
execution.

The verifier is a public admission layer.
It is not an internal VM detail and it is not an optimizer.

## Public Surface

Current verifier surface is centered on:

- `verify_semcode`
- `VerifiedProgram`
- `VerifiedFunction`
- `RejectReport`
- `VerifiedSemCode`
- `VerifiedEntrySemCode`
- `EntryResolutionError`

Canonical token-producing admission path:

- `verify_semcode_token`

Compatibility / legacy admission surface:

- `verify_semcode`

## Verification Scope

Current SemCode verification checks include:

- header validity
- supported version validity
- function and section integrity
- opcode validity
- operand shape validity
- jump-target validity
- string and debug reference validity
- register-budget validity
- call-target validity
- capability consistency with actual opcode usage

## Opcode Admission Matrix

The verifier treats opcode admission as part of SemCode admission.

This matrix documents the current public admission boundary. It uses operation
families rather than binary opcode numbers so that internal encodings can change
without turning this document into a stable bytecode ISA promise.

| Operation family | Admission status | Capability / boundary | Notes |
| --- | --- | --- | --- |
| Ordinary source-derived families: control flow, data movement, literal loading, arithmetic, comparison, calls, and returns | Admitted when produced by supported SemCode and consistent with the emitted contract | No extra capability beyond the artifact contract | Baseline verifier-admitted surface; this document intentionally does not stabilize binary opcode numbers. |
| `SEQUENCE_LEN` | Admitted only when the emitted contract carries `CAP_SEQUENCE_ITERATION` and the header family supports the sequence-iteration contract | Capability-gated | Built-in sequence lowering opcode for the admitted `Sequence(T)` iteration slice. |
| Effect-oriented host-boundary families such as `GateRead`, `GateWrite`, and `PulseEmit` | Admitted only when the emitted contract matches the required capability envelope | Capability-gated / host-boundary | These opcodes do not define capability policy semantics by themselves. |
| Ownership transport payloads admitted through `OWN0` | Admitted structurally only when the ownership transport slice is present and well formed | Header and capability consistency required | This covers the currently documented tuple-only and direct record-field ownership transport slices. |
| Unknown, unsupported, or malformed opcode encodings | Rejected | N/A | Rejection must happen before a successful VM execution path. |
| Opcode streams that fail operand, jump-target, call-target, register-budget, string-reference, or section-integrity checks | Rejected | N/A | These are verifier admission failures, not successful runtime executions. |

Current ownership-specific structural checks for ownership transport include:

- `OWN0` section presence and layout validity when ownership transport is used
- admitted ownership event kind validity
- tuple `AccessPath` payload validity under `SEMCOD11`
- direct record-field `AccessPath` payload validity under `SEMCOD12`
- structural admission for valid `Borrow(Field)` and `Write(Field)` payloads
- header/capability consistency for ownership transport

## Contract Rule

Standard execution uses the chain:

`emit SemCode -> verify_semcode -> (optional) require_entry -> execute`

Important rule:

- VM execution does not replace SemCode admission
- a valid producer path does not waive verifier admission
- entry resolution failure (`EntryResolutionError`) is distinct from verifier rejection (`RejectReport`); missing an entrypoint does not make the artifact itself invalid.

## Separation Rule

`sm-verify` must not become:

- a source parser
- a semantic runtime
- a VM executor
- a general optimizer

It is allowed to reject malformed or contract-inconsistent bytecode only.

Current ownership rule:

- verifier admits ownership payload structurally only
- verifier does not evaluate borrow overlap, release timing, or runtime alias
  policy

## Reject Model

Verifier rejection must preserve:

- the failing verification code
- enough function or offset context to debug the failure
- deterministic diagnostics for the same input artifact

## Verified Execution Rule

The standard `.smc` execution route must require `sm-verify` admission.

Helpers that bypass verification may exist for lower-level testing, but they
must not redefine the public execution contract.

`verify_semcode_token` is the canonical admission boundary for token-first
execution flows. `verify_semcode` remains a compatibility / legacy admission
API and should not be described as the preferred canonical boundary for new
trusted public routes unless a document explicitly distinguishes the two.

## Review Rule

Changes to the verifier require review if they alter:

- what SemCode is considered admissible
- the meaning of an existing verification code
- the structure or deterministic order of reject diagnostics

Required follow-up:

1. update this specification
2. update verifier tests
3. update compatibility or golden tests if public behavior changed
