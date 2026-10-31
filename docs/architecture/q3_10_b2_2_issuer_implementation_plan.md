# Q3.10 B2-2 - Trusted Admission Gateway Implementation Plan

Status: PROPOSED - implementation not authorized by this checkpoint

First evidence kind: RUSTLIKE-PROGRAM-TYPECHECK-v1

## Narrow contract

The first implementation is limited to one immutable source file, no imports,
the fixed RustLike profile, whole-program typechecking and EvidenceOnly. It
grants no VM privilege and does not generalize to arbitrary source sets or
proof kinds.

## Admission sequence

1. Capture immutable source bytes and a canonical source-relative path.
2. Resolve source identity under the existing source-set/path policy.
3. Require the explicit RustLike profile; reject Auto fallback.
4. Independently typecheck the exact captured bytes.
5. Reconstruct the canonical proposition and positive witness from trusted
   observations, not submitted EVR0 claims.
6. Bind source-set, compiler-contract, rule-registry, issuer namespace, epoch,
   session and EvidenceOnly operation.
7. Generate expected EVR0 from those observations.
8. Recompile the exact source with fixed canonical options.
9. Require byte-for-byte equality with submitted SemCode, including metadata and
   function envelopes.
10. Run structural decode and canonical verifier admission under an explicitly
    qualified revision-27 policy.
11. Only a separately authorized B2-3 stage may issue an opaque
    VerifiedEvidence token.

The verifier must not accept a caller digest, issuer label, semantic boolean,
callback or preconstructed token as a substitute.

## Trust objects

| Object | Trust status | Owner |
|---|---|---|
| Captured source bytes | immutable input | gateway |
| EVR0 bytes | untrusted claims until checked | format decoder |
| Typecheck result | trusted observation after approved checker | sm-sema/gateway |
| Recompiled artifact | comparison candidate | sm-ir/sm-emit |
| VerifiedSemCode | existing bytecode admission token | sm-verify |
| VerifiedEvidence | future opaque capability | separately approved gateway |

If later authorized, the token must bind artifact bytes, source identity,
compiler/profile/options identity, proposition identity, operation, epoch and
session. It must not be serializable, forgeable by downstream Rust or
reconstructed from a digest or integer.

## Chicken-and-egg resolution

Revision 27 is structurally recognized but intentionally rejected by canonical
verifier and VM admission. That is the safe state.

The future transition is additive:

1. implement and qualify the gateway independently;
2. reconstruct expected proof and artifact independently;
3. retain fail-closed behavior without qualified gateway context;
4. add an explicitly approved verifier admission policy for gateway output;
5. qualify legacy compatibility and downgrade rejection;
6. only then permit token issuance.

There must be no public issuer switch, temporary allowlist, capability shortcut
or revision relabeling.

## Adversarial matrix

| Scenario | Required result |
|---|---|
| Source A with artifact from B | reject: source/artifact binding mismatch |
| Forged source/compiler/issuer digest | reject: context mismatch |
| Fabricated witness or modified proof graph | reject: rule/proof mismatch |
| Cross-source same-offset witness | reject: origin/file identity mismatch |
| Wrong epoch/session/namespace | reject: admission context mismatch |
| Replayed evidence object | reject: session/epoch/artifact binding mismatch |
| Invalid authority index | reject: issuer-context validation |
| EvidenceOnly escalated to execution operation | reject: operation denied |
| External construction of opaque token | compile/API rejection or admission rejection |
| Legacy artifact promoted to rev27 authority | reject: no legacy upgrade path |
| Metadata-only artifact difference | reject: exact artifact equality failure |
| Function-envelope byte difference | reject: exact artifact equality failure |

Final public diagnostic codes remain UNRESOLVED until D6g catalog/API mapping
is approved. Tests must assert rejection layer and stable internal category
without inventing a public code.

## Independently testable patches

1. Governance/dependency transition and acyclic sm-admission graph.
2. Gateway source capture and immutable identity.
3. Independent RustLike typecheck observation and proposition reconstruction.
4. Canonical reproduction and byte-equality binding.
5. Revision-27 verifier admission policy.
6. Opaque token issuance and red-team qualification.

No patch may skip from structural EVR0 decoding directly to trusted token
issuance.

## Remaining blockers

- no approved sm-admission Cargo transition;
- no final authenticated issuer-context interface;
- no final D6g public diagnostic catalog;
- no qualified revision-27 semantic verifier policy;
- no authorization for opaque-token issuance or VM consumption.
