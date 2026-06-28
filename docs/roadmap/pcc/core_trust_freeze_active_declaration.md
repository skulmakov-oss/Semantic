# CTF-7 — Core Trust Freeze Active Declaration

## Status

Status: ACTIVE DECLARATION

Core Trust Freeze: ACTIVE FOR THE CONSERVATIVE VERIFIED CORE CONTOUR

This declaration activates Core Trust Freeze only for the conservative verified core contour reviewed in CTF-5, CTF-6, and CTF-6a.

It does not declare full project completion, release readiness, production stability, full no_std qualification, or full language completion.

## Baseline

Record:

- PR #1185 merge commit: `37a5c8e pcc: sequence ownership conservative contour`
- PR #1186 merge commit: `7144bc6 docs(trust): map final Core Trust Freeze readiness (#1186)`
- PR #1188 merge commit: `4ab4cc1 docs(trust): draft Core Trust Freeze declaration (#1188)`
- PR #1189 merge commit: `01103d2 docs(trust): record CTF-6 post-merge closeout (#1189)`
- CTF-6a final review commit / PR: `ac8d044 docs(trust): review Core Trust Freeze declaration readiness (#1190)`
- current declaration branch was created from synced `main`

## Active Freeze Scope

The active freeze scope is limited to the conservative verified core contour.

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

## Freeze Meaning

Within this active contour:

- public trust claims must remain verifier-first;
- SemCode format authority must remain in `sm-format`;
- VM execution trust must remain token/admission based;
- runtime ownership claims must remain conservative;
- dynamic sequence ownership must remain `seq[i] -> seq`;
- compatibility/raw helpers must not be promoted to canonical trusted execution;
- UI/Workbench surfaces must not become authority.

Any future change that widens this contour requires a new explicit review and promotion PR.

## Explicit Exclusions

This active declaration excludes:

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

These exclusions remain outside the active freeze scope.

## Guard Requirements

The active freeze contour remains valid only while these gates remain green or explicitly audited:

- `cargo fmt --check`;
- `cargo clippy --workspace --all-targets -- -D warnings`;
- `cargo test --workspace --all-features`;
- selected `tools/7hell` contour;
- trust-boundary guards;
- dependency-boundary guards;
- public claim wording boundaries;
- SemCode authority boundary;
- verifier-first canonical route boundary.

## Future Changes Policy

Changes inside the active freeze contour must be treated as trust-sensitive.

Allowed without widening the freeze contour:

- documentation clarification that does not change claims;
- test strengthening;
- guard strengthening;
- bug fixes that preserve existing semantics and boundaries.

Requires explicit review / promotion:

- new SemCode ownership vocabulary;
- new trusted execution route;
- promotion of raw helpers to canonical execution;
- symbolic/range/iterator ownership;
- full no_std qualification claim;
- UI/Workbench authority expansion;
- release/stability claim widening.

## Non-Claims

This active declaration does not claim:

- full project completion;
- stable release readiness;
- production readiness;
- full no_std qualification;
- symbolic/range/iterator ownership;
- general-purpose alias analysis;
- UI/Workbench authority;
- full language completion.

## Recommended Next Step

Recommended next PR:

`POST-MERGE-4 — CTF-7 Active Declaration Closeout`

after this declaration PR is reviewed and merged.

## Related Roadmap

Pulsar is the internal packed-state acceleration roadmap and remains subordinate to this active contour:

- [docs/roadmap/roadmap_pulsar.md](../roadmap_pulsar.md)

Do not strengthen claims beyond this.
