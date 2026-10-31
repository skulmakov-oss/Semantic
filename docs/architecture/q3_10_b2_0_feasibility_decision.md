# Q3.10-B2-0 — Issuer Feasibility, Rust Boundary Proof, and Minimal Proposition

Status: **BOUNDED INVESTIGATION COMPLETE — B2 IMPLEMENTATION NOT AUTHORIZED**
Repository: `skulmakov-oss/Semantic`
Baseline / worktree HEAD: `69cbdb7edb8a5f7df9053cb1a6140f321e5040e9`
Branch: `feat/q3-10-opaque-evidence-prototype`
Scope: documentation + ephemeral Rust compiler experiment; no production/Cargo/ABI edit.
References: `docs/architecture/q3_10_b1_r3_issuer_semantic_decision.md`,
`docs/architecture/q3_10_b2_implementation_gate.md`,
`docs/spec/evr0_proof_section_v0.md`, `AGENTS.md`, `CONSTRAINTS.md`.

## 1. Verified local facts

- At this baseline `sm-verify` depends on `sm-format`, `sm-runtime-core`,
  and `sm-diagnostic`; it does **not** depend on `sm-front` or `sm-sema`.
  The dependency boundary forbids pulling source analysis into `sm-verify`.
- `sm-sema` depends on `sm-front`, `sm-profile`, `ton618-core`, and
  `sm-diagnostic`; it cannot directly instantiate a private field of
  a type declared in `sm-verify`.
- `smc-cli` already depends on `sm-sema` and `sm-verify`. It can
  orchestrate validated calls without a new crate dependency, but it
  cannot legitimately become the independent owner of verifier policy.
- `sm-front::type_check_program(&Program)` returns
  `Result<(), FrontendError>` and validates the whole parsed program.
  `sm-sema::check_source` emits aggregate report/error, not individually
  authenticated (P, polarity, origin) observations.
- Current Harness: `dependency_changes: false`, Cargo manifests forbidden,
  and new public SemCode revisions require independent explicit owner GO.
- Current `origin/main`, current HEAD and merge-base in this checkout all
  equal `69cbdb7edb8a5f7df9053cb1a6140f321e5040e9`. This is a local
  remote-tracking fact; a fresh fetch would be needed before publication.

## 2. Rust constructor proof: executed in TEMP only

Toolchain: `rustc 1.97.1 (8bab26f4f 2026-07-14)`.

Proof library declared a public `OpaqueEvidence` struct with a private
`_private: ()` field and an exported
`public_mint_from_claim(ProducerClaim { admitted: bool })`.
A separate downstream Rust source attempted two independent attacks:

| Attack | Actual result | Security meaning |
|---|---|---|
| `OpaqueEvidence { _private: () }` from a different crate | Compile rejection E0451, exit 1 | Rust visibility blocks direct struct construction |
| `public_mint_from_claim(ProducerClaim { admitted: true })` | Compile exit 0, runtime exit 0 | Private fields do **not** prevent issuer forgery if a public mint accepts caller assertions |

The example deliberately demonstrates a bad public mint design; it is
**not** existing Semantic code and is not an approved API.
No Rust files from this experiment belong in the repository or PR.

## 3. Is a no-dependency, verifier-minted semantic issuer possible?

Given the baseline graph, the following attractive shortcuts are unsafe:

- **Public claim/boolean callback** into `sm-verify`: forged by native
  callers; no independent semantic truth.
- **`pub trait TrustedIssuer`** implemented by any host or consumer:
  a hostile implementer can return fabricated observations.
- **`sm-verify -> sm-sema` dependency**: violates the existing dependency
  boundary, even if Cargo dependency changes were allowed.
- **Hash-only issuer**: provenance/byte integrity is not semantic correctness.
- **Giving `smc-cli` direct ownership of verifier policy**: conflicts with
  the CLI's adapter-only authority; a combined gateway would need explicit
  ownership/architecture approval.
- **Private `sm-verify::VerifiedEvidence` constructor + external semantic
  checker**: no cross-crate friend access in Rust. Without authenticated
  issuance or an approved bridge, external checking results cannot simply
  be promoted into a verifier-owned token.

**Result:** A fully sound, general-purpose semantic issuer bridge is **not
proved implementable** under the currently approved dependency/visibility
surface. This is a concrete B2-2 authorization blocker, not a reason to
silently change Cargo or weaken the issuer.

## 4. Evaluated integration choices

| Choice | Security | Repo effect | Disposition |
|---|---|---|---|
| A. Make `sm-verify` parse/type-check source | Would independently verify but violates layer | Forbidden dependency/ownership | Reject |
| B. Accept caller-supplied `TrustedObservation` or callback | Forgeable outside the restricted host | Few code changes | Reject |
| C. Build a separate trusted admission gateway that **itself** runs the deterministic `sm-sema` checker and `sm-verify` then privately issues its own opaque admission result | Can be nonforgeable if validation is complete and token has no external mint | Requires explicit new gateway owner, new crate/dependencies or carefully approved host API | **Preferred architecture proposal, NOT approved** |
| D. Cryptographically authenticated issuer attestation with pinned verifier key | Potentially supports cross-process trust | Key lifecycle, crypto dependencies, signature/replay spec | Defer; not needed for initial in-process scope |
| E. Evidence only for verifier-internal bytecode predicates | Feasible without sema issuer | Does not satisfy original semantic proposition goal | Useful later as structural smoke, not B2 qualification |

For option C, the gateway must not own parser semantics or verifier policy.
It delegates the concrete checks to their existing owners; the only trusted
constructor it exposes is an **internally validated**, nonserializable session
token. A public safe constructor accepting **raw source and raw SemCode**
can be legitimate if it independently checks all evidence itself; it must
never accept public prevalidated booleans or claim objects as equivalent
authority. The token's nominal owner would be the combined admission gateway
rather than `sm-verify`; that clarification to the requirement
"verifier-issued" must be explicitly approved before implementation.

## 5. Minimal first proposition candidate: full-program typechecking

The narrow, implementable *candidate*, not a normative language change:

```text
P.kind               = "RUSTLIKE-PROGRAM-TYPECHECK-v1"
P.ast_owner_id       = 0   # reserved synthetic program root
P.semantic_scope_id  = 0   # reserved synthetic root scope
P.type_obligation_id = 0   # reserved program-wide typecheck obligation
```

Constraints:

- Exactly one canonical project-relative `.sm` file; no imports, no
  injected modules, no multi-file namespace ambiguity.
- Trusted host supplies exact immutable UTF-8 source bytes and canonical
  path. The verifier/admission gateway rechecks the EVR0 file, origin,
  manifest digests against these immutable bytes, not claimed values.
- Witness spans the **entire** source byte range `[0, byte_len)`; no
  unqualified `fn` token span is used as evidence for whole-program type.
- Use one fixed approved RustLike `ParserProfile`, canonical grammar
  admission and `SurfaceAuthority::RustLikeOwns` outcome.
  Ambiguity, Logos-only ownership, invalid UTF-8, lexer/parser failures and
  unsupported profiles **reject**; they are not negative type witnesses.
- **Positive** observation iff the exact admitted RustLike `Program`
  satisfies `type_check_program(program) == Ok(())`.
- **Negative** observation iff the same exact admitted/owned parsed program
  reaches type checking and returns `Err(FrontendError)`, subject to an
  approved deterministic error-class scope. No producer-provided negative
  polarity can override the actual checker outcome.
- A synthetically selected P may not stand for any arbitrary per-expression
  type obligation. Do not silently generalize these reserved IDs.
- The program-level result is a deterministic binary predicate for a
  fixed source/profile and contract: a valid T and valid F are possible;
  `N` means no qualifying observation. **S is not demonstrably reachable
  for this first kind under identical trusted source/context**. Keep generic
  quad rules in the abstract model, but never fabricate a contradictory
  witness to claim an admitted S for this typecheck proposition.
- No trusted T-only operation is executable in the VM during B2.

This minimal kind is a credible positive/negative semantic-checker
prototype because existing typecheck code exposes a determinate result.
It still requires a separately qualified bridge and a stable spec for
profile ID, parse/admission authority, negative error classification,
contract identity and source provenance.

## 6. Concrete owner decisions needed before implementation

1. **Architecture:** authorize option C's trusted integrated admission
   gateway and resolve whether its opaque output satisfies the original
   "verifier-issued" requirement. If not, specify an authenticated intercrate
   issuer interface without violating layer boundaries.
2. **Dependency scope:** permit the exact dedicated gateway/Cargo graph
   changes in a named protected governance transition, or provide a
   strictly authorized no-dependency design with an equivalent security
   proof. Do not broaden the standing Harness automatically.
3. **Semantic first slice:** accept or amend the program-wide RustLike
   typecheck proposition tuple and deterministic polarity predicates.
4. **Contract IDs:** freeze canonical `compiler_contract`, parser profile
   identity and verifier registry ID; require independent source/test vectors.
5. **Binary version:** authorize `SEMCOD26` rev 27 and mandatory `ADT0`
   then `EVR0` only after definitive format/limit/reject review.
6. **Qualification:** explicitly gate B2-1 (structural section only) from
   B2-2/3 (trusted semantic issuer and actual opaque evidence token);
   do not claim issuer qualification from `EVR0` decoder tests.

## 7. Next safe checkpoint and definition of done

Recommend **Q3.10-B2-0-R1 / OWNER REVIEW**: review the gateway option,
the exact single-file proposition, and the requested dependency scope
before any structural ABI or constructor implementation. If approved,
first author a narrowly scoped public API/interface RFC with a
compile-fail forging proof and deterministic positive/negative typecheck
fixtures, then ask for B2-1 GO under exact normative contract IDs.

No production code, `Cargo.toml`, `Cargo.lock`, `sm-verify`,
`sm-vm`, C0, protected Harness or bootstrap artifact was edited.
No commit, PR, push or merge is authorized by this document alone.

## 8. Local qualification performed

- Codebase Memory MCP architecture: **read**, indexed checkout project,
  22,050 nodes / 117,148 edges; source function search was completed.
- local-RAG: **read**; 5 documents, 25 chunks, hybrid search.
- Isolated Rust test: private field direct forge **REJECT E0451**;
  public untrusted-claim mint **accepted**, demonstrating its insecurity.
- `git diff --cached --check`: **PASS**, exit 0 (new document staged
  temporarily so the check can inspect its contents).
- `pwsh -NoProfile -File scripts/harness-check.ps1`: **PASS**, exit 0,
  reported exactly this allowed documentation path.
- Trailing-whitespace check: **PASS**, 0 affected lines.
- No repository Cargo/build/format/verifier/VM qualification was executed:
  this is a documentation and external Rust-language feasibility checkpoint.

**Status: Q3.10-B2-0 TECHNICAL FEASIBILITY REVIEW COMPLETE;
TRUSTED ISSUER IMPLEMENTATION BLOCKED PENDING EXPLICIT OWNER DECISION.**
