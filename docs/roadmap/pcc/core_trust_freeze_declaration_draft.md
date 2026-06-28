# CTF-6 — Core Trust Freeze Declaration Draft

## Status

Status: DRAFT / NOT AN ACTIVE FREEZE DECLARATION

Core Trust Freeze: NOT DECLARED COMPLETE BY THIS DOCUMENT

This document drafts the declaration language for a conservative Core Trust Freeze candidate contour. It does not activate the freeze.

## Baseline Evidence

- PR #1185 merge commit: `37a5c8e pcc: sequence ownership conservative contour`
- PR #1186 merge commit: `7144bc6 docs(trust): map final Core Trust Freeze readiness (#1186)`
- PR #1187 merge commit: `89e89dc docs(trust): record CTF-5 post-merge closeout (#1187)`
- current work starts from synced `main`

## Candidate Freeze Scope

The candidate freeze scope is limited to the conservative verified core contour.

Included:

- verifier-first canonical execution route;
- `verify_semcode_token`;
- `VerifiedSemCode` / `VerifiedEntrySemCode`;
- `run_verified_entry_semcode*` as canonical token-first execution wording;
- `sm-format` as SemCode format/decode authority;
- `sm-vm` dependency boundary;
- `sm-verify` admission boundary;
- `prom-cap` capability boundary;
- raw execution compatibility classification;
- runtime ownership conservative contour;
- record field ownership;
- tuple index ownership;
- ADT payload ownership vocabulary / conservative contour;
- static sequence index ownership;
- dynamic sequence fallback: `seq[i] -> seq`;
- public claim wording guardrails.

## Explicit Exclusions

The draft declaration excludes:

- full no_std qualification;
- embedded-ready claims;
- `SequenceIndexDynamic`;
- symbolic dynamic sequence ownership;
- runtime dynamic-index equality;
- range ownership;
- iterator ownership;
- advanced alias reasoning;
- full contract/schema runtime semantics;
- broad Logos/System/Entity/Law qualification;
- UI/Workbench product readiness;
- full language completion;
- release-ready or production-stable claims.

These exclusions are not blockers unless explicitly added to a future freeze scope.

## Draft Declaration Text

Draft wording:

> The Semantic project is ready to declare a conservative Core Trust Freeze candidate contour for the verified execution core described in the CTF readiness map.
>
> The frozen candidate contour is limited to verifier-first admission, SemCode format authority, deterministic VM execution through verified tokens, conservative runtime ownership semantics, and explicit boundary classifications.
>
> This declaration does not claim full language completion, full no_std qualification, symbolic ownership precision, UI product readiness, or release stability.

Important:

This is draft wording only. It must not be treated as an active declaration until a future explicit declaration PR is reviewed and merged.

## Required Final Declaration Gates

Before any final declaration PR, the following must be true:

- `cargo fmt --check` passes;
- `cargo clippy --workspace --all-targets -- -D warnings` passes;
- `cargo test --workspace --all-features` passes;
- `tools/7hell` passes in the selected CI contour;
- trust-boundary guards pass;
- public claim wording remains conservative;
- no new blockers are introduced after this draft;
- final declaration PR explicitly states Core Trust Freeze scope and exclusions.

CTF promotion gates must also be satisfied before any `freeze-candidate -> evidence-backed -> frozen` promotion:

- linked evidence is present;
- the release-facing contract is intended;
- CTF owner approval is explicit;
- overclaim risk has been reviewed;
- compatibility or migration impact is documented where public behavior matters.

## Non-Claims

This draft does not claim:

- Core Trust Freeze complete;
- stable release readiness;
- production readiness;
- full no_std qualification;
- symbolic/range/iterator ownership;
- general-purpose alias analysis;
- UI/Workbench authority;
- full language completion.

## Recommended Next Step

Recommended next PR: `CTF-6a — Core Trust Freeze Declaration Final Review`

That PR may either:

- promote the draft into an active declaration if all gates still pass; or
- record remaining blockers if new evidence appears.

You may polish wording, but do not strengthen claims.
