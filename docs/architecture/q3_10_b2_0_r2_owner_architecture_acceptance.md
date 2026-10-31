# Q3.10-B2-0-R2 — Owner Architecture Acceptance and Gateway Interface Freeze Candidate

Status: **D1–D5 ARCHITECTURAL DIRECTION ACCEPTED; D6 / B2 IMPLEMENTATION NOT APPROVED**
Repository: `skulmakov-oss/Semantic`
Checked baseline and worktree HEAD: `69cbdb7edb8a5f7df9053cb1a6140f321e5040e9`
Branch: `feat/q3-10-opaque-evidence-prototype`
Decision provenance: repository owner's affirmative response in the project conversation on 2026-10-31
to the D1–D5 architecture recommendation following
`docs/architecture/q3_10_b2_0_r1_gateway_owner_api_rfc.md`.
This record captures **technical direction**, not authorization to change
protected governance files or claim a normative SemCode revision.

## 1. Accepted owner decisions

| ID | Accepted direction | Exact boundary |
|---|---|---|
| D1 | Dedicated `sm-admission` gateway crate | It coordinates trusted checks; it does not own parser semantics, SemCode encoding, verifier policies or VM effects |
| D2 | Gateway-issued opaque `VerifiedEvidence` may encapsulate a genuine `sm-verify::VerifiedSemCode` admission result | This explicitly replaces the earlier literal `sm-verify`-constructs-the-token requirement; never use unverified raw bytes |
| D3 | Prepare a **narrow, auditable, separately authorized** governance/dependency transition | Acceptance does not itself edit, waive or expand the active Harness; exact protected paths and owner task authorization still required |
| D4 | First evidence kind: one-file, no-import RustLike whole-program typechecking | Only T is SemCode-backed; F is source-level diagnostic until a separate non-executable transport is approved; S cannot be fabricated for this deterministic kind |
| D5 | Trusted deterministic recompilation with full byte equality to source-bound artifact | Exact canonical source, profile, compiler version, options, EVR0 encoding and resulting verified bytes must be bound; hash-only assertion is insufficient |

D6 (new SemCode revision, registry, limits and stable rejection taxonomy) and
D7 (specific B2 stage implementation GO) are **not** implied by these decisions.
No commit, merge, release, runtime privilege or C0/bootstrap change is authorized.

## 2. Concrete Rust ownership and dependency shape

Provisional package `sm-admission`, version and public API not frozen.
The **intended** dependency edges are exclusively downstream:

```text
sm-front ----> sm-sema                [existing construction relation]
      \           \
       \           v
        ----> sm-admission  <----- sm-format
                |       |
                v       v
           sm-ir/emit  sm-verify ------> sm-format
                |
       (only existing construction dependencies)
```

The diagram communicates logical use, not a directionally exact Cargo graph;
the final minimal dependency list must be validated by `cargo metadata`
before it becomes a public contract. `sm-verify` **MUST NOT** import
`sm-sema`, `sm-front` or `sm-admission`. The gateway must not create a
second semantic rule implementation: it invokes existing certified owners.

At the inspected baseline, root `Cargo.toml` has an explicit workspace
members list. A new dedicated crate therefore necessarily requires at least:
`Cargo.toml` (root members), `crates/sm-admission/Cargo.toml`, and
`Cargo.lock` (if resolution updates it). These files are forbidden by
the standing `dependency_changes: false` envelope. Later CLI integration
could additionally require `crates/smc-cli/Cargo.toml`; that is **not** part
of the initial package transition. A dedicated governance branch and owner
transition record are required; never relax the current Harness during
ordinary implementation.

## 3. Candidate *safe* API surface (not an exported Rust signature)

Conceptual input:
```text
raw snapshot:      canonical relative path + owned immutable UTF-8 bytes
source profile:    fixed admitted RustLike profile (no Auto fallback)
compile options:   canonical codegen settings, explicit optimization/debug values
artifact:          borrowed immutable SemCode bytes
request:           EvidenceOnly / authorized_operation=0
issuer metadata:   gateway-established session/epoch, approved registry & policy
```

Conceptual output:
```text
VerifiedEvidence<'artifact>  // defined by the gateway, private fields
  contains sm_verify::VerifiedSemCode<'artifact> // *actual* admitted bytes
  typed_source_identity
  typed_artifact_identity
  exact proposition identity and positive T witness
  approved contract/profile/options identity
  session and epoch evidence (nonserialized trust state)
  operation marker: EvidenceOnly (never a capability)
```

A public **checking** entry point may take arbitrary raw bytes because
it does every check itself. It may not take a caller-built
`TrustedObservation`, `admitted: bool`, `AuthorityContext` record,
`pub TrustedIssuer` trait implementation, pluggable `Fn() -> bool`
or untrusted precomputed digest as the authority to mint.

The token must not offer `pub new`, `Default`, `Deserialize`,
`From<u32>`, public internal fields, or byte-serialization that permits
reimport as trusted authority. It should be non-`Clone` for the initial
session-scoped profile and bound to the lifetime of the artifact bytes.
A public `artifact_hash` getter remains metadata, not privilege.

The `issuer_namespace`, `authority_context` table index and session ID
in EVR0 are **not credentials**. The gateway establishes/checks the exact
trusted context out of band. In the first slice all operations requiring
runtime capabilities are denied, even when `authorized_operation=1` is
well formed structurally. Session IDs must never create an effect permission.

## 4. Required admission sequence — every predicate is mandatory

1. Host freezes an immutable source snapshot (one UTF-8 file, canonical
   project-relative path, no symlinks escaping root, no imports).
2. Construction check uses the approved fixed grammar profile and verifies
   **exclusive RustLike ownership**, not ambiguous or fallback Auto.
3. Independently typecheck the exact admitted `Program` under the approved
   semantic contract. Require full success for candidate T.
4. Reconstruct the canonical P=(RUSTLIKE-PROGRAM-TYPECHECK-v1,0,0,0),
   source-wide witness span and exact origin/file/manifest identities from
   the same snapshot.
5. Produce the expected EVR0 section from gateway-verified observations
   under the approved registry and nonprivileged operation=0.
   Never take a proposed EVR0 witness as the authoritative oracle.
6. Recompile the immutable source using approved compiler identity,
   language profile, optimization/debug settings and source/issuer
   *public metadata* to produce the entire canonical SemCode artifact.
   Avoid circular acceptance: trusted emission is derived from the
   independently checked source, not copied from untrusted EVR0.
7. Compare **all bytes** of reproduced and submitted SemCode, including
   header, ADT0, EVR0 and function envelopes. Any mismatch rejects.
8. Run canonical `sm-verify` admission over exactly these submitted bytes.
   Preserve the verified token, do not substitute a raw decoder result.
9. Evaluate canonical proof graph, issuer-context indices, parent order,
   proposition equality and polarity. Since the only evidence operation
   is `EvidenceOnly`, no T-only runtime effect exists in this slice.
10. Gateway alone constructs `VerifiedEvidence` with private fields after
    all successful checks; no alternate mint path.

A typecheck-negative source is *not* executable SemCode evidence because
a normal compiler may refuse to emit the artifact. It can be reported as a
source diagnostic without minting `VerifiedEvidence`. For this first kind,
same-source T/F conflict is impossible if checks are deterministic; if both
are claimed, reject as a forged/inconsistent assertion, not an admitted S.
Generic quad N/F/T/S semantics remain unchanged elsewhere.

## 5. Independent isolated Rust feasibility probe

A temporary three-crate Rust sample was compiled on the owner's Windows PC
with `rustc 1.97.1`, separate from this repository:

| Probe | Observed result | Claim limitation |
|---|---|---|
| Mock verifier library compile | exit 0 | Rust crate boundary only |
| Mock gateway library compile, contains verified borrowed token | exit 0 | Type/lifetime shape only |
| Normal caller test, expected valid input, source/artifact mismatch, rejected source | exit 0 (`SAFE_GATEWAY_SKELETON_PASS`) | Simplified byte checks, not real semantic verifier |
| Hostile direct `VerifiedEvidence` struct literal from outside gateway | `E0451`, exit 1 | Confirms privacy under ordinary safe Rust |
| Malicious Rust executable with arbitrary native memory access | NOT TESTED / out of threat model | Requires process boundary if included |

This experiment does **not** prove the actual `sm-admission` crate,
SemCode ABI, source semantics or issuer authorization. It demonstrates that
private construction in the combined gateway is compatible with Rust
cross-crate visibility and the borrowed verifier-token pattern.

## 6. Definition of Done for a future `sm-admission` implementation

- Compile-fail downstream tests for direct token construction, `Default`,
  `Deserialize` and unchecked external `TrustedObservation` injection.
- Integration positive T using real RustLike parser/typechecker, trusted
  reproducible SemCode, canonical `sm-verify` and actual EVR0 DAG.
- Negative tests: valid source A + valid SemCode B substitution, EVR0
  issuer spoof, wrong source set/import, version/contract/profile/options,
  epoch/session/operation, truncation/cycles/duplicate parents, unavailable
  trust root, invalid UTF-8 or grammar authority.
- R3 suite and public API contracts after precisely authorized changes;
  use tests/negative fixtures owned by approved source-authoring guard.
- No opcode, capability, VM execution or production side effect.
- All failures fail closed with bounded work and deterministic reject class.

## 7. Pending gate before B2-1

**B2-1 structural work MAY become possible without dependency changes**, by
modifying only the existing `sm-format` sources and related tests/specs,
but only after a separate explicit owner D6 normative freeze of header,
exact EVR0 section semantics, resource limits and rejection taxonomy.
It must never invent a trusted issuer or mint `VerifiedEvidence`.
B2-2 gateway/issuer work independently awaits a dedicated approved
Cargo/dependency transition (D3 protected-file authorization).

**Current status: Q3.10-B2-0-R2 ARCHITECTURE D1–D5 RECORDED;
D6 ABI FREEZE / D7 B2-1 IMPLEMENTATION GO STILL PENDING.**
