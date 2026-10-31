# Q3.10 EVR0 Trust Boundary

Status: Q3.10-B1-R2 candidate architecture; pending owner contract acceptance. No B2 implementation authority.

## Threat model

The adversary controls Semantic source, generated SemCode bytes, serialized
EVR0 bytes and all untrusted caller arguments. The adversary may replay valid
bytes in another source set, epoch, session or operation. The adversary cannot
forge the trusted issuer context or alter the verifier's approved rule registry.

## Models considered

### A — Independent verification

The verifier checks every derivation against trusted source/context inputs.
This is strongest for local semantic rules, but cannot authenticate external
facts that are not present in the artifact and source context.

### B — Trusted attestation

An independently qualified issuer validates external facts and supplies an
authenticated session context. The verifier checks binding and scope. This is
necessary for facts outside the verifier's reproducible inputs, but the issuer
becomes a separate trust root.

### C — Hybrid (recommended)

Use independent verifier derivation checks for all locally reproducible rules;
use an authenticated, session-scoped issuer context only for explicitly listed
external facts. Keep those authority claims separate from quad state. EVR0
bytes never mint a handle by themselves.

## Boundary objects

| Object | Owner | Can untrusted source construct it? |
|---|---|---:|
| EVR0 bytes | producer/input | Yes, as untrusted bytes only |
| decoded proof claims | decoder | Yes, but not trusted |
| trusted issuer context | admission service | No |
| `VerifiedEvidence` token | verifier | No; verifier-only issuance |
| VM-consumable evidence token | verifier/runtime bridge | No; session- and operation-bound |

## Required B2 sequence

```text
bytes
  -> bounds-safe EVR0 decode
  -> source/context/issuer binding
  -> rule and parent graph validation
  -> derive N/F/T/S
  -> issue opaque session token
  -> consume only for the authorized operation
```

Any failure terminates the sequence. There is no fallback to a public record,
digest, integer handle, source reanalysis or legacy authorization.

## Decision register

| Item | Status |
|---|---|
| Existing SemCode/ADT0/SIG0/OWN0 layout | Accepted existing fact |
| EVR0 after ADT0 before function envelopes | Proposed choice |
| Provisional next header family/revision | Proposed placeholder, owner approval required |
| Hybrid trust model | Recommended proposal |
| Authenticated issuer context | Required unresolved interface |
| EVR0 rule registry | Required unresolved contract |
| VM consumption of evidence token | Required unresolved contract |
| Legacy artifacts gain evidence authority | Rejected |
| New opcode or capability bit | Not proposed in B1 |
| C0/bootstrap changes | Forbidden |

## B1-R1/R2 decision register

R2 supersedes any R1 claim that EVR0 identity equals the frozen SHF0 source-set
digest, or that a structurally well-formed vector is semantically admitted.
The outstanding semantic evaluator and issuer interfaces remain blocking owner
decisions.

| Question | Proposed answer | Approval status |
|---|---|---|
| Proposition reference | Explicit canonical proposition table; proof records use proposition_index | Fully specified proposal |
| Parent encoding | target_proof_id:u32; contiguous selected ranges; strictly increasing prior IDs | Fully specified proposal |
| Authority context | Index into authenticated admission authority table; numeric value has no authority | Fully specified proposal |
| Source identity | EVR0-MANIFEST digest is distinct from the frozen SHF0 source-set digest; independent trusted recomputation and typed mapping of the same files are required | Proposed mapping; issuer interface still unresolved |
| Rule ownership | Exact rule IDs and witness/parent shapes fixed as a candidate; independent semantic oracle required for observations, never hashes alone | Proposed; oracle algorithms and registry approval unresolved |
| Proof-required operations | Explicit operation registry; T-only proof required only where declared | Fully specified proposal |
| Header/revision allocation | New revision family, exact number still provisional | Owner approval required |
| Issuer authentication interface | Out-of-band trusted admission context | Owner approval required |
| VM token consumption | Session-scoped verifier token; exact VM integration boundary unresolved | Owner approval required |
| Final limits and error ABI | Four counts (including propositions <= 65,536), resource limits and public errors proposed | Owner approval required |
| Positive section vector | 241-byte payload, 249-byte section; real SHA-256 source/origin, literal wire bytes; structural-only success | Independently reproducible candidate, not verifier qualification |
| Trusted semantic evaluator | Must reconstruct P and observation from trusted source and approved language rules; producer witness is never authoritative | Owner approval of interface and concrete semantics required |

The document intentionally does not claim that the current repository can issue
or consume `VerifiedEvidence` until B2 implements and qualifies the approved
contract.
