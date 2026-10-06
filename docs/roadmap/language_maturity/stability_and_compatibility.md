# Stability And Compatibility Discipline

Status: current governance companion to SSF-10 (#1378). The canonical technical
compatibility contract is `docs/architecture/artifact_identity_and_trust.md`
(SSF-10 §2); deprecation states and migration rules are in
`docs/roadmap/language_maturity/deprecation_and_migration.md`. This document
is not a release qualification authority and marks no surface stable by itself.

## Goal

Define the long-horizon compatibility and release discipline required for
Semantic to be trusted as a full language platform.

This workstream is about stable user trust across source language, CLI,
standard library, packages, SemCode, and runtime surfaces. It is not only about
bytecode compatibility.

## Current Baseline

Today the repository already has:

- versioned SemCode families
- a profile contract
- release smoke and compatibility checks
- active beta-line compatibility notes

Before SSF-10 it did not yet have one complete language-platform compatibility
policy stack. SSF-10 now defines the per-surface rules and
`deprecation_and_migration.md` governs change; the original questions were:

- which surfaces are stable
- which surfaces are still draft or experimental
- how deprecation works
- how migration guidance should be published
- how breaking changes are reviewed across layers

## Why This Matters

Rust and Python earned maturity through years of stable releases,
compatibility review, migration guidance, and user trust.

Semantic still needs that durability story. Without it, users can compile and
run programs, but they cannot tell which contracts are safe to build on over
time.

## Stability Labels

The language platform should use explicit stability labels.

Required labels:

- `stable`
- `beta`
- `draft`
- `experimental`

Rule:

- no public surface should be left unlabeled once it becomes part of the
  published platform story

## Compatibility Stack

The detailed cross-layer compatibility stack is defined in:

- `docs/roadmap/language_maturity/compatibility_policy_stack.md`

That stack should cover:

- source language
- CLI
- stdlib
- package ecosystem
- SemCode
- runtime/boundary surfaces

## Deprecation And Migration

Deprecation and migration rules are defined in:

- `docs/roadmap/language_maturity/deprecation_and_migration.md`

The core principle is simple:

- stable users should not be surprised by silent surface drift

## Release Channels

The platform should distinguish release channels explicitly.

Expected channels:

- `stable`
- `beta`
- `experimental/nightly-like` if introduced later

Current honest baseline:

- the repository already uses a meaningful beta line
- the future stable line should inherit explicit compatibility policy rather
  than relying on release notes alone

## Support Window Expectations

The first compatibility discipline should define support in relative terms even
before long-term guarantees become ambitious.

Expected initial rule:

- the active stable line receives the strongest compatibility promise
- the active beta line may still evolve, but must publish compatibility notes
- draft and experimental surfaces may change faster, but must remain labeled as
  such

## Non-Goals

This workstream is not intended to:

- rewrite published history
- silently treat draft surfaces as stable
- turn every future feature into an immediate compatibility promise
- promise multi-year support windows before the platform is ready

## Acceptance Criteria

This workstream should be considered materially started only when:

- one compatibility policy exists for source language, CLI, stdlib, and package
  surfaces
- deprecation and migration rules are explicit
- stable release criteria are described beyond a single tag event
- users can tell which parts of the language are stable and which are still
  evolving

## Immediate Next Slice

The immediate next slice for this PR is to define:

- the compatibility policy stack
- deprecation and migration rules
- stability labels and release-channel expectations

because those pieces turn compatibility from a slogan into an operating policy.
