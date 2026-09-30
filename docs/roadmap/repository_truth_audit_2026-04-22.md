# Repository Truth Audit 2026-04-22

Status: historical cleanup baseline (2026-04-22); superseded by later
repository-truth and Stable Foundation audits

## Goal

Capture the first disciplined cleanup baseline after the current `origin/main`
truth audit.

This document does not widen release claims, change semantics, or reopen scope.
It defines the cleanup order required to close repository tails without
introducing new drift.

## Canonical Inputs

- `origin/main` at `99b02833d791119c413ba1e28299ea2be6b2b6e8`
- a private operator checkpoint note, `CHECKPOINT_2026-04-22.md`, that was never
  committed to the repository (its machine-local path is redacted by Issue
  #1969, FND-060); it cannot be reproduced from repository state, so the
  `origin/main` SHA above and the repository contents are the reproducible
  inputs for this audit
- current GitHub PR, issue, and milestone state
- current repository code, tests, and `docs/spec/*` contract bundle

## Current Truth

- the repository is not in an early-language or pre-runtime phase
- the repository is already an owner-split compiler/runtime and PROMETHEUS
  integration stack
- the public contract is centered in `docs/spec/*`
- current GitHub issue state is clean: there are no open issues
- current GitHub tail state is administrative and documentation-heavy, not
  test-red or code-blocked
- current open PRs `#323` and `#324` are green in visible CI and therefore are
  decision tails, not failing tails

## Tail Inventory

### P0 - Administrative Truth Tails

- decide the fate of open PR `#323`
- decide the fate of open PR `#324`
- close GitHub milestone `#23` `M7 UI Application Boundary` if it remains
  completed baseline history rather than an active stream
- close GitHub milestone `#24` `M8 Everyday Expressiveness Foundation` if it
  remains completed baseline history rather than an active stream

### P1 - Top-Level Documentation Drift

- align `ARCHITECTURE.md` with the current owner-split crate architecture or
  retire it as a historical artifact
- audit `docs/spec/cli.md` against the actual `smc-cli` command surface
- audit top-level entrypoint docs so that README, architecture overview, and
  spec bundle point at the same current system shape

### P2 - Roadmap And Narrative Drift

- reconcile roadmap/milestone wording where historical `proposed` language no
  longer matches landed code/spec state
- keep stable-vs-main truth explicit without overstating the published stable
  line
- preserve historical scope notes, but mark them honestly as implemented,
  frozen, proposed, or superseded as appropriate

## Explicit Non-Tails

The following are not currently the primary cleanup blockers:

- failing CI on open PRs `#323` and `#324`
- open GitHub issues
- missing core architecture separation between frontend, IR, verifier, VM, and
  PROMETHEUS runtime crates

These areas may still need maintenance later, but they are not the current
repository truth blockers.

## Cleanup Order

1. freeze this audit baseline
2. decide whether open PR `#323` should merge as the governance/process layer
3. decide whether open PR `#324` should merge, close, or be superseded
4. close stale completed GitHub milestones if no new scoped work is attached to
   them
5. repair top-level documentation drift beginning with `ARCHITECTURE.md` and
   `docs/spec/cli.md`
6. rerun the relevant validation contour for any non-document-only cleanup step
7. merge only when the affected step is green and documented

## Discipline Rule

- cleanup work must be driven by repository truth, not by convenience
- if a proposed cleanup does not reduce drift between code, tests, GitHub, and
  docs, it is not a valid cleanup step
- one cleanup PR must still equal one logical step

## Done Reading

This cleanup baseline is complete only when:

- GitHub milestones reflect actual active versus completed work
- open PRs are either merged or closed for an explicit reason
- top-level documentation no longer describes a substantially older repository
  era
- roadmap, spec, and code can be read together without major narrative
  contradiction
