# Q3.10-B2 — Implementation Gate and Engineering Authorization Candidate

Status: **DRAFT GATE — NOT OWNER ABI GO / NOT AN ACTIVE IMPLEMENTATION TASK**
Task preparation after B1-R2 and B1-R3 decision review.
Baseline: `skulmakov-oss/Semantic@69cbdb7edb8a5f7df9053cb1a6140f321e5040e9`
Related: `docs/spec/evr0_proof_section_v0.md`,
`docs/architecture/q3_10_b1_r3_issuer_semantic_decision.md`.
Preserve all existing blockers and uncommitted user work.

## Why B2 cannot be one blind implementation pass

At the current baseline `sm-format` does not decode `EVR0`,
`sm-verify` does not verify the truth of type propositions, and
`sm-sema::check_source` does not mint a proposition-specific witness.
The active Harness also sets `dependency_changes: false`.
Implementing a public issuer constructor, blindly importing `sm-sema`
into `sm-verify`, treating `POS1` as an alias, or trusting caller-supplied
digests would violate this work's security and/or repository constraints.

## Proposed bounded work packages

### B2-0 — Pre-implementation authority check (no production edits)

- Verify exact `origin/main` ancestry and current task/PR base.
- Check all documentation and approval records for `SEMCOD26`/revision 27,
  mandatory `EVR0` placement, policy and resource limits.
- Construct and security-review a Rust-visible issuance/consumption type
  sketch with no publicly forgeable token. Confirm no forbidden dependency
  edges or Cargo changes, using current crate APIs and a compile-fail test
  plan for untrusted downstream Rust.
- Define the *first* concrete independently decidable proposition kind,
  stable typed AST/semantic scope/obligation IDs, exact positive and negative
  semantic checks, and trusted origin/manifest inputs.
- Define canonical `compiler_contract` and verifier-owned rule-registry
  identities (including version/representation and hash authority).
- Capture explicit OWNER GO for the approved normative choices and the
  precise staged technical scope. If unavailable, stop **BLOCKED**.
- Mandatory Codebase Memory MCP, local-RAG and repository guard skills;
  do not substitute a shell grep for unavailable tooling.

### B2-1 — Structural encoding/decoding (only after B2-0 GO)

- In `sm-format`, add exact approved revision-gated `EVR0` support.
- Decode `ADT0 | EVR0 | function envelopes` in one canonical interpretation.
- Bounds-safe lengths, canonical proposition/proof/witness/parent ordering,
  checked arithmetic and deterministic rejection.
- The structural output remains **untrusted claims**, never an admission token.
- Legacy headers, data interpretation and `SIG0`/`OWN0`/`DBG0` behavior
  remain identical; unknown/relabelled headers fail closed.
- Verify 249-byte proposed *section* sample independently and use separate
  whole-artifact vectors for the actual implementation revision. Do not
  promote structural success to a verified semantic proof.

### B2-2 — Semantic issuer integration (separate qualification checkpoint)

- Trusted Construction-side checker independently reconstructs P and
  verifies polarity against trusted source bytes and approved rule semantics.
- Define a sealed, nonserialized issuer context with authenticated authority
  table, source-set mapping, registry identity, epoch, session and operation.
- The ordinary public API must never mint/deserialize an admitted token from
  an arbitrary Rust record, integer or caller-supplied callback.
- All paths for which independent semantic proof is unavailable reject;
  unknown proof kinds or contexts have no fallback.
- If sealing requires a new crate, Cargo change, new dependency or protected
  authorization, stop and obtain a new explicit task envelope rather than
  crossing repository boundaries.

### B2-3 — Verifier admission and opaque evidence token

- Bind proof P/parents/witnesses to trusted issuer observations and the exact
  verified SemCode artifact and permitted operation.
- Derive N/F/T/S deterministically; S remains valid conflict evidence but
  cannot satisfy T-only authorization.
- Issue an opaque `VerifiedEvidence` only after all checks. No public
  constructors, integer handles, arbitrary byte deserialization or
  unverified compatibility-adapter upgrades.
- Tests must include forged source, raw SemCode, replay across source sets,
  file identity, epoch, session, rule, registry and operation, cycle, duplicated
  witnesses, and a cross-P JOIN. All negative tests must reject, not crash.
- No VM consumption of the token in this checkpoint.

## Mandatory validation and reporting

1. Golden full SemCode artifacts, with true source and trust contexts, at
   least two structurally positive and two hostile negative vectors.
2. Unit tests in `sm-format` and `sm-verify`, real independent semantic
   checker tests, failed downstream Rust construction probes, fuzz/hostile
   length and provenance mutations, legacy version matrix.
3. Determinism: identical trusted inputs and quotas yield identical
   admitted output; negative mismatches report stable failure class.
4. `cargo test --test public_api_contracts --quiet` if API widened;
   normal workspace tests and relevant no_std compatibility where changed.
5. R3-critical review: `pwsh -File scripts/admission_guard.ps1 -FullPreflight`,
   `pwsh -File tools/7hell/run_ci.ps1`, fresh adversarial review;
   `pwsh -File scripts/harness-check.ps1`, `git diff --check`.
6. Exact baseline, HEAD, final tree, worktree state, changed paths, actual
   command exit codes, skipped tests and explicit unresolved risks.
7. Only small logical PR patches; no unrelated refactors, no force-push,
   no protected-governance edits, no merge without explicit OWNER MERGE GO.
8. `Q3.10-B QUALIFIED` requires **actual** verifier issuer/token proof,
   not merely a passing format decoder or a generated golden byte stream.

## Decision register and status

| Decision | Draft engineering direction | Gate |
|---|---|---|
| Trust model | Hybrid: source semantics in Construction; checked issuer in host; EVR0 admission in Execution | Architecture direction |
| Issuer transport | In-process sealed nonserializable typed session | API/constructor proof REQUIRED |
| Verifier responsibilities | Validate structural/graph/context, never source parse | Existing hard invariant |
| Proof kinds | No generic type proof from `check_source` return value | Concrete evaluator required |
| VM consumption | Excluded from B2 | Chosen scope boundary |
| Header | `SEMCOD26`/rev27 candidate | Separate OWNER ABI GO required |
| EVR0 field/limits/rejection public ABI | From B1 candidate pending final registry and API decisions | Separate OWNER ABI GO |
| Dependency changes | Forbidden by active Harness | New owner authorization required if needed |
| Legacy | Normal execution unchanged; no Evidence elevation | Must test |
| Release/merge | Not authorized | Separate qualification and MERGE GO |

**Current classification:**
`Q3.10-B2 GATE PREPARED — NORMATIVE ABI/ISSUER CONSTRUCTOR APPROVAL PENDING`.

This is a reviewable plan, not evidence that an EVR0 encoder, verifier, host
issuer or runtime token exists. No VM or bootstrap completion claim follows.
