# Q3.10-B2-0-R1 — Trusted Admission Gateway / Owner Decision and API RFC

Status: **PROPOSED — for explicit repository-owner technical approval only**
Repository: `skulmakov-oss/Semantic`
Worktree HEAD: `69cbdb7edb8a5f7df9053cb1a6140f321e5040e9`
Branch: `feat/q3-10-opaque-evidence-prototype`
Scope: documentation-only; **no B2 implementation or SemCode revision GO**.
Prerequisites: `q3_10_b2_0_feasibility_decision.md`,
`q3_10_b1_r3_issuer_semantic_decision.md`,
`q3_10_b2_implementation_gate.md`,
`docs/spec/evr0_proof_section_v0.md`.

## 1. Exact baseline evidence and applicable limits

- `sm-front::type_check_program(&Program)` checks a complete parsed program
  and returns `Result<(), FrontendError>`. It does not issue a witnessed
  proposition, source manifest, stable AST-owner ID or SemCode authority.
- `sm-sema::check_source` returns an aggregate `SemanticReport` on success;
  the existing `smc-cli::CliPipeline::compile_source` delegates to
  `sm-ir::compile_program_to_semcode_with_options_debug`.
- `sm-verify::verify_semcode_token` returns `VerifiedSemCode` only after
  static bytecode admission, not independent semantic-source verification.
  Existing `VerifiedEntrySemCode` is an entry-resolved SemCode token,
  **not** a general EVR0 evidence token.
- `smc-cli` already depends on `sm-sema` and `sm-verify`; the latter
  does not depend on `sm-sema`, as required by `CONSTRAINTS.md`.
- The standing Foundation Harness forbids `Cargo.toml` and dependency
  changes, and requires explicit owner GO for ABI changes. This RFC does
  not grant itself those rights.
- The candidate `EVR0` vector is a **249-byte structurally valid section
  only**, with placeholder `compiler_contract` and no actual authenticated
  issuer. No semantic-positive executable golden artifact exists yet.

## 2. Decision: what does "VerifiedEvidence" actually prove?

**Proposed narrow B2 target:**

```text
source S, fixed profile P, config C, artifact B, operation O
  | all bytes immutable for a single admission session
  +-- independent canonical source and parser admission
  +-- independent typecheck of the exact source and obligation
  +-- deterministic compiler reproduction / authenticated provenance
  +-- canonical SemCode structural and verifier admission
  +-- canonical EVR0 proof DAG and exact context checks
  +-- operation/epoch/session/policy/registry consistency
  +--> gateway-owned opaque evidence admission token
```

Source correctness and bytecode verification are **independent predicates**.
Combining their `Ok` outputs without proving source-to-artifact provenance
creates a substitution attack: typecheck a valid source A, submit an unrelated
admitted SemCode artifact B, and claim that B inherits A's semantics. A
`VerifiedEvidence` token must **not** attest to a source-derived semantic
obligation unless the admitted executable artifact is also demonstrably bound
to the **same source, profile, compiler contract and options**.

For the initial implementation, prefer **independent deterministic
recompilation** under fixed, trusted compiler identity and canonical
emission options, with **full byte equality** to the artifact being admitted.
This is stronger than checking only a producer-supplied source digest.
Since EVR0 is to be emitted by that same compiler, the concrete sequence
must explicitly avoid circular trust:

1. Resolve and independently verify the exact immutable source set.
2. Reconstruct the first supported semantic observation from that source.
3. Build the canonical expected EVR0 from those observations and an approved
   registry, revision and admission context, **not** from input EVR0 claims.
4. Reproduce the complete canonical SemCode for that exact source/profile/
   options, including the approved EVR0 header/section.
5. Require exact byte-for-byte equality with supplied SemCode, not merely
   two separately successful validators.
6. Invoke the existing `sm-verify` admission gate and verify EVR0's DAG/
   authority/operation bindings; issue the opaque token only after every
   check succeeds.

This is a **candidate protocol**, not yet implemented and not a claim
that the current compiler can already emit EVR0. If reproducible compilation
cannot be obtained without compiler trust circularity, stop and specify an
independent authenticated provenance mechanism. Do not remove the check.

Do not conflate `EVR0-MANIFEST` with the separately typed frozen
`shf0-source-set-v1` identity. A trusted source-set mapping must
recompute both from the same bytes when bootstrap admission is involved.

## 3. Issuer/token ownership: proposed option C

### Crate-level ownership (requires explicit owner choice)

**Recommended**: a dedicated `sm-admission` gateway (provisional name),
with dependencies **towards** `sm-front/sm-sema`, `sm-format`,
`sm-verify` and the necessary approved emission owner, never the reverse.
This proposed crate is not present, is not approved as a package, and its
addition/Cargo graph are **forbidden by the current Harness**.

The gateway may orchestrate but may NOT redefine existing language
semantics, SemCode structural policy, verifier admission policy or VM rules.
`sm-verify` remains the authority for bytecode and EVR0-graph admission;
the gateway is the authority for the additional *combined*
source-provenance + verifier-issued token. Consequently the initial
`VerifiedEvidence` would be **gateway-issued after successful
`sm-verify` admission**. This is not literally a token constructed inside
`sm-verify`; explicit owner acceptance is mandatory.

Alternative (without a new crate) is a tightly bounded `smc-cli` adapter
which internally performs all checks from raw inputs. It could use existing
dependencies but would risk making CLI a trust-policy owner. Do not
silently select that alternative: it requires separate owner approval
and a concrete public API/visibility review.

### Illustrative interface only (NOT production Rust API)

```text
admit_source_bound_artifact(
    raw_source_set: immutable bytes + canonical paths + import graph,
    raw_semcode: immutable byte slice,
    approved_compiler_profile: frozen enum,
    approved_options: frozen canonical options,
    expected_policy: trusted host policy,
    requested_operation: approved enum
) -> Result<OpaqueVerifiedEvidence, DeterministicEvidenceReject>
```

- The public entrypoint is allowed to receive **raw untrusted inputs**;
  it must independently validate everything. It must NOT accept a
  `bool`, `TrustedObservation`, user-implemented `TrustedIssuer`
  trait, callback, hash, mutable path or raw integer as an authority.
- In-process issuer session/state and evidence object have private
  constructors; no public `new`, `Default`, `Deserialize`, integer
  `From`, unchecked clone or unqualified `pub(crate)` reexport.
- A session token is bound to the admitted artifact, exact source set,
  compiler/profile/options, epoch, registry, policy and operation. The
  authenticated session authority itself is not serialized into SemCode.
- Native code running inside the trusted binary is excluded from the
  hostile-code threat model. A hostile linked Rust crate may call public
  safe APIs with arbitrary raw inputs, but must not be able to mint a
  token for an unchecked source or unrelated SemCode.
- A public function accepting a mutable filesystem path must snapshot
  trusted source bytes **once** and never re-open/re-read them between
  source check, compilation and verifier admission (TOCTOU).
- No direct host I/O within deterministic compiler/verifier/core crates;
  the host adapter must supply the immutable bytes.

## 4. First semantic proposition, exact domain and polarity

Candidate `kind`: `RUSTLIKE-PROGRAM-TYPECHECK-v1`.

Canonical synthetic tuple within a **single-file, no-imports,
fixed-profile** program:

```text
(kind, ast_owner_id, semantic_scope_id, type_obligation_id)
= (RUSTLIKE-PROGRAM-TYPECHECK-v1, 0, 0, 0)
```

The tuple is a reserved whole-program property, **not** a claim that
every AST node has received an individually verified type.

| Input to trusted checker | Result | Meaning |
|---|---|---|
| RustLike-owned source with successful entire-program typecheck | Positive T observation, if all source/context/provenance bindings are valid | Candidate SemCode-backed T proof |
| RustLike-owned parsed source where whole-program typecheck fails | Negative F observation for a *source-level receipt* only | **No successfully emitted SemCode artifact is assumed** |
| Lexer/parser error, unsupported profile, Logos/ambiguous ownership, absent trusted source | Rejection/unavailable evidence, **not negative F** | No typecheck assertion is established |
| Same source/profile with contradictory T and F claims | Reject forged or inconsistent witness for this kind | The deterministic checker cannot generate both |
| Unknown proposition kind, different source, epoch, compiled options or artifact | Fail closed | Never upgrade to `VerifiedEvidence` |

**Critical semantic/transport split:** a typecheck failure normally
prevents ordinary source-to-SemCode emission. Therefore do not require
an `F` witness from that failing program to be embedded in an admitted
executable `EVR0` artifact. The first SemCode-backed slice can only
prove the **T** side. A source-level negative F observation would need
a separately approved, non-executable diagnostic/evidence-record
transport. B2 must not invent one.

Generic quad N/F/T/S semantics remain unchanged. S is a valid
conflict in other proposition families but is not independently
reachable for this deterministic fixed-source whole-program check.
Preserve S semantics in the abstract verifier rule tests; never
fabricate it as a source typecheck result.

Proof witness source span for this kind is the entire single source
file, byte range `[0, source_len)`; the earlier structural vector
using `fn` as a two-byte span is **not** semantic-positive evidence.

## 5. Format/registry decisions still not frozen

- Proposed header `SEMCOD26`, revision 27: no allocation until owner GO.
  `ADT0 | EVR0 | function envelopes` remains a candidate version-gated
  placement; no legacy content sniffing or implicit authority.
- `compiler_contract` currently has placeholder content in the
  structural fixture. It must bind specific compiler implementation,
  language semantics, grammar/profile, EVR0 rules and codegen options.
  Exact canonical representation/version and digest algorithm require
  separate approval; do not invent them in production.
- Rule registry `POS-WITNESS-1` and `OP-T-ONLY-1` are candidates.
  There is **no** approved verifier-side positive proof for arbitrary
  type obligations. A POS witness without independently reconstructed
  source/AST/typechecker semantics fails admission.
- Exact stable public error codes, budget/work limits and trust
  authority mapping remain provisional.
- Proof-required VM operations are not admitted by B2. No VM change,
  opcode, capability-bit redefinition or host effect may be inferred
  from EVR0's `authorized_operation` byte.

## 6. Attack matrix for a future implementation

| Hostile input | Expected behavior |
|---|---|
| Succeeds with source A, supplies unrelated admitted SemCode B | Reject as artifact/provenance mismatch |
| Valid source and SemCode, forged EVR0 `issuer_namespace` | Reject missing/mismatched trusted issuer session |
| Valid structural EVR0, source absent/unverifiable | Reject; never mint |
| Valid EVR0, wrong compiler profile or optimization option | Reject as compilation context mismatch |
| Valid EVR0, source bytes swapped between checks | Reject; immutable snapshot required |
| Caller constructs token struct or `From<u32>` | Compile rejection; no public constructor |
| Caller invokes public `mint(true)` or fake callback | No such admission API |
| Caller replays proof across epoch/session/policy/operation | Reject context mismatch |
| Legacy SemCode with EVR0-like payload bytes | Decode only according to legacy revision; no evidence |
| Typecheck-negative source without executable SemCode | No SemCode-bound evidence; may produce separate diagnostic only |
| T and F claimed for deterministic same-source typecheck P | Reject inconsistency, not accepted S |
| Known S proposition to T-only rule in future admitted family | Reject operation, preserve conflict state |

## 7. Owner decision sheet (NOT automatically approved)

**D1 — gateway owner**: approve an explicitly new gateway crate,
or authorize a narrowly restricted `smc-cli` gateway variant? Default
recommendation: dedicated crate, subordinate to existing semantics.

**D2 — meaning of verifier-issued**: can a gateway-owned token that
encapsulates a successful `sm-verify` result satisfy the requirement?
If strict `sm-verify` issuance is mandatory, provide an alternative
nonforgeable transport and revise this design before any implementation.

**D3 — dependency envelope**: authorize an exact task-scoped
Cargo/new-package governance transition or choose a demonstrably
no-dependency compliant route. The normal Harness remains unchanged.

**D4 — initial proposition**: accept single-file RustLike whole-program
typecheck T only for executable SemCode, with F as out-of-scope source
diagnostic and S unreachable for this kind.

**D5 — source-to-artifact binding**: approve deterministic canonical
recompilation + full byte equality, or provide an equally independently
authenticated source-provenance contract.

**D6 — format/revision**: separately approve the exact EVR0 header,
section ABI, registry, limits and rejection codes. Not conferred by
an owner architecture GO.

**D7 — stage scope**: authorize B2-0 implementation-free contract
finalization first; B2-1 structural decoder, B2-2 issuer and B2-3
token require separately bounded technical GO and qualification.

This RFC is a request for **explicit** owner approval, not a
self-authorization, a substitute for the Harness, a change in
`CONSTRAINTS.md`, or an ABI implementation task.

**Checkpoint status:** `Q3.10-B2-0-R1 OWNER DECISION PACKAGE READY;
B2 IMPLEMENTATION BLOCKED UNTIL D1–D7 ARE RESOLVED`.
