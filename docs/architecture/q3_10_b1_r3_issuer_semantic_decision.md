# Q3.10-B1-R3 — Issuer and Semantic Verification Architecture Decision

Status: **ENGINEERING DECISION CANDIDATE — owner technical review required**
Task: Q3.10-B1-R3, documentation-only; no new public SemCode ABI or B2 implementation authorized.
Baseline: `skulmakov-oss/Semantic@69cbdb7edb8a5f7df9053cb1a6140f321e5040e9`
Inputs: `docs/spec/evr0_proof_section_v0.md`, `q3_10_evr0_trust_boundary.md`,
`q3_10_b1_contract_review.md`, `docs/spec/semcode.md`, `docs/spec/verifier.md`,
`AGENTS.md`, `CONSTRAINTS.md`, active Harness. The existing blocker remains unchanged.

## 1. Observed implementation and non-negotiable boundaries

- `sm-format` is the SemCode format and decode owner; its current canonical
  decode starts from the 8-byte magic and includes revision-gated `ADT0`
  before function envelopes. No `EVR0` revision is currently admitted.
- `sm-verify` issues `VerifiedSemCode` and `VerifiedEntrySemCode` only
  for admitted bytecode. These are **not** semantic evidence tokens.
- `sm-sema::check_source` and `check_rustlike_program` yield validation
  outcomes/diagnostics, **not** proposition-specific, provenance-bound witnesses.
  `sm-front::type_check_program` similarly cannot presently serve as a
  witness-issuing API.
- `CONSTRAINTS.md` forbids source parsing inside `sm-verify` and
  execution-layer dependencies back into frontend/sema internals.
- The stable Foundation Harness forbids dependency/Cargo changes, governance
  changes, unapproved public ABI changes and merge. This decision may not
  authorize its own implementation by modifying that Harness.
- Untrusted source, SemCode bytes, EVR0 bytes, source-provided digests,
  caller-provided indices and caller-provided observations are **not**
  independently verified authority.

## 2. Chosen direction: split local semantic checking from admission

Select the **hybrid architecture**, within the threat model below.

```text
untrusted source bytes
       |
  trusted host I/O (outside deterministic crates)
       |
  sm-front + sm-sema (Construction)
  independently reconstruct P, source origin, obligation and polarity
       |
  trusted in-process issuer; authenticated, typed observation set
       |
       |  [context + independent observations, never naked EVR0 claims]
       v
untrusted SemCode -> sm-format -> sm-verify (Execution)
                                 validate sections, bindings, proof DAG,
                                 allowed rule predicates and operation
                                 |
                       verifier-issued VerifiedEvidence (opaque)
                                 |
                    NO VM consumption during B2
```

**The issuer is not the compiler's `valid=true` bit, source hash, public
integer or EVR0 `issuer_namespace`.** It is a separately trusted checking
component whose output is accepted only in an authenticated admission session.
The issuer's proof of semantic correctness is the independent check of the
exact proposition on authenticated bytes, not merely a statement that a
compiler run succeeded.

## 3. Threat boundary and trust assumptions

Adversary controls Semantic source, emitted SemCode, arbitrary serialized EVR0,
proof metadata and all *untrusted data arguments* supplied to admission. The
adversary can replay across source sets, epochs, sessions and operations.
Adversary does **not** execute arbitrary native Rust within the trusted host
process, replace installed trusted binaries, replace the approved rule registry
or seize the host's issuer credentials. If hostile native Rust is included in
the threat model, in-process Rust type privacy is insufficient: a process
isolation/authenticated IPC contract is separately required, not part of B2.

Preferred B2 trust transport: **in-process, nonserialized typed issuer session**
with private issuance and explicit lifetime. No cryptographic external/remote
issuer, no self-signed EVR0, no implicit trust from compiler_contract digest.
The trusted host owns filesystem access and supply of exact bytes but **cannot
invent semantic rule outcomes**. A trusted checker inside Construction derives
them using the approved semantic contract.

A public arbitrary-caller trait object or callback such as
`fn verify(ev: &EVR0, validator: impl Fn(..) -> bool)` is forbidden as a trust
root. Merely changing this to `dyn Trait` or a public `new(...)` method does
not solve forgery. B2 must prove the sole issuance path and its isolation from
user-controlled arguments. A token must not implement deserialize, public
construction, `From<u32>`, arbitrary `Clone` across sessions or any raw
conversion that grants authority.

## 4. Candidate issuer interface and exact checking responsibilities

The following are **semantic contracts for implementation review**, not
approved exported Rust signatures:

```text
TrustedSourceSet:
  typed EVR0-MANIFEST identity, exact checked file bytes and paths,
  import graph, source-set membership, optional separate SHF0 identity

TrustedSemanticObservation:
  P = (kind, ast_owner_id, semantic_scope_id, type_obligation_id)
  polarity = positive | negative
  file_digest, byte_start, byte_end, origin_digest
  approved semantic-contract identity, provenance of checked rule

TrustedIssuerSession:
  private construction from trusted checker result
  issuer_namespace, policy_namespace, session identity, epoch
  registry identity, allowed operation, authority table
  artifact association (exact admitted SemCode hash)
```

For **every** POS/NEG witness, a trusted Construction-side checker must:

1. Resolve the source set from a trusted host-supplied manifest, not the EVR0
   producer's manifest claim. Validate canonical paths, import graph, bytes,
   file digests and exact origin spans.
2. Parse and type-check under an explicitly fixed compiler/profile/contract.
   Reconstruct the canonical proposition tuple from the trusted typed program,
   including AST-owner ID, semantic scope and type obligation identity.
3. Independently evaluate the claimed positive/negative observation using
   documented type/semantic rules and return its true polarity; never trust
   the witness' polarity from the serialized bytes.
4. Bind the observation to the precise immutable file/range, source-set,
   contract, issuer session, epoch and permitted operation. An S conflict
   must derive from independent same-P positive and negative observations.
5. Reject unknown kinds/rules, unprovable negative claims, missing origins,
   stale inputs, ambiguous AST identity or unavailable semantic evaluator.

`check_source: Result<SemanticReport, SemanticError>` is **not** a sufficient
implementation of this interface: it does not reconstitute the particular
proposition or negative witness. The R2 example `kind=type`, `source=[0,2)`
is currently a structurally valid candidate vector only; the bytes `fn`
are not sufficient evidence of type correctness.

For JOIN/OP rules, `sm-verify` validates already admitted observations,
checks canonical parent references and proposition/authority equality, derives
N/F/T/S from unique authenticated witnesses and denies use of S as T.

## 5. Owners and dependency discipline

| Concern | Contract owner | Rule |
|---|---|---|
| Source parsing and proposition interpretation | `sm-front`/`sm-sema` | No back-dependency from `sm-verify` |
| Exact semantic predicate definitions | `sm-sema` normative semantic spec | Checker derives from trusted source, not EVR0 |
| EVR0 framing and canonical decoding | `sm-format` | Only revision-gated structural parsing |
| Admitted proof rule registry/graph semantics | `sm-verify` | Verifier-owned admission policy; contract ID bound |
| Issuer session authority and trust transport | Trusted host-admission bridge (ownership to be fixed) | No user constructor, no generic self-asserted token |
| VM token consumption | Future task, **not B2** | No VM/host opcode or capability migration in B2 |
| SHF0 source set | Frozen Bootstrap contract | Separate from EVR0-MANIFEST; no repinning |

The actual *cross-crate typed, private issuer transport* is unresolved under
the current dependency envelope. A new neutral carrier crate, new dependency
edge, cross-process signature scheme or public API widening is **not**
implicitly permitted. B2 must first show a compilable architecture diagram
and Rust visibility proof within the approved dependency graph. If this cannot
be achieved without Cargo modifications or changing the trust model: **STOP**,
request a named owner/dependency exception before implementation.

## 6. B2 scope decision and security claim boundaries

**Chosen:** B2 ends at verifier-minted opaque evidence token and verified
proposition graph, after exact independent issuer validation. **No VM
consumption, no new proof-consuming opcode, no Semantic source surface
extension.** VM enforcement is deferred until a separately specified
consumer can be demonstrated and reviewed.

This implies that an EVR0-bearing artifact cannot confer a runtime authority
during B2. Legacy artifacts retain their ordinary verified execution path and
never acquire `VerifiedEvidence`. Evidence-required operations have no
canonical VM execution path until separately authorized. If B2 cannot mint a
token without a public counterfeit constructor, it must remain PARTIAL.

Host session identity is an opaque trusted context identity, **not** a serialized
u32 that grants authority. Deterministic semantic checks must depend only on
source bytes, contract/profile, frozen trusted rule registry and explicitly
supplied issuer context; OS randomness must never enter deterministic
SemCode/trace byte output.

## 7. Remaining decisions — not yet approved normative ABI

1. Whether `SEMCOD26`/revision 27 is available and approved as the new
   mandatory `ADT0 | EVR0` family. No automatic allocation.
2. Precise private, cross-crate issuer bridge with enforceable constructor
   restrictions and zero forbidden dependency edges. **Blocking for B2b**.
3. First concrete supported proposition kind, stable AST/obligation identity
   algorithm, positive and negative evaluation logic with independent
   fixtures. **Blocking for semantic-positive qualification**.
4. Frozen registry digest and `compiler_contract` binding semantics.
5. Resource limits (all four counts, bytes, depth, work units) and public
   versus internal rejection-code commitments.
6. Explicit host Rust attacker assumptions and, if broader, an authenticated
   process boundary with independent security review.

The user authorized continuation of architecture preparation. This file is a
technical choice record and **not** a normative owner signature, merge
approval, ABI deployment authority or claim that B2 is complete.
