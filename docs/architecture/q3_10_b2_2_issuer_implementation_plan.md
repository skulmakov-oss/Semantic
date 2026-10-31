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
8. Invoke a separately qualified canonical revision-27 producer using the same
   immutable source, fixed profile/options and independently checked semantic
   observations.
9. Require the producer to emit canonical ADT0, EVR0 and function envelopes
   without copying submitted claims.
10. Recompile/reconstruct the exact source artifact with fixed canonical
    options and require byte-for-byte equality with submitted SemCode,
    including metadata and function envelopes.
11. Bind the authenticated issuer context and run structural decode plus
    canonical verifier admission under an explicitly qualified revision-27
    policy.
12. Only a separately authorized B2-3 stage may issue an opaque
    VerifiedEvidence token.

The verifier must not accept a caller digest, issuer label, semantic boolean,
callback or preconstructed token as a substitute.

## Qualified canonical revision-27 producer prerequisite

The current producer path in sm-ir/src/legacy_lowering.rs still selects earlier
headers, including MAGIC25, and does not automatically emit revision 27.
Therefore B2-2 cannot claim exact reproduction until this prerequisite is
independently qualified.

The future producer must:

1. accept a trusted immutable source snapshot;
2. use fixed compiler, profile and options identities;
3. derive EVR0 from independently checked semantics;
4. encode canonical EVR0 without copying submitted claims;
5. emit complete canonical ADT0, EVR0 and function envelopes;
6. produce deterministic bytes for identical approved inputs;
7. support exact full-artifact comparison;
8. preserve all legacy producer behavior;
9. avoid alternate verification logic or public admission shortcuts.

Likely owners are sm-ir/sm-emit for production and sm-format for the wire
contract. Required vectors include canonical rev27 artifacts, legacy artifacts,
metadata-only mutations, function-envelope mutations, deterministic repeats
and malformed EVR0. Qualification requires exact bytes, structural decode,
canonical verifier admission and legacy regression coverage. No revision-27
producer implementation is part of this checkpoint.

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

The authenticated issuer-context interface remains OWNER DECISION REQUIRED.
Its future implementation must establish authority internally, bind session
and epoch, compiler contract and rule registry, source-set and artifact
identity, and restrict this first operation to EvidenceOnly. It must not
accept a caller boolean, callback, digest or label as proof, and it must not
provide a serialization path that upgrades untrusted bytes into authority.
The gateway-to-verifier call boundary is therefore only a future qualified
interface; ordinary revision-27 verification remains fail-closed until it is
approved and qualified.

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
4. Qualified canonical revision-27 producer.
5. Canonical reproduction and byte-equality binding.
6. Authenticated issuer-context binding.
7. Revision-27 verifier admission policy.
8. Opaque token issuance and red-team qualification.

No patch may skip from structural EVR0 decoding directly to trusted token
issuance.

## Remaining blockers

- no approved sm-admission Cargo transition;
- no final authenticated issuer-context interface;
- no final semantic-admission D6g diagnostic taxonomy or precedence;
- no qualified canonical revision-27 producer;
- no qualified revision-27 semantic verifier policy;
- no authorization for opaque-token issuance or VM consumption.
